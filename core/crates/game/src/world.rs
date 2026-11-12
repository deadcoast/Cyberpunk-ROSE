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

/// The founding Stash: what the city starts with (Full Design v3.21, The city's clock), each at its condition — a
/// Runner's kit and twenty Parts: two Bunkhouses, the founders' beds and the first batch's at minute four.
pub const STARTING_HAUL: [(&str, u8); 31] = [
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
    ("prt_wire", 80),
    ("prt_bearing", 74),
    ("prt_cloth", 66),
    ("prt_cell_dead", 58),
    ("prt_board_fried", 61),
    ("prt_tarp", 77),
    ("prt_valve", 69),
    ("prt_pipe", 63),
    ("prt_wire", 72),
    ("prt_lens", 55),
    ("prt_bearing", 81),
    ("prt_cloth", 70),
    ("prt_blade_stock", 66),
    ("prt_tarp", 59),
    ("prt_valve", 74),
    ("prt_cell_dead", 62),
    ("prt_board_fried", 68),
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

/// The economy at the founding (Full Design v3.19): a street above every grate of the stack, by stratum, the first
/// grate of the Heart's the Nomads'.
pub fn found_economy(sim: &mat::MatSim, table: &ItemTable, seed: u64) -> economy::Economy {
    let grates: Vec<(u16, u8)> = sim.grates().into_iter().map(|g| (g as u16, sim.stratum_at(g))).collect();
    let first = sim.heart.and_then(|(hx, hy)| logistics::first_grate(sim, hx, hy)).unwrap_or(0) as u16;
    economy::Economy::found(economy::FactionFile::shipped(), table, seed, &grates, first)
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
    /// the campaign (Full Design v3.19, Narrative and Ivan)
    pub campaign: story::Campaign,
}

impl World {
    /// The founding (Full Design, The founding): the Sump with the Heart's Mat fifteen tiles out, the city on it,
    /// the roster, the starting haul.
    pub fn found(seed: u64, table: ItemTable) -> World {
        World::found_with(seed, table, story::Campaign::default(), mat::Genome::default())
    }

    /// New Run (Full Design v3.19, the Game shell): founded from its seed, mode, difficulty and strain code; with a
    /// code, New Game+ carries the profile's laws and Scrapdex.
    pub fn new_run(n: &crate::meta::NewRun, table: ItemTable, profile: &save::Profile) -> World {
        let mut w = World::found_with(n.seed as u64, table, n.campaign(), n.genome());
        if n.code.is_some() {
            crate::meta::carry(profile, &mut w.colony);
        }
        w
    }

    /// The founding of a run in a mode and difficulty, on a genome (New Game+: the strain code's).
    pub fn found_with(seed: u64, table: ItemTable, mut campaign: story::Campaign, genome: mat::Genome) -> World {
        campaign.seed = seed as u32;
        let city = City::game(logistics::founding_with(seed, genome), seed);
        let defs = StructureFile::shipped();
        let mut colony = Colony::new();
        colony.found(&defs, &city.sim).expect("the founding stratum takes the city");
        colony.found_home(&defs, &city.sim);
        let traits = TraitFile::shipped();
        let names = NameFile::shipped();
        let mut roster = Roster::new();
        roster.found(&traits, &names, seed, 1);
        let mut stash = Stash::new();
        starting_haul(&mut stash, &table);
        let mut threats = threat::Threats::new(&city.sim, seed);
        let mut economy = found_economy(&city.sim, &table, seed);
        crate::campaign::found(&campaign, &mut threats, &mut economy);
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
            campaign,
        };
        crate::grow::refresh_mat(&mut w.city, &w.colony);
        crate::ivan::found(&mut w.campaign, &w.city.sim);
        w.campaign.barker = story::Barker::new(seed);
        let notes = w.threats.start_shift(&mut w.city, &mut w.colony, &w.defs, 1, w.threats.is_flood_shift(1));
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
        crate::handoff::tick(&mut self.city, &mut self.colony, self.jam_since, self.day, &mut self.log);
        if self.city.tick.is_multiple_of(logistics::city::REBUILD_EVERY) {
            // the Nerve rules read and act on the cargo field's beat (Full Design v3.18)
            crate::nerve::beat(&self.city, &mut self.colony, &self.defs, Some(&self.threats), &mut self.stash, &mut self.ledger, &self.table, &mut self.log);
            cargo::refresh_demands(&self.colony, &self.defs, &mut self.city, &self.stash, &self.ledger, &self.table);
        }
        let landed = cargo::land(&mut self.city, &mut self.colony, &self.defs, &mut self.stash, &mut self.ledger, &self.table, &mut self.log);
        for (market, contract, item) in landed.delivered.iter().cloned() {
            crate::trade::delivered(&mut self.economy, &mut self.ledger, &mut self.city, &self.table, market, contract, item, &mut self.log);
        }
        crate::ivan::tick(&mut self.campaign, &self.city, &mut self.threats, &mut self.log);
        crate::pressure::tick(&mut self.threats, &mut self.city, &mut self.colony, &self.defs, &self.posts, &mut self.log, &landed);
        self.sales.extend(landed.sold);
        crate::ivan::voice(&mut self.campaign, &self.colony, &self.city.sim, &mut self.threats, &mut self.log, self.city.tick);
        if self.city.tick.is_multiple_of(shift::WORK_TICKS) {
            let (flush_mult, _) = crate::campaign::mults(&self.campaign, &self.city.sim);
            let folk = Folk { roster: &mut self.roster, traits: &self.traits, names: &self.names, died: &mut self.died };
            self.plan = shift::work_cycle(&mut self.city, &mut self.colony, &self.defs, folk, &mut self.stash, &mut self.ledger, &self.table, flush_mult, self.day, &mut self.log);
        }
        if self.city.tick.is_multiple_of(shift::LOOK_TICKS) {
            crate::campaign::onboard(&mut self.campaign, &self.city, &self.colony, &self.defs, &self.roster, &self.ledger, &self.economy, &self.threats);
        }
    }

    /// The run comes home: the roster takes it, the rig goes into the Stash.
    pub fn run_home(&mut self, report: &RunReport, haul: Vec<ItemInstance>) -> (Vec<String>, usize, usize) {
        let name = self.roster.get(report.runner).map(|r| r.full_name()).unwrap_or_default();
        crate::shift::bark_run(&mut self.log, report, &name, !haul.is_empty());
        let mut folk = Folk { roster: &mut self.roster, traits: &self.traits, names: &self.names, died: &mut self.died };
        shift::run_home(&mut folk, &mut self.stash, &self.table, report, haul)
    }

    /// The shift ends; the next one's plan goes to the movers. The route loops run first (Full Design v3.18).
    pub fn end_shift(&mut self) -> Shift {
        let mut folk = Folk { roster: &mut self.roster, traits: &self.traits, names: &self.names, died: &mut self.died };
        let loop_lines = crate::loops::play(&mut self.colony, &mut folk, &mut self.stash, &self.table, Some(&self.threats), &self.city.sim, self.threats.seed, self.day, self.city.tick, &mut self.log);
        let (_, gain_mult) = crate::campaign::mults(&self.campaign, &self.city.sim);
        self.threats.campaign.gain_mult = gain_mult;
        let town = Town { city: &mut self.city, colony: &mut self.colony, defs: &self.defs };
        let store = Store { stash: &mut self.stash, ledger: &mut self.ledger, bench: &mut self.bench, table: &self.table, recipes: &self.recipes };
        let folk = Folk { roster: &mut self.roster, traits: &self.traits, names: &self.names, died: &mut self.died };
        let mut s = shift::end_shift(town, store, folk, self.day, &mut self.log);
        s.people_lines.splice(0..0, loop_lines);
        self.last = Some(s.outcome.clone());
        // the breach and the city's stratum before the threats choose what comes (Full Design v3.19)
        crate::breach::shift_end(&mut self.city, &self.colony, &self.defs, Some(&mut self.threats), &mut self.log);
        self.pressure_end(&s.outcome);
        // the headless city's runs carry no bodies: a recover link here is laid nowhere
        let (lines, _) = crate::trade::end_shift(&mut self.economy, &mut self.ledger, &mut self.stash, &self.table, &mut self.city, &mut self.colony, &mut self.roster, &self.traits, &self.names, Some(&mut self.threats), None, &s.closed.report.to_faction, self.day, self.raiders, &mut self.log);
        s.stash_lines.extend(lines);
        // the campaign reads the city's shift: a chapter's goal, the accord, the Great Cleansing, the ending
        let guards = self.posts.guards.len();
        let mut parts = crate::campaign::Parts { city: &mut self.city, colony: &mut self.colony, defs: &self.defs, threats: &mut self.threats, economy: &mut self.economy, roster: &mut self.roster, died: &mut self.died, stash: &mut self.stash, table: &self.table, log: &mut self.log, guards };
        let story = crate::campaign::shift_end(&mut self.campaign, &mut parts, self.day);
        s.people_lines.extend(story);
        self.day += 1;
        let notes = self.threats.start_shift(&mut self.city, &mut self.colony, &self.defs, self.day, self.threats.is_flood_shift(self.day));
        self.note(notes);
        if self.threats.flood.level > 0 {
            self.log.bark("flood", &[]);
        }
        crate::ivan::shift_end(&mut self.campaign, &mut self.city);
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

impl World {
    /// The parts the campaign moves, for a verb (the accord's answer, the alliance) between shift ends.
    pub fn campaign_parts(&mut self) -> (&mut story::Campaign, crate::campaign::Parts<'_>) {
        let guards = self.posts.guards.len();
        (&mut self.campaign, crate::campaign::Parts { city: &mut self.city, colony: &mut self.colony, defs: &self.defs, threats: &mut self.threats, economy: &mut self.economy, roster: &mut self.roster, died: &mut self.died, stash: &mut self.stash, table: &self.table, log: &mut self.log, guards })
    }
}
