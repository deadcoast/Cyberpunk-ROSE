//! The Mat: a stochastic cellular automaton coupled to a continuous nutrient field.
//! Rule order and parameters follow the GDD (Mat spec) and the Python prototype that validated them.

use crate::genome::{Genome, Params, DECAY};
use crate::rng::Pcg32;
use std::collections::{HashMap, VecDeque};

// ---------------------------------------------------------------- substrate
pub mod sub {
    pub const ROCK: u8 = 0;
    pub const OPEN: u8 = 1;
    pub const PIPE: u8 = 2;
    pub const BRICK: u8 = 3;
    pub const DEPOSIT: u8 = 4;
    pub const WATER: u8 = 5;
    pub const GRATE: u8 = 6;
    pub const CONCRETE: u8 = 7;
    /// rubble across a run chunk: dug to pass; the Mat cannot enter it until it is dug
    pub const COLLAPSE: u8 = 8;
    /// where a shaft crosses from one stratum into the next: sealed until a breach opens it (Full Design v3.19, The
    /// stack and the breach); nothing grows into it or walks it
    pub const SLAB: u8 = 9;
    pub const COUNT: usize = 10;
    /// nutrient conductance per substrate
    pub const COND: [f32; COUNT] = [0.0, 0.40, 0.60, 0.08, 0.40, 0.25, 0.40, 0.04, 0.0, 0.0];
    /// hyphae may enter
    pub const GROWABLE: [bool; COUNT] = [false, true, true, true, true, true, true, true, false, false];
}

// ---------------------------------------------------------------- hyphal states
pub mod hy {
    pub const EMPTY: u8 = 0;
    pub const TIP: u8 = 1;
    pub const CORD: u8 = 2;
    pub const NODE: u8 = 3;
    pub const NECRO: u8 = 4;
}

#[inline]
pub fn is_hyphal(h: u8) -> bool {
    h == hy::TIP || h == hy::CORD || h == hy::NODE
}

const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub tick: u64,
    pub tips: u32,
    pub cords: u32,
    pub nodes: u32,
    pub necro: u32,
    pub nutrient: f32,
    pub reach: u32,
}

/// What the tree and the classes change of the Mat's step (Full Design v3.18, Progression): the game sets them each
/// shift from what stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mods {
    /// the signal range's multiplier (the Hive conduit: 1.5 with Chrome Affinity at 0.6 or more)
    pub range: f32,
    /// the upkeep's (deep tissue: 0.75)
    pub maint: f32,
    /// a Pit's release (the deep pump: 1.5)
    pub release: f32,
    /// Runner class: tips extend two tiles a step
    pub runner: bool,
    /// Hive class: tissue holds twice its armour
    pub hive: bool,
    /// Strain grafting: the city's signal runs into a Bloom's tissue and takes it back
    pub grafting: bool,
}

impl Default for Mods {
    fn default() -> Self {
        Mods { range: 1.0, maint: 1.0, release: 1.0, runner: false, hive: false, grafting: false }
    }
}

/// One stratum (or a stack of them) of the Mat.
#[derive(Clone)]
pub struct MatSim {
    pub w: usize,
    pub h: usize,
    /// rows a stratum (48 in the stack, h for a single stratum) and how many strata the grid holds, the Sump the
    /// lowest (Full Design v3.19, The stack and the breach)
    pub strata_rows: usize,
    pub strata: u8,
    pub genome: Genome,
    pub p: Params,
    pub rng: Pcg32,
    pub sub: Vec<u8>,
    pub n: Vec<f32>,
    pub hy: Vec<u8>,
    pub salt: Vec<bool>,
    age: Vec<i16>,
    stall: Vec<i16>,
    starve: Vec<i16>,
    lonely: Vec<i16>,
    pub sig: Vec<f32>,
    /// damage a hyphal tile has taken against its armour, 1 + 9 x Chrome Affinity (Pass 1; Full Design, Pressure)
    pub wear: Vec<f32>,
    /// lit or bleached this step: it does not mend
    hurt: Vec<bool>,
    /// 0..1, gained each step without signal (Pass 1's Feral Drive); at 1 the tile is feral
    pub ferality: Vec<f32>,
    pub heart: Option<(usize, usize)>,
    /// the Heart burned to nothing: it is no longer a signal source
    pub heart_lost: bool,
    pub relays: Vec<(usize, usize)>,
    pub pits: HashMap<(usize, usize), f32>,
    /// a tended Pit releases x (1 + Shepherd / 10), x (1 + Shepherd / 5) with Shepherd training: by its cell, set
    /// from the shift's plan (Full Design: The 25 structures, the Pit)
    pub pit_boost: HashMap<(usize, usize), f32>,
    /// the organs grown (Full Design v3.18, Organs)
    pub organs: Vec<crate::organ::Organ>,
    /// which organ kinds the step reads (by their index in the defs)
    pub kinds: crate::organ::Kinds,
    /// what the tree, the classes and the organs change of the step (set by the game each shift)
    pub mods: Mods,
    /// Deposits a Root Spur grips, tissue a Chrome Cyst guards (the organ caches)
    pub spur_boost: Vec<bool>,
    pub cyst_guard: Vec<bool>,
    /// Nerve Ganglia: signal sources the Mat grew
    pub ganglia: Vec<(usize, usize)>,
    /// feral tiles taken back into the city's strain since the counter was read (a merge shock)
    pub reclaimed: u32,
    pub tick: u64,
    /// cell indices whose hyphal state changed this tick (for the renderer)
    pub changed: Vec<u32>,
    // constants
    pub d: f32,
    pub decay: f32,
    pub uptake: f32,
    pub pit_rate: f32,
    pub grow_cost: f32,
    pub tip_starter: f32,
    // scratch
    release: Vec<f32>,
    scratch: Vec<f32>,
    scratch2: Vec<f32>,
    cost: Vec<f32>,
}

impl MatSim {
    pub fn new(w: usize, h: usize, seed: u64, genome: Genome) -> Self {
        let mut rng = Pcg32::new(seed);
        let sub = worldgen(w, h, &mut rng);
        Self::on_ground(w, h, h, sub, rng, genome)
    }

    /// The stack (Full Design v3.19): one stratum of `rows` for each of `gens`, the first the Sump at the bottom, each
    /// ceiling's shafts sealed by a slab where they run on into the stratum above.
    pub fn stack(w: usize, rows: usize, gens: &[StratumGen], seed: u64, genome: Genome) -> Self {
        let mut rng = Pcg32::new(seed);
        let sub = worldgen_stack(w, rows, gens, &mut rng);
        Self::on_ground(w, rows * gens.len().max(1), rows, sub, rng, genome)
    }

    fn on_ground(w: usize, h: usize, strata_rows: usize, sub: Vec<u8>, rng: Pcg32, genome: Genome) -> Self {
        let n = w * h;
        MatSim {
            w,
            h,
            strata_rows,
            strata: (h / strata_rows.max(1)).max(1) as u8,
            genome,
            p: genome.params(),
            rng,
            sub,
            n: vec![0.0; n],
            hy: vec![hy::EMPTY; n],
            salt: vec![false; n],
            age: vec![0; n],
            stall: vec![0; n],
            starve: vec![0; n],
            lonely: vec![0; n],
            sig: vec![0.0; n],
            wear: vec![0.0; n],
            hurt: vec![false; n],
            ferality: vec![0.0; n],
            heart: None,
            heart_lost: false,
            relays: Vec::new(),
            pits: HashMap::new(),
            pit_boost: HashMap::new(),
            organs: Vec::new(),
            kinds: crate::organ::Kinds::default(),
            mods: Mods::default(),
            spur_boost: vec![false; n],
            cyst_guard: vec![false; n],
            ganglia: Vec::new(),
            reclaimed: 0,
            tick: 0,
            changed: Vec::new(),
            d: 0.11,
            decay: DECAY,
            uptake: 0.25,
            pit_rate: 0.30,
            grow_cost: 0.04,
            tip_starter: 0.12,
            release: vec![0.0; n],
            scratch: vec![0.0; n],
            scratch2: vec![0.0; n],
            cost: vec![1.0; n],
        }
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    /// The stratum a row lies in, 1 the Sump at the bottom.
    #[inline]
    pub fn stratum_of_row(&self, y: usize) -> u8 {
        self.strata.saturating_sub((y / self.strata_rows.max(1)) as u8).max(1)
    }

    /// The stratum a tile lies in.
    #[inline]
    pub fn stratum_at(&self, i: usize) -> u8 {
        self.stratum_of_row(i / self.w)
    }

    /// A stratum's ceiling: its top row, where its grates are.
    pub fn ceiling(&self, stratum: u8) -> usize {
        (self.strata.saturating_sub(stratum.max(1))) as usize * self.strata_rows
    }

    /// Every grate of the stack as a tile, the Sump's first, west to east within a stratum.
    pub fn grates(&self) -> Vec<usize> {
        (1..=self.strata).flat_map(|s| self.grates_of(s)).collect()
    }

    /// A stratum's grates as tiles, west to east.
    pub fn grates_of(&self, stratum: u8) -> Vec<usize> {
        let y = self.ceiling(stratum);
        (0..self.w).map(|x| y * self.w + x).filter(|&i| self.sub[i] == sub::GRATE).collect()
    }

    /// The crossings out of a stratum's ceiling into the one above, each as its left tile (a slab of two while sealed,
    /// open floor once breached): one beside each grate.
    pub fn crossings_of(&self, stratum: u8) -> Vec<usize> {
        if stratum >= self.strata {
            return Vec::new();
        }
        let y = self.ceiling(stratum) - 1;
        self.grates_of(stratum).into_iter().map(|g| y * self.w + g % self.w).collect()
    }

    /// Whether a crossing is still sealed.
    pub fn sealed(&self, crossing: usize) -> bool {
        self.sub[crossing] == sub::SLAB || self.sub.get(crossing + 1) == Some(&sub::SLAB)
    }

    /// Open a crossing's slab: the breach (Full Design v3.19). Returns whether anything opened.
    pub fn open_slab(&mut self, crossing: usize) -> bool {
        let mut opened = false;
        for i in [crossing, crossing + 1] {
            if self.sub.get(i) == Some(&sub::SLAB) {
                self.sub[i] = sub::OPEN;
                self.changed.push(i as u32);
                opened = true;
            }
        }
        opened
    }

    #[inline]
    fn nb(&self, i: usize, d: usize) -> Option<usize> {
        let (dx, dy) = DIRS[d];
        let x = (i % self.w) as i32 + dx;
        let y = (i / self.w) as i32 + dy;
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            None
        } else {
            Some(y as usize * self.w + x as usize)
        }
    }

    pub fn is_hyphal_at(&self, i: usize) -> bool {
        is_hyphal(self.hy[i])
    }

    fn hyphal_neighbours(&self, i: usize) -> u8 {
        let mut c = 0;
        for d in 0..4 {
            if let Some(j) = self.nb(i, d) {
                if is_hyphal(self.hy[j]) {
                    c += 1;
                }
            }
        }
        c
    }

    fn set_hy(&mut self, i: usize, s: u8) {
        if self.hy[i] != s {
            self.hy[i] = s;
            self.changed.push(i as u32);
            if !is_hyphal(s) {
                self.wear[i] = 0.0;
                self.ferality[i] = 0.0;
            }
        }
    }

    /// Whether a tile is feral tissue (Full Design, Pressure: Blooms).
    #[inline]
    pub fn is_feral(&self, i: usize) -> bool {
        is_hyphal(self.hy[i]) && self.ferality[i] >= 1.0
    }

    /// Feral tissue of this strain is a Bloom: Feral Drive above 0.5 (Pass 1).
    pub fn blooms(&self) -> bool {
        self.genome.0[crate::genome::FERAL] > 0.5
    }

    /// A Bloom's tissue: feral, of a strain that blooms. It is another strain to the city's: the city's signal stops
    /// at it and the two fight where they touch. Feral tissue of a strain that does not bloom is the city's again the
    /// moment its signal reaches it.
    #[inline]
    pub fn is_bloom(&self, i: usize) -> bool {
        self.is_feral(i) && self.blooms()
    }

    /// A hyphal tile's armour (Pass 1: cord armour HP = 1 + 9 x Chrome Affinity).
    pub fn armour(&self) -> f32 {
        1.0 + 9.0 * self.genome.0[crate::genome::CHROME]
    }

    /// A tile's armour with what guards it: twice in a Hive strain, twice again within a Chrome Cyst's reach
    /// (Full Design v3.18).
    pub fn armour_at(&self, i: usize) -> f32 {
        let mut a = self.armour();
        if self.mods.hive {
            a *= 2.0;
        }
        if self.cyst_guard.get(i).copied().unwrap_or(false) {
            a *= 2.0;
        }
        a
    }

    /// The signal range in cord steps: the strain's (24 + 32 x Conductance) and the conduit's.
    pub fn range(&self) -> f32 {
        self.p.heart_range * self.mods.range
    }

    /// White light, bleach underfoot or flame on a tile: `dmg` against its armour; at none left it dies to necrosis.
    /// The Heart and the Relays never die this way. True when the tile died.
    pub fn burn(&mut self, i: usize, dmg: f32) -> bool {
        if !is_hyphal(self.hy[i]) || dmg <= 0.0 {
            return false;
        }
        self.hurt[i] = true;
        let (x, y) = (i % self.w, i / self.w);
        if Some((x, y)) == self.heart || self.relays.contains(&(x, y)) {
            return false;
        }
        self.wear[i] += dmg;
        if self.wear[i] >= self.armour_at(i) {
            self.set_hy(i, hy::NECRO);
            self.age[i] = 0;
            self.starve[i] = 0;
            return true;
        }
        false
    }

    /// Bleach on a tile: its nutrient to nothing.
    pub fn bleach(&mut self, i: usize) {
        if sub::COND[self.sub[i] as usize] > 0.0 {
            self.n[i] = 0.0;
            self.hurt[i] = true;
        }
    }

    // ------------------------------------------------------------ public actions
    pub fn seed_heart(&mut self, x: usize, y: usize) {
        let i = self.idx(x, y);
        self.sub[i] = sub::OPEN;
        self.set_hy(i, hy::NODE);
        self.heart = Some((x, y));
        for d in 0..4 {
            if let Some(j) = self.nb(i, d) {
                if sub::GROWABLE[self.sub[j] as usize] && self.hy[j] == hy::EMPTY {
                    self.set_hy(j, hy::TIP);
                }
            }
        }
        self.splash(x, y, 0.6, true);
    }

    /// A built nerve: a Node that is a signal source and never starves.
    pub fn place_relay(&mut self, x: usize, y: usize) {
        let i = self.idx(x, y);
        self.sub[i] = sub::OPEN;
        self.set_hy(i, hy::NODE);
        for d in 0..4 {
            if let Some(j) = self.nb(i, d) {
                if sub::GROWABLE[self.sub[j] as usize] && self.hy[j] == hy::EMPTY {
                    self.set_hy(j, hy::TIP);
                }
            }
        }
        self.splash(x, y, 0.5, true);
        self.relays.push((x, y));
    }

    fn splash(&mut self, x: usize, y: usize, v: f32, max: bool) {
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let xx = x as i32 + dx;
                let yy = y as i32 + dy;
                if xx >= 0 && yy >= 0 && (xx as usize) < self.w && (yy as usize) < self.h {
                    let i = yy as usize * self.w + xx as usize;
                    if sub::COND[self.sub[i] as usize] > 0.0 {
                        self.n[i] = if max { self.n[i].max(v) } else { (self.n[i] + v).min(1.0) };
                    }
                }
            }
        }
    }

    /// Drop scrap into a Pit. It sits there as stock; the Mat draws it down at its own intake rate.
    pub fn feed(&mut self, x: usize, y: usize, amount: f32) {
        *self.pits.entry((x, y)).or_insert(0.0) += amount;
    }

    /// Fed Chrome drifts the genome: each gene moves by the delivered bias × tier drift and reflects at 0 and 1
    /// (Pass 1: a feed that would carry a gene to 1.08 lands it at 0.92), and the CA parameters follow the new genome
    /// at once. Returns the genes that reflected: (gene, off the ceiling).
    pub fn drift_genome(&mut self, drift: [f32; 10]) -> Vec<(usize, bool)> {
        let mut bounced = Vec::new();
        for (g, d) in drift.iter().enumerate() {
            let mut v = self.genome.0[g] + d;
            if v > 1.0 {
                v = 2.0 - v;
                bounced.push((g, true));
            } else if v < 0.0 {
                v = -v;
                bounced.push((g, false));
            }
            self.genome.0[g] = v.clamp(0.0, 1.0);
        }
        self.p = self.genome.params();
        bounced
    }

    /// Cells of growth one unit of nutrient buys (the Stash's "feed now"): a sprout spends grow_cost + tip_starter
    /// (0.16) from the parent cell, so one nutrient is about six cells; θ_grow is the threshold to start, not a cost.
    pub fn cells_per_nutrient(&self) -> f32 {
        1.0 / (self.grow_cost + self.tip_starter).max(0.01)
    }

    /// The handoff lays a tile of its bypass (Full Design v3.18): a tip on growable empty ground, paid from the
    /// Heart's tile at the growth cost. True when it was laid.
    pub fn lay(&mut self, i: usize) -> bool {
        let Some((hx, hy)) = self.heart else { return false };
        let h = self.idx(hx, hy);
        let cost = self.grow_cost + self.tip_starter;
        if self.hy[i] != hy::EMPTY || self.salt[i] || !sub::GROWABLE[self.sub[i] as usize] || self.n[h] < cost {
            return false;
        }
        self.n[h] -= cost;
        self.set_hy(i, hy::TIP);
        self.age[i] = 0;
        self.stall[i] = 0;
        self.n[i] = (self.n[i] + self.tip_starter).min(1.0);
        true
    }

    /// Bait Smear: paint nutrient straight onto the ground.
    pub fn bait(&mut self, x: usize, y: usize, amount: f32) {
        self.splash(x, y, amount, false);
    }

    pub fn sever(&mut self, x: usize, y: usize) {
        let i = self.idx(x, y);
        if self.hy[i] != hy::EMPTY {
            self.set_hy(i, hy::EMPTY);
            self.n[i] *= 0.5;
        }
    }

    /// Salt Line: a tile the Mat will not enter.
    pub fn salt_tile(&mut self, x: usize, y: usize, on: bool) {
        let i = self.idx(x, y);
        self.salt[i] = on;
        if on && self.hy[i] != hy::EMPTY {
            self.set_hy(i, hy::EMPTY);
        }
    }

    // ------------------------------------------------------------ signal
    pub fn compute_signal(&mut self) {
        for v in self.sig.iter_mut() {
            *v = 0.0;
        }
        let r = self.range();
        let mut dist = std::mem::take(&mut self.scratch);
        for v in dist.iter_mut() {
            *v = -1.0;
        }
        for i in 0..self.cost.len() {
            self.cost[i] = match self.hy[i] {
                hy::NODE => 0.5,
                hy::TIP => 2.0,
                _ => 1.0,
            };
        }
        let bloom = self.blooms();
        let mut dq: VecDeque<usize> = VecDeque::new();
        let mut sources: Vec<(usize, usize)> = Vec::with_capacity(1 + self.relays.len());
        if let Some(hh) = self.heart.filter(|_| !self.heart_lost) {
            sources.push(hh);
        }
        sources.extend(self.relays.iter().copied());
        sources.extend(self.ganglia.iter().copied());
        let graft = self.mods.grafting;
        for (x, y) in sources {
            let i = self.idx(x, y);
            if is_hyphal(self.hy[i]) {
                dist[i] = 0.0;
                dq.push_back(i);
            }
        }
        while let Some(i) = dq.pop_front() {
            let d0 = dist[i];
            for d in 0..4 {
                if let Some(j) = self.nb(i, d) {
                    // the city's signal runs along its own strain, never into a Bloom
                    if is_hyphal(self.hy[j]) && (graft || !(bloom && self.ferality[j] >= 1.0)) {
                        let nd = d0 + self.cost[j];
                        if nd < r && (dist[j] < 0.0 || nd < dist[j]) {
                            dist[j] = nd;
                            dq.push_back(j);
                        }
                    }
                }
            }
        }
        for i in 0..self.sig.len() {
            if dist[i] >= 0.0 {
                self.sig[i] = (1.0 - dist[i] / r).max(0.0);
            }
        }
        self.scratch = dist;
    }

    // ------------------------------------------------------------ nutrient
    fn draw_pits(&mut self) {
        // in a fixed order: two Pits whose footprints touch share cells, and a map's order differs between processes
        let mut keys: Vec<(usize, usize)> = self.pits.keys().copied().collect();
        keys.sort_unstable();
        for (x, y) in keys {
            let stock = self.pits[&(x, y)];
            if stock <= 0.0 {
                continue;
            }
            // footprint 3x3; each cell wants min(pit_rate, 1 - N)
            let boost = self.pit_boost.get(&(x, y)).copied().unwrap_or(1.0);
            let mut cells: Vec<(usize, f32)> = Vec::with_capacity(9);
            let mut want = 0.0f32;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let xx = x as i32 + dx;
                    let yy = y as i32 + dy;
                    if xx < 0 || yy < 0 || xx as usize >= self.w || yy as usize >= self.h {
                        continue;
                    }
                    let i = yy as usize * self.w + xx as usize;
                    if sub::COND[self.sub[i] as usize] > 0.0 {
                        let room = (self.pit_rate * self.mods.release * boost).min(1.0 - self.n[i]).max(0.0);
                        want += room;
                        cells.push((i, room));
                    }
                }
            }
            if want <= 0.0 {
                continue;
            }
            let scale = (stock / want).min(1.0);
            for (i, room) in cells {
                self.n[i] += room * scale;
            }
            self.pits.insert((x, y), stock - want * scale);
        }
    }

    fn step_nutrient(&mut self) {
        let len = self.n.len();
        let cord_k = self.p.cord_k;
        self.draw_pits();

        // 1. diffusion: N' = N + D * sum k_ij (N_j - N_i)
        let mut delta = std::mem::take(&mut self.scratch);
        for i in 0..len {
            let hi = is_hyphal(self.hy[i]);
            let ki = sub::COND[self.sub[i] as usize];
            let mut total = 0.0f32;
            for d in 0..4 {
                if let Some(j) = self.nb(i, d) {
                    let hj = is_hyphal(self.hy[j]);
                    let kj = sub::COND[self.sub[j] as usize];
                    let k = if hi && hj { cord_k } else { ki.min(kj) };
                    total += k * (self.n[j] - self.n[i]);
                }
            }
            delta[i] = self.d * total;
        }
        for i in 0..len {
            self.n[i] += delta[i];
        }

        // 2. uptake: an empty cell next to hyphae gives uptake * N to EACH hyphal neighbour
        for v in delta.iter_mut() {
            *v = 0.0;
        }
        // a Bloom's tissue eats at twice the uptake (Pass 1: its Appetite doubles)
        let bloom = self.blooms();
        for j in 0..len {
            if is_hyphal(self.hy[j]) || sub::COND[self.sub[j] as usize] <= 0.0 {
                continue;
            }
            let amt = self.uptake * self.n[j];
            if amt <= 0.0 {
                continue;
            }
            for d in 0..4 {
                if let Some(i) = self.nb(j, d) {
                    if is_hyphal(self.hy[i]) {
                        let a = if bloom && self.ferality[i] >= 1.0 { amt * 2.0 } else { amt };
                        delta[j] -= a;
                        delta[i] += a;
                    }
                }
            }
        }
        for i in 0..len {
            self.n[i] += delta[i];
        }

        // 3. Heart pump: surplus above a working reserve moves outward (to lower signal), split across links,
        //    each gift capped by the receiver's free capacity
        let reserve = self.p.theta_starve + 0.05;
        let pump = self.p.pump;
        let snap = std::mem::take(&mut self.scratch2);
        let mut snap = snap;
        snap.copy_from_slice(&self.n);
        for v in delta.iter_mut() {
            *v = 0.0;
        }
        for i in 0..len {
            if !is_hyphal(self.hy[i]) || self.sig[i] <= 0.0 {
                continue;
            }
            let surplus = (snap[i] - reserve).max(0.0);
            if surplus <= 0.0 {
                continue;
            }
            let mut c = 0u32;
            for d in 0..4 {
                if let Some(j) = self.nb(i, d) {
                    if is_hyphal(self.hy[j]) && self.sig[j] < self.sig[i] {
                        c += 1;
                    }
                }
            }
            if c == 0 {
                continue;
            }
            let share = pump * surplus / c as f32;
            for d in 0..4 {
                if let Some(j) = self.nb(i, d) {
                    if is_hyphal(self.hy[j]) && self.sig[j] < self.sig[i] {
                        let gift = share.min((1.0 - snap[j]).max(0.0));
                        delta[i] -= gift;
                        delta[j] += gift;
                    }
                }
            }
        }
        for i in 0..len {
            self.n[i] += delta[i];
        }
        self.scratch2 = snap;

        // 4. upkeep, decay, deposits, recycled necrosis, rock, clamp
        let maint = self.p.maint * self.mods.maint;
        let theta_starve = self.p.theta_starve;
        let bloom = self.blooms();
        for i in 0..len {
            let mut v = self.n[i];
            if is_hyphal(self.hy[i]) {
                v -= maint;
                // a Bloom and the city touching fight at the border: both lose 0.05 a step (Pass 1's strains)
                if bloom {
                    let feral = self.ferality[i] >= 1.0;
                    let mut border = false;
                    for d in 0..4 {
                        if let Some(j) = self.nb(i, d) {
                            if is_hyphal(self.hy[j]) && (self.ferality[j] >= 1.0) != feral {
                                border = true;
                            }
                        }
                    }
                    if border {
                        v -= 0.05;
                    }
                }
                // a fed tile out of the light mends its armour a point a step
                if !self.hurt[i] && v > theta_starve {
                    self.wear[i] = (self.wear[i] - 1.0).max(0.0);
                }
                self.hurt[i] = false;
            }
            v -= self.decay * v;
            if self.sub[i] == sub::DEPOSIT {
                // a Root Spur's Deposit feeds the Mat twice (Full Design v3.18, Organs)
                v += if self.spur_boost[i] { 0.008 } else { 0.004 };
            }
            v += self.release[i];
            self.release[i] = 0.0;
            if sub::COND[self.sub[i] as usize] <= 0.0 {
                v = 0.0;
            }
            self.n[i] = v.clamp(0.0, 1.0);
        }
        self.scratch = delta;
    }

    // ------------------------------------------------------------ hyphae
    /// Place a new tip in a growable empty neighbour of `c`; the target may touch at most `max_nbrs` hyphae.
    fn try_sprout(&mut self, c: usize, max_nbrs: u8, ncount: &[u8], occupied: &mut [bool]) -> Option<usize> {
        let need = self.p.theta_grow + self.grow_cost + self.tip_starter;
        if self.n[c] < need {
            return None;
        }
        if self.rng.f32() > 0.7 {
            return None;
        }
        let mut cands: [(usize, f32); 4] = [(0, 0.0); 4];
        let mut k = 0usize;
        let mut wsum = 0.0f32;
        for d in 0..4 {
            if let Some(e) = self.nb(c, d) {
                if occupied[e] || self.hy[e] != hy::EMPTY || self.salt[e] || !sub::GROWABLE[self.sub[e] as usize] {
                    continue;
                }
                let ne = ncount[e];
                if ne > max_nbrs {
                    continue;
                }
                let mut wgt = 0.25 + self.n[e]; // chemotropism: richer ground ahead is favoured, never required
                if ne == 2 {
                    wgt *= 0.15; // closing a loop is rare
                }
                cands[k] = (e, wgt);
                wsum += wgt;
                k += 1;
            }
        }
        if k == 0 {
            return None;
        }
        let mut r = self.rng.f32() * wsum;
        let mut pick = cands[k - 1].0;
        for &(e, wgt) in &cands[..k] {
            if r < wgt {
                pick = e;
                break;
            }
            r -= wgt;
        }
        self.set_hy(pick, hy::TIP);
        self.age[pick] = 0;
        self.stall[pick] = 0;
        // a tip is its parent's strain: it carries the parent's ferality
        self.ferality[pick] = self.ferality[c];
        occupied[pick] = true;
        self.n[c] -= self.grow_cost + self.tip_starter;
        self.n[pick] = (self.n[pick] + self.tip_starter).min(1.0);
        Some(pick)
    }

    fn step_hyphae(&mut self) {
        let len = self.hy.len();
        let old: Vec<u8> = self.hy.clone();
        let mut ncount = vec![0u8; len];
        for i in 0..len {
            ncount[i] = self.hyphal_neighbours(i);
        }
        let theta_starve = self.p.theta_starve;
        for a in self.age.iter_mut() {
            *a = a.saturating_add(1);
        }
        let heart_i = self.heart.map(|(x, y)| y * self.w + x);
        let relay_idx: Vec<usize> = self.relays.iter().map(|&(x, y)| y * self.w + x).collect();

        // starvation and isolation -> necrotic
        for i in 0..len {
            let h = old[i];
            if !is_hyphal(h) {
                self.starve[i] = 0;
                self.lonely[i] = 0;
                continue;
            }
            self.starve[i] = if self.n[i] < theta_starve { self.starve[i] + 1 } else { 0 };
            self.lonely[i] = if ncount[i] == 0 { self.lonely[i] + 1 } else { 0 };
            let mut die = (self.starve[i] >= 24 && (h == hy::CORD || h == hy::NODE)) || (self.starve[i] >= 8 && h == hy::TIP);
            die |= self.lonely[i] >= 12;
            if Some(i) == heart_i || relay_idx.contains(&i) {
                die = false;
            }
            if die {
                self.set_hy(i, hy::NECRO);
                self.age[i] = 0;
                self.starve[i] = 0;
            }
        }

        // isolation -> ferality: tissue without signal gains Pass 1's Feral Drive rate a step and loses it all when
        // signal returns; at 1 it is feral (Full Design, Pressure: Blooms)
        let feral_rate = self.p.feral_rate;
        for i in 0..len {
            if !is_hyphal(self.hy[i]) {
                continue;
            }
            if self.sig[i] > 0.0 {
                // a Bloom's tissue taken back is a merge shock; feral tissue of a strain that does not bloom was
                // never another strain
                if self.ferality[i] >= 1.0 && self.blooms() {
                    self.reclaimed += 1;
                }
                self.ferality[i] = 0.0;
            } else {
                self.ferality[i] = (self.ferality[i] + feral_rate).min(1.0);
            }
        }

        // necrotic -> empty after 30 ticks, releasing nutrient
        for i in 0..len {
            if old[i] == hy::NECRO && self.age[i] >= 30 {
                self.release[i] += 0.35;
                for d in 0..4 {
                    if let Some(j) = self.nb(i, d) {
                        self.release[j] += 0.10;
                    }
                }
                self.set_hy(i, hy::EMPTY);
            }
        }

        // maturation: junction cord with N >= 0.25 for 10 ticks -> node (p = 0.2 / tick)
        for i in 0..len {
            if old[i] == hy::CORD {
                let elig = ncount[i] >= 3 && self.n[i] >= 0.25;
                self.stall[i] = if elig { self.stall[i] + 1 } else { 0 };
                if elig && self.stall[i] >= 10 && self.rng.f32() < 0.2 {
                    self.set_hy(i, hy::NODE);
                    self.age[i] = 0;
                }
            }
        }

        // anastomosis: a tip touching >= 2 hyphal cells has met another strand -> cord
        let mut fused = vec![false; len];
        for i in 0..len {
            if old[i] == hy::TIP && ncount[i] >= 2 {
                fused[i] = true;
                self.set_hy(i, hy::CORD);
                self.age[i] = 0;
            }
        }

        let mut occupied: Vec<bool> = (0..len).map(|i| is_hyphal(old[i]) || self.hy[i] == hy::NECRO).collect();

        // lateral branching: a strand cell (exactly 2 hyphal nbrs) sprouts a side tip
        let lat_p = self.p.branch_p * 0.05;
        for i in 0..len {
            if old[i] == hy::CORD && ncount[i] == 2 && self.rng.f32() < lat_p {
                let _ = self.try_sprout(i, 1, &ncount, &mut occupied);
            }
        }

        // tip extension: one step per tip per tick, in shuffled order
        let mut tips: Vec<usize> = (0..len).filter(|&i| old[i] == hy::TIP && !fused[i] && self.hy[i] == hy::TIP).collect();
        self.rng.shuffle(&mut tips);
        let branch_p = self.p.branch_p;
        let runner = self.mods.runner;
        for t in tips {
            if let Some(p) = self.try_sprout(t, 2, &ncount, &mut occupied) {
                // a Runner strain's tip extends two tiles a step (Pass 2): the new tip goes on at once
                if runner && self.try_sprout(p, 2, &ncount, &mut occupied).is_some() {
                    self.set_hy(p, hy::CORD);
                    self.age[p] = 0;
                }
                if self.rng.f32() > branch_p {
                    self.set_hy(t, hy::CORD); // parent matures unless it forks and stays a tip
                    self.age[t] = 0;
                }
                self.stall[t] = 0;
            } else {
                self.stall[t] += 1;
                if self.stall[t] >= 14 {
                    self.set_hy(t, hy::CORD);
                    self.age[t] = 0;
                    self.stall[t] = 0;
                }
            }
        }
    }

    // ------------------------------------------------------------ tick
    pub fn step(&mut self) -> Stats {
        self.tick += 1;
        self.changed.clear();
        self.compute_signal();
        self.step_hyphae();
        self.step_nutrient();
        self.stats()
    }

    pub fn stats(&self) -> Stats {
        let mut s = Stats { tick: self.tick, ..Default::default() };
        let mut nsum = 0.0f32;
        let mut reach = 0u32;
        let (hx, hy_) = self.heart.unwrap_or((0, 0));
        for i in 0..self.hy.len() {
            match self.hy[i] {
                hy::TIP => s.tips += 1,
                hy::CORD => s.cords += 1,
                hy::NODE => s.nodes += 1,
                hy::NECRO => s.necro += 1,
                _ => {}
            }
            nsum += self.n[i];
            if is_hyphal(self.hy[i]) {
                let x = i % self.w;
                let y = i / self.w;
                let d = (x as i32 - hx as i32).unsigned_abs() + (y as i32 - hy_ as i32).unsigned_abs();
                reach = reach.max(d);
            }
        }
        s.nutrient = nsum;
        s.reach = reach;
        s
    }

    pub fn hyphal_count(&self) -> usize {
        self.hy.iter().filter(|&&h| is_hyphal(h)).count()
    }

    /// 64-bit state digest for determinism tests.
    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for (i, &s) in self.hy.iter().enumerate() {
            h ^= (s as u64) ^ ((self.n[i].to_bits() as u64) << 8) ^ (i as u64);
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        h
    }
}

/// One stratum's ground weights (Full Design v3.19: `data/strata.toml`, after Pass 1's strata table).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StratumGen {
    /// culvert rows: the first's centre, the spacing, and the half-height (1: three rows)
    pub culvert_from: usize,
    pub culvert_every: usize,
    pub culvert_half: usize,
    /// a pipe run this many rows above each culvert (0: none)
    pub pipe_above: usize,
    /// smoothed noise above this is a cave
    pub cave: f32,
    /// a Deposit's chance on open ground
    pub deposit: f32,
    /// rows of water under the lowest culvert
    pub water_rows: usize,
}

impl StratumGen {
    /// The Sump as Pass 1's prototype generated it (and every single-stratum map before the stack).
    pub const SUMP: StratumGen = StratumGen { culvert_from: 12, culvert_every: 13, culvert_half: 1, pipe_above: 2, cave: 0.56, deposit: 0.025, water_rows: 1 };
}

/// Generated sewer stratum: culvert rows with pipe runs, vertical shafts, collapsed caves, brick lining,
/// scrap deposits and a water line. Same layout family as the Python prototype.
pub fn worldgen(w: usize, h: usize, rng: &mut Pcg32) -> Vec<u8> {
    worldgen_band(w, h, rng, &StratumGen::SUMP)
}

/// One stratum by its weights: culvert rows, shafts every 23 columns from 10, smoothed-noise caves, brick lining,
/// Deposits, water under the lowest culvert, and a grate on the top row over each shaft.
pub fn worldgen_band(w: usize, h: usize, rng: &mut Pcg32, g: &StratumGen) -> Vec<u8> {
    let mut s = vec![sub::ROCK; w * h];
    let idx = |x: usize, y: usize| y * w + x;
    let half = g.culvert_half.max(1);
    // culvert rows, a pipe run above each
    let mut y = g.culvert_from.max(half + g.pipe_above);
    while y + half + 1 < h {
        for yy in y - half..=y + half {
            for x in 0..w {
                s[idx(x, yy)] = sub::OPEN;
            }
        }
        if g.pipe_above > 0 {
            for x in 0..w {
                s[idx(x, y - half - g.pipe_above + 1)] = sub::PIPE;
            }
        }
        y += g.culvert_every.max(half * 2 + 3);
    }
    // vertical shafts
    let mut x = 10usize;
    while x + 1 < w {
        for yy in 0..h {
            s[idx(x, yy)] = sub::OPEN;
            s[idx(x + 1, yy)] = sub::OPEN;
        }
        x += 23;
    }
    // caves from smoothed noise
    let mut noise: Vec<f32> = (0..w * h).map(|_| rng.f32()).collect();
    for _ in 0..3 {
        let prev = noise.clone();
        for yy in 0..h {
            for xx in 0..w {
                let gg = |dx: i32, dy: i32| -> f32 {
                    let x2 = (xx as i32 + dx).rem_euclid(w as i32) as usize;
                    let y2 = (yy as i32 + dy).rem_euclid(h as i32) as usize;
                    prev[y2 * w + x2]
                };
                noise[idx(xx, yy)] = (gg(0, 0) + gg(0, -1) + gg(0, 1) + gg(-1, 0) + gg(1, 0)) / 5.0;
            }
        }
    }
    for (i, v) in noise.iter().enumerate() {
        if *v > g.cave {
            s[i] = sub::OPEN;
        }
    }
    // brick lining around open space
    let open: Vec<bool> = s.iter().map(|&v| v == sub::OPEN).collect();
    for yy in 0..h {
        for xx in 0..w {
            let i = idx(xx, yy);
            if s[i] != sub::ROCK {
                continue;
            }
            let near = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                let x2 = (xx as i32 + dx).rem_euclid(w as i32) as usize;
                let y2 = (yy as i32 + dy).rem_euclid(h as i32) as usize;
                open[y2 * w + x2]
            });
            if near {
                s[i] = sub::BRICK;
            }
        }
    }
    // deposits on open ground, water under the lowest culvert
    for v in s.iter_mut() {
        if *v == sub::OPEN && rng.f32() < g.deposit {
            *v = sub::DEPOSIT;
        }
    }
    let mut last_culvert = g.culvert_from.max(half + g.pipe_above);
    let mut yy = last_culvert;
    while yy + half + 1 < h {
        last_culvert = yy;
        yy += g.culvert_every.max(half * 2 + 3);
    }
    for r in 0..g.water_rows {
        let row = last_culvert + half + r;
        if row < h {
            for x in 0..w {
                let i = idx(x, row);
                if s[i] == sub::OPEN {
                    s[i] = sub::WATER;
                }
            }
        }
    }
    // grates: the top row over each shaft (a Deposit that fell there is the grate's: every shaft has one)
    let mut x = 10usize;
    while x + 1 < w {
        s[idx(x, 0)] = sub::GRATE;
        x += 23;
    }
    s
}

/// The stack (Full Design v3.19): `gens[0]` the Sump at the bottom, each above it in order; generated Sump first so
/// a seed's Sump is the single-stratum map's. Every stratum above the Sump has its bottom row sealed — rock, and a
/// slab where a shaft crosses — so nothing passes until a breach.
pub fn worldgen_stack(w: usize, rows: usize, gens: &[StratumGen], rng: &mut Pcg32) -> Vec<u8> {
    let n = gens.len().max(1);
    let mut s = vec![sub::ROCK; w * rows * n];
    for (k, g) in gens.iter().enumerate() {
        let band = worldgen_band(w, rows, rng, g);
        let y0 = (n - 1 - k) * rows;
        s[y0 * w..(y0 + rows) * w].copy_from_slice(&band);
        if k > 0 {
            let bottom = y0 + rows - 1;
            for x in 0..w {
                s[bottom * w + x] = sub::ROCK;
            }
            let mut x = 10usize;
            while x + 1 < w {
                s[bottom * w + x] = sub::SLAB;
                s[bottom * w + x + 1] = sub::SLAB;
                x += 23;
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::*;

    #[test]
    fn the_stack_keeps_the_sump_and_seals_every_crossing_until_a_breach() {
        let g = [StratumGen::SUMP; 3];
        let single = worldgen(192, 48, &mut Pcg32::new(7));
        let mut sim = MatSim::stack(192, 48, &g, 7, Genome::default());
        assert_eq!((sim.h, sim.strata), (144, 3));
        assert_eq!(&sim.sub[96 * 192..], &single[..], "the Sump is the single-stratum map");
        assert_eq!(sim.stratum_of_row(143), 1);
        assert_eq!(sim.stratum_of_row(0), 3);
        assert_eq!(sim.grates_of(1).len(), 8);
        assert!(sim.grates().iter().all(|&i| sim.sub[i] == sub::GRATE));
        let cross = sim.crossings_of(1);
        assert_eq!(cross.len(), 8);
        assert!(cross.iter().all(|&c| sim.sealed(c)));
        // nothing grows from the Sump into the Culverts: the row between is rock and slab
        let between = sim.ceiling(1) - 1;
        assert!((0..192).all(|x| !sub::GROWABLE[sim.sub[between * 192 + x] as usize]));
        assert!(sim.open_slab(cross[0]) && !sim.sealed(cross[0]));
        assert!(sim.crossings_of(3).is_empty());
    }

    fn fed_run(seed: u64, genome: Genome, ticks: u64) -> MatSim {
        let mut sim = MatSim::new(192, 48, seed, genome);
        sim.seed_heart(96, 25);
        for _ in 0..ticks {
            sim.feed(96, 25, 0.6);
            sim.step();
        }
        sim
    }

    #[test]
    fn deterministic() {
        let a = fed_run(11, Genome::default(), 200);
        let b = fed_run(11, Genome::default(), 200);
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.stats().cords, b.stats().cords);
    }

    #[test]
    fn nutrient_bounded_and_finite() {
        let sim = fed_run(7, Genome::default(), 400);
        for &v in &sim.n {
            assert!(v.is_finite() && (0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn fed_network_grows() {
        let sim = fed_run(11, Genome::default(), 350);
        let s = sim.stats();
        assert!(sim.hyphal_count() > 100, "cells {}", sim.hyphal_count());
        assert!(s.reach >= 15, "reach {}", s.reach);
    }

    #[test]
    fn runner_reaches_farther_than_default() {
        let base = fed_run(11, Genome::default(), 350).stats().reach;
        let runner = fed_run(11, Genome::default().with(VIGOR, 0.9).with(CONDUCTANCE, 0.9), 350).stats().reach;
        assert!(runner > base, "runner {} base {}", runner, base);
    }

    #[test]
    fn unfed_network_dies_back() {
        // with nothing to eat — no feed, no Deposits under it — the Mat spends what it holds and dies back; at an
        // eighth of Pass 1's metabolism, after 350 steps of Pass 1's feed, it has lost a third by forty shifts
        // (5000 steps; measured 300 cells to 198)
        let mut sim = fed_run(11, Genome::default(), 350);
        for v in sim.sub.iter_mut() {
            if *v == sub::DEPOSIT {
                *v = sub::OPEN;
            }
        }
        let before = sim.hyphal_count();
        let mut peak = before;
        for _ in 0..5000 {
            sim.step();
            peak = peak.max(sim.hyphal_count());
        }
        assert!(sim.hyphal_count() * 4 < peak * 3, "peak {} after {}", peak, sim.hyphal_count());
    }

    #[test]
    fn transport_conserves_mass_on_a_chain() {
        let mut sim = MatSim::new(64, 5, 1, Genome::default());
        for v in sim.sub.iter_mut() {
            *v = sub::ROCK;
        }
        for x in 0..64 {
            let i = sim.idx(x, 2);
            sim.sub[i] = sub::OPEN;
        }
        sim.heart = Some((2, 2));
        let hi = sim.idx(2, 2);
        sim.hy[hi] = hy::NODE;
        for x in 3..60 {
            let i = sim.idx(x, 2);
            sim.hy[i] = hy::CORD;
        }
        sim.n[hi] = 1.0;
        sim.decay = 0.0;
        sim.p.maint = 0.0;
        sim.compute_signal();
        let before: f32 = sim.n.iter().sum();
        sim.step_nutrient();
        let after: f32 = sim.n.iter().sum();
        assert!((before - after).abs() < 1e-3, "before {} after {}", before, after);
        // and the pump moved something down the chain
        assert!(sim.n[sim.idx(3, 2)] > 0.0);
    }

    #[test]
    fn salt_blocks_growth() {
        let mut sim = MatSim::new(192, 48, 5, Genome::default());
        sim.seed_heart(96, 25);
        for x in 0..192 {
            for y in 0..48 {
                if x == 100 || x == 92 {
                    sim.salt_tile(x, y, true);
                }
            }
        }
        for _ in 0..300 {
            sim.feed(96, 25, 0.6);
            sim.step();
        }
        for y in 0..48 {
            for x in 0..192 {
                if x > 100 || x < 92 {
                    assert!(!sim.is_hyphal_at(sim.idx(x, y)), "grew past salt at {},{}", x, y);
                }
            }
        }
    }
}
