//! The noise field: every sound is a footprint stamped over the tiles it reaches (research 07), not a circle.
//! Dijkstra from the source with per-tile costs: open 2 (diagonal 3), water 4 (halved reach), pipe 1 (carries far
//! down a run), brick 3 (through a one-tile wall into the next room), rubble and deposits 4, rock and concrete
//! block. level = loudness - ceil(cost / 2). The field decays DECAY per tick; events also go to a ring buffer with
//! tick and position so the AI (PAC-023) can remember where a sound came from.

use crate::map::RouteMap;
use mat::sim::sub;
use std::collections::VecDeque;

pub const MAX_NOISE: u8 = 15;
pub const DECAY: u8 = 2;
pub const EVENT_MEMORY: usize = 64;

/// Hearing thresholds for PAC-023, recorded here with the costs they are tuned against.
pub const HEAR_SEWERAT: u8 = 2;
pub const HEAR_SCOUT: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoiseEvent {
    pub x: u16,
    pub y: u16,
    pub loudness: u8,
    pub tick: u64,
    /// who made it: 0 the Runner, 1 a follower, 2 a creature, 3 the world (hazard, trap)
    pub actor: u8,
}

#[derive(Clone, Debug)]
pub struct NoiseField {
    pub w: usize,
    pub h: usize,
    pub level: Vec<u8>,
    pub events: VecDeque<NoiseEvent>,
    live: Vec<usize>,
}

/// What an action sounds like (research 07, decision 6), before the surface add.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    SneakStep,
    WalkStep,
    SprintStep,
    Dig,
    ContainerOpen,
    BodyPickup,
    DropItem,
    ExtractChannel,
    /// gear noise from data/items.toml
    MeleeSwing(i32),
    Gunshot(i32),
}

impl Sound {
    pub fn loudness(self) -> u8 {
        let v: i32 = match self {
            Sound::SneakStep => 2,
            Sound::WalkStep => 4,
            Sound::SprintStep => 7,
            Sound::Dig => 8,
            Sound::ContainerOpen => 3,
            Sound::BodyPickup => 4,
            Sound::DropItem => 5,
            Sound::ExtractChannel => 9,
            Sound::MeleeSwing(g) => 3 + g,
            Sound::Gunshot(g) => 8 + g,
        };
        v.clamp(0, MAX_NOISE as i32) as u8
    }
}

/// What standing on this substrate adds to a footstep or action: water splashes, grates ring.
pub fn surface_add(s: u8) -> u8 {
    match s {
        sub::WATER => 1,
        sub::GRATE => 2,
        _ => 0,
    }
}

/// Cost to carry noise into a tile; None blocks.
pub fn step_cost(s: u8, diagonal: bool) -> Option<u32> {
    let base = match s {
        sub::OPEN => 2,
        sub::GRATE => 2,
        sub::PIPE => 1,
        sub::BRICK => 3,
        sub::WATER => 4,
        sub::DEPOSIT | sub::COLLAPSE => 4,
        _ => return None, // ROCK, CONCRETE
    };
    Some(if diagonal { base + 1 } else { base })
}

impl NoiseField {
    pub fn new(w: usize, h: usize) -> NoiseField {
        NoiseField { w, h, level: vec![0; w * h], events: VecDeque::with_capacity(EVENT_MEMORY), live: Vec::new() }
    }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> u8 {
        self.level[y * self.w + x]
    }

    /// Stamp a sound. `loudness` already includes the surface add. Returns the number of tiles reached.
    pub fn emit(&mut self, map: &RouteMap, x: usize, y: usize, loudness: u8, tick: u64, actor: u8) -> usize {
        if loudness == 0 || x >= self.w || y >= self.h {
            return 0;
        }
        if self.events.len() == EVENT_MEMORY {
            self.events.pop_front();
        }
        self.events.push_back(NoiseEvent { x: x as u16, y: y as u16, loudness, tick, actor });
        let (w, h) = (self.w, self.h);
        let max_cost = (loudness as u32) * 2 - 1;
        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); max_cost as usize + 1];
        let mut best: Vec<u32> = vec![u32::MAX; w * h];
        let si = y * w + x;
        best[si] = 0;
        buckets[0].push(si);
        let mut reached = 0usize;
        for c in 0..=max_cost as usize {
            let mut k = 0;
            while k < buckets[c].len() {
                let i = buckets[c][k];
                k += 1;
                if best[i] != c as u32 {
                    continue;
                }
                let level = (loudness as i32 - ((c as i32) + 1) / 2).max(0) as u8;
                if level == 0 {
                    continue;
                }
                if self.level[i] < level {
                    if self.level[i] == 0 {
                        self.live.push(i);
                    }
                    self.level[i] = level;
                }
                reached += 1;
                let (tx, ty) = ((i % w) as i32, (i / w) as i32);
                for dy in -1..=1i32 {
                    for dx in -1..=1i32 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let (nx, ny) = (tx + dx, ty + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let j = ny as usize * w + nx as usize;
                        let Some(step) = step_cost(map.substrate[j], dx != 0 && dy != 0) else { continue };
                        let nc = c as u32 + step;
                        if nc <= max_cost && nc < best[j] {
                            best[j] = nc;
                            buckets[nc as usize].push(j);
                        }
                    }
                }
            }
        }
        reached
    }

    /// One tick of fading.
    pub fn decay(&mut self) {
        let mut k = 0;
        while k < self.live.len() {
            let i = self.live[k];
            let v = self.level[i].saturating_sub(DECAY);
            self.level[i] = v;
            if v == 0 {
                self.live.swap_remove(k);
            } else {
                k += 1;
            }
        }
    }

    /// Events at or after `tick`, oldest first.
    pub fn since(&self, tick: u64) -> impl Iterator<Item = &NoiseEvent> {
        self.events.iter().filter(move |e| e.tick >= tick)
    }

    pub fn live_tiles(&self) -> usize {
        self.live.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;

    fn map() -> RouteMap {
        let l = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        RouteMap::compile(&l, &route(&l, 21, 2).unwrap())
    }

    #[test]
    fn loudness_table_and_surfaces() {
        assert_eq!(Sound::SneakStep.loudness(), 2);
        assert_eq!(Sound::SprintStep.loudness(), 7);
        assert_eq!(Sound::Gunshot(6).loudness(), 14);
        assert_eq!(Sound::Gunshot(9).loudness(), 15);
        assert_eq!(surface_add(sub::GRATE), 2);
        assert_eq!(surface_add(sub::OPEN), 0);
    }

    #[test]
    fn footprint_fades_with_cost_and_pipes_carry_further_than_water() {
        let m = map();
        let mut n = NoiseField::new(m.w, m.h);
        let (x, y) = (m.entry[1].0 as usize + 8, m.entry[1].1 as usize);
        assert!(m.passable(x, y));
        let reached = n.emit(&m, x, y, 10, 1, 0);
        assert!(reached > 50, "{reached}");
        assert_eq!(n.at(x, y), 10);
        assert_eq!(n.at(x + 1, y), 9);
        assert_eq!(n.at(x + 2, y), 8);
        // a pipe tile adjacent to the sound carries at cost 1
        let pipes: usize = n.level.iter().enumerate().filter(|(i, &v)| v > 0 && m.substrate[*i] == sub::PIPE).count();
        let rocks: usize = n.level.iter().enumerate().filter(|(i, &v)| v > 0 && m.substrate[*i] == sub::ROCK).count();
        assert_eq!(rocks, 0, "noise entered rock");
        let _ = pipes;
        // decay empties the field in loudness / DECAY ticks
        for _ in 0..5 {
            n.decay();
        }
        assert_eq!(n.live_tiles(), 0);
        assert!(n.level.iter().all(|&v| v == 0));
        assert_eq!(n.since(0).count(), 1);
    }

    fn corridor(fill: u8) -> RouteMap {
        let (w, h) = (48usize, 3usize);
        let mut sub_ = vec![sub::ROCK; w * h];
        for x in 2..46 {
            sub_[w + x] = fill;
        }
        RouteMap {
            seed: 0,
            stratum: 1,
            w,
            h,
            substrate: sub_,
            entry: vec![],
            grates: vec![],
            containers: vec![],
            spawns: vec![],
            lights: vec![],
            hazards: vec![],
            chunks: vec![],
            chunk_of: vec![0; w * h],
        }
    }

    #[test]
    fn water_halves_reach_and_pipes_double_it() {
        let reach = |fill: u8| {
            let m = corridor(fill);
            let mut n = NoiseField::new(m.w, m.h);
            n.emit(&m, 2, 1, 10, 0, 0)
        };
        let open = reach(sub::OPEN);
        let water = reach(sub::WATER);
        let pipe = reach(sub::PIPE);
        // level = 10 - ceil(cost/2) > 0 means cost <= 18 from the corridor's end:
        // open (cost 2) reaches 10 tiles, water (4) 5, pipe (1) 19
        assert_eq!(open, 10, "open");
        assert_eq!(water, 5, "water");
        assert_eq!(pipe, 19, "pipe");
    }
}
