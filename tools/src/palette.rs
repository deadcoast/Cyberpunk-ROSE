//! SEEPAGE v2 palette: loaded from the .gpl, grouped into ramps, used for placeholder fills and palette-lock checks.

use crate::png_io::Image;
use std::path::Path;

pub type Rgb = [u8; 3];

pub struct Palette {
    pub entries: Vec<(String, Rgb)>,
}

/// Ramp membership by colour name, in dark-to-light order.
const RAMPS: &[(&str, &[&str])] = &[
    ("noir", &["Void", "Sump", "Culvert", "Concrete Night", "Wet Concrete"]),
    ("industrial", &["Rust Gray", "Rebar", "Dust", "Chalk"]),
    ("mud", &["Silt", "Sewer Brown", "Rust", "Oxide"]),
    ("bile", &["Bile", "Vomit Green", "Pond Scum"]),
    ("drain", &["Drain Blue", "Cistern", "Flood Light"]),
    ("flesh", &["Bruise", "Flesh Dim", "Flesh"]),
    ("ooze", &["Ooze Dark", "Ooze", "Ooze Lit", "Ooze Glow"]),
    ("neon", &["Hot Pink", "Cyan", "Acid Yellow", "Muzzle Orange"]),
    ("white", &["Reliquary Gold", "Vestment White"]),
];

impl Palette {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut entries = Vec::new();
        for line in text.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("Name:") || line.starts_with("Columns:") {
                continue;
            }
            let mut parts = line.split_whitespace();
            let r: u8 = parts.next().and_then(|v| v.parse().ok()).ok_or("bad gpl row")?;
            let g: u8 = parts.next().and_then(|v| v.parse().ok()).ok_or("bad gpl row")?;
            let b: u8 = parts.next().and_then(|v| v.parse().ok()).ok_or("bad gpl row")?;
            let name = parts.collect::<Vec<_>>().join(" ");
            entries.push((name, [r, g, b]));
        }
        if entries.len() < 30 {
            return Err(format!("palette has {} colours, expected 32", entries.len()));
        }
        Ok(Palette { entries })
    }

    pub fn by_name(&self, name: &str) -> Rgb {
        self.entries.iter().find(|(n, _)| n == name).map(|(_, c)| *c).unwrap_or_else(|| panic!("palette has no {name}"))
    }

    pub fn ramp(&self, ramp: &str) -> Vec<Rgb> {
        let names = RAMPS.iter().find(|(n, _)| *n == ramp).map(|(_, v)| *v).unwrap_or(&["Rust Gray", "Rebar", "Dust", "Chalk"]);
        names.iter().map(|n| self.by_name(n)).collect()
    }

    pub fn contains(&self, c: Rgb) -> bool {
        self.entries.iter().any(|(_, p)| *p == c)
    }

    /// Fraction of opaque pixels whose colour is not one of the palette's.
    pub fn off_palette_fraction(&self, img: &Image) -> f32 {
        let mut opaque = 0usize;
        let mut off = 0usize;
        for y in 0..img.h {
            for x in 0..img.w {
                let (c, a) = img.get(x, y);
                if a < 128 {
                    continue;
                }
                opaque += 1;
                if !self.contains(c) {
                    off += 1;
                }
            }
        }
        if opaque == 0 { 0.0 } else { off as f32 / opaque as f32 }
    }
}
