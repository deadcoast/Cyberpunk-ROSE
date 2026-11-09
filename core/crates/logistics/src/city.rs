//! The city on the Mat: sites, cargo packets with cord capacity and demand-weighted routing,
//! and agent columns (structure-of-arrays) moving on flow fields.

use mat::flow::FlowField;
use mat::rng::Pcg32;
use mat::sim::{is_hyphal, sub, MatSim};

pub mod state {
    pub const HOME: u8 = 0;
    pub const TO_MARKET: u8 = 1;
    pub const TO_PIT: u8 = 2;
    pub const TO_HOME: u8 = 3;
}

/// Packets per cord tile before the trunk slows to half speed.
pub const CORD_CAPACITY: u8 = 4;
/// Pit quota: demand is how far below this a Pit's stock sits.
pub const PIT_TARGET: f32 = 40.0;
pub const TIER_NUTRIENT: [f32; 4] = [0.0, 0.18, 0.30, 0.45];

#[derive(Clone, Debug, Default)]
pub struct CityStats {
    pub tick: u64,
    pub agents: usize,
    pub carrying: usize,
    pub packets: usize,
    pub deliveries: u64,
    pub hauls: u64,
    pub jammed_tiles: u32,
}

#[derive(Clone)]
pub struct City {
    pub sim: MatSim,
    pub rng: Pcg32,
    pub markets: Vec<usize>,
    pub pits: Vec<usize>,
    pub pods: Vec<usize>,
    // fields
    pub f_market: FlowField,
    pub f_pod: FlowField,
    pub f_pit: Vec<FlowField>,
    pub f_pit_any: FlowField,
    passable: Vec<bool>,
    cords: Vec<bool>,
    // agents (SoA)
    pub pos: Vec<u32>,
    pub state: Vec<u8>,
    pub dwell: Vec<i16>,
    pub home: Vec<u32>,
    pub phase: Vec<u8>,
    // packets
    pub pk_pos: Vec<u32>,
    pub pk_tier: Vec<u8>,
    pub pk_dest: Vec<u8>,
    /// per-market stockpile of hauled scrap waiting for room on the cord: (tier, destination pit)
    pub market_queue: Vec<Vec<(u8, u8)>>,
    pub load: Vec<u8>,
    pub tick: u64,
    pub deliveries: u64,
    pub hauls: u64,
    pub last_jam: u32,
    pub haul_every: u64,
    pub rebuild_every: u64,
}

impl City {
    /// Place sites on an already-grown Mat and spawn `n_agents` slimos at Pods.
    pub fn new(sim: MatSim, n_agents: usize, market_hints: &[(usize, usize)], seed: u64) -> Self {
        let mut rng = Pcg32::new(seed.wrapping_mul(0x9E37_79B9).wrapping_add(0xC1_7F00));
        let w = sim.w;
        let hyphal: Vec<bool> = sim.hy.iter().map(|&h| is_hyphal(h)).collect();
        let passable: Vec<bool> = (0..sim.n.len()).map(|i| hyphal[i] || sub::COND[sim.sub[i] as usize] > 0.0).collect();
        // markets: nearest hyphal cell to each hint within 20 tiles (a market sits where the Mat reached the grate)
        let hy_cells: Vec<usize> = (0..hyphal.len()).filter(|&i| hyphal[i]).collect();
        let mut markets = Vec::new();
        for &(mx, my) in market_hints {
            let mut best = (usize::MAX, 0usize);
            for &i in &hy_cells {
                let d = (i % w).abs_diff(mx) + (i / w).abs_diff(my);
                if d < best.0 {
                    best = (d, i);
                }
            }
            if best.0 <= 20 && !markets.contains(&best.1) {
                markets.push(best.1);
            }
        }
        let mut pits: Vec<usize> = Vec::new();
        if let Some((x, y)) = sim.heart {
            pits.push(y * w + x);
        }
        for &(x, y) in &sim.relays {
            pits.push(y * w + x);
        }
        // pods: up to 60 nodes, chosen at random
        let mut nodes: Vec<usize> = (0..hyphal.len()).filter(|&i| sim.hy[i] == mat::sim::hy::NODE).collect();
        rng.shuffle(&mut nodes);
        let pods: Vec<usize> = nodes.into_iter().take(60).collect();
        let mut c = City {
            sim,
            rng,
            markets,
            pits,
            pods,
            f_market: FlowField { w, h: 0, dist: vec![], next: vec![] },
            f_pod: FlowField { w, h: 0, dist: vec![], next: vec![] },
            f_pit: Vec::new(),
            f_pit_any: FlowField { w, h: 0, dist: vec![], next: vec![] },
            passable,
            cords: hyphal,
            pos: Vec::new(),
            state: Vec::new(),
            dwell: Vec::new(),
            home: Vec::new(),
            phase: Vec::new(),
            pk_pos: Vec::new(),
            pk_tier: Vec::new(),
            pk_dest: Vec::new(),
            market_queue: Vec::new(),
            load: Vec::new(),
            tick: 0,
            deliveries: 0,
            hauls: 0,
            last_jam: 0,
            haul_every: 15,
            rebuild_every: 20,
        };
        c.load = vec![0; c.sim.n.len()];
        c.market_queue = vec![Vec::new(); c.markets.len()];
        c.rebuild_fields();
        c.spawn_agents(n_agents);
        c
    }

    fn spawn_agents(&mut self, n: usize) {
        if self.pods.is_empty() {
            return;
        }
        for _ in 0..n {
            let home = self.pods[self.rng.below(self.pods.len() as u32) as usize] as u32;
            self.pos.push(home);
            self.home.push(home);
            self.state.push(state::HOME);
            self.dwell.push(self.rng.range(0, 40) as i16);
            self.phase.push(self.rng.below(2) as u8);
        }
    }

    pub fn rebuild_fields(&mut self) {
        let w = self.sim.w;
        let h = self.sim.h;
        for i in 0..self.cords.len() {
            self.cords[i] = is_hyphal(self.sim.hy[i]);
            self.passable[i] = self.cords[i] || sub::COND[self.sim.sub[i] as usize] > 0.0;
        }
        self.f_market = FlowField::build(w, h, &self.passable, &self.markets);
        self.f_pod = FlowField::build(w, h, &self.passable, &self.pods);
        self.f_pit = self.pits.iter().map(|&p| FlowField::build(w, h, &self.cords, &[p])).collect();
        self.f_pit_any = FlowField::build(w, h, &self.cords, &self.pits);
    }

    fn pit_stock(&self, pit: usize) -> f32 {
        let w = self.sim.w;
        *self.sim.pits.get(&(pit % w, pit / w)).unwrap_or(&0.0)
    }

    /// Demand-weighted destination: the Pit with the highest (quota shortfall / distance) from this market.
    fn choose_pit(&self, from: usize) -> Option<u8> {
        let mut best: Option<(f32, u8)> = None;
        for (k, f) in self.f_pit.iter().enumerate() {
            let d = f.dist[from];
            if d < 0 {
                continue;
            }
            let demand = (PIT_TARGET - self.pit_stock(self.pits[k])).max(0.5);
            let score = demand / (1.0 + d as f32);
            if best.map_or(true, |(s, _)| score > s) {
                best = Some((score, k as u8));
            }
        }
        best.map(|(_, k)| k)
    }

    pub fn step(&mut self) -> CityStats {
        self.tick += 1;
        let w = self.sim.w;
        let n = self.pos.len();
        let moving_phase = (self.tick % 2) as u8;

        // --- agents: move by state along the field for that state (half of them on alternate ticks)
        for i in 0..n {
            if self.phase[i] != moving_phase {
                continue;
            }
            let p = self.pos[i] as usize;
            let nxt = match self.state[i] {
                state::TO_MARKET => self.f_market.next[p],
                state::TO_PIT => self.f_pit_any.next[p],
                state::TO_HOME => self.f_pod.next[p],
                _ => self.pos[i],
            };
            self.pos[i] = nxt;
        }
        // --- arrivals and dwell
        let mut delivered_by_agents = 0u64;
        for i in 0..n {
            let p = self.pos[i] as usize;
            match self.state[i] {
                state::TO_MARKET => {
                    if self.f_market.at_source(p) {
                        self.state[i] = state::TO_PIT;
                    } else if !self.f_market.reachable(p) {
                        self.state[i] = state::TO_HOME;
                    }
                }
                state::TO_PIT => {
                    if self.f_pit_any.at_source(p) {
                        self.sim.feed(p % w, p / w, 0.3);
                        delivered_by_agents += 1;
                        self.state[i] = state::TO_HOME;
                    } else if !self.f_pit_any.reachable(p) {
                        self.state[i] = state::TO_HOME;
                    }
                }
                state::TO_HOME => {
                    if self.f_pod.at_source(p) || !self.f_pod.reachable(p) {
                        self.state[i] = state::HOME;
                        self.pos[i] = self.home[i];
                        self.dwell[i] = self.rng.range(20, 80) as i16;
                    }
                }
                _ => {
                    self.dwell[i] -= 1;
                    if self.dwell[i] <= 0 {
                        self.state[i] = state::TO_MARKET;
                    }
                }
            }
        }
        self.deliveries += delivered_by_agents;

        // --- hauls: every `haul_every` ticks each market lands 12-30 units into its stockpile, each routed by demand
        if self.tick % self.haul_every == 0 {
            for mi in 0..self.markets.len() {
                let m = self.markets[mi];
                if let Some(dest) = self.choose_pit(m) {
                    let k = self.rng.range(12, 30);
                    for _ in 0..k {
                        let tier = [1u8, 1, 1, 1, 2, 2, 3][self.rng.below(7) as usize];
                        self.market_queue[mi].push((tier, dest));
                    }
                    self.hauls += 1;
                }
            }
        }

        // --- current load per tile
        for v in self.load.iter_mut() {
            *v = 0;
        }
        for &p in &self.pk_pos {
            let l = &mut self.load[p as usize];
            *l = l.saturating_add(1);
        }

        // --- markets release stock onto the cord one packet at a time while their tile has room
        for mi in 0..self.markets.len() {
            let m = self.markets[mi];
            while self.load[m] < CORD_CAPACITY {
                match self.market_queue[mi].pop() {
                    Some((tier, dest)) => {
                        self.pk_pos.push(m as u32);
                        self.pk_tier.push(tier);
                        self.pk_dest.push(dest);
                        self.load[m] += 1;
                    }
                    None => break,
                }
            }
        }

        // --- packets: one tile per tick toward their Pit; a tile at capacity holds the packet behind it
        let mut jammed = 0u32;
        let mut keep = Vec::with_capacity(self.pk_pos.len());
        for k in 0..self.pk_pos.len() {
            let p = self.pk_pos[k] as usize;
            let f = &self.f_pit[self.pk_dest[k] as usize];
            if !f.reachable(p) {
                // the cord it rode is gone: fall back to any pit
                if self.f_pit_any.reachable(p) {
                    self.pk_dest[k] = 0;
                }
                keep.push(k);
                continue;
            }
            if f.at_source(p) {
                self.sim.feed(p % w, p / w, TIER_NUTRIENT[self.pk_tier[k] as usize]);
                self.deliveries += 1;
                continue;
            }
            let nxt = f.next[p] as usize;
            // a tile at capacity drains at half speed; nothing enters a full tile
            if self.load[p] >= CORD_CAPACITY {
                jammed += 1;
                if self.tick % 2 == 1 {
                    keep.push(k);
                    continue;
                }
            }
            if self.load[nxt] >= CORD_CAPACITY {
                keep.push(k);
                continue;
            }
            self.load[p] = self.load[p].saturating_sub(1);
            self.load[nxt] = self.load[nxt].saturating_add(1);
            self.pk_pos[k] = nxt as u32;
            keep.push(k);
        }
        if keep.len() != self.pk_pos.len() {
            self.pk_pos = keep.iter().map(|&k| self.pk_pos[k]).collect();
            self.pk_tier = keep.iter().map(|&k| self.pk_tier[k]).collect();
            self.pk_dest = keep.iter().map(|&k| self.pk_dest[k]).collect();
        }

        // --- the Mat keeps living underneath
        self.sim.step();
        if self.tick % self.rebuild_every == 0 {
            self.rebuild_fields();
        }
        self.last_jam = jammed;
        self.stats_with(jammed)
    }

    fn stats_with(&self, jammed: u32) -> CityStats {
        CityStats {
            tick: self.tick,
            agents: self.pos.len(),
            carrying: self.state.iter().filter(|&&s| s == state::TO_PIT).count(),
            packets: self.pk_pos.len() + self.market_queue.iter().map(|q| q.len()).sum::<usize>(),
            deliveries: self.deliveries,
            hauls: self.hauls,
            jammed_tiles: jammed,
        }
    }

    pub fn stats(&self) -> CityStats {
        self.stats_with(self.last_jam)
    }

    /// Traffic per tile: slimos + 2 x packets. The Stack view draws this as light.
    pub fn traffic(&self) -> Vec<u16> {
        let mut t = vec![0u16; self.sim.n.len()];
        for &p in &self.pos {
            t[p as usize] = t[p as usize].saturating_add(1);
        }
        for &p in &self.pk_pos {
            t[p as usize] = t[p as usize].saturating_add(2);
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mat::genome::*;

    fn grown() -> MatSim {
        let g = Genome::default().with(VIGOR, 0.8).with(CONDUCTANCE, 0.8);
        let mut sim = MatSim::new(192, 48, 11, g);
        sim.seed_heart(96, 25);
        let relays = [(40, 25), (150, 25), (20, 12), (100, 12), (170, 12), (60, 38), (130, 38)];
        for &(x, y) in &relays {
            sim.place_relay(x, y);
        }
        for _ in 0..500 {
            sim.feed(96, 25, 0.6);
            for &(x, y) in &relays {
                sim.feed(x, y, 0.4);
            }
            sim.step();
        }
        sim
    }

    #[test]
    fn agents_deliver_and_capacity_holds() {
        let sim = grown();
        let mut city = City::new(sim, 3000, &[(60, 12), (130, 12), (96, 38)], 11);
        assert!(!city.markets.is_empty(), "no market placed");
        let mut max_load = 0u8;
        for _ in 0..300 {
            city.step();
            max_load = max_load.max(*city.load.iter().max().unwrap());
        }
        let s = city.stats();
        assert!(s.deliveries > 500, "deliveries {}", s.deliveries);
        assert!(max_load <= CORD_CAPACITY, "max load {}", max_load);
    }

    #[test]
    fn city_is_deterministic() {
        let a = {
            let mut c = City::new(grown(), 1000, &[(60, 12), (130, 12)], 3);
            for _ in 0..150 {
                c.step();
            }
            (c.sim.digest(), c.deliveries, c.pos.iter().map(|&p| p as u64).sum::<u64>())
        };
        let b = {
            let mut c = City::new(grown(), 1000, &[(60, 12), (130, 12)], 3);
            for _ in 0..150 {
                c.step();
            }
            (c.sim.digest(), c.deliveries, c.pos.iter().map(|&p| p as u64).sum::<u64>())
        };
        assert_eq!(a, b);
    }
}
