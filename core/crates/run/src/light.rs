//! The light field: 0-15 per tile (Minecraft's scale), propagated from sources by an 8-connected Dijkstra with
//! integer costs (research 07): orthogonal 2, diagonal 3, water x3; solids stop light but take the arriving level so
//! lit wall faces read. level = emit - ceil(cost / 2); overlapping sources take the max.
//!
//! Static sources (fixtures, grates, the Mat edge) are propagated once per route; dynamic sources (the Runner's
//! lume, muzzle flashes, lamp carriers) are re-propagated each tick and combined by max.

use crate::chunk::substrate_passable;
use crate::map::RouteMap;
use mat::sim::sub;

pub const MAX_LIGHT: u8 = 15;
/// bands (research 07, Brogue's three stealth states)
pub const DARK_MAX: u8 = 2;
pub const DIM_MIN: u8 = 3;
pub const LIT_MIN: u8 = 7;

pub const FIXTURE_EMIT: u8 = 6;
pub const GRATE_EMIT: u8 = 9;
pub const FLASH_EMIT: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub x: u16,
    pub y: u16,
    pub emit: u8,
}

#[derive(Clone, Debug)]
pub struct LightField {
    pub w: usize,
    pub h: usize,
    /// authored and Mat light, propagated once
    pub static_: Vec<u8>,
    /// static max dynamic, the field the game reads
    pub level: Vec<u8>,
    /// tiles touched by the last dynamic pass (cleared before the next)
    touched: Vec<usize>,
    scratch: Vec<u8>,
}

/// Light emitted by a Mat tile with this Lume gene (0..1): 2 + gene * 6, rounded.
pub fn mat_emit(lume_gene: f32) -> u8 {
    (2.0 + lume_gene.clamp(0.0, 1.0) * 6.0).round() as u8
}

impl LightField {
    /// Build from the route's authored lights and grates plus the Mat edge at `mat_emit`.
    pub fn new(map: &RouteMap, mat_lume_gene: f32) -> LightField {
        let n = map.w * map.h;
        let mut f = LightField { w: map.w, h: map.h, static_: vec![0; n], level: vec![0; n], touched: Vec::new(), scratch: vec![0; n] };
        let mut sources: Vec<Source> = map.lights.iter().map(|l| Source { x: l.x, y: l.y, emit: FIXTURE_EMIT }).collect();
        sources.extend(map.grates.iter().map(|&(x, y)| Source { x, y, emit: GRATE_EMIT }));
        let me = mat_emit(mat_lume_gene);
        sources.extend(map.entry.iter().map(|&(x, y)| Source { x, y, emit: me }));
        let mut buf = vec![0u8; map.w * map.h];
        let mut touched = Vec::new();
        propagate(map, &sources, &mut buf, &mut touched);
        f.static_ = buf;
        f.level = f.static_.clone();
        f
    }

    /// Re-propagate the dynamic sources and combine with the static field.
    pub fn set_dynamic(&mut self, map: &RouteMap, sources: &[Source]) {
        for &i in &self.touched {
            self.level[i] = self.static_[i];
            self.scratch[i] = 0;
        }
        self.touched.clear();
        propagate(map, sources, &mut self.scratch, &mut self.touched);
        for &i in &self.touched {
            self.level[i] = self.static_[i].max(self.scratch[i]);
        }
    }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> u8 {
        self.level[y * self.w + x]
    }

    pub fn band(level: u8) -> &'static str {
        if level <= DARK_MAX {
            "dark"
        } else if level < LIT_MIN {
            "dim"
        } else {
            "lit"
        }
    }
}

/// Cost to carry light into a tile: None blocks (light still lands on the tile).
fn step_cost(s: u8, diagonal: bool) -> Option<u32> {
    let base = match s {
        sub::WATER => 6,
        s if substrate_passable(s) => 2,
        _ => return None,
    };
    Some(if diagonal { base + 1 } else { base })
}

/// Dial's algorithm (bucket queue) over integer costs; bounded by 2 * MAX_LIGHT so it is tiny and deterministic.
/// Writes max(existing, arriving) into `out`, records every index it raised in `touched`.
fn propagate(map: &RouteMap, sources: &[Source], out: &mut [u8], touched: &mut Vec<usize>) {
    let (w, h) = (map.w, map.h);
    let max_cost = (MAX_LIGHT as u32) * 2;
    let mut buckets: Vec<Vec<(usize, u8)>> = vec![Vec::new(); max_cost as usize + 1];
    // best cost reached per tile for the current source (cost relative to that source's emit)
    // We run each source separately so "level = emit - ceil(cost/2)" is exact per source; max across sources.
    let mut best: Vec<u32> = vec![u32::MAX; w * h];
    for src in sources {
        if src.emit == 0 || (src.x as usize) >= w || (src.y as usize) >= h {
            continue;
        }
        for b in buckets.iter_mut() {
            b.clear();
        }
        let si = src.y as usize * w + src.x as usize;
        best[si] = 0;
        buckets[0].push((si, src.emit));
        let mut dirty: Vec<usize> = vec![si];
        for c in 0..=max_cost as usize {
            let mut k = 0;
            while k < buckets[c].len() {
                let (i, emit) = buckets[c][k];
                k += 1;
                if best[i] != c as u32 {
                    continue;
                }
                let level = emit as i32 - ((c as i32) + 1) / 2;
                if level <= 0 {
                    continue;
                }
                let level = level as u8;
                if out[i] < level {
                    if out[i] == 0 {
                        touched.push(i);
                    }
                    out[i] = level;
                }
                // solids take light but do not pass it on
                if !substrate_passable(map.substrate[i]) && i != si {
                    continue;
                }
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                for dy in -1..=1i32 {
                    for dx in -1..=1i32 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let j = ny as usize * w + nx as usize;
                        let diagonal = dx != 0 && dy != 0;
                        // light cannot cut a solid corner diagonally
                        if diagonal {
                            let a = (y as usize) * w + nx as usize;
                            let b = (ny as usize) * w + x as usize;
                            if !substrate_passable(map.substrate[a]) && !substrate_passable(map.substrate[b]) {
                                continue;
                            }
                        }
                        let step = match step_cost(map.substrate[j], diagonal) {
                            Some(s) => s,
                            None => {
                                // a solid: lands with the cost of an ordinary step, never relaxes further
                                let nc = c as u32 + if diagonal { 3 } else { 2 };
                                if nc <= max_cost && nc < best[j] {
                                    best[j] = nc;
                                    buckets[nc as usize].push((j, emit));
                                    dirty.push(j);
                                }
                                continue;
                            }
                        };
                        let nc = c as u32 + step;
                        if nc <= max_cost && nc < best[j] {
                            best[j] = nc;
                            buckets[nc as usize].push((j, emit));
                            dirty.push(j);
                        }
                    }
                }
            }
        }
        for i in dirty {
            best[i] = u32::MAX;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;

    fn map() -> RouteMap {
        let l = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        RouteMap::compile(&l, &route(&l, 3, 1).unwrap())
    }

    #[test]
    fn fixtures_make_pools_that_fade_and_stop_at_walls() {
        let mut m = map();
        m.lights.truncate(1);
        m.grates.clear();
        m.entry.clear();
        let f = LightField::new(&m, 0.5);
        let l = m.lights[0];
        assert_eq!(f.at(l.x as usize, l.y as usize), FIXTURE_EMIT);
        // one tile away is one step dimmer, and the pool ends within the emit radius
        let (x, y) = (l.x as usize, l.y as usize);
        let near = [f.at(x + 1, y), f.at(x - 1, y), f.at(x, y + 1), f.at(x, y - 1)];
        assert!(near.iter().any(|&v| v == FIXTURE_EMIT - 1), "{near:?}");
        let far = f.level.iter().enumerate().filter(|(_, &v)| v > 0).map(|(i, _)| ((i % m.w) as i32 - x as i32).abs() + ((i / m.w) as i32 - y as i32).abs()).max().unwrap();
        assert!(far <= 2 * FIXTURE_EMIT as i32 + 2, "pool reaches {far}");
        // light does not pass through rock: a rock tile two away behind a wall stays 0 unless adjacent to lit air
        let lit_rock = f.level.iter().enumerate().filter(|(i, &v)| v > 0 && m.substrate[*i] == sub::ROCK).count();
        let total_rock = m.substrate.iter().filter(|&&s| s == sub::ROCK).count();
        assert!(lit_rock < total_rock / 4, "{lit_rock} of {total_rock} rock lit");
    }

    #[test]
    fn mat_edge_glows_by_gene_and_dynamic_sources_combine() {
        let m = map();
        let mut f = LightField::new(&m, 1.0);
        let (ex, ey) = m.entry[1];
        assert_eq!(f.at(ex as usize, ey as usize), mat_emit(1.0));
        assert_eq!(mat_emit(0.0), 2);
        assert_eq!(mat_emit(1.0), 8);
        let before = f.level.clone();
        f.set_dynamic(&m, &[Source { x: ex + 6, y: ey, emit: 12 }]);
        assert_eq!(f.at(ex as usize + 6, ey as usize), 12);
        assert!(f.level.iter().zip(&before).all(|(a, b)| a >= b));
        // clearing the dynamic source restores the static field exactly
        f.set_dynamic(&m, &[]);
        assert_eq!(f.level, before);
    }

    #[test]
    fn water_costs_three_times_air() {
        let m = map();
        let (wx, wy) = (0..m.h).flat_map(|y| (0..m.w).map(move |x| (x, y))).find(|&(x, y)| m.substrate[m.idx(x, y)] == sub::WATER && x + 4 < m.w && m.substrate[m.idx(x + 1, y)] == sub::WATER && m.substrate[m.idx(x + 2, y)] == sub::WATER).unwrap();
        let mut f = LightField::new(&m, 0.0);
        f.set_dynamic(&m, &[Source { x: wx as u16, y: wy as u16, emit: 9 }]);
        // two water tiles east cost 12 -> level 9 - 6 = 3, unless a drier path around exists (then brighter)
        assert!(f.at(wx + 2, wy) <= 9 - 3, "{}", f.at(wx + 2, wy));
    }
}
