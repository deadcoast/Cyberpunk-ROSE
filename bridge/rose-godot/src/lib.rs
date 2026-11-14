//! GDExtension: exposes the ROSE core to Godot as `RoseSim`.
//! Godot owns the clock and the picture; `RoseSim` owns the one `game::World` the headless proofs play (Phase G,
//! PAC-101), and every number is the World's: the bridge founds it, steps it, ends its shifts and reads it for the
//! screens. `RoseStash` and `RosePeople` are views onto the same World.

use godot::prelude::*;
use logistics::city::state;
use mat::genome::Genome;
use mat::sim::{hy, sub};
use logistics::City;
use mat::MatSim;

mod cargo_bridge;
mod hud_bridge;
mod market_bridge;
mod progress_bridge;
mod story_bridge;
mod colony_bridge;
mod people;
mod profile_bridge;
mod run_bridge;
mod sim_bridge;
mod stash_bridge;

pub use run_bridge::RoseRun;
pub use sim_bridge::RoseSim;
pub use stash_bridge::{RoseItems, RoseStash};
pub(crate) use stash_bridge::{closed_dict, item_dict};


struct RoseExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RoseExtension {
    /// The process's data, installed once as the extension loads and before any class reads it (Phase G, PAC-101):
    /// `data/*.toml` beside the project where it holds a file, else the copies compiled in (an exported game).
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::Scene {
            install_data();
        }
    }
}

/// The data the game reads, and every file of `data/` that did not parse (each also printed as an error).
static DATA_REPORT: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

fn install_data() {
    let dir = godot::classes::ProjectSettings::singleton().globalize_path("res://../data").to_string();
    let (d, errors) = game::data::Data::load(Some(std::path::Path::new(&dir)));
    let mut report: Vec<String> = d.sources.iter().map(|(f, s)| format!("{f} {}", if *s == game::data::Source::Disk { "disk" } else { "compiled" })).collect();
    for e in &errors {
        godot_error!("{e}: the compiled copy stands in");
        report.push(format!("ERROR {e}"));
    }
    if !game::data::install(d) {
        godot_error!("data: installed after a reader took the compiled copies");
        report.push("ERROR data installed late".to_string());
    }
    let _ = DATA_REPORT.set(report);
}

/// A decision for the city's log (Full Design v3.12, The city's log): every verb that changes the city says so.
pub(crate) fn decide(w: &mut game::World, what: String) {
    let t = w.city.tick;
    w.log.push(t, game::Class::Decision, what);
}

/// The process's data (`game::data`): every reader in the bridge goes through it.
pub(crate) fn data() -> &'static game::data::Data {
    game::data::get()
}
