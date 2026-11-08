//! logistics — the city on the Mat: sites, cargo routing with cord capacity, agent columns.

pub mod city;

pub use city::{City, CityStats};

/// The reference scale scenario: one 192x48 stratum, a vigorous conductive strain, one Heart and seven Relays,
/// `grow_ticks` of feeding, then a city of `agents` slimos with three markets.
pub fn scale_scenario(seed: u64, agents: usize, grow_ticks: u64) -> City {
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
    City::new(sim, agents, &[(60, 12), (130, 12), (96, 38)], seed)
}
