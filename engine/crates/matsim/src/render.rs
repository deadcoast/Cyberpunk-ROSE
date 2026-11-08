//! Headless renders in the SEEPAGE palette: the Stack view (circuit board) and a Street view crop.

use crate::city::{state, City};
use crate::sim::{hy, is_hyphal, sub, MatSim};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub type Rgb = [u8; 3];

pub mod pal {
    use super::Rgb;
    pub const VOID: Rgb = [7, 7, 12];
    pub const SUMP: Rgb = [14, 16, 24];
    pub const CULVERT: Rgb = [23, 26, 38];
    pub const RUST_GRAY: Rgb = [74, 79, 94];
    pub const REBAR: Rgb = [107, 111, 123];
    pub const CHALK: Rgb = [181, 180, 176];
    pub const SILT: Rgb = [42, 31, 25];
    pub const RUST: Rgb = [110, 78, 52];
    pub const OXIDE: Rgb = [154, 112, 73];
    pub const DRAIN_BLUE: Rgb = [18, 34, 56];
    pub const CISTERN: Rgb = [31, 61, 99];
    pub const BRUISE: Rgb = [91, 58, 74];
    pub const FLESH_DIM: Rgb = [154, 96, 112];
    pub const FLESH: Rgb = [217, 154, 166];
    pub const MAT_DARK: Rgb = [15, 46, 28];
    pub const MAT: Rgb = [31, 110, 58];
    pub const MAT_LIT: Rgb = [63, 214, 110];
    pub const MAT_GLOW: Rgb = [182, 255, 200];
    pub const HOT_PINK: Rgb = [255, 46, 151];
    pub const CYAN: Rgb = [25, 230, 255];
    pub const ACID: Rgb = [246, 255, 46];
}

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>,
}

impl Canvas {
    pub fn new(w: usize, h: usize, fill: Rgb) -> Self {
        let mut px = vec![0u8; w * h * 3];
        for p in px.chunks_mut(3) {
            p.copy_from_slice(&fill);
        }
        Canvas { w, h, px }
    }

    #[inline]
    pub fn put(&mut self, x: usize, y: usize, c: Rgb) {
        if x < self.w && y < self.h {
            let i = (y * self.w + x) * 3;
            self.px[i..i + 3].copy_from_slice(&c);
        }
    }

    pub fn rect(&mut self, x: i64, y: i64, w: i64, h: i64, c: Rgb) {
        for yy in y.max(0)..(y + h).min(self.h as i64) {
            for xx in x.max(0)..(x + w).min(self.w as i64) {
                self.put(xx as usize, yy as usize, c);
            }
        }
    }

    pub fn scaled(&self, s: usize) -> Canvas {
        let mut out = Canvas::new(self.w * s, self.h * s, pal::SUMP);
        for y in 0..self.h {
            for x in 0..self.w {
                let i = (y * self.w + x) * 3;
                let c = [self.px[i], self.px[i + 1], self.px[i + 2]];
                out.rect((x * s) as i64, (y * s) as i64, s as i64, s as i64, c);
            }
        }
        out
    }

    pub fn save_png(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let wtr = BufWriter::new(file);
        let mut enc = png::Encoder::new(wtr, self.w as u32, self.h as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(std::io::Error::other)?;
        writer.write_image_data(&self.px).map_err(std::io::Error::other)?;
        Ok(())
    }
}

/// Substrate + Mat at 1 px per tile: the base both views share.
pub fn base_tiles(sim: &MatSim, dim: bool) -> Canvas {
    let mut c = Canvas::new(sim.w, sim.h, pal::SUMP);
    for y in 0..sim.h {
        for x in 0..sim.w {
            let i = y * sim.w + x;
            let col = match sim.sub[i] {
                sub::ROCK | sub::CONCRETE => pal::CULVERT,
                sub::BRICK => if dim { pal::SILT } else { pal::RUST },
                sub::PIPE => pal::RUST_GRAY,
                sub::WATER => if dim { pal::DRAIN_BLUE } else { pal::CISTERN },
                sub::DEPOSIT => pal::REBAR,
                sub::GRATE => if dim { pal::REBAR } else { pal::CHALK },
                _ => pal::SUMP,
            };
            c.put(x, y, col);
            if sim.salt[i] {
                c.put(x, y, pal::CHALK);
            }
        }
    }
    for y in 0..sim.h {
        for x in 0..sim.w {
            let i = y * sim.w + x;
            match sim.hy[i] {
                hy::NECRO => c.put(x, y, pal::BRUISE),
                hy::CORD => c.put(x, y, if sim.sig[i] > 0.0 { pal::MAT } else { pal::MAT_DARK }),
                hy::TIP => c.put(x, y, pal::MAT_LIT),
                hy::NODE => c.put(x, y, pal::MAT_GLOW),
                _ => {
                    if !dim && sim.n[i] > 0.12 && sub::COND[sim.sub[i] as usize] > 0.0 && ((x + y) % 2 == 0 || sim.n[i] > 0.35) {
                        c.put(x, y, pal::MAT_DARK);
                    }
                }
            }
        }
    }
    c
}

/// Stack view: the stratum as a circuit board; traffic drawn as light, nothing drawn individually.
pub fn stack_view(city: &City, scale: usize) -> Canvas {
    let sim = &city.sim;
    let mut c = base_tiles(sim, true);
    let traffic = city.traffic();
    for y in 0..sim.h {
        for x in 0..sim.w {
            let t = traffic[y * sim.w + x];
            if t >= 7 {
                c.put(x, y, pal::CHALK);
            } else if t >= 3 {
                c.put(x, y, pal::MAT_GLOW);
            } else if t >= 1 {
                c.put(x, y, pal::MAT_LIT);
            }
        }
    }
    for &p in &city.pods {
        c.put(p % sim.w, p / sim.w, pal::OXIDE);
    }
    for &p in &city.pits {
        c.put(p % sim.w, p / sim.w, pal::ACID);
    }
    for &p in &city.markets {
        c.put(p % sim.w, p / sim.w, pal::HOT_PINK);
    }
    c.scaled(scale)
}

/// Street view: a crop at `scale` px per tile, slimos as glyphs with the ink outline, cargo as pulses.
pub fn street_view(city: &City, x0: usize, y0: usize, tw: usize, th: usize, scale: usize) -> Canvas {
    let sim = &city.sim;
    let base = base_tiles(sim, false);
    let mut c = Canvas::new(tw * scale, th * scale, pal::SUMP);
    for ty in 0..th {
        for tx in 0..tw {
            let sx = x0 + tx;
            let sy = y0 + ty;
            if sx >= sim.w || sy >= sim.h {
                continue;
            }
            let i = (sy * sim.w + sx) * 3;
            let col = [base.px[i], base.px[i + 1], base.px[i + 2]];
            c.rect((tx * scale) as i64, (ty * scale) as i64, scale as i64, scale as i64, col);
        }
    }
    // cargo pulses
    for &p in &city.pk_pos {
        let (px, py) = (p as usize % sim.w, p as usize / sim.w);
        if px >= x0 && px < x0 + tw && py >= y0 && py < y0 + th {
            let cx = ((px - x0) * scale + scale / 2) as i64;
            let cy = ((py - y0) * scale + scale / 2) as i64;
            c.rect(cx - 2, cy - 2, 5, 5, pal::CYAN);
        }
    }
    // agents: up to four glyphs per tile, fanned out
    let mut slot = vec![0u8; sim.n.len()];
    for (k, &p) in city.pos.iter().enumerate() {
        let (px, py) = (p as usize % sim.w, p as usize / sim.w);
        if px < x0 || px >= x0 + tw || py < y0 || py >= y0 + th {
            continue;
        }
        let s = slot[p as usize];
        if s >= 4 {
            continue;
        }
        slot[p as usize] += 1;
        let gx = ((px - x0) * scale + 2 + (s as usize % 2) * 7) as i64;
        let gy = ((py - y0) * scale + 1 + (s as usize / 2) * 7) as i64;
        let carrying = city.state[k] == state::TO_PIT;
        c.rect(gx - 1, gy - 1, 7, 9, pal::VOID);
        c.rect(gx, gy, 5, 7, if carrying { pal::FLESH } else { pal::FLESH_DIM });
        c.put((gx + 2) as usize, (gy + 3) as usize, pal::MAT_GLOW);
        if carrying {
            c.rect(gx + 4, gy - 1, 2, 2, pal::ACID);
        }
    }
    c
}

/// A single Mat frame (no city) for the grow / evidence scenarios.
pub fn mat_view(sim: &MatSim, pits: &[(usize, usize)], scale: usize) -> Canvas {
    let mut c = base_tiles(sim, false);
    for &(x, y) in pits {
        c.put(x, y, pal::HOT_PINK);
    }
    if let Some((x, y)) = sim.heart {
        c.put(x, y, pal::ACID);
    }
    let _ = is_hyphal(0);
    c.scaled(scale)
}
