//! The compiled route: one tile grid with the substrate the `mat` crate understands plus the overlays a run plays
//! with (containers, bodies, spawns, patrol points, lights, hazards, the Mat edge and the grates).

use crate::assemble::Layout;
use crate::chunk::{flood, substrate_passable, ChunkLib, Dir, CHUNK_H, CHUNK_W};
use mat::sim::sub;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerKind {
    Locker,
    Crate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Container {
    pub x: u16,
    pub y: u16,
    pub kind: ContainerKind,
    /// loot tier: the stratum for lockers, one above for crates (PAC-022 reads this)
    pub tier: u8,
    pub chunk: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnKind {
    Sewerat,
    Patrol,
    Corpse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spawn {
    pub x: u16,
    pub y: u16,
    pub kind: SpawnKind,
    pub chunk: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Light {
    pub x: u16,
    pub y: u16,
    /// pool radius in tiles
    pub radius: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HazardKind {
    Gas,
    Collapse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hazard {
    pub x: u16,
    pub y: u16,
    pub kind: HazardKind,
}

#[derive(Clone, Debug)]
pub struct PlacedChunk {
    pub id: String,
    pub kind: String,
    pub stratum: u8,
    /// tile origin in the route map
    pub x0: u16,
    pub y0: u16,
}

#[derive(Clone, Debug)]
pub struct RouteMap {
    pub seed: u64,
    pub stratum: u8,
    pub w: usize,
    pub h: usize,
    /// substrate codes (mat::sim::sub), row-major
    pub substrate: Vec<u8>,
    /// the Mat edge: cells of the entry chunk's west window; the Runner starts here and may extract here
    pub entry: Vec<(u16, u16)>,
    /// grate tiles: extraction for a toll, or free when the grate is held
    pub grates: Vec<(u16, u16)>,
    pub containers: Vec<Container>,
    pub spawns: Vec<Spawn>,
    pub lights: Vec<Light>,
    pub hazards: Vec<Hazard>,
    pub chunks: Vec<PlacedChunk>,
    /// chunk index per tile (u16::MAX where no chunk was placed)
    pub chunk_of: Vec<u16>,
}

pub const LIGHT_RADIUS: u8 = 4;
pub const NO_CHUNK: u16 = u16::MAX;

impl RouteMap {
    pub fn compile(lib: &ChunkLib, lay: &Layout) -> RouteMap {
        let cols = lay.cols();
        let rows = lay.rows();
        let (w, h) = (cols * CHUNK_W, rows * CHUNK_H);
        let mut m = RouteMap {
            seed: lay.seed,
            stratum: lay.stratum,
            w,
            h,
            substrate: vec![sub::ROCK; w * h],
            entry: Vec::new(),
            grates: Vec::new(),
            containers: Vec::new(),
            spawns: Vec::new(),
            lights: Vec::new(),
            hazards: Vec::new(),
            chunks: Vec::with_capacity(lay.placed.len()),
            chunk_of: vec![NO_CHUNK; w * h],
        };
        for (pi, p) in lay.placed.iter().enumerate() {
            let c = &lib.chunks[p.chunk];
            let x0 = ((p.cx - lay.min.0) as usize) * CHUNK_W;
            let y0 = ((p.cy - lay.min.1) as usize) * CHUNK_H;
            m.chunks.push(PlacedChunk { id: c.id.clone(), kind: c.kind.clone(), stratum: c.stratum, x0: x0 as u16, y0: y0 as u16 });
            for y in 0..CHUNK_H {
                for x in 0..CHUNK_W {
                    let g = c.at(x, y);
                    let (gx, gy) = (x0 + x, y0 + y);
                    let i = gy * w + gx;
                    m.chunk_of[i] = pi as u16;
                    let mut s = crate::chunk::glyph_substrate(g);
                    match g {
                        b'<' | b'>' | b'^' | b'v' => {
                            let d = match g {
                                b'<' => Dir::W,
                                b'>' => Dir::E,
                                b'^' => Dir::N,
                                _ => Dir::S,
                            };
                            // joined exits open onto the neighbour; the rest are bricked up
                            s = if p.joined & d.bit() != 0 { sub::OPEN } else { sub::BRICK };
                        }
                        b'@' => {
                            s = sub::OPEN;
                            m.entry.push((gx as u16, gy as u16));
                        }
                        b'G' => m.grates.push((gx as u16, gy as u16)),
                        b'L' => m.containers.push(Container { x: gx as u16, y: gy as u16, kind: ContainerKind::Locker, tier: c.stratum, chunk: pi as u16 }),
                        b'K' => m.containers.push(Container { x: gx as u16, y: gy as u16, kind: ContainerKind::Crate, tier: (c.stratum + 1).min(6), chunk: pi as u16 }),
                        b'R' => m.spawns.push(Spawn { x: gx as u16, y: gy as u16, kind: SpawnKind::Corpse, chunk: pi as u16 }),
                        b'S' => m.spawns.push(Spawn { x: gx as u16, y: gy as u16, kind: SpawnKind::Sewerat, chunk: pi as u16 }),
                        b'O' => m.spawns.push(Spawn { x: gx as u16, y: gy as u16, kind: SpawnKind::Patrol, chunk: pi as u16 }),
                        b'*' => m.lights.push(Light { x: gx as u16, y: gy as u16, radius: LIGHT_RADIUS }),
                        b'!' => m.hazards.push(Hazard { x: gx as u16, y: gy as u16, kind: HazardKind::Gas }),
                        b'X' => m.hazards.push(Hazard { x: gx as u16, y: gy as u16, kind: HazardKind::Collapse }),
                        _ => {}
                    }
                    m.substrate[i] = s;
                }
            }
        }
        m
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    #[inline]
    pub fn passable(&self, x: usize, y: usize) -> bool {
        x < self.w && y < self.h && substrate_passable(self.substrate[self.idx(x, y)])
    }

    pub fn passable_mask(&self) -> Vec<bool> {
        self.substrate.iter().map(|&s| substrate_passable(s)).collect()
    }

    /// Tiles reachable from the Mat edge without digging.
    pub fn reachable(&self) -> Vec<bool> {
        let pass = self.passable_mask();
        let start = self.entry.first().copied().unwrap_or((0, 0));
        flood(&pass, self.w, self.h, (start.0 as usize, start.1 as usize))
    }

    /// Fraction of passable tiles reachable from the Mat edge without digging.
    pub fn connectivity(&self) -> f32 {
        let pass = self.passable_mask();
        let total = pass.iter().filter(|&&p| p).count().max(1);
        let reach = self.reachable().iter().filter(|&&r| r).count();
        reach as f32 / total as f32
    }

    /// Every tile a run can extract from: the Mat edge and the grates.
    pub fn extraction_tiles(&self) -> Vec<(u16, u16)> {
        let mut v = self.entry.clone();
        v.extend(self.grates.iter().copied());
        v
    }

    /// Dig a collapse or deposit tile open (PAC-022/023 call this through the dig action).
    pub fn dig(&mut self, x: usize, y: usize) -> bool {
        let i = self.idx(x, y);
        if matches!(self.substrate[i], sub::COLLAPSE | sub::DEPOSIT) {
            self.substrate[i] = sub::OPEN;
            self.hazards.retain(|h| !(h.kind == HazardKind::Collapse && h.x as usize == x && h.y as usize == y));
            true
        } else {
            false
        }
    }

    /// Counts for the route card and the HUD.
    pub fn digest(&self) -> String {
        format!(
            "s{} seed {} {}x{} tiles {} chunks | {} containers {} bodies {} spawns {} patrols {} lights {} gas {} collapse | {} grates | reach {:.1}%",
            self.stratum,
            self.seed,
            self.w,
            self.h,
            self.chunks.len(),
            self.containers.len(),
            self.spawns.iter().filter(|s| s.kind == SpawnKind::Corpse).count(),
            self.spawns.iter().filter(|s| s.kind == SpawnKind::Sewerat).count(),
            self.spawns.iter().filter(|s| s.kind == SpawnKind::Patrol).count(),
            self.lights.len(),
            self.hazards.iter().filter(|h| h.kind == HazardKind::Gas).count(),
            self.hazards.iter().filter(|h| h.kind == HazardKind::Collapse).count(),
            self.grates.len(),
            self.connectivity() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;

    fn lib() -> ChunkLib {
        ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap()
    }

    #[test]
    fn compiled_routes_are_connected_and_extractable() {
        let l = lib();
        for stratum in 1..=3u8 {
            for seed in 0..150u64 {
                let lay = route(&l, seed, stratum).unwrap();
                let m = RouteMap::compile(&l, &lay);
                assert_eq!(m.chunks.len(), lay.placed.len());
                assert_eq!(m.entry.len(), 3, "seed {seed}: entry window");
                assert!(m.entry.iter().all(|&(x, y)| m.passable(x as usize, y as usize)));
                assert!(!m.extraction_tiles().is_empty());
                let conn = m.connectivity();
                assert!(conn >= 0.95, "seed {seed} s{stratum}: connectivity {conn:.3}\n{}", m.digest());
                // with every collapse dug, nothing passable is left out
                let mut dug = m.clone();
                for hz in m.hazards.iter().filter(|h| h.kind == HazardKind::Collapse) {
                    dug.dig(hz.x as usize, hz.y as usize);
                }
                assert!(dug.connectivity() > 0.9999, "seed {seed} s{stratum}: sealed pocket even after digging");
                // every placed chunk is reachable from the entry
                let reach = m.reachable();
                for (ci, c) in m.chunks.iter().enumerate() {
                    let any = (0..CHUNK_H).any(|y| (0..CHUNK_W).any(|x| reach[m.idx(c.x0 as usize + x, c.y0 as usize + y)]));
                    assert!(any, "seed {seed} s{stratum}: chunk {ci} {} unreachable", c.id);
                }
            }
        }
    }

    #[test]
    fn sealed_exits_are_brick_and_joined_exits_are_open() {
        let l = lib();
        let lay = route(&l, 5, 1).unwrap();
        let m = RouteMap::compile(&l, &lay);
        for (pi, p) in lay.placed.iter().enumerate() {
            let c = lay.chunk(&l, pi);
            for d in c.exit_dirs() {
                let joined = p.joined & d.bit() != 0;
                for (x, y) in d.exit_cells() {
                    let gx = m.chunks[pi].x0 as usize + x;
                    let gy = m.chunks[pi].y0 as usize + y;
                    let s = m.substrate[m.idx(gx, gy)];
                    assert_eq!(s, if joined { sub::OPEN } else { sub::BRICK }, "{} {d:?}", c.id);
                }
            }
        }
    }

    #[test]
    fn overlays_land_on_open_tiles_with_tiers() {
        let l = lib();
        let lay = route(&l, 9, 2).unwrap();
        let m = RouteMap::compile(&l, &lay);
        assert!(!m.containers.is_empty() && !m.lights.is_empty());
        for c in &m.containers {
            assert!(m.passable(c.x as usize, c.y as usize));
            match c.kind {
                ContainerKind::Locker => assert_eq!(c.tier, 2),
                ContainerKind::Crate => assert_eq!(c.tier, 3),
            }
        }
        for s in &m.spawns {
            assert!(m.passable(s.x as usize, s.y as usize));
        }
        for hz in &m.hazards {
            match hz.kind {
                HazardKind::Gas => assert!(m.passable(hz.x as usize, hz.y as usize)),
                HazardKind::Collapse => assert_eq!(m.substrate[m.idx(hz.x as usize, hz.y as usize)], sub::COLLAPSE),
            }
        }
    }

    #[test]
    fn digging_opens_collapse_and_deposit() {
        let l = lib();
        let mut found = None;
        for seed in 0..100u64 {
            let lay = route(&l, seed, 1).unwrap();
            let m = RouteMap::compile(&l, &lay);
            if let Some(h) = m.hazards.iter().find(|h| h.kind == HazardKind::Collapse).copied() {
                found = Some((m, h));
                break;
            }
        }
        let (mut m, h) = found.expect("some S1 route has a collapse");
        let before = m.hazards.len();
        assert!(m.dig(h.x as usize, h.y as usize));
        assert!(m.passable(h.x as usize, h.y as usize));
        assert_eq!(m.hazards.len(), before - 1);
        assert!(!m.dig(h.x as usize, h.y as usize));
    }
}
