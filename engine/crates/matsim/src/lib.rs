//! matsim — the Cyberpunk: ROSE core.
//!
//! * `sim`    the Mat: stochastic cellular automaton + nutrient field (the floor the city stands on)
//! * `flow`   multi-source flow fields for thousands of movers
//! * `city`   sites, cargo packets with cord capacity and demand routing, agent columns
//! * `render` headless Stack / Street / Mat renders in the SEEPAGE palette
//!
//! Everything is deterministic from (seed, inputs): one PCG32 stream per simulation.

pub mod city;
pub mod flow;
pub mod genome;
pub mod render;
pub mod rng;
pub mod sim;

pub use city::{City, CityStats};
pub use genome::{Genome, Params};
pub use sim::{MatSim, Stats};

/// The reference scale scenario: one 192x48 stratum, a vigorous conductive strain, one Heart and seven Relays,
/// 500 ticks of feeding, then a city of `agents` slimos with three markets.
pub fn scale_scenario(seed: u64, agents: usize, grow_ticks: u64) -> City {
    use genome::{CONDUCTANCE, VIGOR};
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
