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
    /// the surface's answer (Full Design v3.14, Pressure)
    pub threats: threat::Threats,
    /// who stands Guard and tends the Pens this shift (the shift's plan)
    pub posts: threat::Posts,
    /// Oathkeepers a run killed this shift: their Attention counts in the Heart's zone
    pub run_kills: u32,
    /// the surface's factions (Full Design v3.15, Economy)
    pub economy: economy::Economy,
    /// the shift's plan: (slimo, job, structure)
    pub plan: Vec<(u32, people::Job, u32)>,
    /// raiders walk the Board's routes (clear-raiders contracts are offered); the headless city's Board is S1's
    pub raiders: bool,
    /// the sales landed since the caller last took them (`rose economy`'s report)
    pub sales: Vec<cargo::Sale>,
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
        let threats = threat::Threats::new(&city.sim, seed);
        let grates: Vec<u16> = (0..city.sim.w).filter(|&x| city.sim.sub[x] == mat::sim::sub::GRATE).map(|x| x as u16).collect();
        let first = city.sim.heart.and_then(|(hx, hy)| logistics::first_grate(&city.sim, hx, hy)).unwrap_or(0) as u16;
        let economy = economy::Economy::found(economy::FactionFile::shipped(), &table, seed, &grates, first, 1);
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
            threats,
            posts: threat::Posts::default(),
            run_kills: 0,
            economy,
            plan: Vec::new(),
            raiders: crate::trade::raiders_walk(1),
            sales: Vec::new(),
        };
        crate::grow::refresh_mat(&mut w.city, &w.colony);
        let notes = w.threats.start_shift(&mut w.city, &mut w.colony, &w.defs, 1, shift::shift_of_day(1) == 2);
        w.note(notes);
        w.publish();
        w.movers();
        w.trade_start();
        w
    }

    pub fn note(&mut self, notes: Vec<threat::Note>) {
        crate::pressure::note(&mut self.log, self.city.tick, notes);
    }

    /// Publish the consumers (after building, a quota set, a stockpile painted).
    pub fn publish(&mut self) {
        let c = cargo::consumers(&self.colony, &self.defs, &self.city, &self.stash, &self.ledger, &self.table);
        self.city.set_consumers(c);
    }

    pub fn movers(&mut self) {
        self.plan = shift::refresh_movers(&mut self.city, &self.colony, &self.defs, &self.roster, &self.traits, self.day);
        self.posts = threat::Posts::from_plan(&self.plan, &self.roster);
    }

    /// The surface's side of a shift's start: see `trade::start_shift`.
    fn trade_start(&mut self) {
        crate::trade::start_shift(&mut self.economy, &mut self.ledger, &self.city, &self.colony, &self.defs, &self.roster, &self.names, &self.plan, Some(&self.threats), self.day, self.raiders, &mut self.log);
    }

    /// One city tick: slimos walk, cargo rides and lands, the Mat steps on its clock.
    pub fn tick(&mut self) {
        self.city.step();
        self.jam_since = if self.city.last_jam > 0 { Some(self.jam_since.unwrap_or(self.city.tick)) } else { None };
        if self.city.tick.is_multiple_of(logistics::city::REBUILD_EVERY) {
            cargo::refresh_demands(&self.colony, &self.defs, &mut self.city, &self.stash, &self.ledger, &self.table);
        }
        let landed = cargo::land(&mut self.city, &mut self.colony, &self.defs, &mut self.stash, &mut self.ledger, &self.table, &mut self.log);
        for (market, contract, item) in landed.delivered.iter().cloned() {
            crate::trade::delivered(&mut self.economy, &mut self.ledger, &mut self.city, &self.table, market, contract, item, &mut self.log);
        }
        crate::pressure::tick(&mut self.threats, &mut self.city, &mut self.colony, &self.defs, &self.posts, &mut self.log, &landed);
        self.sales.extend(landed.sold);
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
        self.pressure_end(&s.outcome);
        let mut s = s;
        // the headless city's runs carry no bodies: a recover link here is laid nowhere
        let (lines, _) = crate::trade::end_shift(&mut self.economy, &mut self.ledger, &mut self.stash, &self.table, &mut self.city, &mut self.colony, &mut self.roster, &self.traits, &self.names, Some(&mut self.threats), None, &s.closed.report.to_faction, self.day, self.raiders, &mut self.log);
        s.stash_lines.extend(lines);
        self.day += 1;
        let notes = self.threats.start_shift(&mut self.city, &mut self.colony, &self.defs, self.day, shift::shift_of_day(self.day) == 2);
        self.note(notes);
        self.movers();
        self.trade_start();
        s
    }

    /// The threats' side of the shift's end (Full Design v3.14, Pressure): see `pressure::end`.
    fn pressure_end(&mut self, o: &colony::ShiftOutcome) {
        let kills = std::mem::take(&mut self.run_kills);
        crate::pressure::end(&mut self.threats, &mut self.city, &mut self.colony, &self.defs, &mut self.roster, &mut self.stash, &self.table, &mut self.log, &mut self.died, o, self.day, kills);
    }

    pub fn alerts(&self) -> Vec<alerts::Alert> {
        alerts::current(&self.colony, &self.defs, &self.city, &self.roster, &self.stash, &self.ledger, self.last.as_ref(), self.jam_since, Some(&self.threats), Some(&self.economy))
    }
}
