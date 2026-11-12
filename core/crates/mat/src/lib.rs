//! mat — the Mat: a stochastic cellular automaton coupled to a nutrient field.
//!
//! * `sim`    the CA, nutrient transport, signal BFS, relays, pits, player verbs
//! * `flow`   multi-source flow fields (shared with logistics and run)
//! * `genome` ten genes and their expression into parameters
//! * `organ`  organs: where they grow, the growth, what the step reads of them
//! * `rng`    PCG32, one stream per simulation
//!
//! Deterministic from (seed, inputs). No engine dependency.

pub mod flow;
pub mod genome;
pub mod organ;
pub mod rng;
pub mod sim;

pub use genome::{Genome, Params};
pub use organ::{Organ, OrganDef, Site};
pub use sim::{MatSim, Mods, Stats, StratumGen};
