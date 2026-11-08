//! rose-tools — the asset pipeline for Cyberpunk: ROSE.
//!
//!   rose-tools placeholders   manifest -> game/assets/placeholders/*.png, one visual language
//!   rose-tools validate       game/assets/art/*.png against size, SEEPAGE v2 palette, outline rule
//!   rose-tools prompts        manifest -> docs/pixellab-prompts.md (per-slot generation briefs)
//!   rose-tools sheet          every placeholder (and art) on one labelled contact sheet
//!   rose-tools all            the four above, in order

mod font;
mod manifest;
mod palette;
mod png_io;

use clap::{Parser, Subcommand};
use manifest::{Manifest, Slot};
use palette::{Palette, Rgb};
use png_io::Image;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "rose-tools", version, about = "Cyberpunk: ROSE asset pipeline")]
struct Cli {
    /// repository root (defaults to the directory containing Cargo.toml above this binary, or cwd)
    #[arg(long)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Placeholders,
    Validate,
    Prompts,
    Sheet,
    All,
}

struct Paths {
    root: PathBuf,
    manifest: PathBuf,
    placeholders: PathBuf,
    art: PathBuf,
    palette: PathBuf,
    items: PathBuf,
    docs: PathBuf,
}

impl Paths {
    fn new(root: PathBuf) -> Self {
        Paths {
            manifest: root.join("game/assets/manifest"),
            placeholders: root.join("game/assets/placeholders"),
            art: root.join("game/assets/art"),
            palette: root.join("game/assets/palette/SEEPAGE_v2.gpl"),
            items: root.join("data/items.toml"),
            docs: root.join("docs"),
            root,
        }
    }
}

fn find_root() -> PathBuf {
    let mut p = std::env::current_dir().expect("cwd");
    loop {
        if p.join("Cargo.toml").exists() && p.join("game").exists() {
            return p;
        }
        if !p.pop() {
            return std::env::current_dir().expect("cwd");
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let paths = Paths::new(cli.root.unwrap_or_else(find_root));
    let pal = Palette::load(&paths.palette).unwrap_or_else(|e| die(&format!("palette: {e}")));
    let manifest = Manifest::load(&paths.manifest, &paths.items).unwrap_or_else(|e| die(&format!("manifest: {e}")));
    println!();
    println!("  ROSE-TOOLS  {}", dim(&format!("{} slots in {} files, {} items expanded", manifest.slots.len(), manifest.files, manifest.expanded_items)));
    println!("  {}", dim(&"-".repeat(72)));
    let ok = match cli.cmd {
        Cmd::Placeholders => placeholders(&paths, &manifest, &pal),
        Cmd::Validate => validate(&paths, &manifest, &pal),
        Cmd::Prompts => prompts(&paths, &manifest),
        Cmd::Sheet => sheet(&paths, &manifest, &pal),
        Cmd::All => placeholders(&paths, &manifest, &pal) && prompts(&paths, &manifest) && sheet(&paths, &manifest, &pal) && validate(&paths, &manifest, &pal),
    };
    println!();
    if !ok {
        std::process::exit(1);
    }
}

fn die(msg: &str) -> ! {
    eprintln!("  error: {msg}");
    std::process::exit(2)
}

fn dim(s: &str) -> String {
    use std::io::IsTerminal;
    if std::io::stdout().is_terminal() { format!("\x1b[2m{s}\x1b[0m") } else { s.to_string() }
}

fn row(cols: &[(&str, usize)]) {
    let mut line = String::from("  ");
    for (c, w) in cols {
        line.push_str(&format!("{:<w$}", c, w = *w));
        line.push(' ');
    }
    println!("{}", line.trim_end());
}

// ---------------------------------------------------------------- placeholders
fn placeholder_image(slot: &Slot, state: &str, pal: &Palette) -> Image {
    let (w, h) = (slot.size[0] as usize, slot.size[1] as usize);
    let frames = slot.frames.max(1) as usize;
    let mut img = Image::new(w * frames, h);
    let ramp = pal.ramp(&slot.ramp);
    let (dark, mid, light) = (ramp[0], ramp[ramp.len() / 2], ramp[ramp.len() - 1]);
    let void = pal.by_name("Void");
    let chalk = pal.by_name("Chalk");
    // UI frames and buttons are 9-sliced at runtime: a clean nine-patch (dashed Void outer line, solid inner line,
    // flat fill) and no tag, because anything in the centre gets stretched across the panel
    if slot.kind == "ui" && slot.silhouette == "frame" {
        let fill = pal.by_name(if slot.id.contains("button") { "Culvert" } else { "Sump" });
        let inner = pal.by_name(if state == "hover" { "Cyan" } else if state == "down" { "Rust Gray" } else { "Rebar" });
        for f in 0..frames {
            let x0 = f * w;
            img.rect(x0, 0, w, h, fill);
            img.rect(x0 + 2, 2, w - 4, 1, inner);
            img.rect(x0 + 2, h - 3, w - 4, 1, inner);
            img.rect(x0 + 2, 2, 1, h - 4, inner);
            img.rect(x0 + w - 3, 2, 1, h - 4, inner);
            for x in 0..w {
                if (x / 2) % 2 == 0 {
                    img.put(x0 + x, 0, void);
                    img.put(x0 + x, h - 1, void);
                }
            }
            for y in 0..h {
                if (y / 2) % 2 == 0 {
                    img.put(x0, y, void);
                    img.put(x0 + w - 1, y, void);
                }
            }
        }
        return img;
    }
    for f in 0..frames {
        let x0 = f * w;
        // fill
        img.rect(x0, 0, w, h, mid);
        // silhouette hint in the ramp's light step on a dark bed
        match slot.silhouette.as_str() {
            "capsule" => {
                img.rect(x0 + w / 4, h / 8, w / 2, h - h / 4, dark);
                img.rect(x0 + w / 4 + 1, h / 8 + 1, w / 2 - 2, h - h / 4 - 2, light);
                img.rect(x0 + w / 4 + 1, h / 8 + 1, w / 2 - 2, (h / 6).max(1), dark); // head line
            }
            "rect" => {
                let m = (w.min(h) / 8).max(1);
                img.rect(x0 + m, m, w - 2 * m, h - 2 * m, dark);
                img.rect(x0 + m + 1, m + 1, w - 2 * m - 2, h - 2 * m - 2, light);
            }
            "hatch" => {
                for y in 0..h {
                    for x in 0..w {
                        if (x + y + f * 3) % 6 == 0 {
                            img.put(x0 + x, y, dark);
                        }
                    }
                }
            }
            "disc" => {
                let r = (w.min(h) / 2) as i64 - 2;
                let (cx, cy) = ((x0 + w / 2) as i64, (h / 2) as i64);
                for y in 0..h as i64 {
                    for x in 0..w as i64 {
                        let dx = x + x0 as i64 - cx;
                        let dy = y - cy;
                        let d2 = dx * dx + dy * dy;
                        if d2 <= r * r {
                            img.put((x0 as i64 + x) as usize, y as usize, if d2 >= (r - 2) * (r - 2) { dark } else { light });
                        }
                    }
                }
            }
            "frame" => {
                let m = (w.min(h) / 8).max(2);
                img.rect(x0 + m, m, w - 2 * m, h - 2 * m, light);
                img.rect(x0 + m + 2, m + 2, w.saturating_sub(2 * m + 4), h.saturating_sub(2 * m + 4), mid);
            }
            "strip" => {
                let t = (h / 3).max(2);
                img.rect(x0, h / 2 - t / 2, w, t, dark);
                img.rect(x0, h / 2 - t / 2 + 1, w, 1, light);
            }
            "pile" => {
                let base = h - 2;
                let mut half = w / 2;
                let mut y = base;
                while half > 1 && y > h / 3 {
                    img.rect(x0 + w / 2 - half, y, half * 2, 1, if y % 2 == 0 { dark } else { light });
                    half = half.saturating_sub(w / 10 + 1);
                    y -= 1;
                }
            }
            _ => {}
        }
        // dashed 1 px Void border: the placeholder mark
        for x in 0..w {
            if (x / 2) % 2 == 0 {
                img.put(x0 + x, 0, void);
                img.put(x0 + x, h - 1, void);
            }
        }
        for y in 0..h {
            if (y / 2) % 2 == 0 {
                img.put(x0, y, void);
                img.put(x0 + w - 1, y, void);
            }
        }
        // frame tick marks along the top
        if frames > 1 && w >= 8 {
            for k in 0..=f.min(w / 3) {
                img.put(x0 + 2 + k * 2, 1, chalk);
            }
        }
        // tag at bottom-left, state digit after it; world tiles carry it on their first variant only,
        // so a painted stratum stays readable while still marked
        if w >= 16 && h >= 9 && !(slot.kind == "tile" && f > 0) {
            let mut label = slot.tag.clone();
            if !state.is_empty() {
                label.push(' ');
                label.push_str(&state.chars().take(3).collect::<String>());
            }
            font::draw(&mut img, x0 + 2, h - 8, &label, chalk, void);
        }
    }
    img
}

fn placeholders(paths: &Paths, manifest: &Manifest, pal: &Palette) -> bool {
    std::fs::create_dir_all(&paths.placeholders).expect("placeholders dir");
    let mut written = 0usize;
    for slot in &manifest.slots {
        for state in &slot.states {
            let img = placeholder_image(slot, state, pal);
            let file = paths.placeholders.join(slot.file_name(state));
            img.save(&file).unwrap_or_else(|e| die(&format!("{}: {e}", file.display())));
            written += 1;
        }
    }
    // the font sheet placeholder is the real built-in font
    let sheet = font::sheet(pal.by_name("Chalk"), pal.by_name("Void"));
    sheet.save(&paths.placeholders.join("ui.font.mono.png")).expect("font sheet");
    row(&[("placeholders", 14), (&format!("{written} files written to {}", rel(&paths.root, &paths.placeholders)), 60)]);
    true
}

// ---------------------------------------------------------------- validate
fn validate(paths: &Paths, manifest: &Manifest, pal: &Palette) -> bool {
    let mut ok = true;
    // 1. every slot has a placeholder for every state
    let mut missing = 0;
    for slot in &manifest.slots {
        for state in &slot.states {
            if !paths.placeholders.join(slot.file_name(state)).exists() {
                missing += 1;
                ok = false;
            }
        }
    }
    row(&[("placeholders", 14), (&if missing == 0 { "every slot and state has a placeholder".to_string() } else { format!("{missing} missing") }, 60)]);
    // 2. art files: known slot, exact size, palette lock, outline
    let mut checked = 0;
    let mut failed = 0;
    if let Ok(rd) = std::fs::read_dir(&paths.art) {
        let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "png")).collect();
        entries.sort();
        for path in entries {
            checked += 1;
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let (slot, state) = match manifest.slot_for_file(&name) {
                Some(s) => s,
                None => {
                    row(&[("art", 14), (&format!("{name}: no manifest slot"), 60)]);
                    failed += 1;
                    continue;
                }
            };
            let img = match Image::load(&path) {
                Ok(i) => i,
                Err(e) => {
                    row(&[("art", 14), (&format!("{name}: {e}"), 60)]);
                    failed += 1;
                    continue;
                }
            };
            let want_w = slot.size[0] as usize * slot.frames.max(1) as usize;
            let want_h = slot.size[1] as usize;
            let mut problems: Vec<String> = Vec::new();
            if img.w != want_w || img.h != want_h {
                problems.push(format!("size {}x{} (manifest {}x{})", img.w, img.h, want_w, want_h));
            }
            let off = pal.off_palette_fraction(&img);
            if off > 0.02 {
                problems.push(format!("{:.1}% off-palette", off * 100.0));
            }
            if matches!(slot.kind.as_str(), "sprite" | "icon" | "vfx") {
                // outline rule (art bible): the darkest step of the object's ramp, stepping to Void on dark ground
                let mut allowed = vec![pal.by_name("Void")];
                allowed.push(pal.ramp(&slot.ramp)[0]);
                let bad = outline_violation_fraction(&img, &allowed);
                if bad > 0.02 {
                    problems.push(format!("{:.1}% of edge pixels outside the outline colours", bad * 100.0));
                }
            }
            if problems.is_empty() {
                row(&[("art", 14), (&format!("{name}  ok{}", if state.is_empty() { String::new() } else { format!("  [{state}]") }), 60)]);
            } else {
                failed += 1;
                row(&[("art", 14), (&format!("{name}: {}", problems.join("; ")), 60)]);
            }
        }
    }
    row(&[("art", 14), (&format!("{checked} checked, {failed} failed"), 60)]);
    ok && failed == 0
}

/// Fraction of opaque pixels with a transparent 4-neighbour whose colour is not an allowed outline colour.
fn outline_violation_fraction(img: &Image, allowed: &[Rgb]) -> f32 {
    let mut edge = 0usize;
    let mut bad = 0usize;
    for y in 0..img.h {
        for x in 0..img.w {
            let (c, a) = img.get(x, y);
            if a < 128 {
                continue;
            }
            let transparent_nb = [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let nx = x as i64 + dx;
                let ny = y as i64 + dy;
                nx < 0 || ny < 0 || nx >= img.w as i64 || ny >= img.h as i64 || img.get(nx as usize, ny as usize).1 < 128
            });
            if transparent_nb {
                edge += 1;
                if !allowed.contains(&c) {
                    bad += 1;
                }
            }
        }
    }
    if edge == 0 { 0.0 } else { bad as f32 / edge as f32 }
}

// ---------------------------------------------------------------- prompts
const STYLE_SUFFIX: &str = "Sewer Noir 32 pixel art. Palette SEEPAGE v2 only. One hard light, blue-shifted shadows, three value steps, two-tone ordered dither, no anti-aliasing, no smooth gradients. Saturated colour only where something emits light. Sprites and icons carry a one-pixel black selective outline; tiles carry none.";

fn prompts(paths: &Paths, manifest: &Manifest) -> bool {
    let mut out = String::new();
    out.push_str("PixelLab prompt sheet\n=====================\n\n");
    out.push_str("Generated by `rose-tools prompts` from game/assets/manifest. Every generation uses `generate_image_bitforge` against the named anchor at style strength 70 or higher, then `rose-tools validate`.\n\n");
    out.push_str(&format!("Style suffix appended to every prompt:\n\n    {STYLE_SUFFIX}\n\n"));
    let mut by_file: BTreeMap<String, Vec<&Slot>> = BTreeMap::new();
    for s in &manifest.slots {
        if s.expanded_from.is_some() {
            continue;
        }
        by_file.entry(s.source.clone()).or_default().push(s);
    }
    for (file, slots) in by_file {
        out.push_str(&format!("{file}\n{}\n\n", "-".repeat(file.len())));
        for s in slots {
            out.push_str(&format!("### {}\n\n", s.id));
            out.push_str(&format!("- kind {} · one frame {}x{} · {} frame{} · states {}\n", s.kind, s.size[0], s.size[1], s.frames, if s.frames == 1 { "" } else { "s" }, if s.states.len() == 1 && s.states[0].is_empty() { "single".to_string() } else { s.states.join(", ") }));
            out.push_str(&format!("- anchor `{}` · ramp {} · status {}\n", if s.anchor.is_empty() { "(this is an anchor)" } else { &s.anchor }, s.ramp, s.status));
            out.push_str(&format!("- prompt: {}\n", s.prompt));
            out.push_str(&format!("- negative: {}\n\n", s.negative));
        }
    }
    out.push_str(&format!("Item icons\n----------\n\n{} icons expand from data/items.toml under `icon.item.<id>`; the prompt per icon is the item's name and codex line with the icon.item prompt as the frame.\n", manifest.expanded_items));
    std::fs::create_dir_all(&paths.docs).expect("docs");
    let file = paths.docs.join("pixellab-prompts.md");
    std::fs::write(&file, out).expect("write prompts");
    row(&[("prompts", 14), (&format!("wrote {}", rel(&paths.root, &file)), 60)]);
    true
}

// ---------------------------------------------------------------- contact sheet
fn sheet(paths: &Paths, manifest: &Manifest, pal: &Palette) -> bool {
    let sump = pal.by_name("Sump");
    let chalk = pal.by_name("Chalk");
    let void = pal.by_name("Void");
    let dust = pal.by_name("Dust");
    let cols = 6usize;
    let cell_w = 196usize;
    let cell_h = 110usize;
    // one cell per slot (first state, first 4 frames shown), items collapsed into one row of 12
    let slots: Vec<&Slot> = manifest.slots.iter().filter(|s| s.expanded_from.is_none()).collect();
    let items: Vec<&Slot> = manifest.slots.iter().filter(|s| s.expanded_from.is_some()).collect();
    let rows = (slots.len() + cols - 1) / cols + 1;
    let mut img = Image::new(cols * cell_w + 16, rows * cell_h + 40);
    img.rect(0, 0, img.w, img.h, sump);
    font::draw_scaled(&mut img, 8, 8, "SEWER NOIR 32  PLACEHOLDER CONTACT SHEET", chalk, sump, 2);
    font::draw_scaled(&mut img, 8, 22, &format!("{} SLOTS  {} ITEM ICONS  EVERY CELL IS A PLACEHOLDER UNTIL ART LANDS", slots.len(), items.len()), dust, sump, 1);
    for (i, s) in slots.iter().enumerate() {
        let cx = 8 + (i % cols) * cell_w;
        let cy = 36 + (i / cols) * cell_h;
        // label on top, image below it: a cell reads top-down
        font::draw(&mut img, cx + 4, cy + 2, &s.id.to_uppercase(), chalk, sump);
        font::draw(&mut img, cx + 4, cy + 9, &format!("{}X{} F{} S{}", s.size[0], s.size[1], s.frames, s.states.len()), dust, sump);
        let file = paths.placeholders.join(s.file_name(&s.states[0]));
        if let Ok(ph) = Image::load(&file) {
            let show_w = ph.w.min(s.size[0] as usize * 4).min(cell_w - 8);
            let scale = if s.size[1] as usize <= 32 && show_w * 2 <= cell_w - 8 { 2 } else { 1 };
            let show_h = (ph.h * scale).min(cell_h - 22);
            img.blit_scaled(&ph, cx + 4, cy + 18, show_w, show_h / scale, scale, void);
        }
    }
    // item icon strip
    let iy = 36 + ((slots.len() + cols - 1) / cols) * cell_h;
    font::draw(&mut img, 8, iy, &format!("ICON.ITEM.* ({} ICONS, FIRST 36 SHOWN)", items.len()), chalk, sump);
    for (k, s) in items.iter().take(36).enumerate() {
        let file = paths.placeholders.join(s.file_name(""));
        if let Ok(ph) = Image::load(&file) {
            img.blit_scaled(&ph, 8 + (k % 36) * 32, iy + 8, 32, 32, 1, void);
        }
    }
    let out = paths.placeholders.join("_contact_sheet.png");
    img.save(&out).expect("sheet");
    row(&[("sheet", 14), (&format!("wrote {} ({}x{})", rel(&paths.root, &out), img.w, img.h), 60)]);
    true
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).display().to_string()
}
