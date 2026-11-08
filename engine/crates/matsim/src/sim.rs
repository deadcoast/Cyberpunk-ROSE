//! The Mat: a stochastic cellular automaton coupled to a continuous nutrient field.
//! Rule order and parameters follow the GDD (Mat spec) and the Python prototype that validated them.

use crate::genome::{Genome, Params};
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
    pub const COUNT: usize = 8;
    /// nutrient conductance per substrate
    pub const COND: [f32; COUNT] = [0.0, 0.40, 0.60, 0.08, 0.40, 0.25, 0.40, 0.04];
    /// hyphae may enter
    pub const GROWABLE: [bool; COUNT] = [false, true, true, true, true, true, true, true];
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

/// One stratum (or a stack of them) of the Mat.
#[derive(Clone)]
pub struct MatSim {
    pub w: usize,
    pub h: usize,
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
    pub heart: Option<(usize, usize)>,
    pub relays: Vec<(usize, usize)>,
    pub pits: HashMap<(usize, usize), f32>,
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
        let n = w * h;
        let mut rng = Pcg32::new(seed);
        let sub = worldgen(w, h, &mut rng);
        MatSim {
            w,
            h,
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
            heart: None,
            relays: Vec::new(),
            pits: HashMap::new(),
            tick: 0,
            changed: Vec::new(),
            d: 0.11,
            decay: 0.001,
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
        let r = self.p.heart_range;
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
        let mut dq: VecDeque<usize> = VecDeque::new();
        let mut sources: Vec<(usize, usize)> = Vec::with_capacity(1 + self.relays.len());
        if let Some(hh) = self.heart {
            sources.push(hh);
        }
        sources.extend(self.relays.iter().copied());
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
                    if is_hyphal(self.hy[j]) {
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
        let keys: Vec<(usize, usize)> = self.pits.keys().copied().collect();
        for (x, y) in keys {
            let stock = self.pits[&(x, y)];
            if stock <= 0.0 {
                continue;
            }
            // footprint 3x3; each cell wants min(pit_rate, 1 - N)
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
                        let room = self.pit_rate.min(1.0 - self.n[i]).max(0.0);
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
                        delta[j] -= amt;
                        delta[i] += amt;
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
        let maint = self.p.maint;
        for i in 0..len {
            let mut v = self.n[i];
            if is_hyphal(self.hy[i]) {
                v -= maint;
            }
            v -= self.decay * v;
            if self.sub[i] == sub::DEPOSIT {
                v += 0.004;
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
    fn try_sprout(&mut self, c: usize, max_nbrs: u8, ncount: &[u8], occupied: &mut [bool]) -> bool {
        let need = self.p.theta_grow + self.grow_cost + self.tip_starter;
        if self.n[c] < need {
            return false;
        }
        if self.rng.f32() > 0.7 {
            return false;
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
            return false;
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
        occupied[pick] = true;
        self.n[c] -= self.grow_cost + self.tip_starter;
        self.n[pick] = (self.n[pick] + self.tip_starter).min(1.0);
        true
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
                self.try_sprout(i, 1, &ncount, &mut occupied);
            }
        }

        // tip extension: one step per tip per tick, in shuffled order
        let mut tips: Vec<usize> = (0..len).filter(|&i| old[i] == hy::TIP && !fused[i] && self.hy[i] == hy::TIP).collect();
        self.rng.shuffle(&mut tips);
        let branch_p = self.p.branch_p;
        for t in tips {
            if self.try_sprout(t, 2, &ncount, &mut occupied) {
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

/// Generated sewer stratum: culvert rows with pipe runs, vertical shafts, collapsed caves, brick lining,
/// scrap deposits and a water line. Same layout family as the Python prototype.
pub fn worldgen(w: usize, h: usize, rng: &mut Pcg32) -> Vec<u8> {
    let mut s = vec![sub::ROCK; w * h];
    let idx = |x: usize, y: usize| y * w + x;
    // culvert rows every 13 tiles starting at 12, a pipe run two above
    let mut y = 12usize;
    while y + 2 < h {
        for yy in y - 1..=y + 1 {
            for x in 0..w {
                s[idx(x, yy)] = sub::OPEN;
            }
        }
        for x in 0..w {
            s[idx(x, y - 2)] = sub::PIPE;
        }
        y += 13;
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
                let g = |dx: i32, dy: i32| -> f32 {
                    let x2 = (xx as i32 + dx).rem_euclid(w as i32) as usize;
                    let y2 = (yy as i32 + dy).rem_euclid(h as i32) as usize;
                    prev[y2 * w + x2]
                };
                noise[idx(xx, yy)] = (g(0, 0) + g(0, -1) + g(0, 1) + g(-1, 0) + g(1, 0)) / 5.0;
            }
        }
    }
    for i in 0..w * h {
        if noise[i] > 0.56 {
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
    // deposits on open ground, water on the row below the lowest culvert
    for i in 0..w * h {
        if s[i] == sub::OPEN && rng.f32() < 0.025 {
            s[i] = sub::DEPOSIT;
        }
    }
    let mut last_culvert = 12usize;
    let mut yy = 12usize;
    while yy + 2 < h {
        last_culvert = yy;
        yy += 13;
    }
    if last_culvert + 1 < h {
        for x in 0..w {
            let i = idx(x, last_culvert + 1);
            if s[i] == sub::OPEN {
                s[i] = sub::WATER;
            }
        }
    }
    // grates: one open tile at the top row above each shaft top
    let mut x = 10usize;
    while x + 1 < w {
        if s[idx(x, 0)] == sub::OPEN {
            s[idx(x, 0)] = sub::GRATE;
        }
        x += 23;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::*;

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
        let mut sim = fed_run(11, Genome::default(), 350);
        let before = sim.hyphal_count();
        for _ in 0..900 {
            sim.step();
        }
        assert!(sim.hyphal_count() < before, "before {} after {}", before, sim.hyphal_count());
        assert!(sim.stats().necro > 0 || sim.hyphal_count() < before / 2);
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
