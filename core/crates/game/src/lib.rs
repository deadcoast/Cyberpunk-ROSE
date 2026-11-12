//! game — the shift whole, headless (Full Design v3.12, Technical architecture): the run home, the colony's shift,
//! the Stash's side, the cargo out and in, the stockpiles settling, the roster's records, the alerts and the city's
//! log. The Godot bridge calls these with its parts; `rose city` owns them all and plays twenty shifts.

pub mod alerts;
pub mod breach;
pub mod campaign;
pub mod cargo;
pub mod city;
pub mod grow;
pub mod handoff;
pub mod log;
pub mod loops;
pub mod nerve;
pub mod pressure;
pub mod shift;
pub mod siege;
pub mod store;
pub mod trade;
pub mod world;

pub use alerts::{Alert, Level};
pub use log::{CityLog, Class, Event};
pub use shift::{end_shift, refresh_movers, run_home, run_report, shift_of_day, Folk, Shift, Store, Town};
pub use world::World;

#[cfg(test)]
mod tests;
