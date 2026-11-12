//! Seeded assembly: 8-14 chunks laid on a chunk grid by matching opposite exits, growing from the entry chunk.
//! Same (library, seed, stratum) gives the same layout, so a route card can be re-run and learned.

use crate::chunk::{Chunk, ChunkLib, Dir};
use mat::rng::Pcg32;
use std::collections::HashMap;

pub const MIN_CHUNKS: usize = 8;
pub const MAX_CHUNKS: usize = 14;

#[derive(Clone, Debug)]
pub struct Placed {
    pub chunk: usize, // index into the library
    pub cx: i32,
    pub cy: i32,
    /// exits that join a neighbour (Dir bits); the rest are sealed at compile time
    pub joined: u8,
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub seed: u64,
    pub stratum: u8,
    pub placed: Vec<Placed>,
    /// chunk-grid bounding box, inclusive
    pub min: (i32, i32),
    pub max: (i32, i32),
}

impl Layout {
    pub fn cols(&self) -> usize {
        (self.max.0 - self.min.0 + 1) as usize
    }
    pub fn rows(&self) -> usize {
        (self.max.1 - self.min.1 + 1) as usize
    }
    pub fn entry(&self) -> &Placed {
        &self.placed[0]
    }
}

/// Pick the chunk count for a route from its seed: 8..=14, biased to the middle so most routes are a full shift.
pub fn route_len(seed: u64) -> usize {
    let mut r = Pcg32::new(seed ^ 0x5EED_C0DE);
    let a = r.below((MAX_CHUNKS - MIN_CHUNKS + 1) as u32) as usize;
    let b = r.below((MAX_CHUNKS - MIN_CHUNKS + 1) as u32) as usize;
    MIN_CHUNKS + (a + b) / 2
}

/// A route for the card: chunk count from the seed, then `assemble`.
pub fn route(lib: &ChunkLib, seed: u64, stratum: u8) -> Result<Layout, String> {
    assemble(lib, seed, stratum, route_len(seed))
}

/// Lay `n` chunks of `stratum`. The entry chunk sits at (0,0) with the Mat to its west; every further chunk is
/// drawn weighted from the stratum pool for an open exit, repeats damped, until `n` are placed or nothing fits.
pub fn assemble(lib: &ChunkLib, seed: u64, stratum: u8, n: usize) -> Result<Layout, String> {
    let n = n.clamp(1, 64);
    let entry_idx = lib.chunks.iter().position(|c| c.stratum == stratum && c.is_entry).ok_or_else(|| format!("stratum {stratum} has no entry chunk"))?;
    let pool: Vec<usize> = lib.chunks.iter().enumerate().filter(|(_, c)| c.stratum == stratum && c.weight > 0).map(|(i, _)| i).collect();
    if pool.is_empty() {
        return Err(format!("stratum {stratum} has no chunks to draw"));
    }
    let mut rng = Pcg32::new(seed.wrapping_mul(0x9E37_79B9).wrapping_add(stratum as u64));
    let mut placed: Vec<Placed> = vec![Placed { chunk: entry_idx, cx: 0, cy: 0, joined: 0 }];
    let mut grid: HashMap<(i32, i32), usize> = HashMap::from([((0, 0), 0)]); // cell -> placed index
    let mut uses: HashMap<usize, u32> = HashMap::new();
    // open exits: (placed index, dir)
    let mut frontier: Vec<(usize, Dir)> = lib.chunks[entry_idx].exit_dirs().map(|d| (0, d)).collect();

    while placed.len() < n && !frontier.is_empty() {
        // prefer the most recently opened exits a little, so routes read as a path rather than a blob
        let pick = if frontier.len() > 1 && rng.f32() < 0.5 { frontier.len() - 1 } else { rng.below(frontier.len() as u32) as usize };
        let (from, dir) = frontier.swap_remove(pick);
        let (fx, fy) = (placed[from].cx, placed[from].cy);
        let (dx, dy) = dir.delta();
        let target = (fx + dx, fy + dy);
        if target.0 < 0 {
            continue; // west of the entry column is the Mat; that exit is sealed
        }
        if let Some(&other) = grid.get(&target) {
            // already a chunk there: join if it has the facing exit, otherwise this exit is sealed
            if lib.chunks[placed[other].chunk].has_exit(dir.opposite()) {
                placed[from].joined |= dir.bit();
                placed[other].joined |= dir.opposite().bit();
                frontier.retain(|&(p, d)| !(p == other && d == dir.opposite()));
            }
            continue;
        }
        // candidates: pool chunks with the facing exit. A chunk's weight is also its cap per route (a vault
        // once, a culvert up to three times); only when every fitting chunk is at its cap does the cap lift,
        // and repeats are damped either way.
        let fitting: Vec<usize> = pool.iter().copied().filter(|&ci| lib.chunks[ci].has_exit(dir.opposite())).collect();
        if fitting.is_empty() {
            continue; // nothing in this stratum fits that way; sealed
        }
        let under_cap: Vec<usize> = fitting.iter().copied().filter(|&ci| *uses.get(&ci).unwrap_or(&0) < lib.chunks[ci].weight).collect();
        let candidates = if under_cap.is_empty() { &fitting } else { &under_cap };
        let mut weights: Vec<(usize, f32)> = candidates
            .iter()
            .map(|&ci| {
                let u = *uses.get(&ci).unwrap_or(&0) as f32;
                (ci, lib.chunks[ci].weight as f32 / ((1.0 + u) * (1.0 + u)))
            })
            .collect();
        // when the route is nearly full, prefer dead-end-ish chunks (fewer exits) so it closes cleanly
        if placed.len() + 1 >= n {
            for (ci, w) in weights.iter_mut() {
                let exits = lib.chunks[*ci].exits.count_ones() as f32;
                *w /= exits;
            }
        }
        let total: f32 = weights.iter().map(|(_, w)| w).sum();
        let mut roll = rng.f32() * total;
        let mut chosen = weights[weights.len() - 1].0;
        for (ci, w) in &weights {
            if roll < *w {
                chosen = *ci;
                break;
            }
            roll -= w;
        }
        *uses.entry(chosen).or_insert(0) += 1;
        let idx = placed.len();
        placed.push(Placed { chunk: chosen, cx: target.0, cy: target.1, joined: dir.opposite().bit() });
        placed[from].joined |= dir.bit();
        grid.insert(target, idx);
        // the new chunk's other exits: join neighbours that face them now, else open them on the frontier
        for d in lib.chunks[chosen].exit_dirs() {
            if d == dir.opposite() {
                continue;
            }
            let (ex, ey) = d.delta();
            let nb = (target.0 + ex, target.1 + ey);
            if let Some(&other) = grid.get(&nb) {
                if lib.chunks[placed[other].chunk].has_exit(d.opposite()) {
                    placed[idx].joined |= d.bit();
                    placed[other].joined |= d.opposite().bit();
                    frontier.retain(|&(p, dd)| !(p == other && dd == d.opposite()));
                }
            } else {
                frontier.push((idx, d));
            }
        }
    }

    let min = (placed.iter().map(|p| p.cx).min().unwrap(), placed.iter().map(|p| p.cy).min().unwrap());
    let max = (placed.iter().map(|p| p.cx).max().unwrap(), placed.iter().map(|p| p.cy).max().unwrap());
    Ok(Layout { seed, stratum, placed, min, max })
}

impl Layout {
    pub fn chunk<'a>(&self, lib: &'a ChunkLib, i: usize) -> &'a Chunk {
        &lib.chunks[self.placed[i].chunk]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib() -> ChunkLib {
        ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap()
    }

    #[test]
    fn route_lengths_stay_in_band() {
        for seed in 0..500u64 {
            let n = route_len(seed);
            assert!((MIN_CHUNKS..=MAX_CHUNKS).contains(&n));
        }
    }

    #[test]
    fn assembles_requested_count_for_every_stratum() {
        let l = lib();
        for stratum in 1..=6u8 {
            for seed in 0..200u64 {
                let lay = route(&l, seed, stratum).unwrap();
                assert!((MIN_CHUNKS..=MAX_CHUNKS).contains(&lay.placed.len()), "seed {seed} s{stratum}: {} chunks", lay.placed.len());
                assert_eq!(lay.placed.len(), route_len(seed), "seed {seed} s{stratum} fell short");
                assert!(l.chunks[lay.entry().chunk].is_entry);
                assert_eq!((lay.entry().cx, lay.entry().cy), (0, 0));
                assert!(lay.placed.iter().all(|p| p.cx >= 0), "seed {seed}: a chunk west of the Mat edge");
            }
        }
    }

    #[test]
    fn deterministic_per_seed() {
        let l = lib();
        let a = route(&l, 77, 2).unwrap();
        let b = route(&l, 77, 2).unwrap();
        assert_eq!(a.placed.len(), b.placed.len());
        for (p, q) in a.placed.iter().zip(&b.placed) {
            assert_eq!((p.chunk, p.cx, p.cy, p.joined), (q.chunk, q.cx, q.cy, q.joined));
        }
        let c = route(&l, 78, 2).unwrap();
        assert!(a.placed.iter().zip(&c.placed).any(|(p, q)| p.chunk != q.chunk || (p.cx, p.cy) != (q.cx, q.cy)));
    }

    #[test]
    fn joins_are_mutual_and_face_each_other() {
        let l = lib();
        for seed in 0..100u64 {
            let lay = route(&l, seed, 1).unwrap();
            let grid: HashMap<(i32, i32), usize> = lay.placed.iter().enumerate().map(|(i, p)| ((p.cx, p.cy), i)).collect();
            for (i, p) in lay.placed.iter().enumerate() {
                for d in Dir::ALL {
                    if p.joined & d.bit() == 0 {
                        continue;
                    }
                    let (dx, dy) = d.delta();
                    let o = grid[&(p.cx + dx, p.cy + dy)];
                    assert!(lay.placed[o].joined & d.opposite().bit() != 0, "seed {seed}: {i} joins {o} one way");
                    assert!(lay.chunk(&l, i).has_exit(d) && lay.chunk(&l, o).has_exit(d.opposite()));
                }
            }
        }
    }
}
