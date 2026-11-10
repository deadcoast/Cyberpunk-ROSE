//! The run's Flood (Full Design v3.14, the Threats table): on a Flood shift the water in every chunk that holds some
//! rises the day's roll over 90 s from a moment the route card prints; a Runner under water 15 s goes Down.

use crate::creature::{ticks, RunFloodRules, TICK_HZ};
use crate::map::RouteMap;
use mat::rng::Pcg32;
use mat::sim::sub;
use std::collections::VecDeque;

/// A route's Flood on a Flood shift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunFlood {
    /// tiles the water rises (0: a Flood shift whose water holds)
    pub rise: u8,
    /// the tick the rise starts, and the ticks it takes
    pub start: u64,
    pub ticks: u64,
}

impl RunFlood {
    /// The Flood for a route on a Flood shift whose roll is `rise`. The moment is seeded by the route and the day, so
    /// the card and the run agree on it.
    pub fn plan(map_seed: u64, day: u32, rise: u8, shift_ticks: u64, rules: &RunFloodRules) -> RunFlood {
        // mixed first: routes of one save have neighbouring seeds, and the generator's first draw follows its seed
        let mut z = map_seed ^ ((day as u64) << 32) ^ 0xF100D;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        let mut r = Pcg32::new(z ^ (z >> 31));
        let lo = (shift_ticks as f32 * rules.from) as u64;
        let hi = ((shift_ticks as f32 * rules.to) as u64).max(lo + 1);
        let start = lo + r.below((hi - lo).min(u32::MAX as u64) as u32) as u64;
        RunFlood { rise, start, ticks: ticks(rules.rise_s).max(1) as u64 }
    }

    /// The run's Flood: this shift's when it is a Flood shift, else the next shift's from the overtime's first tick
    /// when that one is (the run clock and the city clock are one clock).
    pub fn for_shift(map_seed: u64, day: u32, flood: Option<u8>, next: Option<u8>, shift_ticks: u64, rules: &RunFloodRules) -> Option<RunFlood> {
        flood.map(|rise| RunFlood::plan(map_seed, day, rise, shift_ticks, rules)).or_else(|| next.map(|rise| RunFlood { rise, start: shift_ticks, ticks: ticks(rules.rise_s).max(1) as u64 }))
    }

    /// Tiles risen at `tick`: the k-th lands k / rise of the way through the rise.
    pub fn level_at(&self, tick: u64) -> u8 {
        if self.rise == 0 || tick < self.start {
            return 0;
        }
        ((tick - self.start) * self.rise as u64 / self.ticks).min(self.rise as u64) as u8
    }

    /// The moment the rise starts, in seconds of the shift (what the card prints).
    pub fn start_s(&self) -> u64 {
        self.start / TICK_HZ as u64
    }
}

/// The route's water before any Flood.
pub fn base_water(map: &RouteMap) -> Vec<bool> {
    map.substrate.iter().map(|&s| s == sub::WATER).collect()
}

/// Raise each chunk's water `level` tiles over `base` (the route's water before the Flood). Each body of base water
/// in a chunk (its tiles joined inside the chunk) takes the open tiles of its chunk joined to it that lie no more than
/// `level` above its surface and no lower than its floor — the water rises, it does not pour down shafts. Returns the
/// tiles that turned to Water, in order. Calling it again with a higher level adds to what is there.
pub fn raise(map: &mut RouteMap, base: &[bool], level: u8) -> Vec<usize> {
    let (w, h) = (map.w, map.h);
    let n = w * h;
    let mut body = vec![false; n];
    let mut turned = Vec::new();
    if level == 0 {
        return turned;
    }
    for start in 0..n {
        if !base[start] || body[start] {
            continue;
        }
        let chunk = map.chunk_of[start];
        // the body: base water joined to this tile inside its chunk
        let mut tiles = vec![start];
        body[start] = true;
        let mut k = 0;
        while k < tiles.len() {
            let i = tiles[k];
            k += 1;
            for j in neighbours(i, w, h) {
                if base[j] && !body[j] && map.chunk_of[j] == chunk {
                    body[j] = true;
                    tiles.push(j);
                }
            }
        }
        let surface = tiles.iter().map(|&i| i / w).min().unwrap_or(0);
        let floor = tiles.iter().map(|&i| i / w).max().unwrap_or(0);
        let top = surface.saturating_sub(level as usize);
        // the rise: open or already-water tiles of the chunk joined to the body, between `top` and its floor
        let mut seen = vec![false; n];
        let mut q: VecDeque<usize> = tiles.iter().copied().collect();
        for &i in &tiles {
            seen[i] = true;
        }
        while let Some(i) = q.pop_front() {
            for j in neighbours(i, w, h) {
                if seen[j] || map.chunk_of[j] != chunk {
                    continue;
                }
                let y = j / w;
                if y < top || y > floor {
                    continue;
                }
                let s = map.substrate[j];
                if s == sub::OPEN {
                    map.substrate[j] = sub::WATER;
                    turned.push(j);
                } else if s != sub::WATER {
                    continue;
                }
                seen[j] = true;
                q.push_back(j);
            }
        }
    }
    turned
}

fn neighbours(i: usize, w: usize, h: usize) -> impl Iterator<Item = usize> {
    let (x, y) = (i % w, i / w);
    [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < h).then(|| i + w)].into_iter().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;
    use crate::creature::CreatureFile;

    fn map(seed: u64, stratum: u8) -> RouteMap {
        let lib = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        RouteMap::compile(&lib, &route(&lib, seed, stratum).unwrap())
    }

    #[test]
    fn the_water_rises_by_level_inside_its_chunks_and_never_below_its_floor() {
        let mut m = map(11, 1);
        let base = base_water(&m);
        assert!(base.iter().any(|&b| b), "the route holds water");
        let chunks_with_water: std::collections::BTreeSet<u16> = (0..base.len()).filter(|&i| base[i]).map(|i| m.chunk_of[i]).collect();
        let one = raise(&mut m, &base, 1);
        assert!(!one.is_empty(), "a tile of rise turned nothing");
        let two = raise(&mut m, &base, 2);
        assert!(!two.is_empty(), "the second tile added nothing");
        for &i in one.iter().chain(&two) {
            assert!(chunks_with_water.contains(&m.chunk_of[i]), "water rose in a chunk that held none");
        }
        // every risen tile is within two rows above some base water of its chunk
        let w = m.w;
        for &i in one.iter().chain(&two) {
            let (x, y) = (i % w, i / w);
            let ok = (0..base.len()).any(|j| base[j] && m.chunk_of[j] == m.chunk_of[i] && j / w <= y + 2);
            assert!(ok, "tile {x},{y} rose too far");
        }
    }

    #[test]
    fn the_card_and_the_run_agree_on_when_and_the_level_climbs_over_the_rise() {
        let defs = CreatureFile::shipped();
        let a = RunFlood::plan(11, 6, 3, 9600, &defs.flood);
        let b = RunFlood::plan(11, 6, 3, 9600, &defs.flood);
        assert_eq!(a, b);
        assert!(a.start < 4800, "the rise starts in the shift's first half");
        assert_eq!(a.level_at(a.start), 0);
        assert_eq!(a.level_at(a.start + a.ticks / 3), 1);
        assert_eq!(a.level_at(a.start + a.ticks), 3);
        assert_eq!(a.level_at(a.start + 10 * a.ticks), 3);
        assert_eq!(RunFlood::plan(11, 6, 0, 9600, &defs.flood).level_at(u64::MAX / 2), 0);
    }
}
