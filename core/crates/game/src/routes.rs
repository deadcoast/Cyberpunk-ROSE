//! What the city knows of its Board routes between runs (Full Design: the Route Board; Fog; Death and recovery): how
//! many runs each route has had, what its fog remembers, the bodies on it and the followers stranded there. It is the
//! World's, so a save carries it; the game's screens read it through the bridge. A route is (stratum, seed).

use std::collections::BTreeMap;

/// A body on a route: one of ours, or a recover contract's laid for the city to fetch (`contract`).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub seed: u64,
    pub stratum: u8,
    pub x: u16,
    pub y: u16,
    pub chunk: String,
    pub day: u32,
    pub runner: String,
    /// the rig, postcard-encoded (`inventory::Rig`), Spored gear already removed
    pub rig: Vec<u8>,
    pub raided: bool,
    pub contract: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteBook {
    /// runs a route has had: first-run generosity reads 0
    pub runs: BTreeMap<(u8, u64), u16>,
    /// the fog's memory of a route, one bit a tile (`run::Fog::memory_bits`)
    pub memory: BTreeMap<(u8, u64), Vec<u8>>,
    pub bodies: Vec<Body>,
    /// followers left at an extraction tile, by route
    pub stranded: BTreeMap<(u8, u64), Vec<u32>>,
}

impl RouteBook {
    pub fn runs_of(&self, stratum: u8, seed: u64) -> u16 {
        self.runs.get(&(stratum, seed)).copied().unwrap_or(0)
    }

    pub fn note_run(&mut self, stratum: u8, seed: u64) {
        *self.runs.entry((stratum, seed)).or_insert(0) += 1;
    }

    pub fn memory_of(&self, stratum: u8, seed: u64) -> &[u8] {
        self.memory.get(&(stratum, seed)).map_or(&[], |m| m.as_slice())
    }

    /// A run's fog memory replaces the route's (an empty one keeps what was known).
    pub fn remember(&mut self, stratum: u8, seed: u64, bits: Vec<u8>) {
        if !bits.is_empty() {
            self.memory.insert((stratum, seed), bits);
        }
    }

    pub fn bodies_on(&self, stratum: u8, seed: u64) -> impl Iterator<Item = &Body> {
        self.bodies.iter().filter(move |b| b.stratum == stratum && b.seed == seed)
    }

    /// The followers a run left on a route (an empty list keeps the last).
    pub fn strand(&mut self, stratum: u8, seed: u64, ids: Vec<u32>) {
        if !ids.is_empty() {
            self.stranded.insert((stratum, seed), ids);
        }
    }

    pub fn stranded_on(&self, stratum: u8, seed: u64) -> &[u32] {
        self.stranded.get(&(stratum, seed)).map_or(&[], |s| s.as_slice())
    }

    /// Recover contracts' bodies as laid (`trade::Laid`): they join our dead on their route, raiders roll on them as
    /// on ours.
    pub fn lay(&mut self, laid: &[crate::trade::Laid]) {
        for l in laid {
            let rig = postcard::to_allocvec(&l.corpse.rig).unwrap_or_default();
            self.bodies.push(Body { seed: l.route, stratum: l.stratum, x: l.corpse.x, y: l.corpse.y, chunk: l.corpse.chunk.clone(), day: l.corpse.day, runner: l.name.clone(), rig, raided: l.corpse.raided, contract: Some(l.contract) });
        }
    }
}
