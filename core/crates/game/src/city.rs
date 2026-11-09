//! `rose city` (Full Design v3.12, The city's log; Build plan, Phase C's proof): the whole game headless for a
//! number of shifts. Each shift the steward gives its standing orders, the eligible slimo of best Scav runs the
//! first route card of S1 as the bot with a rig packed from the Stash, the city ticks through the run at 2 Hz, the
//! haul comes home and the shift ends in core. The steward's orders are this design's stand-in for a player, not a
//! strategy the game recommends. The assertions and the merged run-and-city landing cadence go to the caller.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use colony::{QuotaSet, SIGNAL_FLOOR};
use inventory::{Rig, Verb};
use items::{Category, ItemInstance, ItemTable};
use people::{Injury, RunReport, Skill};
use run::telemetry::Class as RunClass;
use run::{ChunkLib, CreatureFile, LootTables, RouteMap, RunResult, RunSetup};

use crate::log::Class;
use crate::{store, World};

/// The steward's build plan, one ghost at a time at the nearest anchor to the Heart — but the Pit, which goes at
/// the nearest anchor to the first grate (Pass 2's first session: a Pod, then a second Pit baited toward the grate,
/// then the market reached).
pub const BUILD_PLAN: [&str; 8] = ["pod", "pit", "lantern", "rack", "pod", "bunkhouse", "charter_board", "salt_works"];
/// The Rack's stockpile: eight cells around it.
pub const RACK_CELLS: usize = 8;
/// The Heart's Pit and the grate's Pit each kept at 2 nutrient, Uncommon at most.
pub const HEART_KEEP: QuotaSet = QuotaSet { target: 2.0, ceiling: 1 };
/// The kit the bot carries, packed from the Stash where the Stash has it: the pipe in the hand, the lantern, the
/// pry bar, the antiseptic and the salt in the quick slots (run::bot::bot_rig's kit).
pub const KIT_HAND: &str = "ger_pipe";
pub const KIT_QUICK: [&str; 4] = ["ger_lantern", "ger_pry_bar", "org_antiseptic", "prt_salt"];
/// A shift with no one to run lasts the shortest route's clock (8:00) at 2 Hz.
pub const IDLE_SHIFT_TICKS: u64 = 960;
/// The city's clock is 2 Hz, the run's 20 Hz.
pub const RUN_TICKS_PER_CITY_TICK: u64 = 10;

#[derive(Clone, Debug, Default)]
pub struct ShiftRecord {
    pub day: u32,
    pub runner: String,
    pub result: String,
    /// the shift's length in seconds (the run's)
    pub seconds: f32,
    pub run_landings: usize,
    pub city_landings: usize,
    /// the merged run-and-city landing moments' median gap, seconds
    pub merged_gap: f32,
    pub run_gap: f32,
    pub built: Vec<String>,
    pub decisions: Vec<String>,
    pub population: usize,
    pub stash: usize,
    /// on the Heart's floor for want of room in the grid
    pub floor: usize,
    pub in_rooms: usize,
    pub eddies: u32,
    /// the shift's money: earned at the markets, spent on lantern oil and paste
    pub earned: u32,
    pub spent: u32,
    /// meals: from the Mat under the home, from paste
    pub paste_meals: u32,
    /// the Mat: its cells (Tip, Cord, Node) and the nutrient it holds
    pub mat_cells: usize,
    pub mat_nutrient: f32,
    /// shipments of the last shift end still on the Mat when this shift's run ended
    pub late: usize,
    pub starving: usize,
    pub heart_signal: f32,
    pub alerts: Vec<String>,
    pub digest: u64,
}

#[derive(Clone, Debug, Default)]
pub struct CityRun {
    pub seed: u64,
    pub shifts: Vec<ShiftRecord>,
    /// the merged log: run and city events on one clock, a block a shift
    pub log: String,
    pub plan_built: usize,
    pub failures: Vec<String>,
}

/// One moment on the merged clock, seconds from the shift's start.
struct Moment {
    t: f32,
    from: &'static str,
    class: &'static str,
    what: String,
}

/// The nearest anchor to `from` (in tiles walked on the grid, ties by row then column) where the def may stand.
pub fn nearest_spot(w: &World, def: &str, from: (u16, u16)) -> Option<(u16, u16)> {
    let mut best: Option<(u32, u16, u16)> = None;
    for y in 0..w.city.sim.h as u16 {
        for x in 0..w.city.sim.w as u16 {
            let d = (x as i32 - from.0 as i32).unsigned_abs() + (y as i32 - from.1 as i32).unsigned_abs();
            if best.is_some_and(|b| (d, y, x) >= (b.0, b.2, b.1)) {
                continue;
            }
            if w.colony.can_place(&w.defs, &w.city.sim, def, x, y).is_ok() {
                best = Some((d, x, y));
            }
        }
    }
    best.map(|b| (b.1, b.2))
}

fn heart_xy(w: &World) -> (u16, u16) {
    w.city.sim.heart.map(|(x, y)| (x as u16, y as u16)).unwrap_or((0, 0))
}

/// The steward's standing orders, once a shift before the run.
#[derive(Clone, Debug, Default)]
pub struct Steward {
    next: usize,
    current: Option<u32>,
    rack_painted: bool,
    quota_set: bool,
}

impl Steward {
    fn decide(w: &mut World, what: String, out: &mut Vec<String>) {
        w.log.push(w.city.tick, Class::Decision, what.clone());
        out.push(what);
    }

    pub fn act(&mut self, w: &mut World) -> Vec<String> {
        let mut out = Vec::new();
        // the Heart's Pit kept at 2 nutrient, Uncommon at most
        if !self.quota_set {
            if let Some(h) = w.colony.heart(&w.defs).map(|s| s.id) {
                w.colony.set_quota(&w.defs, h, Some(HEART_KEEP));
                Self::decide(w, "the Heart keeps 2.0 nutrient, Uncommon at most".into(), &mut out);
            }
            self.quota_set = true;
        }
        // every Curio and Tech sold at the first grate while the Mat reaches it from the Heart
        if w.city.from_heart(logistics::Kind::Market, 0).is_some() {
            let sell: Vec<u32> = w.stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && matches!(w.table.def(it.def).category, Category::Curio | Category::Tech)).map(|(h, _, _)| h).collect();
            let n = sell.len();
            for h in sell {
                w.ledger.queue(&mut w.stash, h, Verb::Sell { market: 0 }, w.day);
            }
            if n > 0 {
                Self::decide(w, format!("{n} Curio and Tech to sell at the first grate"), &mut out);
            }
        }
        // the build plan, one ghost at a time
        let standing = self.current.and_then(|id| w.colony.get(id)).map(|s| s.built);
        if standing == Some(true) || (self.current.is_some() && standing.is_none()) {
            if standing == Some(true) && BUILD_PLAN[self.next] == "rack" && !self.rack_painted {
                self.paint_rack(w, self.current.unwrap(), &mut out);
            }
            if standing == Some(true) && BUILD_PLAN[self.next] == "pit" {
                let pit = self.current.unwrap();
                w.colony.set_quota(&w.defs, pit, Some(HEART_KEEP));
                Self::decide(w, "the grate's Pit keeps 2.0 nutrient, Uncommon at most".into(), &mut out);
            }
            self.current = None;
            self.next += 1;
        }
        if self.current.is_none() && self.next < BUILD_PLAN.len() {
            let def = BUILD_PLAN[self.next];
            let mw = w.city.sim.w;
            let from = match (def, w.city.market_cells.iter().flatten().next()) {
                ("pit", Some(&g)) => ((g % mw) as u16, (g / mw) as u16),
                _ => heart_xy(w),
            };
            if let Some((x, y)) = nearest_spot(w, def, from) {
                if let Ok(id) = w.colony.place(&w.defs, &w.city.sim, def, x, y) {
                    self.current = Some(id);
                    Self::decide(w, format!("{} placed at {x},{y}", w.defs.get(def).map_or(def, |d| d.name.as_str())), &mut out);
                }
            }
        }
        // every petition answered by placing what it asks, nearest the district's hub
        let asks: Vec<(String, (u16, u16), String)> = w.colony.districts.iter().filter_map(|d| d.petition.as_ref().and_then(|p| p.asks.first().cloned()).map(|a| (a, d.hub, d.name.clone()))).collect();
        for (def, hub, name) in asks {
            let pending = w.colony.structures.iter().any(|s| !s.built && w.defs.structure[s.def].id == def);
            if pending {
                continue;
            }
            if let Some((x, y)) = nearest_spot(w, &def, hub) {
                if w.colony.place(&w.defs, &w.city.sim, &def, x, y).is_ok() {
                    Self::decide(w, format!("{name}'s petition: {} placed at {x},{y}", w.defs.get(&def).map_or(def.as_str(), |d| d.name.as_str())), &mut out);
                }
            }
        }
        w.publish();
        out
    }

    /// The Rack's stockpile: the eight cells nearest its anchor that take a zone.
    fn paint_rack(&mut self, w: &mut World, rack: u32, out: &mut Vec<String>) {
        let Some(r) = w.colony.get(rack).cloned() else { return };
        let (mw, mh) = (w.city.sim.w as i32, w.city.sim.h as i32);
        let mut cells: Vec<(i32, u32)> = Vec::new();
        for dy in -3..=3i32 {
            for dx in -4..=5i32 {
                let (x, y) = (r.x as i32 + dx, r.y as i32 + dy);
                if x < 0 || y < 0 || x >= mw || y >= mh {
                    continue;
                }
                let c = (y * mw + x) as u32;
                if w.colony.occupied(&w.defs, x as u16, y as u16).is_none() && colony::Colony::zone_cell_ok(&w.city.sim, c as usize) && w.colony.zone_at(c).is_none() {
                    cells.push((dx.abs() + dy.abs(), c));
                }
            }
        }
        cells.sort_unstable();
        let pick: Vec<u32> = cells.into_iter().take(RACK_CELLS).map(|c| c.1).collect();
        if let Ok(z) = w.colony.paint_stockpile(&w.defs, &w.city.sim, rack, &pick) {
            self.rack_painted = true;
            Self::decide(w, format!("stockpile {z} painted at the Rack, {} cells", pick.len()), out);
        }
    }

    pub fn plan_built(&self, w: &World) -> usize {
        let mut left: Vec<&str> = BUILD_PLAN.to_vec();
        let mut n = 0;
        for s in w.colony.structures.iter().filter(|s| s.built) {
            if let Some(k) = left.iter().position(|d| *d == w.defs.structure[s.def].id) {
                left.remove(k);
                n += 1;
            }
        }
        n
    }
}

/// Take the first unpinned item of a def from any room of the Stash.
fn take_kit(w: &mut World, id: &str) -> Option<ItemInstance> {
    let idx = w.table.id_of(id)?;
    let h = w.stash.handles().into_iter().find(|(_, _, it)| it.def == idx && !it.pinned).map(|(h, _, _)| h)?;
    w.stash.take(h)
}

/// The rig packed from the Stash for this Runner.
fn pack(w: &mut World, rows: u8, carry: f32) -> Rig {
    let mut rig = Rig::new(8, rows.clamp(3, 8), 4, carry);
    rig.gear.hands[0] = take_kit(w, KIT_HAND);
    for (i, id) in KIT_QUICK.iter().enumerate() {
        rig.quick[i] = take_kit(w, id);
    }
    rig
}

fn digest(w: &World) -> u64 {
    let mut h = DefaultHasher::new();
    w.city.tick.hash(&mut h);
    w.city.sim.hy.hash(&mut h);
    for s in &w.colony.structures {
        (s.id, s.def, s.x, s.y, s.built, s.tier, s.progress, s.paid).hash(&mut h);
    }
    let mut items: Vec<(u32, u8)> = w.stash.handles().into_iter().map(|(_, _, it)| (it.def.0 as u32, it.condition)).collect();
    items.sort_unstable();
    items.hash(&mut h);
    (w.ledger.eddies, w.ledger.in_flight.len(), w.stash.floor.len()).hash(&mut h);
    for r in &w.roster.records {
        (r.id, format!("{:?}", r.status), r.needs, r.priorities).hash(&mut h);
        for s in Skill::all() {
            r.skill(s).hash(&mut h);
        }
    }
    for e in &w.log.events {
        (e.tick, e.class.name(), &e.what).hash(&mut h);
    }
    h.finish()
}

fn median(v: &mut [f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

/// Landing moments a second apart are one (the run's report reads them so), and the gaps between them.
fn gaps(times: &mut [f32]) -> Vec<f32> {
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut moments: Vec<f32> = Vec::new();
    for &t in times.iter() {
        if moments.last().is_none_or(|&m| t - m > 1.0) {
            moments.push(t);
        }
    }
    moments.windows(2).map(|p| p[1] - p[0]).collect()
}

/// Play the city for `shifts` shifts from `seed` (the game's founding: the stratum grown 500 ticks); `each` hears
/// every shift as it ends.
pub fn play(seed: u64, shifts: u32, each: &mut dyn FnMut(usize, &ShiftRecord)) -> CityRun {
    let table = ItemTable::from_toml(items::ITEMS_TOML).expect("items.toml");
    let lib = ChunkLib::from_toml(run::CHUNKS_TOML).expect("chunks.toml");
    let (tables, creatures) = (LootTables::shipped(), CreatureFile::shipped());
    let mut w = World::found(seed, 500, table.clone());
    let mut steward = Steward::default();
    let route_seed = seed * 7 + 1;
    let map = RouteMap::compile(&lib, &run::route(&lib, route_seed, 1).expect("the first route of S1"));
    let mut memory: Vec<u8> = Vec::new();
    let mut runs = 0u16;
    let mut out = CityRun { seed, ..Default::default() };
    for _ in 0..shifts {
        let day = w.day;
        let mut rec = ShiftRecord { day, ..Default::default() };
        let start_tick = w.city.tick;
        let log_from = w.log.events.len();
        rec.decisions = steward.act(&mut w);
        // the Runner: the eligible slimo of best Scav, alone
        let runner = w.roster.eligible().into_iter().max_by_key(|r| (r.skill(Skill::Scav), std::cmp::Reverse(r.id))).map(|r| r.id);
        let riding: Vec<u32> = w.ledger.in_flight.iter().map(|s| s.id).collect();
        let mut moments: Vec<Moment> = Vec::new();
        let mut run_times: Vec<f32> = Vec::new();
        let outcome = runner.map(|id| {
            let r = w.roster.get(id).unwrap();
            let setup = r.runner_setup(&w.traits);
            rec.runner = r.full_name();
            let rig = pack(&mut w, setup.rig_rows, setup.carry_kg());
            w.roster.depart(id, &[]);
            w.movers();
            let lume = w.city.sim.genome.0[mat::genome::LUME];
            let nonce = w.city.tick * 131 + day as u64;
            let mem = if memory.is_empty() { None } else { Some(memory.as_slice()) };
            let rs = RunSetup { mat_lume_gene: lume, memory: mem, run_nonce: nonce, route_runs: runs, day, rig, runner: setup, followers: Vec::new(), corpses: Vec::new() };
            run::bot::play_setup(map.clone(), table.clone(), tables.clone(), creatures.clone(), rs)
        });
        let run_ticks = outcome.as_ref().map_or(IDLE_SHIFT_TICKS * RUN_TICKS_PER_CITY_TICK, |o| o.ticks);
        // the city through the run
        for _ in 0..run_ticks / RUN_TICKS_PER_CITY_TICK {
            w.tick();
        }
        rec.late = w.ledger.in_flight.iter().filter(|s| riding.contains(&s.id)).count();
        // home
        if let (Some(id), Some(o)) = (runner, outcome.as_ref()) {
            runs += 1;
            memory = o.route_memory.clone();
            rec.result = match o.result {
                RunResult::Extracted { .. } => "extracted".into(),
                RunResult::Died { .. } => "died".into(),
            };
            for e in &o.log.entries {
                let t = e.tick as f32 / run::creature::TICK_HZ as f32;
                if e.class == RunClass::Landing {
                    run_times.push(t);
                }
                moments.push(Moment { t, from: "run", class: e.class.tag(), what: format!("{} {}", e.kind, e.detail).trim().to_string() });
            }
            let extracted = matches!(o.result, RunResult::Extracted { .. });
            let report = RunReport {
                runner: id,
                followers: Vec::new(),
                extracted,
                runner_died: !extracted,
                followers_extracted: Vec::new(),
                followers_stranded: Vec::new(),
                followers_dead: Vec::new(),
                pulls: o.report.pulls as u32,
                items_taken: o.items_taken,
                kills: o.kills,
                hits_survived: o.hits_survived,
                follower_strikes: o.follower_strikes,
                injuries: o.injuries.iter().filter_map(|s| Injury::from_status(s)).collect(),
            };
            let mut haul: Vec<ItemInstance> = o.rig.as_ref().filter(|_| extracted).map(|rig| store::rig_items(rig, &w.table)).unwrap_or_default();
            haul.extend(o.mat_haul.iter().cloned());
            haul.extend(o.regrown.iter().cloned());
            w.run_home(&report, haul);
        } else {
            rec.result = "no one to run".into();
        }
        let s = w.end_shift();
        rec.seconds = run_ticks as f32 / run::creature::TICK_HZ as f32;
        // the city's events on the same clock
        let mut city_times: Vec<f32> = Vec::new();
        for e in &w.log.events[log_from..] {
            let t = (e.tick - start_tick) as f32 / 2.0;
            if e.class == Class::Landing {
                city_times.push(t);
            }
            moments.push(Moment { t, from: "city", class: match e.class { Class::Landing => "LAND", Class::Loss => "LOSS", Class::Decision => "DECI" }, what: e.what.clone() });
        }
        rec.run_landings = run_times.len();
        rec.city_landings = city_times.len();
        rec.run_gap = median(&mut gaps(&mut run_times.clone()));
        let mut merged: Vec<f32> = run_times.iter().chain(city_times.iter()).copied().collect();
        rec.merged_gap = median(&mut gaps(&mut merged));
        rec.built = s.outcome.built.iter().filter_map(|id| w.colony.get(*id)).map(|st| w.defs.structure[st.def].name.clone()).collect();
        rec.population = w.roster.records.len();
        rec.stash = w.stash.count();
        rec.in_rooms = w.stash.rooms.iter().map(|r| r.items.len()).sum();
        rec.floor = w.stash.floor.len();
        rec.eddies = w.ledger.eddies;
        rec.earned = s.closed.report.earned;
        rec.spent = s.outcome.upkeep + s.outcome.paste_eddies;
        rec.paste_meals = s.outcome.paste_meals;
        let ms = w.city.sim.stats();
        rec.mat_cells = (ms.tips + ms.cords + ms.nodes) as usize;
        rec.mat_nutrient = ms.nutrient;
        rec.starving = w.roster.records.iter().filter(|r| r.is_up() && r.need(people::Need::Fed) < 10).count();
        rec.heart_signal = w.city.heart_cell().map_or(0.0, |i| w.city.sim.sig[i]);
        rec.alerts = w.alerts().into_iter().map(|a| a.text).collect();
        rec.digest = digest(&w);
        // the shift's block of the merged log
        moments.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap().then(a.from.cmp(b.from)));
        out.log.push_str(&format!("shift {}  day {}  {}  {}  {:.0} s\n", out.shifts.len() + 1, day, if rec.runner.is_empty() { "-" } else { &rec.runner }, rec.result, rec.seconds));
        for m in &moments {
            out.log.push_str(&format!("  {:>3}:{:04.1}  {:<4} {}  {}\n", (m.t / 60.0) as u32, m.t % 60.0, m.from, m.class, m.what));
        }
        each(out.shifts.len(), &rec);
        out.shifts.push(rec);
    }
    out.plan_built = steward.plan_built(&w);
    // the assertions (the design's Phase C proof)
    for (k, r) in out.shifts.iter().enumerate().skip(1) {
        if r.late > 0 {
            out.failures.push(format!("shift {}: {} of the last shift's cargo still on the Mat", k + 1, r.late));
        }
    }
    for (k, r) in out.shifts.iter().enumerate() {
        if r.starving > 0 {
            out.failures.push(format!("shift {}: {} starving", k + 1, r.starving));
        }
        if r.heart_signal < SIGNAL_FLOOR {
            out.failures.push(format!("shift {}: the Heart's core at signal {:.2}", k + 1, r.heart_signal));
        }
    }
    if shifts as usize >= BUILD_PLAN.len() * 2 && out.plan_built < BUILD_PLAN.len() {
        out.failures.push(format!("the build plan stands at {} of {}", out.plan_built, BUILD_PLAN.len()));
    }
    out
}

/// Two plays of one seed, compared shift by shift: the city replays bit-exact or says where it parted.
pub fn replays(seed: u64, shifts: u32, each: &mut dyn FnMut(usize, &ShiftRecord)) -> Result<CityRun, String> {
    let a = play(seed, shifts, each);
    let b = play(seed, shifts, &mut |_, _| {});
    for (k, (x, y)) in a.shifts.iter().zip(b.shifts.iter()).enumerate() {
        if x.digest != y.digest {
            return Err(format!("the city parted from its replay at shift {}", k + 1));
        }
    }
    if a.log != b.log {
        return Err("the merged log parted from its replay".into());
    }
    Ok(a)
}
