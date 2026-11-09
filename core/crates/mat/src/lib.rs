//! mat — the Mat: a stochastic cellular automaton coupled to a nutrient field.
//!
//! * `sim`    the CA, nutrient transport, signal BFS, relays, pits, player verbs
//! * `flow`   multi-source flow fields (shared with logistics and run)
//! * `genome` ten genes and their expression into parameters
//! * `rng`    PCG32, one stream per simulation
//!
//! Deterministic from (seed, inputs). No engine dependency.

pub mod flow;
pub mod genome;
pub mod rng;
pub mod sim;

pub use genome::{Genome, Params};
pub use sim::{MatSim, Stats};
