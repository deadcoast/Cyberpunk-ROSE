//! The run. Phase B of the design: authored 32 x 24 chunks (`chunk`), seeded assembly of 8-14 of them into a
//! route (`assemble`), the compiled tile map a run is played on (`map`), and the fields played against it:
//! light (`light`), sight (`fov`), fog of war with route memory (`fog`), noise (`noise`), all ticked together by
//! `senses`; and what a route gives (`loot`: tables, sources, the rig intake). Threats, combat and extraction land in
//! their own modules as PAC-023..024 close.
//!
//! Perspective and traversal. Every ROSE view is a side-on cross-section. Slimos are ooze: they cling to and
//! crawl along any surface, so a Runner moves in four directions through every passable tile (open floor and air,
//! water, grates) and shafts need no ladders. Gravity is a rule for dropped items and bodies, not for slimos.
//! Collapse and deposit tiles are solid until dug.

pub mod assemble;
pub mod chunk;
pub mod creature;
pub mod fog;
pub mod fov;
pub mod light;
pub mod loot;
pub mod map;
pub mod noise;
pub mod runner;
pub mod senses;
pub mod sim;

pub use assemble::{assemble, route, Layout, Placed, MAX_CHUNKS, MIN_CHUNKS};
pub use chunk::{Chunk, ChunkLib, Dir, CHUNK_H, CHUNK_W};
pub use creature::{Creature, CreatureFile, Kind, Phase};
pub use fog::Fog;
pub use light::{LightField, Source};
pub use loot::{LootState, LootTables, OpenMethod, OpenResult, SourceKind, SourceState};
pub use map::{Container, ContainerKind, Hazard, HazardKind, Light, RouteMap, Spawn, SpawnKind};
pub use noise::{NoiseEvent, NoiseField, Sound};
pub use runner::{Action, Life, Runner, StatusKind};
pub use sim::{Event, RunInput, RunResult, RunSetup, RunSim};
pub use senses::{notice_range, Lume, Motion, Senses};

/// The authored chunk library shipped with the game (data/chunks.toml). Tools and tests use it; the game may
/// load a newer file from disk and fall back to this.
pub const CHUNKS_TOML: &str = include_str!("../../../../data/chunks.toml");
