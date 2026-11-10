//! The city on the Mat (Full Design v3.11, Colony mode: The city's clock, Cargo). The city ticks; the Mat steps every
//! `mat_every` ticks; cargo packets ride the Mat one tile a tick toward their consumer along a signal-weighted field,
//! four to a tile; the roster's slimos walk between bed and work. The scale proof's synthetic crowd and market hauls
//! (Phase A) run only when the city is `synthetic` (`rose scale`, the load test of these arrays).

use std::collections::HashMap;

use mat::flow::FlowField;
use mat::rng::Pcg32;
use mat::sim::{is_hyphal, sub, MatSim};

use crate::field::{Field, UNREACHED};

pub mod state {
    pub const HOME: u8 = 0;
    /// the scale proof's crowd
    pub const TO_MARKET: u8 = 1;
    pub const TO_PIT: u8 = 2;
    pub const TO_HOME: u8 = 3;
    /// the roster's movers
    pub const TO_WORK: u8 = 4;
    pub const AT_WORK: u8 = 5;
}

/// Packets per tile before it slows to half speed and takes no more.
pub const CORD_CAPACITY: u8 = 4;
/// Ticks between field rebuilds (Pass 2: every 20 ticks from the Mat's current shape).
pub const REBUILD_EVERY: u64 = 20;
/// Ticks per Mat step in the game: 125 steps in a 12.5-minute shift at 2 Hz (research 16).
pub const MAT_EVERY: u64 = 12;
/// A slimo walks a tile every two ticks.
pub const WALK_EVERY: u64 = 2;
/// A market is a grate with Mat this close (the Seep Market's rule).
pub const MARKET_REACH: usize = 2;
/// The scale proof's Pit target and synthetic tier nutrients.
pub const PIT_TARGET: f32 = 40.0;
pub const TIER_NUTRIENT: [f32; 4] = [0.0, 0.18, 0.30, 0.45];

/// What a consumer takes (Full Design: Consumers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Pit,
    Market,
    Still,
    /// the Stash, at the Heart
    Heart,
    /// a stockpile (its zone id), at its store's anchor: the Stash's settling
    Room,
}

impl Kind {
    pub fn code(self) -> u8 {
        match self {
            Kind::Pit => 0,
            Kind::Market => 1,
            Kind::Still => 2,
            Kind::Heart => 3,
            Kind::Room => 4,
        }
    }
}

/// A consumer the bridge registers: its kind and key (Pit index, market index, Still structure id), the Mat cell it
/// takes cargo at, its demand (0–1, the share of its quota not met) and whether it lost signal priority.
#[derive(Clone, Debug, PartialEq)]
pub struct Consumer {
    pub kind: Kind,
    pub key: u32,
    pub cell: usize,
    pub demand: f32,
    pub low: bool,
}

/// A packet that reached its consumer this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arrival {
    pub cargo: u32,
    pub kind: Kind,
    pub key: u32,
}

/// A packet waiting at its source tile for room on the cord.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pending {
    at: usize,
    cargo: u32,
    kind: Kind,
    key: u32,
}

/// One of the roster on the map: its bed and, while its job has it, its work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mover {
    pub id: u32,
    pub home: usize,
    pub work: Option<usize>,
}

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

/// The scale proof's synthetic city (Phase A): pods, a crowd walking market → nearest Pit → home, market hauls.
#[derive(Clone)]
pub struct Crowd {
    pub pods: Vec<usize>,
    pub f_market: FlowField,
    pub f_pod: FlowField,
    pub f_pit_any: FlowField,
    pub dwell: Vec<i16>,
    pub phase: Vec<u8>,
    pub haul_every: u64,
}

#[derive(Clone)]
pub struct City {
    pub sim: MatSim,
    pub rng: Pcg32,
    pub markets: Vec<usize>,
    /// the Mat cell each market takes its cargo at; None while the Mat does not reach its grate
    pub market_cells: Vec<Option<usize>>,
    pub pits: Vec<usize>,
    pub consumers: Vec<Consumer>,
    /// fields toward every consumer cell and the Heart, rebuilt every REBUILD_EVERY ticks
    fields: HashMap<usize, Field>,
    walk_fields: HashMap<usize, FlowField>,
    mat_mask: Vec<bool>,
    passable: Vec<bool>,
    // agents (SoA): the roster's movers, or the scale proof's crowd
    pub ids: Vec<u32>,
    pub pos: Vec<u32>,
    pub state: Vec<u8>,
    pub home: Vec<u32>,
    pub goal: Vec<u32>,
    // packets (SoA)
    pub pk_pos: Vec<u32>,
    pub pk_cargo: Vec<u32>,
    pub pk_kind: Vec<Kind>,
    pub pk_key: Vec<u32>,
    pub pk_cell: Vec<u32>,
    outbox: Vec<Pending>,
    pub load: Vec<u8>,
    /// tiles that were full while cargo waited to enter them last tick: the jams (Pass 2: above capacity)
    pub jam: Vec<bool>,
    jam_now: Vec<bool>,
    arrivals: Vec<Arrival>,
    spills: Vec<u32>,
    pub tick: u64,
    pub deliveries: u64,
    pub hauls: u64,
    pub last_jam: u32,
    pub mat_every: u64,
    pub crowd: Option<Crowd>,
}

fn mask_of(sim: &MatSim) -> (Vec<bool>, Vec<bool>) {
    let mat: Vec<bool> = sim.hy.iter().map(|&h| is_hyphal(h)).collect();
    let passable: Vec<bool> = (0..sim.n.len()).map(|i| mat[i] || sub::COND[sim.sub[i] as usize] > 0.0).collect();
    (mat, passable)
}

impl City {
    /// The game's city on a grown Mat: no crowd, no synthetic hauls; the Mat steps every MAT_EVERY ticks; its
    /// markets are the grates the Mat reaches.
    pub fn game(sim: MatSim, seed: u64) -> Self {
        let mut c = Self::bare(sim, seed);
        c.mat_every = MAT_EVERY;
        c.pits = c.sim.heart.iter().map(|&(x, y)| y * c.sim.w + x).collect();
        c.refresh_markets();
        c.rebuild_fields();
        c
    }

    fn bare(sim: MatSim, seed: u64) -> Self {
        let rng = Pcg32::new(seed.wrapping_mul(0x9E37_79B9).wrapping_add(0xC1_7F00));
        let (mat_mask, passable) = mask_of(&sim);
        let n = sim.n.len();
        City {
            sim,
            rng,
            markets: Vec::new(),
            market_cells: Vec::new(),
            pits: Vec::new(),
            consumers: Vec::new(),
            fields: HashMap::new(),
            walk_fields: HashMap::new(),
            mat_mask,
            passable,
            ids: Vec::new(),
            pos: Vec::new(),
            state: Vec::new(),
            home: Vec::new(),
            goal: Vec::new(),
            pk_pos: Vec::new(),
            pk_cargo: Vec::new(),
            pk_kind: Vec::new(),
            pk_key: Vec::new(),
            pk_cell: Vec::new(),
            outbox: Vec::new(),
            load: vec![0; n],
            jam: vec![false; n],
            jam_now: vec![false; n],
            arrivals: Vec::new(),
            spills: Vec::new(),
            tick: 0,
            deliveries: 0,
            hauls: 0,
            last_jam: 0,
            mat_every: 1,
            crowd: None,
        }
    }

    /// The scale proof (Phase A): `n_agents` synthetic slimos at random Nodes walking market → nearest Pit → home and
    /// feeding 0.3 on arrival, markets at the Mat nearest each hint landing 12–30 synthetic units every 15 ticks,
    /// routed by demand to the Heart and the Relays. The Mat steps every tick, as the proof measured it.
    pub fn new(sim: MatSim, n_agents: usize, market_hints: &[(usize, usize)], seed: u64) -> Self {
        let mut c = Self::bare(sim, seed);
        let w = c.sim.w;
        let hy_cells: Vec<usize> = (0..c.mat_mask.len()).filter(|&i| c.mat_mask[i]).collect();
        for &(mx, my) in market_hints {
            let mut best = (usize::MAX, 0usize);
            for &i in &hy_cells {
                let d = (i % w).abs_diff(mx) + (i / w).abs_diff(my);
                if d < best.0 {
                    best = (d, i);
                }
            }
            if best.0 <= 20 && !c.markets.contains(&best.1) {
                c.markets.push(best.1);
                c.market_cells.push(Some(best.1));
            }
        }
        if let Some((x, y)) = c.sim.heart {
            c.pits.push(y * w + x);
        }
        for &(x, y) in &c.sim.relays {
            c.pits.push(y * w + x);
        }
        let mut nodes: Vec<usize> = (0..c.mat_mask.len()).filter(|&i| c.sim.hy[i] == mat::sim::hy::NODE).collect();
        c.rng.shuffle(&mut nodes);
        let pods: Vec<usize> = nodes.into_iter().take(60).collect();
        let blank = || FlowField { w, h: 0, dist: vec![], next: vec![] };
        c.crowd = Some(Crowd { pods, f_market: blank(), f_pod: blank(), f_pit_any: blank(), dwell: Vec::new(), phase: Vec::new(), haul_every: 15 });
        c.consumers = c.pits.iter().enumerate().map(|(k, &p)| Consumer { kind: Kind::Pit, key: k as u32, cell: p, demand: 1.0, low: false }).collect();
        c.rebuild_fields();
        c.spawn_crowd(n_agents);
        c
    }

    pub fn synthetic(&self) -> bool {
        self.crowd.is_some()
    }

    fn spawn_crowd(&mut self, n: usize) {
        let Some(cr) = self.crowd.as_mut() else { return };
        if cr.pods.is_empty() {
            return;
        }
        for k in 0..n {
            let home = cr.pods[self.rng.below(cr.pods.len() as u32) as usize] as u32;
            self.ids.push(k as u32);
            self.pos.push(home);
            self.home.push(home);
            self.goal.push(u32::MAX);
            self.state.push(state::HOME);
            cr.dwell.push(self.rng.range(0, 40) as i16);
            cr.phase.push(self.rng.below(2) as u8);
        }
    }

    /// Markets: every grate with Mat within MARKET_REACH. A grate once reached keeps its index (queued sales point
    /// at it); while the Mat does not reach it its cell is None and no cargo goes there.
    pub fn refresh_markets(&mut self) {
        let (w, h) = (self.sim.w, self.sim.h);
        for x in 0..w {
            if self.sim.sub[x] != sub::GRATE {
                continue;
            }
            let mut best: Option<(usize, usize)> = None;
            for y in 0..=MARKET_REACH.min(h - 1) {
                for xx in x.saturating_sub(MARKET_REACH)..=(x + MARKET_REACH).min(w - 1) {
                    let i = y * w + xx;
                    let d = y + xx.abs_diff(x);
                    if d <= MARKET_REACH && self.mat_mask[i] && best.map_or(true, |(bd, _)| d < bd) {
                        best = Some((d, i));
                    }
                }
            }
            let grate = x;
            match self.markets.iter().position(|&m| m == grate) {
                Some(k) => self.market_cells[k] = best.map(|b| b.1),
                None => {
                    if let Some((_, cell)) = best {
                        self.markets.push(grate);
                        self.market_cells.push(Some(cell));
                    }
                }
            }
        }
    }

    /// The consumers the bridge publishes (Pits, Stills); the markets the Mat reaches join them; the fields follow.
    pub fn set_consumers(&mut self, consumers: Vec<Consumer>) {
        self.consumers = consumers;
        self.rebuild_fields();
    }

    /// A grate's number on the stratum, west to east from 1 (the threats' zones and every line of the log name grates so).
    pub fn grate_number(&self, x: usize) -> usize {
        1 + (0..x.min(self.sim.w)).filter(|&c| self.sim.sub[c] == sub::GRATE).count()
    }

    /// The number of the grate a market stands under.
    pub fn market_grate(&self, market: usize) -> usize {
        self.markets.get(market).map_or(market + 1, |&x| self.grate_number(x))
    }

    pub fn heart_cell(&self) -> Option<usize> {
        self.sim.heart.map(|(x, y)| y * self.sim.w + x)
    }

    pub fn rebuild_fields(&mut self) {
        let (w, h) = (self.sim.w, self.sim.h);
        let (mat_mask, passable) = mask_of(&self.sim);
        self.mat_mask = mat_mask;
        self.passable = passable;
        if !self.synthetic() {
            // the markets the Mat reaches are consumers with an order's demand
            self.refresh_markets();
            self.consumers.retain(|c| c.kind != Kind::Market);
            for (k, cell) in self.market_cells.iter().enumerate() {
                if let Some(cell) = cell {
                    self.consumers.push(Consumer { kind: Kind::Market, key: k as u32, cell: *cell, demand: 1.0, low: false });
                }
            }
        }
        let mut cells: Vec<usize> = self.consumers.iter().map(|c| c.cell).collect();
        cells.extend(self.heart_cell());
        cells.sort_unstable();
        cells.dedup();
        self.fields.clear();
        for c in cells {
            self.fields.insert(c, Field::build(w, h, &self.mat_mask, &self.sim.sig, c));
        }
        self.walk_fields.clear();
        if let Some(cr) = self.crowd.as_mut() {
            cr.f_market = FlowField::build(w, h, &self.passable, &self.markets);
            cr.f_pod = FlowField::build(w, h, &self.passable, &cr.pods);
            cr.f_pit_any = FlowField::build(w, h, &self.mat_mask, &self.pits);
        }
    }

    /// Weighted tiles from `from` to a consumer cell, None when it cannot be reached.
    pub fn distance(&self, from: usize, cell: usize) -> Option<f32> {
        let f = self.fields.get(&cell)?;
        f.reachable(from).then(|| f.tiles(from))
    }

    /// Weighted tiles from the Heart to a consumer, for the quotas' pull order.
    pub fn from_heart(&self, kind: Kind, key: u32) -> Option<f32> {
        let heart = self.heart_cell()?;
        let c = self.consumers.iter().find(|c| c.kind == kind && c.key == key)?;
        self.distance(heart, c.cell)
    }

    /// The reachable consumer of a kind with the highest demand per tile of distance from `from`; a low-priority
    /// one only when no other is reachable; ties toward the Heart.
    pub fn choose(&self, kind: Kind, from: usize) -> Option<(u32, usize)> {
        let heart = self.heart_cell();
        let mut best: Option<(bool, f32, f32, u32, usize)> = None;
        for c in self.consumers.iter().filter(|c| c.kind == kind) {
            let Some(d) = self.distance(from, c.cell) else { continue };
            let score = c.demand.max(0.0) / (1.0 + d);
            let to_heart = heart.and_then(|hc| self.distance(hc, c.cell)).unwrap_or(f32::MAX);
            let better = match best {
                None => true,
                Some((low, s, th, _, _)) => {
                    if c.low != low {
                        !c.low
                    } else if (score - s).abs() > 1e-6 {
                        score > s
                    } else {
                        to_heart < th
                    }
                }
            };
            if better {
                best = Some((c.low, score, to_heart, c.key, c.cell));
            }
        }
        best.map(|b| (b.3, b.4))
    }

    fn cell_of(&self, kind: Kind, key: u32) -> Option<usize> {
        if kind == Kind::Heart {
            return self.heart_cell();
        }
        self.consumers.iter().find(|c| c.kind == kind && c.key == key).map(|c| c.cell)
    }

    /// Put a packet on the Mat at `at` (the Heart, a Still) bound for a consumer; it waits there for room.
    pub fn dispatch(&mut self, at: usize, cargo: u32, kind: Kind, key: u32) {
        self.outbox.push(Pending { at, cargo, kind, key });
    }

    pub fn take_arrivals(&mut self) -> Vec<Arrival> {
        std::mem::take(&mut self.arrivals)
    }

    pub fn take_spills(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.spills)
    }

    /// Cargo still waiting at its source or riding, as (cargo, kind, key).
    pub fn in_transit(&self) -> Vec<(u32, Kind, u32)> {
        let mut v: Vec<(u32, Kind, u32)> = self.outbox.iter().map(|p| (p.cargo, p.kind, p.key)).collect();
        for k in 0..self.pk_pos.len() {
            v.push((self.pk_cargo[k], self.pk_kind[k], self.pk_key[k]));
        }
        v
    }

    /// The roster on the map this shift. Slimos already placed keep their tile; new ones start at their bed.
    pub fn set_movers(&mut self, movers: &[Mover]) {
        let old: HashMap<u32, u32> = self.ids.iter().zip(self.pos.iter()).map(|(&i, &p)| (i, p)).collect();
        self.ids.clear();
        self.pos.clear();
        self.state.clear();
        self.home.clear();
        self.goal.clear();
        for m in movers {
            let p = old.get(&m.id).copied().unwrap_or(m.home as u32);
            self.ids.push(m.id);
            self.pos.push(p);
            self.home.push(m.home as u32);
            let goal = m.work.unwrap_or(m.home);
            self.goal.push(goal as u32);
            self.state.push(match (m.work.is_some(), p as usize == goal) {
                (true, true) => state::AT_WORK,
                (true, false) => state::TO_WORK,
                (false, true) => state::HOME,
                (false, false) => state::TO_HOME,
            });
        }
    }

    pub fn step(&mut self) -> CityStats {
        self.tick += 1;
        if self.synthetic() {
            self.step_crowd();
        } else if self.tick % WALK_EVERY == 0 {
            self.step_movers();
        }
        self.recount_load();
        self.release();
        let jammed = self.move_packets();
        if self.tick % self.mat_every == 0 {
            self.sim.step();
        } else {
            self.sim.changed.clear();
        }
        if self.tick % REBUILD_EVERY == 0 {
            self.rebuild_fields();
            self.reroute();
        }
        self.last_jam = jammed;
        self.stats()
    }

    fn step_movers(&mut self) {
        let (w, h) = (self.sim.w, self.sim.h);
        for i in 0..self.pos.len() {
            let goal = self.goal[i] as usize;
            let p = self.pos[i] as usize;
            if p == goal {
                self.state[i] = if self.state[i] == state::TO_WORK || self.state[i] == state::AT_WORK { state::AT_WORK } else { state::HOME };
                continue;
            }
            let passable = &self.passable;
            let f = self.walk_fields.entry(goal).or_insert_with(|| FlowField::build(w, h, passable, &[goal]));
            if f.reachable(p) {
                self.pos[i] = f.next[p];
            }
            // no way there on Mat or floor: they stay where they are
        }
    }

    fn step_crowd(&mut self) {
        let w = self.sim.w;
        let moving = (self.tick % 2) as u8;
        let Some(cr) = self.crowd.as_mut() else { return };
        for i in 0..self.pos.len() {
            if cr.phase[i] != moving {
                continue;
            }
            let p = self.pos[i] as usize;
            self.pos[i] = match self.state[i] {
                state::TO_MARKET => cr.f_market.next[p],
                state::TO_PIT => cr.f_pit_any.next[p],
                state::TO_HOME => cr.f_pod.next[p],
                _ => self.pos[i],
            };
        }
        let mut fed: Vec<usize> = Vec::new();
        for i in 0..self.pos.len() {
            let p = self.pos[i] as usize;
            match self.state[i] {
                state::TO_MARKET => {
                    if cr.f_market.at_source(p) {
                        self.state[i] = state::TO_PIT;
                    } else if !cr.f_market.reachable(p) {
                        self.state[i] = state::TO_HOME;
                    }
                }
                state::TO_PIT => {
                    if cr.f_pit_any.at_source(p) {
                        fed.push(p);
                        self.state[i] = state::TO_HOME;
                    } else if !cr.f_pit_any.reachable(p) {
                        self.state[i] = state::TO_HOME;
                    }
                }
                state::TO_HOME => {
                    if cr.f_pod.at_source(p) || !cr.f_pod.reachable(p) {
                        self.state[i] = state::HOME;
                        self.pos[i] = self.home[i];
                        cr.dwell[i] = self.rng.range(20, 80) as i16;
                    }
                }
                _ => {
                    cr.dwell[i] -= 1;
                    if cr.dwell[i] <= 0 {
                        self.state[i] = state::TO_MARKET;
                    }
                }
            }
        }
        let haul_every = cr.haul_every;
        for p in fed {
            self.sim.feed(p % w, p / w, 0.3);
            self.deliveries += 1;
        }
        // market hauls: 12–30 synthetic units at each market, each a packet to the Pit with the most demand
        if self.tick % haul_every == 0 {
            for c in self.consumers.iter_mut() {
                let stock = *self.sim.pits.get(&(c.cell % w, c.cell / w)).unwrap_or(&0.0);
                c.demand = ((PIT_TARGET - stock) / PIT_TARGET).max(0.0125);
            }
            for mi in 0..self.markets.len() {
                let m = self.markets[mi];
                if let Some((key, _)) = self.choose(Kind::Pit, m) {
                    let k = self.rng.range(12, 30);
                    for _ in 0..k {
                        let tier = [1u8, 1, 1, 1, 2, 2, 3][self.rng.below(7) as usize];
                        self.outbox.push(Pending { at: m, cargo: tier as u32, kind: Kind::Pit, key });
                    }
                    self.hauls += 1;
                }
            }
        }
    }

    fn recount_load(&mut self) {
        for v in self.load.iter_mut() {
            *v = 0;
        }
        for &p in &self.pk_pos {
            let l = &mut self.load[p as usize];
            *l = l.saturating_add(1);
        }
    }

    /// Where a packet at `at` can go: its own consumer if reachable, else the best reachable one of its kind; a
    /// stockpile out of reach sends it to the Heart. None when nothing of its kind can be reached.
    fn reachable_target(&self, at: usize, kind: Kind, key: u32) -> Option<(Kind, u32, usize)> {
        let own = self.cell_of(kind, key).filter(|&c| self.distance(at, c).is_some());
        match (kind, own) {
            (_, Some(c)) => Some((kind, key, c)),
            (Kind::Heart, None) => None,
            (Kind::Room, None) => self.heart_cell().filter(|&h| self.distance(at, h).is_some()).map(|h| (Kind::Heart, 0, h)),
            _ => self.choose(kind, at).map(|(k, c)| (kind, k, c)),
        }
    }

    /// Sources release their waiting cargo onto the Mat while their tile has room, oldest first, each only toward
    /// a consumer it can reach (Full Design v3.12, Cargo). Cargo from the Stash with nothing of its kind in reach
    /// is the Stash's again at once; cargo for the Stash waits at its source, off the Mat, until the Heart can be
    /// reached — so an unreachable consumer never fills the tile everything else leaves from.
    fn release(&mut self) {
        if self.outbox.is_empty() {
            return;
        }
        let mut keep = Vec::new();
        for p in std::mem::take(&mut self.outbox) {
            let mut p = p;
            let cell = match self.reachable_target(p.at, p.kind, p.key) {
                Some((kind, key, cell)) => {
                    p.kind = kind;
                    p.key = key;
                    Some(cell)
                }
                None if p.kind != Kind::Heart => {
                    self.arrivals.push(Arrival { cargo: p.cargo, kind: Kind::Heart, key: 0 });
                    continue;
                }
                None => None,
            };
            let on_mat = self.mat_mask.get(p.at).copied().unwrap_or(false);
            match cell {
                Some(cell) if on_mat && self.load[p.at] < CORD_CAPACITY => {
                    self.pk_pos.push(p.at as u32);
                    self.pk_cargo.push(p.cargo);
                    self.pk_kind.push(p.kind);
                    self.pk_key.push(p.key);
                    self.pk_cell.push(cell as u32);
                    self.load[p.at] += 1;
                }
                _ => {
                    if on_mat && self.load[p.at] >= CORD_CAPACITY {
                        self.jam_now[p.at] = true;
                    }
                    keep.push(p)
                }
            }
        }
        self.outbox = keep;
    }

    /// One tile a tick toward the consumer; a full tile takes no more, and a full tile cargo is waiting to enter is
    /// a jam: its packets move at half speed. Two packets meeting head-on between full tiles trade places, so a
    /// cord carrying both ways jams but never locks (Full Design v3.12, Capacity and jams). Returns the jammed tiles.
    fn move_packets(&mut self) -> u32 {
        let w = self.sim.w;
        let n = self.pk_pos.len();
        let mut keep = Vec::with_capacity(n);
        let mut moved = vec![false; n];
        // packets by the tile they start the tick on, for the head-on trade
        let mut by_tile: HashMap<u32, Vec<usize>> = HashMap::new();
        for (k, &p) in self.pk_pos.iter().enumerate() {
            by_tile.entry(p).or_default().push(k);
        }
        for k in 0..n {
            if moved[k] {
                keep.push(k);
                continue;
            }
            let p = self.pk_pos[k] as usize;
            // the tile stopped being Mat: step onto Mat beside it, or spill
            if !self.mat_mask[p] {
                let side = [1i32, -1, w as i32, -(w as i32)].iter().map(|d| p as i32 + d).find(|&j| j >= 0 && (j as usize) < self.mat_mask.len() && self.mat_mask[j as usize] && self.load[j as usize] < CORD_CAPACITY);
                match side {
                    Some(j) => {
                        self.load[p] = self.load[p].saturating_sub(1);
                        self.load[j as usize] += 1;
                        self.pk_pos[k] = j as u32;
                        keep.push(k);
                    }
                    None => {
                        self.load[p] = self.load[p].saturating_sub(1);
                        self.spills.push(self.pk_cargo[k]);
                    }
                }
                continue;
            }
            let Some((d, nxt)) = self.fields.get(&(self.pk_cell[k] as usize)).map(|f| (f.dist[p], f.next[p] as usize)) else {
                keep.push(k);
                continue;
            };
            if d == 0 {
                self.load[p] = self.load[p].saturating_sub(1);
                self.arrive(k);
                continue;
            }
            if d == UNREACHED {
                keep.push(k);
                continue;
            }
            if self.jam[p] && self.tick % 2 == 1 {
                keep.push(k);
                continue;
            }
            if self.load[nxt] >= CORD_CAPACITY {
                // one coming the other way trades tiles with it
                let other = by_tile.get(&(nxt as u32)).and_then(|v| {
                    v.iter().copied().find(|&j| {
                        j != k && !moved[j] && self.pk_pos[j] as usize == nxt && self.fields.get(&(self.pk_cell[j] as usize)).is_some_and(|f| f.dist[nxt] != 0 && f.dist[nxt] != UNREACHED && f.next[nxt] as usize == p)
                    })
                });
                match other {
                    Some(j) => {
                        self.pk_pos[k] = nxt as u32;
                        self.pk_pos[j] = p as u32;
                        moved[j] = true;
                        moved[k] = true;
                    }
                    None => self.jam_now[nxt] = true,
                }
                keep.push(k);
                continue;
            }
            moved[k] = true;
            self.load[p] = self.load[p].saturating_sub(1);
            self.load[nxt] = self.load[nxt].saturating_add(1);
            self.pk_pos[k] = nxt as u32;
            keep.push(k);
        }
        if keep.len() != self.pk_pos.len() {
            self.pk_pos = keep.iter().map(|&k| self.pk_pos[k]).collect();
            self.pk_cargo = keep.iter().map(|&k| self.pk_cargo[k]).collect();
            self.pk_kind = keep.iter().map(|&k| self.pk_kind[k]).collect();
            self.pk_key = keep.iter().map(|&k| self.pk_key[k]).collect();
            self.pk_cell = keep.iter().map(|&k| self.pk_cell[k]).collect();
        }
        std::mem::swap(&mut self.jam, &mut self.jam_now);
        self.jam_now.iter_mut().for_each(|j| *j = false);
        self.jam.iter().filter(|&&j| j).count() as u32
    }

    fn arrive(&mut self, k: usize) {
        self.deliveries += 1;
        if self.synthetic() {
            let p = self.pk_cell[k] as usize;
            let w = self.sim.w;
            self.sim.feed(p % w, p / w, TIER_NUTRIENT[(self.pk_cargo[k] as usize).min(3)]);
            return;
        }
        self.arrivals.push(Arrival { cargo: self.pk_cargo[k], kind: self.pk_kind[k], key: self.pk_key[k] });
    }

    /// After a rebuild: a packet whose consumer cannot be reached re-chooses the best reachable one of its kind.
    fn reroute(&mut self) {
        for k in 0..self.pk_pos.len() {
            let p = self.pk_pos[k] as usize;
            let reach = self.fields.get(&(self.pk_cell[k] as usize)).is_some_and(|f| f.reachable(p));
            let still_there = self.pk_kind[k] == Kind::Heart || self.consumers.iter().any(|c| c.kind == self.pk_kind[k] && c.key == self.pk_key[k] && c.cell == self.pk_cell[k] as usize);
            if self.pk_kind[k] == Kind::Room && reach {
                continue;
            }
            if reach && still_there {
                continue;
            }
            // cargo for the Stash, or for a stockpile the Mat no longer reaches, goes to the Heart
            if self.pk_kind[k] == Kind::Heart || self.pk_kind[k] == Kind::Room {
                if let Some(h) = self.heart_cell() {
                    self.pk_kind[k] = Kind::Heart;
                    self.pk_key[k] = 0;
                    self.pk_cell[k] = h as u32;
                }
                continue;
            }
            match self.choose(self.pk_kind[k], p) {
                Some((key, cell)) => {
                    self.pk_key[k] = key;
                    self.pk_cell[k] = cell as u32;
                }
                // nothing of its kind in reach: it turns back and is the Stash's when it lands at the Heart
                None => {
                    if let Some(h) = self.heart_cell() {
                        self.pk_kind[k] = Kind::Heart;
                        self.pk_key[k] = 0;
                        self.pk_cell[k] = h as u32;
                    }
                }
            }
        }
    }

    pub fn stats(&self) -> CityStats {
        CityStats {
            tick: self.tick,
            agents: self.pos.len(),
            carrying: if self.synthetic() { self.state.iter().filter(|&&s| s == state::TO_PIT).count() } else { self.state.iter().filter(|&&s| s == state::AT_WORK).count() },
            packets: self.pk_pos.len() + self.outbox.len(),
            deliveries: self.deliveries,
            hauls: self.hauls,
            jammed_tiles: self.last_jam,
        }
    }

    /// Traffic per tile: slimos + 2 × packets. The Stack view draws this as light.
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
    fn the_scale_proof_delivers_and_capacity_holds() {
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
    fn the_scale_proof_is_deterministic() {
        let run = || {
            let mut c = City::new(grown(), 1000, &[(60, 12), (130, 12)], 3);
            for _ in 0..150 {
                c.step();
            }
            (c.sim.digest(), c.deliveries, c.pos.iter().map(|&p| p as u64).sum::<u64>())
        };
        assert_eq!(run(), run());
    }

    fn game_city() -> City {
        let mut c = City::game(grown(), 11);
        let w = c.sim.w;
        let pits: Vec<Consumer> = c.sim.relays.clone().iter().enumerate().map(|(k, &(x, y))| Consumer { kind: Kind::Pit, key: k as u32 + 1, cell: y * w + x, demand: 1.0, low: false }).collect();
        c.set_consumers(pits);
        c
    }

    #[test]
    fn the_game_city_has_no_phantoms_and_its_mat_keeps_its_own_clock() {
        let mut c = game_city();
        assert!(!c.synthetic() && c.pos.is_empty());
        assert!(!c.markets.is_empty(), "the founding Mat reaches a grate");
        for (k, &g) in c.markets.iter().enumerate() {
            assert_eq!(c.sim.sub[g], sub::GRATE, "a market is a grate");
            let cell = c.market_cells[k].expect("reached");
            assert!(c.sim.hy[cell] != 0 && (cell / c.sim.w) <= MARKET_REACH);
        }
        let t0 = c.sim.tick;
        for _ in 0..(MAT_EVERY * 10) {
            c.step();
        }
        assert_eq!(c.sim.tick - t0, 10, "the Mat steps once every {MAT_EVERY} ticks");
        assert_eq!(c.deliveries, 0, "nothing rides that nobody sent");
    }

    #[test]
    fn cargo_goes_by_demand_per_tile_and_arrives_at_its_consumer() {
        let mut c = game_city();
        let heart = c.heart_cell().unwrap();
        // the Relay with the highest demand per tile of distance from the Heart wins
        let best = c.choose(Kind::Pit, heart).unwrap();
        let mut scores: Vec<(f32, u32)> = c.consumers.iter().filter_map(|k| c.distance(heart, k.cell).map(|d| (k.demand / (1.0 + d), k.key))).collect();
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        assert_eq!(best.0, scores[0].1);
        // raise the demand of a farther one far enough and it wins
        let far = scores.last().unwrap().1;
        for k in c.consumers.iter_mut() {
            k.demand = if k.key == far { 1.0 } else { 0.01 };
        }
        assert_eq!(c.choose(Kind::Pit, heart).unwrap().0, far);
        // a packet to it arrives, and only once
        c.dispatch(heart, 7, Kind::Pit, far);
        let mut got = Vec::new();
        for _ in 0..400 {
            c.step();
            got.extend(c.take_arrivals());
        }
        assert_eq!(got, vec![Arrival { cargo: 7, kind: Kind::Pit, key: far }]);
    }

    #[test]
    fn a_trunk_holds_four_to_a_tile_and_jams_behind() {
        let mut c = game_city();
        let heart = c.heart_cell().unwrap();
        let key = c.choose(Kind::Pit, heart).unwrap().0;
        for k in 0..60 {
            c.dispatch(heart, k, Kind::Pit, key);
        }
        let mut max_load = 0;
        let mut jammed = 0;
        let mut landed = 0;
        for _ in 0..600 {
            let s = c.step();
            max_load = max_load.max(*c.load.iter().max().unwrap());
            jammed = jammed.max(s.jammed_tiles);
            landed += c.take_arrivals().len();
        }
        assert!(max_load <= CORD_CAPACITY);
        assert!(jammed >= 1, "sixty at once back up");
        assert_eq!(landed, 60, "and all of them land");
    }

    #[test]
    fn a_severed_consumer_reroutes_its_cargo() {
        let mut c = game_city();
        let heart = c.heart_cell().unwrap();
        let (key, cell) = c.choose(Kind::Pit, heart).unwrap();
        c.dispatch(heart, 1, Kind::Pit, key);
        c.step();
        // the consumer goes away (its Pit is lost); at the next rebuild the packet takes the next best
        c.consumers.retain(|k| k.key != key);
        let mut got = Vec::new();
        for _ in 0..800 {
            c.step();
            got.extend(c.take_arrivals());
        }
        assert_eq!(got.len(), 1);
        assert_ne!(got[0].key, key, "it went elsewhere");
        let _ = cell;
    }

    #[test]
    fn movers_walk_to_work_and_home() {
        let mut c = game_city();
        let heart = c.heart_cell().unwrap();
        let work = c.consumers[0].cell;
        c.set_movers(&[Mover { id: 5, home: heart, work: Some(work) }]);
        for _ in 0..800 {
            c.step();
        }
        assert_eq!(c.pos[0] as usize, work);
        assert_eq!(c.state[0], state::AT_WORK);
        c.set_movers(&[Mover { id: 5, home: heart, work: None }]);
        assert_eq!(c.state[0], state::TO_HOME, "keeps its tile, heads home");
        for _ in 0..800 {
            c.step();
        }
        assert_eq!((c.pos[0] as usize, c.state[0]), (heart, state::HOME));
    }
}
