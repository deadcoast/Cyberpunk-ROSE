//! A 3x5 pixel font for placeholder tags and contact-sheet labels. Each glyph is five rows of three bits.
//! The shipped game font (5x7) is a Phase F asset; this one is the tool's own.

use crate::palette::Rgb;
use crate::png_io::Image;

const GLYPH_W: usize = 3;
const GLYPH_H: usize = 5;

fn glyph(c: char) -> [u8; 5] {
    match c.to_ascii_uppercase() {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b011],
        'V' => [0b101, 0b101, 0b101, 0b010, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b010, 0b101, 0b101, 0b101, 0b010],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b110, 0b001, 0b010, 0b001, 0b110],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b110, 0b001, 0b110],
        '6' => [0b011, 0b100, 0b110, 0b101, 0b010],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b010, 0b101, 0b010, 0b101, 0b010],
        '9' => [0b010, 0b101, 0b011, 0b001, 0b110],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '_' => [0b000, 0b000, 0b000, 0b000, 0b111],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        '(' => [0b010, 0b100, 0b100, 0b100, 0b010],
        ')' => [0b010, 0b001, 0b001, 0b001, 0b010],
        '*' => [0b101, 0b010, 0b111, 0b010, 0b101],
        '?' => [0b110, 0b001, 0b010, 0b000, 0b010],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '=' => [0b000, 0b111, 0b000, 0b111, 0b000],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        ' ' => [0; 5],
        _ => [0b111, 0b101, 0b101, 0b101, 0b111],
    }
}

/// Draw text at (x, y) with a 1 px gap between glyphs. `bg` is painted one pixel around the text run.
pub fn draw(img: &mut Image, x: usize, y: usize, text: &str, fg: Rgb, bg: Rgb) {
    draw_scaled(img, x, y, text, fg, bg, 1)
}

pub fn draw_scaled(img: &mut Image, x: usize, y: usize, text: &str, fg: Rgb, bg: Rgb, scale: usize) {
    let n = text.chars().count();
    if n == 0 {
        return;
    }
    let run_w = n * (GLYPH_W + 1) * scale + scale;
    let run_h = (GLYPH_H + 2) * scale;
    img.rect(x.saturating_sub(scale), y.saturating_sub(scale), run_w, run_h, bg);
    for (i, c) in text.chars().enumerate() {
        let g = glyph(c);
        for (row, bits) in g.iter().enumerate() {
            for col in 0..GLYPH_W {
                if bits & (0b100 >> col) != 0 {
                    img.rect(x + (i * (GLYPH_W + 1) + col) * scale, y + row * scale, scale, scale, fg);
                }
            }
        }
    }
}

/// The full glyph sheet, 96 cells of 4x6, ASCII 32..127, as the ui.font.mono placeholder. Transparent between
/// the strokes: a glyph cell drawn over a panel must not paint the panel (a space is nothing, not a block).
pub fn sheet(fg: Rgb, _bg: Rgb) -> Image {
    let mut img = Image::new(96 * 4, 6);
    for code in 32u8..128 {
        let g = glyph(code as char);
        let x0 = (code as usize - 32) * 4;
        for (row, bits) in g.iter().enumerate() {
            for col in 0..GLYPH_W {
                if bits & (0b100 >> col) != 0 {
                    img.put(x0 + col, row, fg);
                }
            }
        }
    }
    img
}
