//! The whole city owned in one place, headless: what `rose city` plays and the tests drive. The game's bridge holds
//! the same parts in its Godot objects and calls the same functions.

use bench::{BenchQueue, RecipeFile};
use colony::{Colony, LawFile, StructureFile};
use inventory::{Ledger, Stash};
use items::{ItemInstance, ItemTable};
use logistics::City;
use people::{NameFile, Record, Roster, RunReport, TraitFile};

use crate::log::CityLog;
use crate::shift::{self, Folk, Shift, Store, Town};
use crate::{alerts, cargo};

/// The founding Stash: what the city starts with (Full Design, the starting haul), each at its condition.
pub const STARTING_HAUL: [(&str, u8); 14] = [
    ("chr_street_eye", 82),
    ("prt_pipe", 71),
    ("ger_nail_rifle", 64),
    ("org_paste", 90),
    ("cur_ooze_toy", 77),
    ("chr_lamp_core", 68),
    ("prt_plate", 85),
    ("tec_shard_corpo", 73),
    ("ger_pipe", 66),
    ("ger_lantern", 79),
    ("ger_pry_bar", 88),
    ("org_antiseptic", 95),
    ("prt_salt", 92),
    ("org_stim", 70),
];

/// Put the starting haul into a Stash.
pub fn starting_haul(stash: &mut Stash, table: &ItemTable) {
    for (id, cond) in STARTING_HAUL {
        if let Some(idx) = table.id_of(id) {
            let mut it = ItemInstance::new(idx);
            it.condition = cond;
            let _ = stash.grid.auto_place(table.def(idx), it);
        }
    }
}

pub struct World {
    pub city: City,
    pub colony: Colony,
    pub defs: StructureFile,
    pub laws: LawFile,
    pub stash: Stash,
    pub ledger: Ledger,
    pub bench: BenchQueue,
    pub table: ItemTable,
    pub recipes: RecipeFile,
    pub roster: Roster,
    pub traits: TraitFile,
    pub names: NameFile,
    pub died: Vec<Record>,
    pub log: CityLog,
    pub day: u32,
    pub jam_since: Option<u64>,
    pub last: Option<colony::ShiftOutcome>,
}

impl World {
    /// The founding (Full Design, The founding): the Sump with the Heart's Mat fifteen tiles out, the city on it,
    /// the roster, the starting haul.
    pub fn found(seed: u64, table: ItemTable) -> World {
        let city = City::game(logistics::founding(seed), seed);
        let defs = StructureFile::shipped();
        let mut colony = Colony::new();
        colony.found(&defs, &city.sim).expect("the founding stratum takes the city");
        let traits = TraitFile::shipped();
        let names = NameFile::shipped();
        let mut roster = Roster::new();
        roster.found(&traits, &names, seed, 1);
        let mut stash = Stash::new();
        starting_haul(&mut stash, &table);
        let mut w = World {
            city,
            colony,
            defs,
            laws: LawFile::shipped(),
            stash,
            ledger: Ledger::new(),
            bench: BenchQueue::new(),
            table,
            recipes: RecipeFile::shipped(),
            roster,
            traits,
            names,
            died: Vec::new(),
            log: CityLog::default(),
            day: 1,
            jam_since: None,
            last: None,
        };
        w.publish();
        w.movers();
        w
    }

    /// Publish the consumers (after building, a quota set, a stockpile painted).
    pub fn publish(&mut self) {
        let c = cargo::consumers(&self.colony, &self.defs, &self.city, &self.stash, &self.ledger, &self.table);
        self.city.set_consumers(c);
    }

    pub fn movers(&mut self) {
        shift::refresh_movers(&mut self.city, &self.colony, &self.defs, &self.roster, &self.traits, self.day);
    }

    /// One city tick: slimos walk, cargo rides and lands, the Mat steps on its clock.
    pub fn tick(&mut self) {
        self.city.step();
        self.jam_since = if self.city.last_jam > 0 { Some(self.jam_since.unwrap_or(self.city.tick)) } else { None };
        if self.city.tick.is_multiple_of(logistics::city::REBUILD_EVERY) {
            cargo::refresh_demands(&self.colony, &self.defs, &mut self.city, &self.stash, &self.ledger, &self.table);
        }
        cargo::land(&mut self.city, &mut self.colony, &self.defs, &mut self.stash, &mut self.ledger, &self.table, &mut self.log);
    }

    /// The run comes home: the roster takes it, the rig goes into the Stash.
    pub fn run_home(&mut self, report: &RunReport, haul: Vec<ItemInstance>) -> (Vec<String>, usize, usize) {
        let mut folk = Folk { roster: &mut self.roster, traits: &self.traits, names: &self.names, died: &mut self.died };
        shift::run_home(&mut folk, &mut self.stash, &self.table, report, haul)
    }

    /// The shift ends; the next one's plan goes to the movers.
    pub fn end_shift(&mut self) -> Shift {
        let town = Town { city: &mut self.city, colony: &mut self.colony, defs: &self.defs };
        let store = Store { stash: &mut self.stash, ledger: &mut self.ledger, bench: &mut self.bench, table: &self.table, recipes: &self.recipes };
        let folk = Folk { roster: &mut self.roster, traits: &self.traits, names: &self.names, died: &mut self.died };
        let s = shift::end_shift(town, store, folk, self.day, &mut self.log);
        self.last = Some(s.outcome.clone());
        self.day += 1;
        self.movers();
        s
    }

    pub fn alerts(&self) -> Vec<alerts::Alert> {
        alerts::current(&self.colony, &self.defs, &self.city, &self.roster, &self.stash, &self.ledger, self.last.as_ref(), self.jam_since)
    }
}
