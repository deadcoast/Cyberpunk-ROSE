//! Saves and the Board's routes (Full Design: Saves; the Route Board; Death and recovery). A save is the World's
//! snapshot with a header and the shell's view state (`game::World::save_file`); the shell writes the bytes to its
//! slots and reads them back. The routes' runs, fog memory, bodies and stranded followers are the World's
//! (`game::routes`), so a save carries them and every screen reads them here.

use godot::prelude::*;

use crate::RoseSim;

fn header_dict(f: &save::SaveFile) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("why", "");
    d.set("seed", f.seed as i64);
    d.set("day", f.day as i64);
    d.set("tick", f.tick as i64);
    d.set("label", f.label.as_str());
    d.set("created_unix", f.created_unix as i64);
    d.set("saved_unix", f.saved_unix as i64);
    d.set("strain_code", f.strain_code.clone().unwrap_or_default().as_str());
    d.set("difficulty", f.difficulty as i64);
    d
}

fn route_key(stratum: i64, seed: i64) -> (u8, u64) {
    (stratum.clamp(0, 255) as u8, seed as u64)
}

#[godot_api(secondary)]
impl RoseSim {
    /// The city as a save file's bytes: the World's snapshot, the header, `ui` (the shell's view state) beside it.
    /// Empty when there is no city.
    #[func]
    fn save_bytes(&self, ui: PackedByteArray, created_unix: i64, saved_unix: i64) -> PackedByteArray {
        let Some(w) = self.world.as_ref() else { return PackedByteArray::new() };
        PackedByteArray::from(w.save_file(ui.to_vec(), created_unix.max(0) as u64, saved_unix.max(0) as u64).encode().as_slice())
    }

    /// What a save file says of itself without loading it: {why ("" or why it does not read), seed, day, tick, label,
    /// created_unix, saved_unix, strain_code, difficulty}.
    #[func]
    fn save_header(bytes: PackedByteArray) -> VarDictionary {
        match save::SaveFile::decode(bytes.as_slice()) {
            Ok(f) => header_dict(&f),
            Err(e) => {
                let mut d = VarDictionary::new();
                d.set("why", e.to_string().as_str());
                d
            }
        }
    }

    /// Load a save: the World restored in place of whatever city this held. {why: "" or why not, and the header's
    /// fields, ui: the shell's view state as it was saved}. A save that does not read changes nothing.
    #[func]
    fn load_bytes(&mut self, bytes: PackedByteArray) -> VarDictionary {
        let f = match save::SaveFile::decode(bytes.as_slice()) {
            Ok(f) => f,
            Err(e) => {
                let mut d = VarDictionary::new();
                d.set("why", e.to_string().as_str());
                return d;
            }
        };
        match game::World::restore(&f.world, crate::data().items.clone()) {
            Ok(w) => {
                self.world = Some(w);
                let mut d = header_dict(&f);
                d.set("ui", &PackedByteArray::from(f.ui.as_slice()));
                d
            }
            Err(e) => {
                let mut d = VarDictionary::new();
                d.set("why", e.as_str());
                d
            }
        }
    }

    /// The city's day: the shift it is in (the World's own count, which a save carries).
    #[func]
    fn day(&self) -> i64 {
        self.world.as_ref().map_or(1, |w| w.day as i64)
    }

    // ------------------------------------------------------------ the Board's routes (game::routes)

    /// Runs a route has had (first-run generosity reads 0).
    #[func]
    fn route_runs(&self, stratum: i64, seed: i64) -> i64 {
        let (s, k) = route_key(stratum, seed);
        self.world.as_ref().map_or(0, |w| w.routes.runs_of(s, k) as i64)
    }

    #[func]
    fn note_route_run(&mut self, stratum: i64, seed: i64) {
        let (s, k) = route_key(stratum, seed);
        if let Some(w) = self.world.as_mut() {
            w.routes.note_run(s, k);
        }
    }

    /// What a route's fog remembers, one bit a tile (empty: never run).
    #[func]
    fn route_memory(&self, stratum: i64, seed: i64) -> PackedByteArray {
        let (s, k) = route_key(stratum, seed);
        self.world.as_ref().map_or(PackedByteArray::new(), |w| PackedByteArray::from(w.routes.memory_of(s, k)))
    }

    #[func]
    fn remember_route(&mut self, stratum: i64, seed: i64, bits: PackedByteArray) {
        let (s, k) = route_key(stratum, seed);
        if let Some(w) = self.world.as_mut() {
            w.routes.remember(s, k, bits.to_vec());
        }
    }

    /// The bodies on a route: [{seed, stratum, x, y, chunk, day, runner, rig: bytes, raided, contract (-1 ours)}].
    #[func]
    fn bodies_on(&self, stratum: i64, seed: i64) -> VarArray {
        let (s, k) = route_key(stratum, seed);
        let mut out = VarArray::new();
        let Some(w) = self.world.as_ref() else { return out };
        for b in w.routes.bodies_on(s, k) {
            let mut d = VarDictionary::new();
            d.set("seed", b.seed as i64);
            d.set("stratum", b.stratum as i64);
            d.set("x", b.x as i64);
            d.set("y", b.y as i64);
            d.set("chunk", b.chunk.as_str());
            d.set("day", b.day as i64);
            d.set("runner", b.runner.as_str());
            d.set("rig", &PackedByteArray::from(b.rig.as_slice()));
            d.set("raided", b.raided);
            d.set("contract", b.contract.map_or(-1, |c| c as i64));
            out.push(&d.to_variant());
        }
        out
    }

    /// One of ours fell on a route: its body joins the route ({x, y, chunk, day, rig: bytes, raided}, the Runner's name).
    #[func]
    fn add_body(&mut self, stratum: i64, seed: i64, body: VarDictionary, runner: GString) {
        let (s, k) = route_key(stratum, seed);
        let Some(w) = self.world.as_mut() else { return };
        let get_i = |key: &str| body.get(key).and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0);
        let rig: PackedByteArray = body.get("rig").and_then(|v| v.try_to::<PackedByteArray>().ok()).unwrap_or_default();
        let chunk: GString = body.get("chunk").and_then(|v| v.try_to::<GString>().ok()).unwrap_or_default();
        let raided = body.get("raided").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);
        w.routes.bodies.push(game::routes::Body { seed: k, stratum: s, x: get_i("x").clamp(0, u16::MAX as i64) as u16, y: get_i("y").clamp(0, u16::MAX as i64) as u16, chunk: chunk.to_string(), day: get_i("day").max(0) as u32, runner: runner.to_string(), rig: rig.to_vec(), raided, contract: None });
    }

    /// The followers left on a route.
    #[func]
    fn stranded_on(&self, stratum: i64, seed: i64) -> PackedInt32Array {
        let (s, k) = route_key(stratum, seed);
        self.world.as_ref().map_or(PackedInt32Array::new(), |w| PackedInt32Array::from(w.routes.stranded_on(s, k).iter().map(|&i| i as i32).collect::<Vec<_>>().as_slice()))
    }

    #[func]
    fn strand(&mut self, stratum: i64, seed: i64, ids: PackedInt32Array) {
        let (s, k) = route_key(stratum, seed);
        if let Some(w) = self.world.as_mut() {
            w.routes.strand(s, k, ids.as_slice().iter().filter(|&&i| i >= 0).map(|&i| i as u32).collect());
        }
    }
}
