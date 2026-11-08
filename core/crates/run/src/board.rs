//! The route card: what can be known about a route before a run, computed from the compiled map, the creature
//! file and the route's memory bits (Full Design v3.3, The Route Board). The Board draws this and counts nothing.

use crate::creature::CreatureFile;
use crate::map::RouteMap;
use mat::sim::sub;

/// The danger index: expected trouble from the authored spawns. Each burrow brings the mean pack of the stratum's
/// mix, each creature weighted by `[spawn].threat` (crawler 1, burster 1.5, maw 3, scout 3); each chalk-marked
/// chunk brings two scouts at the stratum's patrol chance. The band is the index per chunk, because the shift
/// length already tells the player the route is long: LOW < 1.5, MEDIUM < 2.5, HIGH ≥ 2.5. Measured over 60 seeds
/// per stratum at launch: S1 ≈ 1.2 per chunk (LOW), S3 ≈ 2.1 (MEDIUM), S2 ≈ 2.4 (MEDIUM to HIGH: the culvert
/// chunks carry the most chalk).
pub const DENSITY_MEDIUM: f32 = 1.5;
pub const DENSITY_HIGH: f32 = 2.5;

#[derive(Clone, Debug, PartialEq)]
pub struct RouteCard {
    pub chunks: usize,
    pub shift_ticks: u64,
    /// the danger index for the whole route
    pub danger: f32,
    /// the index per chunk, which the band reads
    pub density: f32,
    pub burrows: usize,
    pub patrol_chunks: usize,
    /// remembered tiles / passable tiles, 0..1
    pub known: f32,
    /// containers whose tile is remembered: (lockers, crates)
    pub known_lockers: usize,
    pub known_crates: usize,
    /// water tiles / passable tiles, 0..1
    pub water: f32,
    pub grates: usize,
}

impl RouteCard {
    pub fn danger_band(&self) -> &'static str {
        if self.density >= DENSITY_HIGH {
            "high"
        } else if self.density >= DENSITY_MEDIUM {
            "medium"
        } else {
            "low"
        }
    }

    pub fn shift_seconds(&self) -> u64 {
        self.shift_ticks / crate::creature::TICK_HZ as u64
    }
}

fn remembered(bits: Option<&[u8]>, n: usize, i: usize) -> bool {
    match bits {
        Some(b) if b.len() == n.div_ceil(8) => b[i / 8] & (1 << (i % 8)) != 0,
        _ => false,
    }
}

/// The card for a compiled route; `memory` is the route's memory bits from the save (None before any run).
pub fn card(map: &RouteMap, defs: &CreatureFile, memory: Option<&[u8]>) -> RouteCard {
    let n = map.w * map.h;
    let passable = map.passable_mask();
    let total = passable.iter().filter(|&&p| p).count().max(1);
    let mut seen = 0usize;
    let mut water = 0usize;
    for i in 0..n {
        if !passable[i] {
            continue;
        }
        if remembered(memory, n, i) {
            seen += 1;
        }
        if map.substrate[i] == sub::WATER {
            water += 1;
        }
    }
    let burrows = map.spawns.iter().filter(|s| s.kind == crate::map::SpawnKind::Sewerat).count();
    let mut patrol_chunks: Vec<u16> = map.spawns.iter().filter(|s| s.kind == crate::map::SpawnKind::Patrol).map(|s| s.chunk).collect();
    patrol_chunks.sort_unstable();
    patrol_chunks.dedup();
    let pack_mean = (defs.spawn.pack[0] as f32 + defs.spawn.pack[1] as f32) / 2.0;
    // a burrow's creature, weighted by the stratum's mix (maws only come up in nest and vault chunks, which the
    // mix already prices low)
    let mix = defs.spawn.burrow(map.stratum);
    let mix_total: f32 = mix.iter().sum::<u32>().max(1) as f32;
    let per_creature = (mix[0] as f32 * defs.spawn.threat[0] + mix[1] as f32 * defs.spawn.threat[1] + mix[2] as f32 * defs.spawn.threat[2]) / mix_total;
    let danger = burrows as f32 * pack_mean * per_creature + patrol_chunks.len() as f32 * defs.spawn.patrol(map.stratum) * 2.0 * defs.spawn.threat[3];
    let density = danger / map.chunks.len().max(1) as f32;
    // grates as groups of adjacent grate tiles, not tiles
    let mut grate_groups = 0usize;
    let mut seen_g = vec![false; map.grates.len()];
    for i in 0..map.grates.len() {
        if seen_g[i] {
            continue;
        }
        grate_groups += 1;
        let mut stack = vec![i];
        seen_g[i] = true;
        while let Some(k) = stack.pop() {
            let (x, y) = map.grates[k];
            for (j, &(ox, oy)) in map.grates.iter().enumerate() {
                if !seen_g[j] && (ox as i32 - x as i32).abs() + (oy as i32 - y as i32).abs() == 1 {
                    seen_g[j] = true;
                    stack.push(j);
                }
            }
        }
    }
    let mut known_lockers = 0;
    let mut known_crates = 0;
    for c in &map.containers {
        if remembered(memory, n, c.y as usize * map.w + c.x as usize) {
            match c.kind {
                crate::map::ContainerKind::Locker => known_lockers += 1,
                crate::map::ContainerKind::Crate => known_crates += 1,
            }
        }
    }
    RouteCard {
        chunks: map.chunks.len(),
        shift_ticks: defs.shift.shift_ticks(map.chunks.len()),
        danger,
        density,
        burrows,
        patrol_chunks: patrol_chunks.len(),
        known: seen as f32 / total as f32,
        known_lockers,
        known_crates,
        water: water as f32 / total as f32,
        grates: grate_groups,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;

    fn map(seed: u64, stratum: u8) -> RouteMap {
        let lib = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        RouteMap::compile(&lib, &route(&lib, seed, stratum).unwrap())
    }

    #[test]
    fn a_fresh_card_knows_the_route_but_not_its_contents() {
        let defs = CreatureFile::shipped();
        let m = map(11, 1);
        let c = card(&m, &defs, None);
        assert_eq!(c.chunks, m.chunks.len());
        assert_eq!(c.known, 0.0);
        assert_eq!((c.known_lockers, c.known_crates), (0, 0));
        assert!(c.shift_seconds() >= 480 && c.shift_seconds() <= 900, "shift {}", c.shift_seconds());
        assert_eq!(c.shift_seconds(), 480 + 70 * (m.chunks.len() as u64).saturating_sub(8));
    }

    #[test]
    fn memory_reveals_containers_and_deeper_strata_are_more_dangerous() {
        let defs = CreatureFile::shipped();
        let m = map(11, 1);
        // remember every tile: every container is known, the route is fully known
        let n = m.w * m.h;
        let bits = vec![0xFFu8; n.div_ceil(8)];
        let c = card(&m, &defs, Some(&bits));
        assert!((c.known - 1.0).abs() < 1e-6);
        assert_eq!(c.known_lockers + c.known_crates, m.containers.len());
        // S1 has no patrols and the lightest mix; a deeper route carries more trouble per chunk
        let m3 = map(11, 3);
        let c3 = card(&m3, &defs, None);
        assert!(c3.density > c.density, "S3 {} vs S1 {} per chunk", c3.density, c.density);
        assert_eq!(c.danger_band(), "low");
        assert_ne!(c3.danger_band(), "low");
    }
}
