//! logistics — the city on the Mat: the city's clock, consumers and their signal-weighted fields, cargo packets with
//! cord capacity, demand routing and re-routing, the roster's movers (Full Design v3.11, Colony mode: Cargo).

pub mod city;
pub mod field;

pub use city::{Arrival, City, CityStats, Consumer, Kind, Mover};

/// The founding stratum (and the scale proof's): one 192x48 stratum, a vigorous conductive strain, one Heart and
/// seven Relays, `grow_ticks` of feeding. The Mat the city is founded on (Full Design: The founding).
pub fn founding_mat(seed: u64, grow_ticks: u64) -> mat::MatSim {
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
    City::new(founding_mat(seed, grow_ticks), agents, &[(60, 12), (130, 12), (96, 38)], seed)
}
