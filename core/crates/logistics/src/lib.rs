//! logistics — the city on the Mat: the city's clock, consumers and their signal-weighted fields, cargo packets with
//! cord capacity, demand routing and re-routing, the roster's movers (Full Design v3.11, Colony mode: Cargo).

pub mod city;
pub mod field;

pub use city::{Arrival, City, CityStats, Consumer, Kind, Mover};

/// Six Common (Junk) Chrome at 0.18 nutrient each.
pub const FOUNDING_PIT: f32 = 6.0 * 0.18;
/// The founding's feed a step (Pass 1's tuning rate), FOUNDING_BAIT of it painted ahead of the Mat toward the grate.
pub const FOUNDING_FEED: f32 = 0.6;
pub const FOUNDING_BAIT: f32 = 0.3;
pub const FOUNDING_HEART: (usize, usize) = (96, 25);
/// The founding gives up growing after this many steps (seeds 11-26 reach the grate in 77-439).
pub const FOUNDING_STEPS: usize = 4000;

/// The game's founding (Full Design, The founding): Pass 2's first session to 5:30 — the Heart and a Pit of six Junk
/// Chrome, "a grate visible 40 tiles east; the Mat is 15 tiles out; second Pit, baited east", "the Mat reaches the
/// grate" — which in this game's clock falls inside the first shift's run. The Sump stratum and a vigorous, conductive
/// strain, the Heart at its centre grown from alone on the founding feed, FOUNDING_BAIT of it a step painted on the
/// ground ahead of the Mat nearest the first grate, until the Mat reaches that grate (tissue within
/// city::MARKET_REACH of it, joined to the Heart); then the Heart's Pit holding six Junk Chrome.
pub fn founding(seed: u64) -> mat::MatSim {
    use mat::genome::{CONDUCTANCE, VIGOR};
    use mat::{Genome, MatSim};
    let g = Genome::default().with(VIGOR, 0.8).with(CONDUCTANCE, 0.8);
    let mut sim = MatSim::new(192, 48, seed, g);
    let (hx, hy) = FOUNDING_HEART;
    sim.seed_heart(hx, hy);
    if let Some(gx) = first_grate(&sim, hx, hy) {
        let way = ground_distance(&sim, gx);
        for _ in 0..FOUNDING_STEPS {
            if reaches_grate(&sim, gx) {
                break;
            }
            sim.feed(hx, hy, FOUNDING_FEED - FOUNDING_BAIT);
            if let Some(t) = frontier(&sim, &way).1 {
                sim.n[t] = (sim.n[t] + FOUNDING_BAIT).min(1.0);
            }
            sim.step();
        }
    }
    sim.pits.insert((hx, hy), FOUNDING_PIT);
    sim
}

/// Whether the Mat joined to the Heart has tissue within MARKET_REACH of the grate at column `gx` (the test a market
/// is found by).
pub fn reaches_grate(sim: &mat::MatSim, gx: usize) -> bool {
    let joined = joined_to_heart(sim);
    (0..joined.len()).any(|i| joined[i] && i / sim.w + (i % sim.w).abs_diff(gx) <= city::MARKET_REACH)
}

/// The tissue joined to the Heart along the Mat (tips, cords and nodes).
pub fn joined_to_heart(sim: &mat::MatSim) -> Vec<bool> {
    use mat::sim::hy;
    let mut seen = vec![false; sim.hy.len()];
    let Some((hx, hy_)) = sim.heart else { return seen };
    let w = sim.w;
    let start = hy_ * w + hx;
    seen[start] = true;
    let mut q = std::collections::VecDeque::from([start]);
    while let Some(i) = q.pop_front() {
        let (x, y) = (i % w, i / w);
        for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
            if nx < w && ny < sim.h {
                let j = ny * w + nx;
                if !seen[j] && matches!(sim.hy[j], hy::TIP | hy::CORD | hy::NODE) {
                    seen[j] = true;
                    q.push_back(j);
                }
            }
        }
    }
    seen
}

/// The grate nearest a tile, by columns across and rows up (grates are on the top row).
pub fn first_grate(sim: &mat::MatSim, hx: usize, hy: usize) -> Option<usize> {
    (0..sim.w).filter(|&x| sim.sub[x] == mat::sim::sub::GRATE).min_by_key(|&x| (x.abs_diff(hx) + hy, x))
}

/// Tiles from the grate at column `gx` (top row) to every cell, walked over the ground the Mat can grow on (rock and
/// collapse are not); `usize::MAX` where no way leads.
pub fn ground_distance(sim: &mat::MatSim, gx: usize) -> Vec<usize> {
    let (w, h) = (sim.w, sim.h);
    let mut d = vec![usize::MAX; w * h];
    d[gx] = 0;
    let mut q = std::collections::VecDeque::from([gx]);
    while let Some(i) = q.pop_front() {
        let (x, y) = (i % w, i / w);
        for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
            if nx < w && ny < h {
                let j = ny * w + nx;
                if d[j] == usize::MAX && mat::sim::sub::GROWABLE[sim.sub[j] as usize] {
                    d[j] = d[i] + 1;
                    q.push_back(j);
                }
            }
        }
    }
    d
}

/// The Mat's frontier along `way` (ground_distance to a grate): the least distance of its tissue, and the empty ground
/// the Mat touches nearest the grate.
pub fn frontier(sim: &mat::MatSim, way: &[usize]) -> (usize, Option<usize>) {
    use mat::sim::{hy, sub};
    let w = sim.w;
    let toward = |i: usize| way[i];
    let mut near = usize::MAX;
    let mut ahead: Option<(usize, usize)> = None;
    for i in 0..sim.hy.len() {
        if matches!(sim.hy[i], hy::TIP | hy::CORD | hy::NODE) {
            near = near.min(toward(i));
            continue;
        }
        if sim.hy[i] != hy::EMPTY || !sub::GROWABLE[sim.sub[i] as usize] || sim.salt[i] {
            continue;
        }
        let (x, y) = (i % w, i / w);
        let touches = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)].into_iter().any(|(nx, ny)| nx < w && ny < sim.h && matches!(sim.hy[ny * w + nx], hy::TIP | hy::CORD | hy::NODE));
        if touches && toward(i) != usize::MAX && ahead.is_none_or(|b| (toward(i), i) < b) {
            ahead = Some((toward(i), i));
        }
    }
    (near, ahead.map(|a| a.1))
}

/// The scale proof's stratum: one 192x48 stratum, a vigorous conductive strain, one Heart and seven Relays,
/// `grow_ticks` of feeding (Phase A's load test; `rose scale` only — the game founds on `founding`).
pub fn scale_mat(seed: u64, grow_ticks: u64) -> mat::MatSim {
    use mat::genome::{CONDUCTANCE, VIGOR};
    use mat::{Genome, MatSim};
    let g = Genome::default().with(VIGOR, 0.8).with(CONDUCTANCE, 0.8);
    let mut sim = MatSim::new(192, 48, seed, g);
    sim.seed_heart(96, 25);
    let relays = [(40, 25), (150, 25), (20, 12), (100, 12), (170, 12), (60, 38), (130, 38)];
    for &(x, y) in &relays {
        sim.place_relay(x, y);
    }
    for _ in 0..grow_ticks {
        sim.feed(96, 25, 0.6);
        for &(x, y) in &relays {
            sim.feed(x, y, 0.4);
        }
        sim.step();
    }
    sim
}

/// The reference scale scenario (Phase A's proof): the founding stratum with a synthetic city of `agents` slimos and
/// three markets landing synthetic hauls. `rose scale` and the scale tests only; the game's city is `City::game`.
pub fn scale_scenario(seed: u64, agents: usize, grow_ticks: u64) -> City {
    City::new(scale_mat(seed, grow_ticks), agents, &[(60, 12), (130, 12), (96, 38)], seed)
}
