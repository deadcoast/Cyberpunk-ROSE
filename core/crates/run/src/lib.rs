//! The run. Phase B of the design: authored 32 x 24 chunks (`chunk`), seeded assembly of 8-14 of them into a
//! route (`assemble`), and the compiled tile map a run is played on (`map`). Fog, light, noise, loot, threats,
//! combat and extraction land in their own modules as PAC-021..024 close.
//!
//! Perspective and traversal. Every ROSE view is a side-on cross-section. Slimos are ooze: they cling to and
//! crawl along any surface, so a Runner moves in four directions through every passable tile (open floor and air,
//! water, grates) and shafts need no ladders. Gravity is a rule for dropped items and bodies, not for slimos.
//! Collapse and deposit tiles are solid until dug.

pub mod assemble;
pub mod chunk;
pub mod map;

pub use assemble::{assemble, route, Layout, Placed, MAX_CHUNKS, MIN_CHUNKS};
pub use chunk::{Chunk, ChunkLib, Dir, CHUNK_H, CHUNK_W};
pub use map::{Container, ContainerKind, Hazard, HazardKind, Light, RouteMap, Spawn, SpawnKind};

/// The authored chunk library shipped with the game (data/chunks.toml). Tools and tests use it; the game may
/// load a newer file from disk and fall back to this.
pub const CHUNKS_TOML: &str = include_str!("../../../../data/chunks.toml");
