//! Authored chunks: 32 x 24 glyph grids loaded from data/chunks.toml and validated on load.
//!
//! Glyph legend (the single source for authoring, tools/gen/gen_chunks.py mirrors it):
//! `#` rock, `.` open, `B` brick, `P` pipe, `~` water, `G` grate (extraction), `D` scrap deposit (dig),
//! `X` collapse (dig to pass), `*` light fixture, `!` gas pocket, `L` locker, `K` crate, `R` corpse,
//! `S` sewerat spawn, `O` patrol point, `@` entry (the Mat edge), `<` `>` `^` `v` exits.
//!
//! Exits sit in canonical windows so any east meets any west and any north any south:
//! horizontal exits occupy rows 11-13 of column 0 or 31; vertical exits occupy columns 14-17 of row 0 or 23.

use mat::sim::sub;
use serde::Deserialize;
use std::collections::VecDeque;

pub const CHUNK_W: usize = 32;
pub const CHUNK_H: usize = 24;
pub const EXIT_ROWS: std::ops::RangeInclusive<usize> = 11..=13;
pub const EXIT_COLS: std::ops::RangeInclusive<usize> = 14..=17;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

    pub fn opposite(self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::E => Dir::W,
            Dir::S => Dir::N,
            Dir::W => Dir::E,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::N => (0, -1),
            Dir::E => (1, 0),
            Dir::S => (0, 1),
            Dir::W => (-1, 0),
        }
    }

    pub fn bit(self) -> u8 {
        match self {
            Dir::N => 1,
            Dir::E => 2,
            Dir::S => 4,
            Dir::W => 8,
        }
    }

    pub fn glyph(self) -> char {
        match self {
            Dir::N => '^',
            Dir::E => '>',
            Dir::S => 'v',
            Dir::W => '<',
        }
    }

    /// The cells of this chunk edge that an exit in this direction occupies, in chunk coordinates.
    pub fn exit_cells(self) -> Vec<(usize, usize)> {
        match self {
            Dir::N => EXIT_COLS.map(|x| (x, 0)).collect(),
            Dir::S => EXIT_COLS.map(|x| (x, CHUNK_H - 1)).collect(),
            Dir::W => EXIT_ROWS.map(|y| (0, y)).collect(),
            Dir::E => EXIT_ROWS.map(|y| (CHUNK_W - 1, y)).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub id: String,
    pub stratum: u8,
    pub kind: String,
    /// relative pick weight and the most times it appears in one route while alternatives remain;
    /// 0 = never picked at random (entry chunks)
    pub weight: u32,
    pub note: String,
    /// row-major glyphs, CHUNK_W * CHUNK_H
    pub glyphs: Vec<u8>,
    /// exits present (Dir bits)
    pub exits: u8,
    /// true for the chunk the Mat edge opens into (has `@` cells)
    pub is_entry: bool,
}

impl Chunk {
    #[inline]
    pub fn at(&self, x: usize, y: usize) -> u8 {
        self.glyphs[y * CHUNK_W + x]
    }

    pub fn has_exit(&self, d: Dir) -> bool {
        self.exits & d.bit() != 0
    }

    pub fn exit_dirs(&self) -> impl Iterator<Item = Dir> + '_ {
        Dir::ALL.into_iter().filter(move |d| self.has_exit(*d))
    }
}

/// Substrate code a glyph compiles to, and whether a Runner can stand in it before anything is dug.
pub fn glyph_substrate(g: u8) -> u8 {
    match g {
        b'#' => sub::ROCK,
        b'B' => sub::BRICK,
        b'P' => sub::PIPE,
        b'~' => sub::WATER,
        b'G' => sub::GRATE,
        b'D' => sub::DEPOSIT,
        b'X' => sub::COLLAPSE,
        // open floor and everything that stands on open floor
        _ => sub::OPEN,
    }
}

pub fn substrate_passable(s: u8) -> bool {
    matches!(s, sub::OPEN | sub::WATER | sub::GRATE)
}

pub fn glyph_passable(g: u8) -> bool {
    substrate_passable(glyph_substrate(g))
}

const KNOWN: &[u8] = b"#.BP~GDX*!LKRSO@<>^v";

#[derive(Deserialize)]
struct ChunkRow {
    id: String,
    stratum: u8,
    kind: String,
    weight: u32,
    #[serde(default)]
    note: String,
    rows: Vec<String>,
}

#[derive(Deserialize)]
struct ChunkFile {
    chunk: Vec<ChunkRow>,
}

#[derive(Clone, Debug)]
pub struct ChunkLib {
    pub chunks: Vec<Chunk>,
}

impl ChunkLib {
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let f: ChunkFile = toml::from_str(text).map_err(|e| format!("chunks.toml: {e}"))?;
        let mut chunks = Vec::with_capacity(f.chunk.len());
        for r in f.chunk {
            chunks.push(parse_chunk(r)?);
        }
        let mut ids: Vec<&str> = chunks.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        if let Some(w) = ids.windows(2).find(|w| w[0] == w[1]) {
            return Err(format!("duplicate chunk id {}", w[0]));
        }
        for s in 1..=6u8 {
            let n = chunks.iter().filter(|c| c.stratum == s).count();
            if n > 0 && !chunks.iter().any(|c| c.stratum == s && c.is_entry) {
                return Err(format!("stratum {s} has {n} chunks but no entry chunk"));
            }
        }
        Ok(ChunkLib { chunks })
    }

    pub fn get(&self, id: &str) -> Option<&Chunk> {
        self.chunks.iter().find(|c| c.id == id)
    }

    pub fn strata(&self) -> Vec<u8> {
        let mut v: Vec<u8> = self.chunks.iter().map(|c| c.stratum).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    pub fn entry_for(&self, stratum: u8) -> Option<&Chunk> {
        self.chunks.iter().find(|c| c.stratum == stratum && c.is_entry)
    }

    pub fn pool_for(&self, stratum: u8) -> Vec<&Chunk> {
        self.chunks.iter().filter(|c| c.stratum == stratum && c.weight > 0).collect()
    }
}

fn parse_chunk(r: ChunkRow) -> Result<Chunk, String> {
    let id = r.id;
    if r.rows.len() != CHUNK_H {
        return Err(format!("{id}: {} rows, expected {CHUNK_H}", r.rows.len()));
    }
    let mut glyphs = Vec::with_capacity(CHUNK_W * CHUNK_H);
    for (y, row) in r.rows.iter().enumerate() {
        if row.len() != CHUNK_W {
            return Err(format!("{id}: row {y} has {} glyphs, expected {CHUNK_W}", row.len()));
        }
        for (x, &g) in row.as_bytes().iter().enumerate() {
            if !KNOWN.contains(&g) {
                return Err(format!("{id}: unknown glyph {:?} at {x},{y}", g as char));
            }
            glyphs.push(g);
        }
    }
    let mut chunk = Chunk { id, stratum: r.stratum, kind: r.kind, weight: r.weight, note: r.note, glyphs, exits: 0, is_entry: false };
    validate(&mut chunk)?;
    Ok(chunk)
}

/// Structural rules every chunk must meet. Fills `exits` and `is_entry`.
fn validate(c: &mut Chunk) -> Result<(), String> {
    let id = c.id.clone();
    if !(1..=6).contains(&c.stratum) {
        return Err(format!("{id}: stratum {} out of 1..=6", c.stratum));
    }
    // exits and entry: every exit window is all-or-nothing, and exit glyphs live only in their window
    for d in Dir::ALL {
        let cells = d.exit_cells();
        let marks: Vec<u8> = cells.iter().map(|&(x, y)| c.at(x, y)).collect();
        let is_exit = marks.iter().all(|&g| g == d.glyph() as u8);
        let is_entry = d == Dir::W && marks.iter().all(|&g| g == b'@');
        let none = marks.iter().all(|&g| g != d.glyph() as u8 && g != b'@');
        if !(is_exit || is_entry || none) {
            return Err(format!("{id}: {d:?} exit window is partly marked; an exit fills rows 11-13 / cols 14-17"));
        }
        if is_exit {
            c.exits |= d.bit();
        }
        if is_entry {
            c.is_entry = true;
        }
    }
    for y in 0..CHUNK_H {
        for x in 0..CHUNK_W {
            let g = c.at(x, y);
            let on_border = x == 0 || y == 0 || x == CHUNK_W - 1 || y == CHUNK_H - 1;
            let is_mark = matches!(g, b'<' | b'>' | b'^' | b'v' | b'@');
            if is_mark && !on_border {
                return Err(format!("{id}: exit glyph {:?} off the border at {x},{y}", g as char));
            }
            if on_border && !is_mark && glyph_passable(g) {
                return Err(format!("{id}: open border cell at {x},{y}; borders are solid except in exit windows"));
            }
        }
    }
    if c.exits == 0 && !c.is_entry {
        return Err(format!("{id}: no exits"));
    }
    if c.is_entry && c.weight != 0 {
        return Err(format!("{id}: entry chunks carry weight 0 (they are placed, never drawn)"));
    }
    if c.is_entry && !c.has_exit(Dir::E) {
        return Err(format!("{id}: an entry chunk needs an east exit; the Mat is to the west"));
    }
    // every exit (and the entry) reaches every other without digging
    let passable: Vec<bool> = c.glyphs.iter().map(|&g| glyph_passable(g)).collect();
    let mut ports: Vec<(usize, usize)> = Vec::new();
    for d in Dir::ALL {
        if c.has_exit(d) || (d == Dir::W && c.is_entry) {
            ports.push(d.exit_cells()[0]);
        }
    }
    let reach = flood(&passable, CHUNK_W, CHUNK_H, ports[0]);
    for p in &ports[1..] {
        if !reach[p.1 * CHUNK_W + p.0] {
            return Err(format!("{id}: exit at {},{} is not reachable from the others without digging", p.0, p.1));
        }
    }
    let total = passable.iter().filter(|&&p| p).count();
    let reached = reach.iter().filter(|&&r| r).count();
    if (reached as f32) < total as f32 * 0.6 {
        return Err(format!("{id}: only {reached} of {total} passable cells reach the exits; too much is sealed"));
    }
    Ok(())
}

/// 4-connected flood over `passable`, returning the reached mask.
pub fn flood(passable: &[bool], w: usize, h: usize, start: (usize, usize)) -> Vec<bool> {
    let mut seen = vec![false; w * h];
    let si = start.1 * w + start.0;
    if !passable[si] {
        return seen;
    }
    seen[si] = true;
    let mut q = VecDeque::from([si]);
    while let Some(i) = q.pop_front() {
        let (x, y) = (i % w, i / w);
        let mut push = |nx: usize, ny: usize| {
            let j = ny * w + nx;
            if passable[j] && !seen[j] {
                seen[j] = true;
                q.push_back(j);
            }
        };
        if x > 0 {
            push(x - 1, y);
        }
        if x + 1 < w {
            push(x + 1, y);
        }
        if y > 0 {
            push(x, y - 1);
        }
        if y + 1 < h {
            push(x, y + 1);
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib() -> ChunkLib {
        ChunkLib::from_toml(crate::CHUNKS_TOML).expect("shipped chunks parse")
    }

    #[test]
    fn shipped_library_parses_with_three_strata() {
        let l = lib();
        assert!(l.chunks.len() >= 14, "{} chunks", l.chunks.len());
        assert_eq!(l.strata(), vec![1, 2, 3]);
        for s in 1..=3 {
            assert!(l.entry_for(s).is_some());
            assert!(l.pool_for(s).len() >= 4, "stratum {s} pool too small");
        }
    }

    #[test]
    fn exits_only_on_edges_and_windows_are_whole() {
        let l = lib();
        for c in &l.chunks {
            for d in c.exit_dirs() {
                for (x, y) in d.exit_cells() {
                    assert_eq!(c.at(x, y), d.glyph() as u8, "{} {d:?}", c.id);
                }
            }
            for y in 1..CHUNK_H - 1 {
                for x in 1..CHUNK_W - 1 {
                    assert!(!matches!(c.at(x, y), b'<' | b'>' | b'^' | b'v' | b'@'), "{}: exit glyph inside at {x},{y}", c.id);
                }
            }
        }
    }

    #[test]
    fn every_chunk_has_light_or_is_dark_by_design() {
        // the design says dark by default with authored pools; at least the entries carry a fixture
        let l = lib();
        for s in 1..=3 {
            let e = l.entry_for(s).unwrap();
            assert!(e.glyphs.contains(&b'*'), "{}: entry without a light", e.id);
        }
    }

    #[test]
    fn rejects_bad_chunks() {
        let mut rows = vec!["#".repeat(32); 24];
        rows[12] = format!("<{}>", ".".repeat(30));
        let mk = |rows: &Vec<String>| {
            format!(
                "[[chunk]]\nid=\"t\"\nstratum=1\nkind=\"x\"\nweight=1\nrows=[{}]\n[[chunk]]\nid=\"e\"\nstratum=1\nkind=\"entry\"\nweight=0\nrows=[{}]\n",
                rows.iter().map(|r| format!("\"{r}\"")).collect::<Vec<_>>().join(","),
                entry_rows().iter().map(|r| format!("\"{r}\"")).collect::<Vec<_>>().join(",")
            )
        };
        // partial exit window
        assert!(ChunkLib::from_toml(&mk(&rows)).unwrap_err().contains("partly marked"));
        rows[11] = rows[12].clone();
        rows[13] = rows[12].clone();
        assert!(ChunkLib::from_toml(&mk(&rows)).is_ok());
        // open border cell
        rows[5] = format!("#{}.", "#".repeat(30));
        assert!(ChunkLib::from_toml(&mk(&rows)).unwrap_err().contains("open border"));
        rows[5] = "#".repeat(32);
        // exit sealed behind a collapse
        rows[12] = format!("<{}XX{}>", ".".repeat(14), ".".repeat(14));
        rows[11] = rows[12].clone();
        rows[13] = rows[12].clone();
        assert!(ChunkLib::from_toml(&mk(&rows)).unwrap_err().contains("not reachable"));
    }

    fn entry_rows() -> Vec<String> {
        let mut rows = vec!["#".repeat(32); 24];
        for y in 11..=13 {
            rows[y] = format!("@{}>", ".".repeat(30));
        }
        rows[10] = format!("#{}#", "*".repeat(30));
        rows
    }
}
