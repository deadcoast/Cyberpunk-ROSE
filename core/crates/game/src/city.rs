//! `rose city` (Full Design v3.13, The city's log; Build plan, Phase C's proof): the whole game headless for a
//! number of shifts. Each shift the steward gives its standing orders, the eligible slimo of best Scav runs the
//! first route card of S1 as the bot with a rig packed from the Stash, the city ticks through the run at the bot's
//! pace, the haul comes home and the shift ends in core. The steward's orders are this design's stand-in for a
//! player, not a strategy the game recommends. The assertions and the merged run-and-city landing cadence go to the
//! caller.

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

/// The steward's build plan, one ghost at a time at the nearest anchor to the Heart; after the first Pod it waits
/// while the cord to the first grate is lost and the steering has work (Pass 2's order: the Pod, then the grate).
pub const BUILD_PLAN: [&str; 7] = ["pod", "lantern", "rack", "pod", "bunkhouse", "charter_board", "salt_works"];
/// The steering beside it (Pass 1: a stratum is crossed with Relays; Pass 2: "breach a stratum with Relays"): when
/// the cord to the first grate is lost, one Relay ghost at a time at the anchor nearest the grate on the Heart's side
/// of the break, the next when it stands, until the Mat reaches the grate again; at most this many standing.
pub const MAX_RELAYS: usize = 8;
/// The Rack's stockpile: eight cells around it.
pub const RACK_CELLS: usize = 8;
/// The Heart's Pit kept at 2 nutrient, Uncommon at most.
pub const HEART_KEEP: QuotaSet = QuotaSet { target: 2.0, ceiling: 1 };
/// The kit the bot carries, packed from the Stash where the Stash has it: the pipe in the hand, the lantern, the
/// pry bar, the antiseptic and the salt in the quick slots (run::bot::bot_rig's kit).
pub const KIT_HAND: &str = "ger_pipe";
pub const KIT_QUICK: [&str; 4] = ["ger_lantern", "ger_pry_bar", "org_antiseptic", "prt_salt"];
/// A shift with no one to run lasts the shortest route's clock (8:00) at 2 Hz.
pub const IDLE_SHIFT_TICKS: u64 = 960;
/// The city's clock is 2 Hz, the run's 20 Hz.
pub const RUN_TICKS_PER_CITY_TICK: u64 = 10;
/// The bot plays a route at BOT_FACTOR times a player's pace (run::telemetry), so the city keeps that pace through
/// its run: BOT_FACTOR city ticks for every ten of the bot's run ticks, and a shift does to the city what a player's
/// shift does (125 Mat steps in 12.5 minutes, The city's clock). The merged log is on the bot's clock.
pub fn city_ticks(run_ticks: u64) -> u64 {
    (run_ticks as f32 * run::telemetry::BOT_FACTOR / RUN_TICKS_PER_CITY_TICK as f32).round() as u64
}

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
    /// at the shift's end: the Mat reaches the first grate's market from the Heart; the steering can win it back (a
    /// Relay ghost out toward it, or room and an anchor for one)
    pub reached: bool,
    pub steering: bool,
    pub heart_signal: f32,
    pub alerts: Vec<String>,
    pub digest: u64,
    /// the economy: the markets the Mat reaches from the Heart, the shift's sales, the widest any demand's walk stood
    /// from its buy-list value
    pub markets: usize,
    pub sales: Vec<crate::cargo::Sale>,
    pub drift: f32,
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

/// The nearest anchor to `from` (tiles across plus tiles down, ties by row then column) where the def may stand.
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

/// The grate nearest the Heart (Pass 2's "a grate visible 40 tiles east"), by tiles across and up, as a tile.
pub fn first_grate(w: &World) -> Option<usize> {
    let (hx, hy) = heart_xy(w);
    logistics::first_grate(&w.city.sim, hx as usize, hy as usize)
}

/// The anchor joined to the Heart where a Relay may stand nearest the first grate along the ground the Mat can grow
/// on (a shaft that dead-ends under rock is not nearer for being close across; Mat cut off from the Heart is not the
/// city's side of a break), ties by row then column.
pub fn relay_spot(w: &World) -> Option<(u16, u16)> {
    let g = first_grate(w)?;
    let way = logistics::ground_distance(&w.city.sim, g);
    let joined = logistics::joined_to_heart(&w.city.sim);
    let sw = w.city.sim.w;
    (0..way.len()).filter(|&i| way[i] != usize::MAX && joined[i]).filter(|&i| w.colony.can_place(&w.defs, &w.city.sim, "relay", (i % sw) as u16, (i / sw) as u16).is_ok()).min_by_key(|&i| (way[i], i)).map(|i| ((i % sw) as u16, (i / sw) as u16))
}

/// Whether the Mat reaches the first grate's market from the Heart.
pub fn reaches_first_grate(w: &World) -> bool {
    let Some(g) = first_grate(w) else { return false };
    w.city.markets.iter().position(|&m| m == g).is_some_and(|k| w.city.from_heart(logistics::Kind::Market, k as u32).is_some())
}

fn heart_xy(w: &World) -> (u16, u16) {
    w.city.sim.heart.map(|(x, y)| (x as u16, y as u16)).unwrap_or((0, 0))
}

/// What the steward does with the haul's Chrome (Full Design v3.15, Economy: the proof). Feeding — the founding's
/// standing order — keeps the Heart at 2.0 nutrient (the quota pulls the Common and Uncommon Chrome); trading sets no
/// quota of its own and sells the Chrome past a Relay's six with the rest of the surplus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Haul {
    #[default]
    Feed,
    Trade,
}

/// A stall takes five of a category a shift from the steward.
pub const PER_STALL: usize = 5;
/// Chrome the steward keeps for the steering: a Relay's six.
pub const CHROME_RESERVE: usize = 6;
/// Parts the steward keeps when it pays for research: a Bunkhouse's ten.
pub const RESEARCH_KEEP_PARTS: u32 = 10;

/// The steward's standing orders, once a shift before the run.
#[derive(Clone, Debug, Default)]
pub struct Steward {
    pub haul: Haul,
    next: usize,
    current: Option<u32>,
    rack_painted: bool,
    quota_set: bool,
    /// the steering's Relay ghost
    relay: Option<u32>,
    /// research nodes the caller wants before the cheapest open one (`rose campaign`'s chapters)
    pub wanted: Vec<&'static str>,
    /// research only the wanted (`rose campaign`: the Chrome the rest of the tree costs is the climb's)
    pub only_wanted: bool,
}

impl Steward {
    fn decide(w: &mut World, what: String, out: &mut Vec<String>) {
        w.log.push(w.city.tick, Class::Decision, what.clone());
        out.push(what);
    }

    pub fn act(&mut self, w: &mut World) -> Vec<String> {
        let mut out = Vec::new();
        // feeding: the Heart's Pit kept at 2 nutrient, Uncommon at most
        if !self.quota_set && self.haul == Haul::Feed {
            if let Some(h) = w.colony.heart(&w.defs).map(|s| s.id) {
                w.colony.set_quota(&w.defs, h, Some(HEART_KEEP));
                Self::decide(w, "the Heart keeps 2.0 nutrient, Uncommon at most".into(), &mut out);
            }
            self.quota_set = true;
        }
        // the haul: contracts it can fill, the surplus to the best stalls in reach (or the Chrome to the ground),
        // protection when a Patrol is due at a Gang grate
        self.contracts(w, &mut out);
        self.research(w, &mut out);
        self.route_loop(w, &mut out);
        self.surplus(w, &mut out);
        self.services(w, &mut out);
        // the steering: whenever the cord to the first grate is lost, a Relay ghost at the anchor nearest the grate on
        // the Heart's side of the break, the next when it stands
        if self.relay.and_then(|id| w.colony.get(id)).is_none_or(|s| s.built) {
            self.relay = None;
        }
        let standing = w.colony.structures.iter().filter(|s| s.built && w.defs.structure[s.def].id == "relay").count();
        if self.relay.is_none() && !reaches_first_grate(w) && standing < MAX_RELAYS {
            if let Some((x, y)) = relay_spot(w) {
                if let Ok(id) = w.colony.place(&w.defs, &w.city.sim, "relay", x, y) {
                    self.relay = Some(id);
                    Self::decide(w, format!("Relay placed at {x},{y} toward the first grate"), &mut out);
                }
            }
        }
        // the build plan, one ghost at a time
        let standing = self.current.and_then(|id| w.colony.get(id)).map(|s| s.built);
        if standing == Some(true) || (self.current.is_some() && standing.is_none()) {
            if standing == Some(true) && BUILD_PLAN[self.next] == "rack" && !self.rack_painted {
                self.paint_rack(w, self.current.unwrap(), &mut out);
            }
            self.current = None;
            self.next += 1;
        }
        // after the first Pod the grate comes first (Pass 2's order): the plan waits while the steering has work
        let grate_first = self.next >= 1 && !reaches_first_grate(w) && self.steering(w);
        if self.current.is_none() && self.next < BUILD_PLAN.len() && !grate_first {
            let def = BUILD_PLAN[self.next];
            if let Some((x, y)) = nearest_spot(w, def, heart_xy(w)) {
                if let Ok(id) = w.colony.place(&w.defs, &w.city.sim, def, x, y) {
                    self.current = Some(id);
                    Self::decide(w, format!("{} placed at {x},{y}", w.defs.get(def).map_or(def, |d| d.name.as_str())), &mut out);
                }
            }
        }
        // every petition answered by placing the first thing it asks that can stand (a Lume Farm wants a Lume Bulb,
        // a Lantern Post will do), nearest the district's hub
        let asks: Vec<(Vec<String>, (u16, u16), String)> = w.colony.districts.iter().filter_map(|d| d.petition.as_ref().map(|p| (p.asks.clone(), d.hub, d.name.clone()))).collect();
        for (defs, hub, name) in asks {
            if defs.iter().any(|def| w.colony.structures.iter().any(|s| !s.built && w.defs.structure[s.def].id == *def)) {
                continue;
            }
            for def in defs {
                if let Some((x, y)) = nearest_spot(w, &def, hub) {
                    if w.colony.place(&w.defs, &w.city.sim, &def, x, y).is_ok() {
                        Self::decide(w, format!("{name}'s petition: {} placed at {x},{y}", w.defs.get(&def).map_or(def.as_str(), |d| d.name.as_str())), &mut out);
                        break;
                    }
                }
            }
        }
        w.publish();
        out
    }

    /// The research standing order (this design's steward; Full Design v3.18, Research): with no project standing,
    /// the wanted nodes first (`rose campaign`'s defences), else study a blueprint the Stash holds that opens a tier,
    /// else start the first node of the lowest open tier the Stash pays for while it keeps a Bunkhouse's Parts and a
    /// Relay's Chrome.
    fn research(&mut self, w: &mut World, out: &mut Vec<String>) {
        if w.colony.progress.tech.project.is_some() {
            return;
        }
        let f = progress::TechFile::shipped();
        let held: Vec<String> = w.stash.handles().into_iter().map(|(_, _, it)| w.table.def(it.def).id.clone()).collect();
        let parts = w.stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == items::Category::Part).count() as u32;
        let chrome = w.stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == items::Category::Chrome).count() as u32;
        // the wanted first, in their order: the node once it can be picked and paid, with no reserve kept (it is what
        // the reserves are for), else the blueprint in hand that opens its tier
        for id in self.wanted.clone() {
            let Some(n) = f.get(id) else { continue };
            if w.colony.progress.tech.has(id) {
                continue;
            }
            let p = progress::Project::Node(n.id.clone());
            if w.colony.progress.tech.cannot_pick(&p).is_none() {
                let (cp, cc) = f.cost_of(&p);
                if parts >= cp && chrome >= cc && crate::grow::pick_project(&mut w.colony, &mut w.stash, &mut w.ledger, &w.table, p).is_ok() {
                    Self::decide(w, format!("research: {}", n.name), out);
                }
                return;
            }
            let opens = held.iter().find(|item| f.blueprint(item) == Some((n.branch_index(), n.tier))).cloned();
            if let Some(item) = opens {
                if crate::grow::pick_project(&mut w.colony, &mut w.stash, &mut w.ledger, &w.table, progress::Project::Study(item)).is_ok() {
                    let name = w.colony.progress.tech.project_name().unwrap_or_default();
                    Self::decide(w, format!("research: {name}"), out);
                    return;
                }
            }
        }
        if self.only_wanted {
            return;
        }
        let mut studies: Vec<(u8, String)> = held.iter().filter_map(|id| f.blueprint(id).map(|(_, t)| (t, id.clone()))).collect();
        studies.sort();
        for (_, item) in studies {
            let p = progress::Project::Study(item);
            if crate::grow::pick_project(&mut w.colony, &mut w.stash, &mut w.ledger, &w.table, p).is_ok() {
                let name = w.colony.progress.tech.project_name().unwrap_or_default();
                Self::decide(w, format!("research: {name}"), out);
                return;
            }
        }
        let mut nodes: Vec<&progress::tech::NodeDef> = f.node.iter().filter(|n| w.colony.progress.tech.cannot_pick(&progress::Project::Node(n.id.clone())).is_none()).collect();
        // then the cheapest tier
        nodes.sort_by_key(|n| n.tier);
        for n in nodes {
            let p = progress::Project::Node(n.id.clone());
            let (cp, cc) = f.cost_of(&p);
            if parts < cp + RESEARCH_KEEP_PARTS || chrome < cc + CHROME_RESERVE as u32 {
                continue;
            }
            if crate::grow::pick_project(&mut w.colony, &mut w.stash, &mut w.ledger, &w.table, p).is_ok() {
                Self::decide(w, format!("research: {}", n.name), out);
                return;
            }
        }
    }

    /// The loop standing order (this design's steward; Full Design v3.18, Route loops): once Route automation stands
    /// and S1's route has seen two runs home, the eligible slimo second by Scav loops it (the first is the Runner).
    fn route_loop(&mut self, w: &mut World, out: &mut Vec<String>) {
        if !w.colony.loops.is_empty() || !w.colony.progress.tech.flag("route_loops") || w.colony.homes_from(1) < colony::LOOP_AFTER {
            return;
        }
        let mut crew: Vec<&people::Record> = w.roster.eligible();
        crew.sort_by_key(|r| (std::cmp::Reverse(r.skill(Skill::Scav)), r.id));
        if let Some(r) = crew.get(1) {
            let (id, name) = (r.id, r.full_name());
            if w.colony.set_loop(1, Some(id)).is_ok() {
                Self::decide(w, format!("{name} loops the S1 route"), out);
            }
        }
    }

    /// The kit kept for the Runner: every piece of it in the Stash, the spares for the run a Runner does not come
    /// home from, and the best weapons and armour the Stash holds.
    fn kit(w: &World) -> Vec<u32> {
        let defs: Vec<items::ItemId> = std::iter::once(KIT_HAND).chain(KIT_QUICK).filter_map(|id| w.table.id_of(id)).collect();
        let mut v: Vec<u32> = w.stash.handles().into_iter().filter(|(_, _, it)| defs.contains(&it.def)).map(|(h, _, _)| h).collect();
        v.extend(WORN.iter().filter_map(|(slot, kind)| best_gear(w, slot, kind)));
        v
    }

    /// The markets the Mat reaches from the Heart.
    pub fn in_reach(w: &World) -> Vec<usize> {
        (0..w.city.markets.len()).filter(|&k| w.city.from_heart(logistics::Kind::Market, k as u32).is_some()).collect()
    }

    /// A deliver link in hand takes what fits as it comes; then each faction in reach with no chain in hand has its
    /// first three-link deliver offer the Stash can fill now taken and sent, or its first three-link host offer while
    /// a bed is free (a longer chain asks more than one route's hauls bring in its shifts).
    fn contracts(&mut self, w: &mut World, out: &mut Vec<String>) {
        let kit = Self::kit(w);
        let market_of = |w: &World, f: u8| crate::trade::market_of(&w.economy, &w.city, &w.colony, f).map(|m| m as usize);
        for c in w.economy.s.contracts.clone() {
            if let (economy::Kind::Deliver, Some(k)) = (c.kind, market_of(w, c.faction)) {
                self.send(w, &c, k, &kit, out);
            }
        }
        let (beds, taken) = w.colony.bed_count(&w.defs);
        let mut free_beds = ((beds - taken).max(0) as usize).saturating_sub(w.roster.waiting.len());
        for o in w.economy.s.offers.clone() {
            let Some(k) = market_of(w, o.faction) else { continue };
            if o.links > 3 || w.economy.s.contracts.iter().any(|c| c.faction == o.faction) {
                continue;
            }
            let c = w.economy.first_link(&o, w.day);
            let take = match c.kind {
                economy::Kind::Deliver => self.fitting(w, &c, &kit).len() >= c.ask.n as usize,
                economy::Kind::Host if free_beds > 0 => {
                    free_beds -= 1;
                    true
                }
                _ => false,
            };
            if !take {
                continue;
            }
            if let Ok((line, _)) = crate::trade::accept(&mut w.economy, (&w.city, &w.colony), &mut w.roster, &w.traits, &w.names, &w.table, None, o.id, w.day, &mut w.log) {
                out.push(line);
                if let Some(held) = w.economy.s.contracts.iter().find(|x| x.id == o.id).cloned() {
                    if held.kind == economy::Kind::Deliver {
                        self.send(w, &held, k, &kit, out);
                    }
                }
            }
        }
    }

    /// What fits a deliver link, up to what it still asks and is not already riding, sent to its market.
    fn send(&self, w: &mut World, c: &economy::Contract, k: usize, kit: &[u32], out: &mut Vec<String>) {
        let riding = w.ledger.in_flight.iter().filter(|s| s.contract == Some(c.id)).count() + w.ledger.queue.iter().filter(|q| matches!(q.verb, Verb::Deliver { contract, .. } if contract == c.id)).count();
        let need = (c.ask.n as usize).saturating_sub(c.got as usize + riding);
        let send: Vec<u32> = self.fitting(w, c, kit).into_iter().take(need).collect();
        let n = send.len();
        for h in send {
            w.ledger.queue(&mut w.stash, h, Verb::Deliver { market: k as u8, contract: c.id }, w.day);
        }
        if n > 0 {
            let line = format!("{n} to the {} at grate {}", w.economy.name(c.faction), w.city.market_grate(k));
            Self::decide(w, line, out);
        }
    }

    /// The Stash's items that meet a deliver link, cheapest first, the kit kept and the Parts the build plan still
    /// needs.
    fn fitting(&self, w: &World, c: &economy::Contract, kit: &[u32]) -> Vec<u32> {
        let mut v: Vec<(u32, u32, bool)> = w.stash.handles().into_iter().filter(|(h, _, it)| !it.pinned && !kit.contains(h) && w.economy.fits(c, it, &w.table)).map(|(h, _, it)| (w.table.def(it.def).value, h, w.table.def(it.def).category == Category::Part)).collect();
        v.sort_unstable();
        let need: u32 = BUILD_PLAN[self.next.min(BUILD_PLAN.len())..].iter().filter_map(|d| w.defs.get(d)).map(|d| d.cost.parts).sum();
        let mut spare = (w.stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == Category::Part).count() as u32).saturating_sub(need);
        v.into_iter()
            .filter(|&(_, _, part)| {
                if !part {
                    return true;
                }
                let ok = spare > 0;
                spare = spare.saturating_sub(1);
                ok
            })
            .map(|(_, h, _)| h)
            .collect()
    }

    /// The surplus — Tech, Curio, Contraband, Gear the kit does not use and, trading, Chrome past a Relay's six; Parts are kept
    /// for building, Organics for the clinic, Keys and Relics never leave — goes to whichever market in reach pays
    /// most for each piece now, five of a category a stall a shift.
    fn surplus(&mut self, w: &mut World, out: &mut Vec<String>) {
        let reach = Self::in_reach(w);
        let kit = Self::kit(w);
        let chrome = self.haul == Haul::Trade;
        // a blueprint that opens a tier the tree has not opened is research, never surplus (Full Design v3.18)
        let f = progress::TechFile::shipped();
        let keep = |id: &str| f.blueprint(id).is_some_and(|(b, t)| w.colony.progress.tech.open[b] < t);
        let mut items: Vec<(u32, u32, Category)> = w
            .stash
            .handles()
            .into_iter()
            .filter(|(h, _, it)| !it.pinned && !kit.contains(h) && !keep(&w.table.def(it.def).id))
            .map(|(h, _, it)| (w.table.def(it.def).value, h, w.table.def(it.def).category))
            .filter(|(_, _, c)| matches!(c, Category::Tech | Category::Curio | Category::Contraband | Category::Gear) || (chrome && *c == Category::Chrome))
            .collect();
        items.sort_unstable_by_key(|&(v, h, _)| (std::cmp::Reverse(v), h));
        // the Chrome reserve: the cheapest six stay
        let reserve: Vec<u32> = items.iter().filter(|i| i.2 == Category::Chrome).rev().take(CHROME_RESERVE).map(|i| i.1).collect();
        items.retain(|i| !reserve.contains(&i.1));
        let mut sent: Vec<(usize, u32)> = Vec::new();
        for (_, h, cat) in items {
            let Some(it) = w.stash.get_any(h).cloned() else { continue };
            let best = reach
                .iter()
                .copied()
                .filter_map(|k| {
                    let ahead = w.ledger.queue.iter().filter(|q| q.verb == Verb::Sell { market: k as u8 } && w.table.def(q.item.def).category == cat).count();
                    if ahead >= PER_STALL {
                        return None;
                    }
                    let glut = w.ledger.glut_units(k as u8, cat as usize) + ahead as u32;
                    w.ledger.best(&w.table, &it, k as u8, glut).map(|(p, _)| (p, std::cmp::Reverse(k)))
                })
                .max();
            if let Some((p, std::cmp::Reverse(k))) = best {
                if w.ledger.queue(&mut w.stash, h, Verb::Sell { market: k as u8 }, w.day) {
                    sent.push((k, p));
                }
            }
        }
        for &k in &reach {
            let here: Vec<u32> = sent.iter().filter(|s| s.0 == k).map(|s| s.1).collect();
            if !here.is_empty() {
                Self::decide(w, format!("{} to sell at grate {}, about {} e", here.len(), w.city.market_grate(k), here.iter().sum::<u32>()), out);
            }
        }
    }

    /// The services it buys (Full Design v3.16, Economy: the proof): protection when a Patrol is due at a Gang grate,
    /// intel when a reached zone stands within 15 of the Sweep line, the Badges' timings when one stands within 10 of
    /// any line, the Nomads' convoy every shift it sells at their stall, a hire when a bed stands free and the eddies
    /// cover it twice. A service the faction's open tier does not stock is not bought.
    fn services(&mut self, w: &mut World, out: &mut Vec<String>) {
        let reached: Vec<usize> = Self::in_reach(w).into_iter().map(|k| w.city.markets[k]).collect();
        let zones: Vec<(f32, usize)> = w.threats.zones.iter().enumerate().filter(|(_, z)| reached.contains(&z.grate)).map(|(k, z)| (z.attention, k)).collect();
        let stages = w.threats.rules.stage.clone();
        let mut want: Vec<(Option<u8>, &str)> = Vec::new();
        let ix = |w: &World, id: &str| w.economy.file.index(id);
        let gangs = w.economy.file.index("gangs");
        if w.economy.s.protection == 0 && w.threats.due.iter().any(|d| d.stage == 0 && w.economy.faction_at(w.threats.zones[d.zone as usize].grate) == gangs) {
            want.push((gangs, "protection"));
        }
        let sweep = stages.iter().find(|s| s.id == "sweep").map_or(f32::MAX, |s| s.at);
        if w.economy.s.intel == 0 && zones.iter().any(|&(a, _)| a < sweep && a >= sweep - 15.0) {
            want.push((ix(w, "corpo"), "intel"));
        }
        if w.economy.s.timings == 0 && zones.iter().any(|&(a, _)| stages.iter().any(|s| a < s.at && a >= s.at - 10.0)) {
            want.push((ix(w, "badges"), "timings"));
        }
        if let Some(n) = w.economy.file.index("nomads") {
            if let Some(m) = crate::trade::market_of(&w.economy, &w.city, &w.colony, n) {
                if w.ledger.queue.iter().any(|q| q.verb == Verb::Sell { market: m }) {
                    want.push((Some(n), "convoy"));
                }
            }
        }
        let (beds, taken) = w.colony.bed_count(&w.defs);
        if beds - taken > w.roster.waiting.len() as i32 && w.ledger.eddies >= 2 * w.economy.file.services.hire {
            let f = (0..w.economy.file.factions.len() as u8).find(|&f| w.economy.tier_open(f) >= 4 && w.economy.service_entry(f, "hire").is_some() && crate::trade::market_of(&w.economy, &w.city, &w.colony, f).is_some());
            if f.is_some() {
                want.push((f, "hire"));
            }
        }
        for (f, svc) in want {
            let Some(f) = f else { continue };
            let Some(m) = crate::trade::market_of(&w.economy, &w.city, &w.colony, f) else { continue };
            let Some(entry) = w.economy.service_entry(f, svc) else { continue };
            if w.ledger.orders.iter().any(|o| o.faction == f && o.entry == entry) {
                continue;
            }
            let plan = w.plan.clone();
            if let Ok(each) = crate::trade::order(&mut w.economy, &mut w.ledger, &w.table, &w.city, &w.colony, &w.defs, &w.roster, &plan, m, f, entry, 1) {
                let line = format!("{svc} from the {} at grate {}: {each} e", w.economy.name(f), w.city.market_grate(m as usize));
                Self::decide(w, line, out);
            }
        }
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

    /// Whether the steering can win the first grate: a Relay ghost out toward it, or room for one more Relay and an
    /// anchor to put it on.
    pub fn steering(&self, w: &World) -> bool {
        let standing = w.colony.structures.iter().filter(|s| s.built && w.defs.structure[s.def].id == "relay").count();
        self.relay.and_then(|id| w.colony.get(id)).is_some_and(|s| !s.built) || (standing < MAX_RELAYS && relay_spot(w).is_some())
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

/// The Stash's best gear for a slot and kind (the weapon of most damage, the armour of most): the Runner goes out in it
/// rather than the kit's pipe when the runs have brought better home (this design's steward).
fn best_gear(w: &World, slot: &str, kind: &str) -> Option<u32> {
    w.stash
        .handles()
        .into_iter()
        .filter(|(_, _, it)| !it.pinned)
        .filter_map(|(h, _, it)| {
            let g = w.table.def(it.def).gear.as_ref()?;
            (g.slot == slot && g.kind == kind).then_some((g.dmg + g.armour, w.table.def(it.def).value, h))
        })
        .max_by_key(|&(score, value, h)| (score, value, std::cmp::Reverse(h)))
        .map(|(_, _, h)| h)
}

/// The best gear the Runner wears: the melee and the ranged weapon for the hands, the head, body and feet armour.
const WORN: [(&str, &str); 5] = [("hand", "melee"), ("hand", "ranged"), ("head", "armour"), ("body", "armour"), ("feet", "armour")];

/// The rig packed from the Stash for this Runner: the best weapons and armour it holds (the kit's pipe when it holds
/// none better), the kit's quick items.
fn pack(w: &mut World, rows: u8, carry: f32) -> Rig {
    let mut rig = Rig::new(8, rows.clamp(3, 8), 4, carry);
    let picks: Vec<Option<u32>> = WORN.iter().map(|(slot, kind)| best_gear(w, slot, kind)).collect();
    let mut take = |h: Option<u32>| h.and_then(|h| w.stash.take(h));
    rig.gear.hands[0] = take(picks[0]);
    rig.gear.hands[1] = take(picks[1]);
    rig.gear.head = take(picks[2]);
    rig.gear.body = take(picks[3]);
    rig.gear.feet = take(picks[4]);
    if rig.gear.hands[0].is_none() {
        rig.gear.hands[0] = take_kit(w, KIT_HAND);
    }
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
    for st in &w.economy.s.standing {
        (st.rep, st.favour, st.traded, st.opened).hash(&mut h);
    }
    (w.economy.s.contracts.len(), w.economy.s.offers.len(), w.ledger.orders.len()).hash(&mut h);
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

/// Play the city for `shifts` shifts from `seed` (the game's founding: the Mat steered to the first grate); `each`
/// hears every shift as it ends.
pub fn play(seed: u64, shifts: u32, each: &mut dyn FnMut(usize, &ShiftRecord)) -> CityRun {
    play_world(seed, shifts, each).0
}

/// The same, and the World as the last shift left it (the Cleansing proof plays on from it).
pub fn play_world(seed: u64, shifts: u32, each: &mut dyn FnMut(usize, &ShiftRecord)) -> (CityRun, World) {
    play_with(seed, shifts, Haul::Feed, each)
}

/// The same with the steward's haul given (`rose economy` plays both).
pub fn play_with(seed: u64, shifts: u32, haul: Haul, each: &mut dyn FnMut(usize, &ShiftRecord)) -> (CityRun, World) {
    let table = ItemTable::from_toml(items::ITEMS_TOML).expect("items.toml");
    let mut p = Player::new(World::found(seed, table), seed, haul);
    for k in 0..shifts as usize {
        let r = p.shift(1, &mut |_| Vec::new());
        each(k, r);
    }
    let Player { w, steward, mut out, .. } = p;
    out.plan_built = steward.plan_built(&w);
    // the assertions (the design's Phase C proof)
    for (k, r) in out.shifts.iter().enumerate().skip(1) {
        if r.late > 0 {
            out.failures.push(format!("shift {}: {} of the last shift's cargo still on the Mat", k + 1, r.late));
        }
    }
    for (k, r) in out.shifts.iter().enumerate() {
        if !r.reached && !r.steering {
            out.failures.push(format!("shift {}: the cord to the first grate is lost and no Relay can go out to win it back", k + 1));
        }
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
    (out, w)
}

/// One stratum's Board route as the headless game runs it: its map, what the fog remembers, its runs so far.
struct Route {
    stratum: u8,
    map: RouteMap,
    memory: Vec<u8>,
    runs: u16,
}

/// The whole game headless, a shift at a time (`rose city`, `rose cleansing`, `rose economy`, `rose campaign`): the
/// steward's standing orders and the caller's, the eligible slimo of best Scav on a stratum's Board route as the bot,
/// alone, with the kit packed from the Stash; the city through the run at the bot's pace, the haul home, the shift's
/// end in core.
pub struct Player {
    pub w: World,
    pub steward: Steward,
    pub out: CityRun,
    seed: u64,
    table: ItemTable,
    lib: ChunkLib,
    tables: LootTables,
    creatures: CreatureFile,
    routes: Vec<Route>,
}

impl Player {
    pub fn new(w: World, seed: u64, haul: Haul) -> Player {
        let table = w.table.clone();
        Player {
            w,
            steward: Steward { haul, ..Steward::default() },
            out: CityRun { seed, ..Default::default() },
            seed,
            table,
            lib: ChunkLib::from_toml(run::CHUNKS_TOML).expect("chunks.toml"),
            tables: LootTables::shipped(),
            creatures: CreatureFile::shipped(),
            routes: Vec::new(),
        }
    }

    fn route(&mut self, stratum: u8) -> usize {
        if let Some(k) = self.routes.iter().position(|r| r.stratum == stratum) {
            return k;
        }
        let map = RouteMap::compile(&self.lib, &run::route(&self.lib, run::route_seed(self.seed, stratum), stratum).expect("a Board route"));
        self.routes.push(Route { stratum, map, memory: Vec::new(), runs: 0 });
        self.routes.len() - 1
    }

    /// One shift with the run on `stratum`'s Board route (none when no one can run); `orders` are the caller's,
    /// given after the steward's and before the run.
    pub fn shift(&mut self, stratum: u8, orders: &mut dyn FnMut(&mut World) -> Vec<String>) -> &ShiftRecord {
        let rk = self.route(stratum);
        let w = &mut self.w;
        let day = w.day;
        let mut rec = ShiftRecord { day, ..Default::default() };
        let start_tick = w.city.tick;
        let log_from = w.log.events.len();
        rec.decisions = self.steward.act(w);
        rec.decisions.extend(orders(w));
        // the Runner: the eligible slimo of best Scav, alone; a contract's guest and a loop's slimo stay home
        let hosts: Vec<u32> = w.economy.s.contracts.iter().filter_map(|c| c.host).collect();
        let looping: Vec<u32> = w.colony.loops.iter().map(|l| l.slimo).collect();
        let runner = w.roster.eligible().into_iter().filter(|r| !hosts.contains(&r.id) && !looping.contains(&r.id)).max_by_key(|r| (r.skill(Skill::Scav), std::cmp::Reverse(r.id))).map(|r| r.id);
        let riding: Vec<u32> = w.ledger.in_flight.iter().map(|s| s.id).collect();
        let mut moments: Vec<Moment> = Vec::new();
        let mut run_times: Vec<f32> = Vec::new();
        let route = &mut self.routes[rk];
        let outcome = runner.map(|id| {
            let r = w.roster.get(id).unwrap();
            let mut setup = r.runner_setup(&w.traits);
            setup.flags.extend(crate::grow::run_flags(&w.colony, Some(&w.threats), &w.city.sim));
            rec.runner = r.full_name();
            let rig = pack(w, setup.rig_rows, setup.carry_kg());
            w.roster.depart(id, &[]);
            w.movers();
            let lume = w.city.sim.genome.0[mat::genome::LUME];
            let nonce = w.city.tick * 131 + day as u64;
            let mem = if route.memory.is_empty() { None } else { Some(route.memory.as_slice()) };
            let rs = RunSetup { mat_lume_gene: lume, memory: mem, run_nonce: nonce, route_runs: route.runs, day, rig, runner: setup, followers: Vec::new(), corpses: Vec::new(), flood: w.threats.flood_roll(), next_flood: w.threats.is_flood_shift(day + 1).then(|| w.threats.roll_for(day + 1)) };
            run::bot::play_setup(route.map.clone(), self.table.clone(), self.tables.clone(), self.creatures.clone(), rs)
        });
        let run_ticks = outcome.as_ref().map_or(IDLE_SHIFT_TICKS * RUN_TICKS_PER_CITY_TICK, |o| o.ticks);
        // the city through the run, at the bot's pace
        let ticks = if outcome.is_some() { city_ticks(run_ticks) } else { IDLE_SHIFT_TICKS };
        for _ in 0..ticks {
            w.tick();
        }
        rec.late = w.ledger.in_flight.iter().filter(|s| riding.contains(&s.id)).count();
        // home
        if let (Some(id), Some(o)) = (runner, outcome.as_ref()) {
            route.runs += 1;
            w.run_kills += o.oath_kills;
            w.economy.raider_kills(o.raider_kills);
            for k in &o.killed {
                if let Some(name) = w.colony.progress.collections.trophy("kill", k, day) {
                    w.log.push(w.city.tick, crate::log::Class::Landing, format!("Trophy: {name}"));
                }
            }
            route.memory = o.route_memory.clone();
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
            if extracted {
                w.colony.home_from(stratum);
            }
            // Ivan's line and the campaign's note of it (chapter 5 reads a Vault run's best item)
            crate::campaign::run_home(&mut w.campaign, &mut w.log, &w.table, stratum, &rec.runner, extracted, !extracted, o.rig.as_ref().filter(|_| extracted));
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
            let t = (e.tick - start_tick) as f32 / 2.0 / if outcome.is_some() { run::telemetry::BOT_FACTOR } else { 1.0 };
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
        rec.reached = reaches_first_grate(w);
        rec.steering = self.steward.steering(w);
        rec.heart_signal = w.city.heart_cell().map_or(0.0, |i| w.city.sim.sig[i]);
        rec.alerts = w.alerts().into_iter().map(|a| a.text).collect();
        rec.markets = Steward::in_reach(w).len();
        rec.sales = std::mem::take(&mut w.sales);
        rec.drift = (0..w.economy.file.factions.len() as u8).flat_map(|f| economy::CATEGORIES.map(|c| w.economy.drift(f, c).abs())).fold(0.0, f32::max);
        rec.digest = digest(w);
        // the shift's block of the merged log
        moments.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap().then(a.from.cmp(b.from)));
        self.out.log.push_str(&format!("shift {}  day {}  {}  {}  {:.0} s\n", self.out.shifts.len() + 1, day, if rec.runner.is_empty() { "-" } else { &rec.runner }, rec.result, rec.seconds));
        for m in &moments {
            self.out.log.push_str(&format!("  {:>3}:{:04.1}  {:<4} {}  {}\n", (m.t / 60.0) as u32, m.t % 60.0, m.from, m.class, m.what));
        }
        self.out.shifts.push(rec);
        self.out.shifts.last().unwrap()
    }
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
