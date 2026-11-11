//! The shift, whole (Full Design: Shift end; Colony mode; v3.12): the run comes home, the colony works and pays,
//! the Stash's side follows, the cargo leaves, the stockpiles settle, the roster takes its work and its contexts.
//! The bridge and the twenty-shift city both call this; no view decides any of it.

use std::collections::HashMap;

use bench::{BenchQueue, RecipeFile};
use colony::{Colony, ShiftOutcome, StructureFile};
use inventory::{Ledger, Stash};
use items::{ItemInstance, ItemTable};
use logistics::City;
use people::{NameFile, Record, Roster, RunReport, ShiftContext, ShiftWork, Skill, Stat, Status, TraitFile};

use crate::cargo;
use crate::log::{CityLog, Class};
use crate::store::{self, Closed};

/// The city and its colony.
pub struct Town<'a> {
    pub city: &'a mut City,
    pub colony: &'a mut Colony,
    pub defs: &'a StructureFile,
}

/// The Stash, its ledger and the Bench.
pub struct Store<'a> {
    pub stash: &'a mut Stash,
    pub ledger: &'a mut Ledger,
    pub bench: &'a mut BenchQueue,
    pub table: &'a ItemTable,
    pub recipes: &'a RecipeFile,
}

/// The roster, its traits, the names newcomers take and the dead since the last shift end (for the survivors' Mood).
pub struct Folk<'a> {
    pub roster: &'a mut Roster,
    pub traits: &'a TraitFile,
    pub names: &'a NameFile,
    pub died: &'a mut Vec<Record>,
}

#[derive(Clone, Debug, Default)]
pub struct Shift {
    pub outcome: ShiftOutcome,
    pub closed: Closed,
    pub stash_lines: Vec<String>,
    pub people_lines: Vec<String>,
    /// items the stockpiles drew or caught, items the haulers shelved
    pub settled: usize,
    pub hauled: u32,
}

/// The shift of the day for a colony day: 0 Dim, 1 Dark, 2 Flood.
pub fn shift_of_day(day: u32) -> u8 {
    ((day.max(1) - 1) % 3) as u8
}

/// The run comes home (before the shift ends): the roster takes the run, the rig's items go into the Stash (what the
/// grid cannot hold lies on the Heart's floor). Returns the record lines, items into the grid, onto the floor.
pub fn run_home(folk: &mut Folk, stash: &mut Stash, table: &ItemTable, report: &RunReport, haul: Vec<ItemInstance>) -> (Vec<String>, usize, usize) {
    let lines = run_report(folk, report);
    let (placed, floor) = store::take_haul(stash, table, haul);
    (lines, placed, floor)
}

/// The roster takes the run: XP, bonds, injuries, the dead to the memorial. Returns the record lines.
pub fn run_report(folk: &mut Folk, report: &RunReport) -> Vec<String> {
    let before: Vec<Record> = std::iter::once(report.runner).chain(report.followers.iter().copied()).filter_map(|id| folk.roster.get(id).cloned()).collect();
    let memorial_before = folk.roster.memorial.len();
    folk.roster.apply_run(folk.traits, report);
    folk.died.extend(folk.roster.memorial[memorial_before..].iter().cloned());
    let mut lines = Vec::new();
    for b in &before {
        match folk.roster.get(b.id) {
            None => lines.push(format!("{} died on the run; to the memorial", b.full_name())),
            Some(a) => {
                for s in Skill::all() {
                    if a.skill(s) > b.skill(s) {
                        lines.push(format!("{}: {} {} to {}", a.full_name(), s.name(), b.skill(s), a.skill(s)));
                    }
                }
                for bd in &a.bonds {
                    if bd.bonded && !b.bonds.iter().any(|x| x.with == bd.with && x.bonded) {
                        let who = folk.roster.get(bd.with).map(|x| x.full_name()).unwrap_or_default();
                        lines.push(format!("{} and {} are bonded", a.full_name(), who));
                    }
                }
                if a.status == Status::Stranded {
                    lines.push(format!("{} is stranded on the route", a.full_name()));
                }
                for (inj, _) in a.injuries.iter().filter(|(i, _)| !b.injuries.iter().any(|(j, _)| j == i)) {
                    lines.push(format!("{} came home {}", a.full_name(), inj.name()));
                }
            }
        }
    }
    lines
}

/// The roster's side of the shift: the assignments become shift jobs, the finished crafts, the sales and the hauls
/// become XP for the slimos in those slots, the clinic treats, every record reads its context, and the dead's Voice
/// reaches the survivors. Returns the record lines.
fn people_side(folk: &mut Folk, o: &ShiftOutcome, sold: u32, crafted: u32, hauls: &[(u32, u32)]) -> Vec<String> {
    let traits = folk.traits;
    let roster = &mut *folk.roster;
    let mut lines: Vec<String> = Vec::new();
    for (slimo, job, _) in &o.assignments {
        if let Some(r) = roster.get_mut(*slimo) {
            if r.is_up() && r.shift_job.is_none() {
                r.shift_job = Some(*job);
                r.status = Status::Working(*job);
            }
        }
    }
    let mut work = ShiftWork { tend_shifts_by: o.work.tend_shifts_by.clone(), identified_by: o.work.identified_by.clone(), hauls_by: hauls.to_vec(), research_by: o.work.research_by.clone(), build_by: o.work.build_by.clone(), ..Default::default() };
    if !o.work.crafts_by.is_empty() {
        for k in 0..crafted as usize {
            work.crafts_by.push(o.work.crafts_by[k % o.work.crafts_by.len()]);
        }
    }
    if !o.sellers.is_empty() && sold > 0 {
        let each = (sold / o.sellers.len() as u32).max(1);
        for s in &o.sellers {
            work.sales_by.push((*s, each));
        }
    }
    // the clinic: each treated patient by a medic in turn (Healer's Hands treats at 2x)
    for (k, patient) in o.treated.iter().enumerate() {
        let medic = o.work.treatments_by.get(k % o.work.treatments_by.len().max(1)).copied();
        let healer = medic.and_then(|m| roster.get(m)).is_some_and(|m| m.has_flag(traits, "treat_x2"));
        if let Some(r) = roster.get_mut(*patient) {
            if r.treat(traits, healer) > 0 {
                if let Some(m) = medic {
                    work.treatments_by.push(m);
                }
                lines.push(format!("{} was treated at the Lung", r.full_name()));
            }
        }
    }
    let before: Vec<Record> = roster.records.clone();
    let died = std::mem::take(folk.died);
    let contexts: HashMap<u32, ShiftContext> = roster
        .records
        .iter()
        .map(|r| {
            let stranded = r.status == Status::Stranded;
            let mut cx = o.contexts.get(&r.id).copied().unwrap_or(ShiftContext { on_mat: !stranded, wet: stranded, asleep: !stranded, ..Default::default() });
            cx.death_voice = roster.death_voice_for(r, &died);
            // a Memorial in the district, or the memorial wall, halves a death's hit (Full Design v3.18)
            if cx.memorial {
                cx.death_voice /= 2;
            }
            (r.id, cx)
        })
        .collect();
    roster.end_shift(traits, &work, |r| contexts.get(&r.id).copied().unwrap_or_default());
    for (b, a) in before.iter().zip(roster.records.iter()) {
        for s in Skill::all() {
            if a.skill(s) > b.skill(s) {
                lines.push(format!("{}: {} {} to {}", a.full_name(), s.name(), b.skill(s), a.skill(s)));
            }
        }
        let (lb, la) = (b.unrest_line(traits), a.unrest_line(traits));
        if la > lb {
            lines.push(format!("{} is {}", a.full_name(), ["content", "ready to strike", "ready to defect", "petitioning"][la.min(3) as usize]));
        }
        if a.stat(Stat::Body) < b.stat(Stat::Body) {
            lines.push(format!("{} is starving: Body {}", a.full_name(), a.stat(Stat::Body)));
        }
    }
    for r in roster.records.iter_mut() {
        if matches!(r.status, Status::Working(_)) {
            r.status = Status::Idle;
        }
        r.shift_job = None;
    }
    lines
}

/// The shift whole. `day` picks the shift of the day. Every landing, loss and the shift's own events go to `log`.
pub fn end_shift(town: Town, st: Store, mut folk: Folk, day: u32, log: &mut CityLog) -> Shift {
    let Town { city, colony, defs } = town;
    let Store { stash, ledger, bench, table, recipes } = st;
    let sod = shift_of_day(day);
    let eddies = ledger.eddies;
    let outcome = {
        let roster = &*folk.roster;
        let traits = folk.traits;
        let mut pay = |c: colony::Cost| store::take_cost(stash, ledger, table, c);
        colony.end_shift(defs, &mut city.sim, roster, traits, sod, eddies, &mut pay)
    };
    let tick = city.tick;
    for &(sid, dark) in &outcome.power {
        if let Some(s) = colony.get(sid) {
            let d = &defs.structure[s.def];
            if dark {
                log.push(tick, Class::Loss, format!("{} at {},{} is dark: its cord carries no power", d.name, s.x, s.y));
            } else {
                log.push(tick, Class::Landing, format!("{} at {},{} has power again", d.name, s.x, s.y));
            }
        }
    }
    // the Stash's side: the Appraisal Table, oil and paste, spoiling
    let mut stash_lines = Vec::new();
    if outcome.identifies > 0 {
        let n = store::identify_n(stash, outcome.identifies as usize);
        if n > 0 {
            stash_lines.push(format!("the Appraisal Table named {n} item{}", if n == 1 { "" } else { "s" }));
        }
    }
    let spend = outcome.upkeep + outcome.paste_eddies;
    if spend > 0 {
        ledger.eddies = ledger.eddies.saturating_sub(spend);
        stash_lines.push(format!("{spend} e for oil and paste"));
    }
    let spoiled = stash.spoil(table);
    if spoiled > 0 {
        stash_lines.push(format!("{spoiled} Organic{} spoiled a little outside a Cold Store", if spoiled == 1 { "" } else { "s" }));
    }
    // the stockpiles as they stand now, then the cargo: products, the queue, the pulls; the haulers; the settling
    store::sync_rooms(stash, colony, defs, city);
    stash_lines.extend(cargo::dispatch_shift(city, colony, defs, stash, ledger, table, &outcome));
    let hauls = store::haul(stash, &outcome.haulers);
    let hauled: u32 = hauls.iter().map(|h| h.1).sum();
    if hauled > 0 {
        stash_lines.push(format!("the haulers shelved {hauled}"));
    }
    let settled = store::settle(city, colony, defs, stash, ledger, table);
    if settled > 0 {
        stash_lines.push(format!("{settled} on the cords to the stockpiles"));
    }
    // the Bench's recipe tiers follow Chrome's and the best Bench standing (Full Design v3.18, Research)
    let bench_tier = colony.structures.iter().filter(|s| s.built && defs.structure[s.def].id == "bench").map(|s| s.tier).max().unwrap_or(0);
    bench.tier_open = colony.progress.tech.recipe_tier(bench_tier);
    let faster = crate::trade::faster(city, colony, defs);
    let closed = store::close(stash, ledger, bench, table, recipes, outcome.bench_slots as usize, &faster);
    // the Heart's tier is the city's; the grid follows it
    stash.set_tier(colony.heart_tier(defs));
    let mut people_lines = people_side(&mut folk, &outcome, closed.report.sold, closed.placed, &hauls);
    // the flush (Recruitment): a batch at a grate when its time comes; beds shelter the waiting, the rest go feral
    let mut rng = mat::rng::Pcg32::new(0xF1A5_0000 ^ ((day as u64) << 16) ^ folk.roster.next_id as u64);
    folk.roster.census = colony.progress.tech.flag("census");
    folk.roster.d4_runs = colony.progress.tech.flag("d4_runs");
    let mut came = folk.roster.flush(folk.traits, folk.names, &mut rng, day);
    // each further grate the Mat reaches keeps its own clock (Full Design v3.18, City tiers)
    let further: Vec<u16> = city.markets.iter().zip(city.market_cells.iter()).skip(1).filter(|(_, c)| c.is_some()).map(|(g, _)| *g as u16).collect();
    came += folk.roster.flush_grates(folk.traits, folk.names, &mut rng, day, &further);
    if came > 0 {
        log.push(tick, Class::Landing, format!("{came} newcomer{} came down a grate", if came == 1 { "" } else { "s" }));
        people_lines.push(format!("{came} newcomer{} came down a grate", if came == 1 { "" } else { "s" }));
    }
    let (beds, taken) = colony.bed_count(defs);
    let reached = city.market_cells.iter().any(|m| m.is_some());
    let (joined, feral) = folk.roster.shelter((beds - taken).max(0) as usize, reached);
    for n in &joined {
        log.push(tick, Class::Landing, format!("{n} joined the city"));
        people_lines.push(format!("{n} joined the city"));
    }
    for n in &feral {
        log.push(tick, Class::Loss, format!("{n} went feral in the drain"));
        people_lines.push(format!("{n} went feral in the drain, never sheltered"));
    }
    if !joined.is_empty() {
        colony.assign_homes(defs, folk.roster, folk.traits, &city.sim);
    }
    // the shift's own events
    for sid in &outcome.built {
        if let Some(s) = colony.get(*sid) {
            log.push(tick, Class::Landing, format!("{} built", defs.structure[s.def].name));
        }
    }
    for sid in &outcome.upgraded {
        if let Some(s) = colony.get(*sid) {
            log.push(tick, Class::Landing, format!("{} upgraded to tier {}", defs.structure[s.def].name, s.tier));
        }
    }
    // mending (Full Design v3.17): a line a structure, the builders' and a foreman's stock's together
    let mut mended: Vec<(u32, i32)> = Vec::new();
    for &(sid, hp) in &outcome.mended {
        match mended.iter_mut().find(|m| m.0 == sid) {
            Some(m) => m.1 += hp,
            None => mended.push((sid, hp)),
        }
    }
    for (sid, hp) in mended {
        if let Some(s) = colony.get(sid) {
            let d = &defs.structure[s.def];
            log.push(tick, Class::Landing, format!("{} at {},{} mended {hp} HP ({}/{})", d.name, s.x, s.y, s.hp, colony.hp_max(defs, s)));
        }
    }
    for did in &outcome.districts_formed {
        if let Some(d) = colony.districts.iter().find(|d| d.id == *did) {
            log.push(tick, Class::Landing, format!("{} is a district", d.name));
        }
    }
    for (did, _) in &outcome.elections {
        if let Some(d) = colony.districts.iter().find(|d| d.id == *did) {
            log.push(tick, Class::Landing, format!("{} elected a foreman", d.name));
        }
    }
    for (law, did) in &outcome.laws_arrived {
        log.push(tick, Class::Landing, format!("{law} reached district {did}"));
    }
    for did in &outcome.strikes {
        log.push(tick, Class::Loss, format!("district {did} on strike"));
    }
    for did in &outcome.ready_to_defect {
        log.push(tick, Class::Loss, format!("district {did} ready to defect"));
    }
    for (x, y) in &outcome.cut {
        log.push(tick, Class::Landing, format!("a cord cut at {x},{y}"));
    }
    if let Some(done) = &outcome.research_done {
        let what = match done {
            progress::Done::Node(id) => format!("researched: {}", progress::TechFile::shipped().get(id).map_or(id.as_str(), |n| n.name.as_str())),
            progress::Done::Opened(b, t) => format!("{} tier {t} is open", progress::tech::BRANCH_NAMES[*b]),
        };
        log.push(tick, Class::Landing, what);
    }
    // the progression: organs, classes, the Strainbook, the collections (Full Design v3.18)
    stash_lines.extend(crate::grow::shift_end(city, colony, defs, stash, table, folk.roster.records.len(), day, log));
    Shift { outcome, closed, stash_lines, people_lines, settled, hauled }
}

/// The roster on the map for shift `day`: the colony's plan for that shift. Returns the plan (who stands Guard where
/// is the threats' to read).
pub fn refresh_movers(city: &mut City, colony: &Colony, defs: &StructureFile, roster: &Roster, traits: &TraitFile, day: u32) -> Vec<(u32, people::Job, u32)> {
    if city.synthetic() {
        return Vec::new();
    }
    let plan = colony.assign_jobs(defs, roster, traits, shift_of_day(day));
    // a tended Pit releases x (1 + Shepherd / 10), Shepherd training x (1 + Shepherd / 5)
    let per = if colony.progress.tech.flag("shepherd_x2") { 5.0 } else { 10.0 };
    city.sim.pit_boost.clear();
    for &(slimo, job, sid) in &plan {
        if job != people::Job::Tend {
            continue;
        }
        let pit = colony.get(sid).is_some_and(|s| matches!(defs.structure[s.def].id.as_str(), "pit" | "heart"));
        if let (true, Some(cell), Some(r)) = (pit, colony.cargo_cell(defs, &city.sim, sid), roster.get(slimo)) {
            let w = city.sim.w;
            city.sim.pit_boost.insert((cell % w, cell / w), 1.0 + r.skill(Skill::Shepherd) as f32 / per);
        }
    }
    let movers = cargo::movers(colony, defs, city, roster, &plan);
    city.set_movers(&movers);
    plan
}
