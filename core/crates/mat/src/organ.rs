//! Organs (Full Design v3.18, Progression: Organs; Pass 1's Genome): an organ grows by itself at a Node of the city's
//! strain once a gene crosses its line. The kinds are data (`data/strain.toml`, read by the progress crate); this
//! module holds where one may grow and the growth itself.

use crate::sim::{hy, is_hyphal, sub, MatSim};

/// Where an organ may grow (Full Design v3.18, the Organs table's Site column).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    /// any Node
    Any,
    /// a Node within 2 tiles of water
    Water,
    /// a Node in a passage 1-2 tiles across
    Passage,
    /// a Node 24 or more cords from the Heart and every Relay
    Far,
    /// a Node on or beside a Deposit
    Deposit,
    /// a Node with four or more hyphal tiles round it
    Junction,
    /// a Node within 6 tiles of feral tissue
    Feral,
    /// a Node on a trunk: tissue on two opposite sides
    Trunk,
}

impl Site {
    pub fn from_key(k: &str) -> Option<Site> {
        Some(match k {
            "any" => Site::Any,
            "water" => Site::Water,
            "passage" => Site::Passage,
            "far" => Site::Far,
            "deposit" => Site::Deposit,
            "junction" => Site::Junction,
            "feral" => Site::Feral,
            "trunk" => Site::Trunk,
            _ => return None,
        })
    }
}

/// An organ kind as the Mat needs it: the gene and its line, and the site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrganDef {
    pub gene: usize,
    pub line: f32,
    pub site: Site,
}

/// A grown organ: its kind (an index into the organ defs) and its tile. `rule` is the organ's edited rule (Organ
/// rules): 0 is the default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Organ {
    pub kind: u8,
    pub i: u32,
    pub rule: u8,
}

/// Cords from the Heart and every Relay a Nerve Ganglion grows past (Full Design v3.18).
pub const FAR_CORDS: f32 = 24.0;

/// Ground a slimo cannot stand in: the walls of a passage.
fn wall(s: u8) -> bool {
    matches!(s, sub::ROCK | sub::CONCRETE | sub::COLLAPSE)
}

impl MatSim {
    /// The organs of a kind the strain holds.
    pub fn organs_of(&self, kind: u8) -> impl Iterator<Item = &Organ> + '_ {
        self.organs.iter().filter(move |o| o.kind == kind)
    }

    /// Nodes of the city's strain: hyphal Nodes with signal.
    pub fn node_count(&self) -> usize {
        self.hy.iter().filter(|&&h| h == hy::NODE).count()
    }

    fn cheb_any(&self, i: usize, r: i32, f: impl Fn(usize) -> bool) -> bool {
        let (x, y) = ((i % self.w) as i32, (i / self.w) as i32);
        for dy in -r..=r {
            for dx in -r..=r {
                let (xx, yy) = (x + dx, y + dy);
                if xx >= 0 && yy >= 0 && (xx as usize) < self.w && (yy as usize) < self.h && f(yy as usize * self.w + xx as usize) {
                    return true;
                }
            }
        }
        false
    }

    fn passage(&self, i: usize) -> bool {
        let (x, y) = ((i % self.w) as i32, (i / self.w) as i32);
        for (dx, dy) in [(1i32, 0i32), (0, 1)] {
            let mut a = 0;
            let mut b = 0;
            for k in 1..=3 {
                let (xx, yy) = (x - dx * k, y - dy * k);
                if xx < 0 || yy < 0 || xx as usize >= self.w || yy as usize >= self.h || wall(self.sub[yy as usize * self.w + xx as usize]) {
                    a = k;
                    break;
                }
            }
            for k in 1..=3 {
                let (xx, yy) = (x + dx * k, y + dy * k);
                if xx < 0 || yy < 0 || xx as usize >= self.w || yy as usize >= self.h || wall(self.sub[yy as usize * self.w + xx as usize]) {
                    b = k;
                    break;
                }
            }
            // walls a and b tiles off on the two sides: the passage is a + b - 1 across
            if a > 0 && b > 0 && a + b - 1 <= 2 {
                return true;
            }
        }
        false
    }

    /// Whether a Node meets a site rule.
    pub fn site_ok(&self, i: usize, site: Site) -> bool {
        match site {
            Site::Any => true,
            Site::Water => self.cheb_any(i, 2, |j| self.sub[j] == sub::WATER),
            Site::Passage => self.passage(i),
            Site::Far => {
                let r = self.range();
                self.sig[i] > 0.0 && (1.0 - self.sig[i]) * r >= FAR_CORDS
            }
            Site::Deposit => self.cheb_any(i, 1, |j| self.sub[j] == sub::DEPOSIT),
            Site::Junction => {
                let mut c = 0;
                let (x, y) = ((i % self.w) as i32, (i / self.w) as i32);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (xx, yy) = (x + dx, y + dy);
                        if (dx, dy) != (0, 0) && xx >= 0 && yy >= 0 && (xx as usize) < self.w && (yy as usize) < self.h && is_hyphal(self.hy[yy as usize * self.w + xx as usize]) {
                            c += 1;
                        }
                    }
                }
                c >= 4
            }
            Site::Feral => self.cheb_any(i, 6, |j| self.is_feral(j)),
            Site::Trunk => {
                let (x, y) = (i % self.w, i / self.w);
                let h = |xx: usize, yy: usize| is_hyphal(self.hy[yy * self.w + xx]);
                (x > 0 && x + 1 < self.w && h(x - 1, y) && h(x + 1, y)) || (y > 0 && y + 1 < self.h && h(x, y - 1) && h(x, y + 1))
            }
        }
    }

    /// Grow the organs whose lines the genome crosses (Full Design v3.18, Organs): each grows one at the eligible
    /// Node with the most nutrient — signalled, not feral, its site met, no organ within `spacing` tiles and
    /// `covered` false (no footprint over it) — while the strain holds fewer than 1 + Nodes / `per_nodes` of it.
    /// Returns the organs grown: (kind, tile).
    pub fn grow_organs(&mut self, defs: &[OrganDef], spacing: i32, per_nodes: usize, covered: &dyn Fn(usize) -> bool) -> Vec<(u8, usize)> {
        self.prune_organs();
        let cap = 1 + self.node_count() / per_nodes.max(1);
        let mut grown = Vec::new();
        for (k, d) in defs.iter().enumerate() {
            if self.genome.0[d.gene] <= d.line || self.organs_of(k as u8).count() >= cap {
                continue;
            }
            let mut best: Option<(f32, usize)> = None;
            for i in 0..self.hy.len() {
                if self.hy[i] != hy::NODE || self.sig[i] <= 0.0 || self.is_feral(i) || covered(i) {
                    continue;
                }
                if !self.site_ok(i, d.site) {
                    continue;
                }
                let (x, y) = ((i % self.w) as i32, (i / self.w) as i32);
                let near = self.organs.iter().any(|o| {
                    let (ox, oy) = ((o.i as usize % self.w) as i32, (o.i as usize / self.w) as i32);
                    (ox - x).abs() <= spacing && (oy - y).abs() <= spacing
                });
                if near {
                    continue;
                }
                if best.is_none_or(|(n, _)| self.n[i] > n) {
                    best = Some((self.n[i], i));
                }
            }
            if let Some((_, i)) = best {
                self.organs.push(Organ { kind: k as u8, i: i as u32, rule: 0 });
                grown.push((k as u8, i));
            }
        }
        if !grown.is_empty() {
            self.refresh_organ_cache(defs);
        }
        grown
    }

    /// An organ dies with its Node. Returns the organs lost: (kind, tile).
    pub fn prune_organs(&mut self) -> Vec<(u8, usize)> {
        let mut lost = Vec::new();
        let hy = &self.hy;
        self.organs.retain(|o| {
            let ok = hy[o.i as usize] == hy::NODE;
            if !ok {
                lost.push((o.kind, o.i as usize));
            }
            ok
        });
        lost
    }

    /// The organ standing on a tile.
    pub fn organ_at(&self, i: usize) -> Option<&Organ> {
        self.organs.iter().find(|o| o.i as usize == i)
    }

    /// The caches the step reads: the Deposits a Root Spur grips, the tissue a Chrome Cyst guards, the Nerve
    /// Ganglia that are signal sources. `spur`, `cyst`, `ganglion` are the kinds' indices in the defs.
    pub fn refresh_organ_cache(&mut self, _defs: &[OrganDef]) {
        let len = self.hy.len();
        self.spur_boost = vec![false; len];
        self.cyst_guard = vec![false; len];
        self.ganglia.clear();
        let w = self.w as i32;
        let h = self.h as i32;
        for o in self.organs.clone() {
            let i = o.i as usize;
            let (x, y) = ((i % self.w) as i32, (i / self.w) as i32);
            if Some(o.kind) == self.kinds.spur {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (xx, yy) = (x + dx, y + dy);
                        if xx >= 0 && yy >= 0 && xx < w && yy < h {
                            let j = (yy * w + xx) as usize;
                            if self.sub[j] == sub::DEPOSIT {
                                self.spur_boost[j] = true;
                            }
                        }
                    }
                }
            }
            if Some(o.kind) == self.kinds.cyst {
                for dy in -4..=4 {
                    for dx in -4..=4 {
                        let (xx, yy) = (x + dx, y + dy);
                        if xx >= 0 && yy >= 0 && xx < w && yy < h {
                            self.cyst_guard[(yy * w + xx) as usize] = true;
                        }
                    }
                }
            }
            if Some(o.kind) == self.kinds.ganglion {
                self.ganglia.push((x as usize, y as usize));
            }
        }
    }

    /// A Graft Bud's work at the shift's end: feral tissue within 6 joins the city's strain. Returns the tiles taken.
    pub fn graft_buds(&mut self) -> u32 {
        let Some(g) = self.kinds.graft else { return 0 };
        let buds: Vec<usize> = self.organs_of(g).map(|o| o.i as usize).collect();
        let mut n = 0;
        for b in buds {
            let (x, y) = ((b % self.w) as i32, (b / self.w) as i32);
            for dy in -6i32..=6 {
                for dx in -6i32..=6 {
                    let (xx, yy) = (x + dx, y + dy);
                    if xx >= 0 && yy >= 0 && (xx as usize) < self.w && (yy as usize) < self.h {
                        let j = yy as usize * self.w + xx as usize;
                        if self.is_feral(j) {
                            self.ferality[j] = 0.0;
                            if self.blooms() {
                                n += 1;
                            }
                        }
                    }
                }
            }
        }
        self.reclaimed += n;
        n
    }
}

/// The organ kinds the Mat's own step reads, by their index in the defs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Kinds {
    pub spur: Option<u8>,
    pub cyst: Option<u8>,
    pub ganglion: Option<u8>,
    pub graft: Option<u8>,
}
