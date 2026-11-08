//! RGBA8 image buffer with PNG load/save. Nearest-neighbour only; pixel art never interpolates.

use crate::palette::Rgb;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>, // RGBA
}

impl Image {
    pub fn new(w: usize, h: usize) -> Self {
        Image { w, h, px: vec![0; w * h * 4] }
    }

    #[inline]
    pub fn put(&mut self, x: usize, y: usize, c: Rgb) {
        if x < self.w && y < self.h {
            let i = (y * self.w + x) * 4;
            self.px[i] = c[0];
            self.px[i + 1] = c[1];
            self.px[i + 2] = c[2];
            self.px[i + 3] = 255;
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> (Rgb, u8) {
        let i = (y * self.w + x) * 4;
        ([self.px[i], self.px[i + 1], self.px[i + 2]], self.px[i + 3])
    }

    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgb) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                self.put(xx, yy, c);
            }
        }
    }

    /// Blit the top-left `src_w` x `src_h` of `src` at integer `scale`, transparent pixels filled with `bg`.
    pub fn blit_scaled(&mut self, src: &Image, x: usize, y: usize, src_w: usize, src_h: usize, scale: usize, bg: Rgb) {
        for sy in 0..src_h.min(src.h) {
            for sx in 0..src_w.min(src.w) {
                let (c, a) = src.get(sx, sy);
                let c = if a < 128 { bg } else { c };
                self.rect(x + sx * scale, y + sy * scale, scale, scale, c);
            }
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let w = BufWriter::new(file);
        let mut enc = png::Encoder::new(w, self.w as u32, self.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(std::io::Error::other)?;
        writer.write_image_data(&self.px).map_err(std::io::Error::other)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let decoder = png::Decoder::new(std::io::BufReader::new(file));
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let mut buf = vec![0; reader.output_buffer_size().unwrap_or(0)];
        let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
        let (w, h) = (info.width as usize, info.height as usize);
        let bytes = &buf[..info.buffer_size()];
        let mut img = Image::new(w, h);
        match (info.color_type, info.bit_depth) {
            (png::ColorType::Rgba, png::BitDepth::Eight) => img.px.copy_from_slice(bytes),
            (png::ColorType::Rgb, png::BitDepth::Eight) => {
                for i in 0..w * h {
                    img.px[i * 4..i * 4 + 3].copy_from_slice(&bytes[i * 3..i * 3 + 3]);
                    img.px[i * 4 + 3] = 255;
                }
            }
            (png::ColorType::Grayscale, png::BitDepth::Eight) => {
                for i in 0..w * h {
                    img.px[i * 4..i * 4 + 3].copy_from_slice(&[bytes[i]; 3]);
                    img.px[i * 4 + 3] = 255;
                }
            }
            (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => {
                for i in 0..w * h {
                    img.px[i * 4..i * 4 + 3].copy_from_slice(&[bytes[i * 2]; 3]);
                    img.px[i * 4 + 3] = bytes[i * 2 + 1];
                }
            }
            (ct, bd) => return Err(format!("unsupported png format {ct:?}/{bd:?} (export RGBA8 or RGB8)")),
        }
        Ok(img)
    }
}
