//! `rose campaign` (Full Design v3.20, Phase E's proof): the campaign headless at the bot's pace, chapter 1 to an
//! ending. The twenty-shift steward's standing orders (`city::Steward`) and, over them, a campaign steward's: climb the
//! stack (a Relay and a Pit at the frontier toward the nearest crossing, a Relay at its slab, a grate key where one is
//! held), beds for the slimos, the forge and the market, the accord answered, the defences, a Pen, the Still, runs on
//! the stratum a chapter asks for, Ivan's Parley for the alliance, the feed steered toward one ending's genes with
//! Ivan's hand, the Great Cleansing met or let in. The campaign steward is this design's stand-in for a player who
//! wants a given ending, not a strategy the game recommends.

use colony::QuotaSet;
use items::{Category, Tier};
use story::campaign::{Accord, Mode};

use crate::city::{nearest_spot, Haul, Player, ShiftRecord};
use crate::World;

/// The ending a scripted campaign plays for: its genes (the pair the ending scores), whether it accepts the accord,
/// whether it meets the Great Cleansing.
#[derive(Clone, Copy, Debug)]
pub struct Aim {
    pub ending: &'static str,
    pub genes: [usize; 2],
    pub accept: bool,
    pub defend: bool,
}

/// The five scripts (the genes are Full Design v3.19's Endings; Scattered lets the Great Cleansing in).
pub fn aims() -> [Aim; 5] {
    use mat::genome::*;
    [
        Aim { ending: "garden", genes: [LUME, THRIFT], accept: true, defend: true },
        Aim { ending: "hive", genes: [CHROME, CONDUCTANCE], accept: false, defend: true },
        Aim { ending: "world_eater", genes: [APPETITE, FERAL], accept: false, defend: true },
        Aim { ending: "accord", genes: [THRIFT, TOXICITY], accept: true, defend: true },
        Aim { ending: "scattered", genes: [LUME, THRIFT], accept: true, defend: false },
    ]
}

/// One shift of the campaign as the report reads it.
#[derive(Clone, Debug, Default)]
pub struct CampaignShift {
    pub day: u32,
    pub chapter: u8,
    pub stratum: u8,
    pub run_on: u8,
    pub result: String,
    pub population: usize,
    pub beds: i32,
    pub cells: usize,
    /// the highest row the Mat joined to the Heart holds (0 is the Undercroft's ceiling)
    pub top_row: usize,
    pub genes: [f32; 10],
    /// the research standing and the tiers open (Chrome, Flesh, Mycelium); Gates and Turrets built
    pub research: String,
    pub open: [u8; 3],
    pub defences: usize,
    /// the Bloom's tiles and Nodes, the crawlers standing
    pub bloom: (usize, usize, usize),
    /// Parts and Chrome in the Stash (unpinned), eddies
    pub stock: (usize, usize, u32),
    pub decisions: Vec<String>,
    pub story: Vec<String>,
}

/// What the campaign steward keeps between shifts.
#[derive(Clone, Debug, Default)]
pub struct CampaignSteward {
    pub aim: Option<Aim>,
    /// the frontier's last distance to its target and the shifts it has stood
    last_way: usize,
    stood: u32,
    /// whether the Pits' quotas are feeding
    feeding: bool,
}

/// Defences the steward raises before a Cleansing (this design's: the `rose cleansing` proof held the Heart with two
/// Gates and a manned Turret).
/// Chrome the steward keeps in the Stash for research and Relays (a tier-3 node's eight and a Relay's six), and the
/// eddies it never spends on it.
const CHROME_WANTED: usize = 14;
const EDDY_RESERVE: u32 = 100;
/// Scrap Turrets by the Heart.
const TURRETS: usize = 2;
/// Beds kept ahead of the slimos.
const SPARE_BEDS: i32 = 1;
/// A frontier Pit's quota: nutrient kept, Uncommon at most.
const FRONT_KEEP: QuotaSet = QuotaSet { target: 4.0, ceiling: 1 };

fn count(w: &World, def: &str, built: bool) -> usize {
    w.colony.structures.iter().filter(|s| w.defs.structure[s.def].id == def && (s.built || !built)).count()
}

fn heart_xy(w: &World) -> (u16, u16) {
    w.city.sim.heart.map(|(x, y)| (x as u16, y as u16)).unwrap_or((0, 0))
}

fn place_near(w: &mut World, def: &str, from: (u16, u16), why: &str, out: &mut Vec<String>) -> Option<u32> {
    let (x, y) = nearest_spot(w, def, from)?;
    let id = w.colony.place(&w.defs, &w.city.sim, def, x, y).ok()?;
    let name = w.defs.get(def).map_or(def.to_string(), |d| d.name.clone());
    out.push(format!("{name} placed at {x},{y}{why}"));
    Some(id)
}

impl CampaignSteward {
    /// The orders of a shift, after the standing ones and before the run.
    pub fn act(&mut self, w: &mut World) -> Vec<String> {
        let mut out = Vec::new();
        self.answer(w, &mut out);
        self.jobs(w, &mut out);
        self.beds(w, &mut out);
        self.chapter_builds(w, &mut out);
        self.buy(w, &mut out);
        self.climb(w, &mut out);
        self.steer(w, &mut out);
        self.ivan(w, &mut out);
        w.publish();
        for o in &out {
            w.log.push(w.city.tick, crate::log::Class::Decision, o.clone());
        }
        out
    }

    /// The stratum the run goes to: the Vault while chapter 5 waits on a Runner home from it, else the lowest the Board
    /// offers.
    pub fn run_on(&self, w: &World) -> u8 {
        let board: Vec<u8> = logistics::board_strata(w.threats.stratum, w.city.sim.strata).collect();
        if w.campaign.chapter == 5 && !w.campaign.vault_home && board.contains(&5) {
            return 5;
        }
        board.first().copied().unwrap_or(1)
    }

    fn answer(&mut self, w: &mut World, out: &mut Vec<String>) {
        let accept = self.aim.is_some_and(|a| a.accept);
        if matches!(w.campaign.accord, Some(Accord::Waiting { .. })) {
            let day = w.day;
            let (c, mut p) = w.campaign_parts();
            if crate::campaign::answer_accord(c, &mut p, accept, day).is_ok() {
                out.push(format!("the envoy answered: {}", if accept { "accepted" } else { "refused" }));
            }
        }
    }

    /// The Job grid: while a project stands, the slimo of best Tweak that can research (Tweak 2) puts Research
    /// first; before a Cleansing (chapter 3 on, and the Great one) Conscription is signed when Authority allows.
    fn jobs(&mut self, w: &mut World, out: &mut Vec<String>) {
        // a researcher for every Research slot the Benches open (one a Bench below tier 3)
        let research = people::Job::Research;
        let benches = count(w, "bench", true);
        let first = w.roster.records.iter().filter(|r| r.is_up() && r.priorities[research.index()] == 4).count();
        if w.colony.progress.tech.project.is_some() && first < benches {
            let mut picks: Vec<(u8, u32, String)> = w.roster.records.iter().filter(|r| r.is_up() && r.priorities[research.index()] != 4 && r.skill(people::Skill::Tweak) >= research.gate()).map(|r| (r.skill(people::Skill::Tweak), r.id, r.full_name())).collect();
            picks.sort_by_key(|p| (std::cmp::Reverse(p.0), p.1));
            for (_, id, name) in picks.into_iter().take(benches - first) {
                if let Some(r) = w.roster.get_mut(id) {
                    r.priorities[research.index()] = 4;
                    out.push(format!("{name} puts Research first"));
                }
            }
        }
        // a Guard for every seat the Gates and Turrets open, the best Fight first
        let guard = people::Job::Guard;
        let seats = count(w, "gate", true) + count(w, "turret", true);
        let manned = w.roster.records.iter().filter(|r| r.is_up() && r.priorities[guard.index()] == 4).count();
        if manned < seats {
            let mut picks: Vec<(u8, u32, String)> = w.roster.records.iter().filter(|r| r.is_up() && r.priorities[guard.index()] != 4 && r.priorities[research.index()] != 4).map(|r| (r.skill(people::Skill::Fight), r.id, r.full_name())).collect();
            picks.sort_by_key(|p| (std::cmp::Reverse(p.0), p.1));
            for (_, id, name) in picks.into_iter().take(seats - manned) {
                if let Some(r) = w.roster.get_mut(id) {
                    r.priorities[guard.index()] = 4;
                    out.push(format!("{name} stands Guard first"));
                }
            }
        }
        // while ghosts wait, Build first for the rest (the Job grid's column): the defences and the climb are built
        // by hands, and mending takes them first
        let ghosts = w.colony.structures.iter().any(|s| !s.built);
        let build = people::Job::Build;
        for r in w.roster.records.iter_mut().filter(|r| r.priorities[research.index()] != 4 && r.priorities[guard.index()] != 4) {
            let want = if ghosts { 4 } else { 2 };
            if r.priorities[build.index()] != 0 {
                r.priorities[build.index()] = want;
            }
        }
        let threatened = w.campaign.chapter >= 3 && self.aim.is_none_or(|a| a.defend || w.campaign.chapter < 7);
        if threatened && w.colony.charter.law("conscription").is_none() {
            if let Some(law) = colony::LawFile::shipped().get("conscription").cloned() {
                if w.colony.sign(&w.defs, &w.city.sim, &law).is_ok() {
                    out.push("Conscription signed: every D2 is militia while the white light is down".into());
                }
            }
        }
    }

    fn beds(&mut self, w: &mut World, out: &mut Vec<String>) {
        let (beds, _) = w.colony.bed_count(&w.defs);
        let pending = w.colony.structures.iter().any(|s| !s.built && matches!(w.defs.structure[s.def].id.as_str(), "pod" | "bunkhouse"));
        if !pending && beds < w.roster.records.len() as i32 + SPARE_BEDS {
            let def = if w.colony.progress.tech.structure_locked("bunkhouse").is_none() { "bunkhouse" } else { "pod" };
            place_near(w, def, heart_xy(w), " for the slimos without beds", out);
        }
    }

    /// The chapters' structures: the forge and the market (2), the defences before a Cleansing (3, 7), a Pen (4),
    /// the Still (5).
    fn chapter_builds(&mut self, w: &mut World, out: &mut Vec<String>) {
        let ch = w.campaign.chapter;
        let pending = |w: &World, def: &str| w.colony.structures.iter().any(|s| !s.built && w.defs.structure[s.def].id == def);
        let heart = heart_xy(w);
        // a second Tweaker Bench: a second Research slot (Full Design v3.18: tier 2 by shift 15 asks two researchers)
        if ch >= 2 && count(w, "bench", false) < 2 {
            place_near(w, "bench", heart, " (a second Research slot)", out);
        }
        if ch >= 2 && count(w, "forge", false) == 0 {
            place_near(w, "forge", heart, " (chapter 2)", out);
        }
        if ch >= 2 && count(w, "market", false) == 0 {
            if let Some(g) = crate::city::first_grate(w) {
                let sw = w.city.sim.w;
                place_near(w, "market", ((g % sw) as u16, (g / sw) as u16), " at the first grate (chapter 2)", out);
            }
        }
        // the defences, as the tree opens them: two Gates across the way down from the first grate (the `rose
        // cleansing` proof's), Turrets by the Heart
        let defend = self.aim.is_none_or(|a| a.defend) || w.campaign.chapter < 7;
        if defend && count(w, "gate", false) == 0 && w.colony.progress.tech.structure_locked("gate").is_none() {
            if let Some(g) = crate::city::first_grate(w) {
                out.extend(crate::siege::gates(w, g));
            }
        }
        if defend && count(w, "turret", false) < TURRETS && !pending(w, "turret") {
            place_near(w, "turret", heart, " by the Heart", out);
        }
        if ch >= 4 && count(w, "pen", false) == 0 {
            place_near(w, "pen", heart, " (chapter 4: the Bloom's crawlers)", out);
        }
        if ch >= 5 && count(w, "still", false) == 0 {
            place_near(w, "still", heart, " (chapter 5)", out);
        }
    }

    /// The climb (Full Design v3.20, The Mat's growth): toward the nearest crossing out of the city's stratum, a Relay
    /// on its slab's reach breaks it at the shift's end; short of it, a Relay at the frontier when none stands within
    /// six and a Pit fed by quota there when the frontier has stood a shift; a held grate key opens the slab at once.
    fn climb(&mut self, w: &mut World, out: &mut Vec<String>) {
        let s = w.threats.stratum;
        if s >= w.city.sim.strata {
            return;
        }
        let sw = w.city.sim.w;
        let joined = logistics::joined_to_heart(&w.city.sim);
        // the crossing nearest the Mat along the ground, by the floor under its slab
        let mut best: Option<(usize, usize, Vec<usize>)> = None;
        for c in w.city.sim.crossings_of(s) {
            let probe = if w.city.sim.sealed(c) { c + sw } else { c };
            let way = logistics::ground_distance(&w.city.sim, probe);
            if let Some(d) = (0..way.len()).filter(|&i| joined[i] && way[i] != usize::MAX).map(|i| way[i]).min() {
                if best.as_ref().is_none_or(|b| d < b.0) {
                    best = Some((d, c, way));
                }
            }
        }
        let Some((d, c, way)) = best else { return };
        let (cx, cy) = ((c % sw) as i32, (c / sw) as i32);
        if w.city.sim.sealed(c) {
            // a key opens it now
            let key = crate::breach::crossings(&w.city.sim).into_iter().find(|k| k.tile == c).map(|k| k.key).unwrap_or_default();
            if !key.is_empty() && w.stash.handles().iter().any(|(_, _, it)| w.table.def(it.def).id == key) {
                if let Ok(line) = crate::breach::open_with_key(&mut w.city, Some(&mut w.threats), &mut w.stash, &w.table, c, &mut w.log) {
                    out.push(line);
                    return;
                }
            }
            // a Relay in its reach breaks it at the shift's end
            let reach = crate::breach::BREACH_REACH;
            let near = |x: i32, y: i32| (x - cx).abs().min((x - cx - 1).abs()).max((y - cy).abs()) <= reach;
            let has = w.colony.structures.iter().any(|st| w.defs.structure[st.def].id == "relay" && near(st.x as i32, st.y as i32));
            if !has {
                let spot = (0..joined.len()).filter(|&i| joined[i] && near((i % sw) as i32, (i / sw) as i32)).filter(|&i| w.colony.can_place(&w.defs, &w.city.sim, "relay", (i % sw) as u16, (i / sw) as u16).is_ok()).min_by_key(|&i| (way[i], i));
                if let Some(i) = spot {
                    let (x, y) = ((i % sw) as u16, (i / sw) as u16);
                    if w.colony.place(&w.defs, &w.city.sim, "relay", x, y).is_ok() {
                        out.push(format!("Relay placed at {x},{y} on the slab at {cx},{cy}"));
                        return;
                    }
                }
            }
        }
        // the frontier: has it stood?
        self.stood = if d >= self.last_way { self.stood + 1 } else { 0 };
        self.last_way = d;
        let anchor = |w: &World, def: &str| (0..joined.len()).filter(|&i| joined[i] && way[i] != usize::MAX).filter(|&i| w.colony.can_place(&w.defs, &w.city.sim, def, (i % sw) as u16, (i / sw) as u16).is_ok()).min_by_key(|&i| (way[i], i));
        let within = |w: &World, def: &str, x: u16, y: u16, r: i32| w.colony.structures.iter().any(|st| w.defs.structure[st.def].id == def && (st.x as i32 - x as i32).abs().max((st.y as i32 - y as i32).abs()) <= r);
        let pending = w.colony.structures.iter().any(|st| !st.built && matches!(w.defs.structure[st.def].id.as_str(), "relay" | "pit"));
        if pending {
            return;
        }
        if let Some(i) = anchor(w, "relay") {
            let (x, y) = ((i % sw) as u16, (i / sw) as u16);
            if !within(w, "relay", x, y, 6) && w.colony.place(&w.defs, &w.city.sim, "relay", x, y).is_ok() {
                out.push(format!("Relay placed at {x},{y}, {d} tiles from the crossing at {cx},{cy}"));
                return;
            }
        }
        if self.stood >= 1 {
            if let Some(i) = anchor(w, "pit") {
                let (x, y) = ((i % sw) as u16, (i / sw) as u16);
                if !within(w, "pit", x, y, 3) {
                    if let Ok(id) = w.colony.place(&w.defs, &w.city.sim, "pit", x, y) {
                        w.colony.set_quota(&w.defs, id, Some(FRONT_KEEP));
                        out.push(format!("a Pit at the frontier {x},{y}, {d} tiles from the crossing, keeping {:.0}", FRONT_KEEP.target));
                    }
                }
            }
        }
    }

    /// Chrome bought with the eddies past a reserve, the cheapest a unit at any stall in reach, while the Stash holds
    /// less than a research node's and a Relay's (the haul's Chrome feeds the Heart; research and the climb buy theirs).
    fn buy(&mut self, w: &mut World, out: &mut Vec<String>) {
        let chrome = w.stash.handles().iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == Category::Chrome).count();
        let riding: u16 = w.ledger.orders.iter().map(|o| o.units).sum();
        if chrome + riding as usize >= CHROME_WANTED || w.ledger.eddies < EDDY_RESERVE {
            return;
        }
        let mut best: Option<(u32, u8, u8, u16, u16)> = None;
        for m in crate::city::Steward::in_reach(w) {
            let g = w.city.markets[m];
            let Some(f) = w.economy.faction_at(g) else { continue };
            for (entry, units, each) in w.economy.stock(f, &w.table, 0, false, false) {
                let is_chrome = w.economy.stock_item(f, entry).is_some_and(|i| w.table.def(i).category == Category::Chrome);
                if is_chrome && units > 0 && best.is_none_or(|b| each < b.0) {
                    best = Some((each, m as u8, f, entry, units));
                }
            }
        }
        let Some((each, m, f, entry, units)) = best else { return };
        let afford = ((w.ledger.eddies - EDDY_RESERVE) / each.max(1)) as u16;
        let n = units.min(afford).min((CHROME_WANTED - chrome) as u16);
        if n == 0 {
            return;
        }
        let plan = w.plan.clone();
        if crate::trade::order(&mut w.economy, &mut w.ledger, &w.table, &w.city, &w.colony, &w.defs, &w.roster, &plan, m, f, entry, n).is_ok() {
            out.push(format!("{n} Chrome ordered from the {} at {each} e each", w.economy.name(f)));
        }
    }

    /// The feed: the Pits' quotas take Common Chrome only (a hundredth of a bias a feed), and only the Chrome past
    /// what research and the Relays want — the Stash's Chrome is research first, the ground second (this design's
    /// steward); Ivan's hand feeds the item that pulls hardest toward the aim at three times its bias.
    fn steer(&mut self, w: &mut World, out: &mut Vec<String>) {
        let chrome = w.stash.handles().iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == Category::Chrome).count();
        let feeding = chrome >= CHROME_WANTED;
        let pits: Vec<(u32, bool)> = w.colony.pits(&w.defs).into_iter().filter_map(|id| w.colony.get(id).map(|s| (id, w.defs.structure[s.def].id == "heart"))).collect();
        for (id, heart) in pits {
            let target = if !feeding { 0.0 } else if heart { crate::city::HEART_KEEP.target } else { FRONT_KEEP.target };
            let q = QuotaSet { target, ceiling: 0 };
            if w.colony.get(id).and_then(|s| s.quota) != Some(q) {
                w.colony.set_quota(&w.defs, id, Some(q));
            }
        }
        if feeding != self.feeding {
            self.feeding = feeding;
            out.push(if feeding { "the Pits feed again: the Stash holds the Chrome research wants".into() } else { "the Pits stop feeding: research and the Relays want the Chrome".into() });
        }
    }

    /// Ivan: in chapter 6 to the market of the faction he can ally, to Parley; otherwise at the Heart, feeding by hand
    /// the item that pulls hardest toward the aim.
    fn ivan(&mut self, w: &mut World, out: &mut Vec<String>) {
        let day = w.day;
        if w.campaign.ivan.fell {
            return;
        }
        if w.campaign.chapter == 6 && w.economy.s.ally.is_none() {
            if let Ok(line) = crate::ivan::parley(&mut w.campaign, &w.city, &mut w.colony, &mut w.economy, day, &mut w.log) {
                out.push(line);
            }
            // walk to the market of the faction with the best standing
            let best = (0..w.economy.s.standing.len() as u8).filter(|&f| !w.economy.file.factions[f as usize].accord).filter_map(|f| crate::trade::market_of(&w.economy, &w.city, &w.colony, f).map(|m| (w.economy.s.standing[f as usize].rep, w.economy.s.standing[f as usize].favour, m))).max_by_key(|&(r, fav, m)| (r, fav, std::cmp::Reverse(m)));
            if let Some((_, _, m)) = best {
                let g = w.city.markets[m as usize];
                let sw = w.city.sim.w;
                let _ = crate::ivan::order(&mut w.campaign, &w.city.sim, (g % sw) as u16, (g / sw + 1) as u16);
            }
            return;
        }
        let Some(aim) = self.aim else { return };
        if crate::ivan::pit_here(&w.campaign, &w.city, &w.colony, &w.defs).is_none() {
            let (hx, hy) = heart_xy(w);
            let _ = crate::ivan::order(&mut w.campaign, &w.city.sim, hx, hy);
            return;
        }
        let best = w.stash.handles().into_iter().filter(|(_, _, it)| matches!(w.table.def(it.def).category, Category::Chrome | Category::Relic)).map(|(h, _, it)| {
            let d = w.table.def(it.def);
            let b = d.bias_vector();
            let mine: f32 = aim.genes.iter().map(|&g| b[g]).sum();
            let rivals: f32 = rival_genes(aim).iter().map(|&g| b[g]).sum();
            ((mine - rivals) * tier_drift(d.tier), h)
        }).filter(|&(s, _)| s > 0.0).max_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(b.1.cmp(&a.1)));
        if let Some((_, h)) = best {
            if let Ok(line) = crate::ivan::feed(&mut w.campaign, &mut w.city, &mut w.colony, &w.defs, &mut w.stash, &w.table, h, day, &mut w.log) {
                out.push(line);
            }
        }
    }
}

fn tier_drift(t: Tier) -> f32 {
    match t {
        Tier::Common => 0.01,
        Tier::Uncommon => 0.03,
        Tier::Rare => 0.07,
        Tier::Blackline => 0.15,
        Tier::Relic => 0.40,
    }
}

/// The genes of the endings an aim must not raise (the other pairs, less the aim's own).
fn rival_genes(aim: Aim) -> Vec<usize> {
    use mat::genome::*;
    let pairs = [[LUME, THRIFT], [CHROME, CONDUCTANCE], [APPETITE, FERAL], [THRIFT, TOXICITY]];
    let mut v: Vec<usize> = pairs.iter().flatten().copied().filter(|g| !aim.genes.contains(g)).collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// The campaign's result: every shift, and whether and where it ended.
#[derive(Clone, Debug, Default)]
pub struct CampaignRun {
    pub seed: u64,
    pub aim: String,
    pub shifts: Vec<CampaignShift>,
    pub ending: Option<String>,
    pub warchest: Option<u32>,
    pub log: String,
    pub seconds: f32,
}

/// Play a campaign for an aim, up to `max` shifts or its ending; `each` hears every shift.
pub fn play(seed: u64, aim: Aim, max: u32, each: &mut dyn FnMut(&CampaignShift, &ShiftRecord)) -> (CampaignRun, World) {
    let started = std::time::Instant::now();
    let table = items::ItemTable::from_toml(items::ITEMS_TOML).expect("items.toml");
    let w = World::found_with(seed, table, story::Campaign::new(Mode::Campaign, story::Difficulty::Culvert), mat::Genome::default());
    let mut p = Player::new(w, seed, Haul::Feed);
    p.steward.wanted = vec!["c1_forge", "c2_gate", "c2_turret", "f1_pens", "f2_still"];
    let mut cs = CampaignSteward { aim: Some(aim), ..Default::default() };
    let mut out = CampaignRun { seed, aim: aim.ending.to_string(), ..Default::default() };
    for _ in 0..max {
        let run_on = cs.run_on(&p.w);
        let story_from = p.w.log.events.len();
        let rec = p.shift(run_on, &mut |w| cs.act(w)).clone();
        let w = &p.w;
        let joined = logistics::joined_to_heart(&w.city.sim);
        let cs_row = CampaignShift {
            day: rec.day,
            chapter: w.campaign.chapter,
            stratum: w.threats.stratum,
            run_on,
            result: rec.result.clone(),
            population: w.roster.records.len(),
            beds: w.colony.bed_count(&w.defs).0,
            cells: joined.iter().filter(|&&j| j).count(),
            top_row: (0..joined.len()).filter(|&i| joined[i]).map(|i| i / w.city.sim.w).min().unwrap_or(0),
            genes: w.city.sim.genome.0,
            research: w.colony.progress.tech.project_name().unwrap_or_default(),
            open: [w.colony.progress.tech.open[0], w.colony.progress.tech.open[1], w.colony.progress.tech.open[2]],
            defences: count(w, "gate", true) + count(w, "turret", true),
            stock: {
                let n = |c: Category| w.stash.handles().iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == c).count();
                (n(Category::Part), n(Category::Chrome), w.ledger.eddies)
            },
            bloom: {
                let sim = &w.city.sim;
                let b: Vec<usize> = (0..sim.hy.len()).filter(|&i| sim.is_bloom(i)).collect();
                (b.len(), b.iter().filter(|&&i| sim.hy[i] == mat::sim::hy::NODE).count(), w.threats.bodies.iter().filter(|b| b.role == threat::Role::Crawler && !b.dead).count())
            },
            decisions: rec.decisions.clone(),
            story: w.log.events[story_from..].iter().filter(|e| e.what.contains("chapter") || e.what.contains("reached") || e.what.contains("slab") || e.what.contains("ending") || e.what.contains("envoy") || e.what.contains("accord") || e.what.contains("alliance")).map(|e| e.what.clone()).collect(),
        };
        each(&cs_row, &rec);
        out.shifts.push(cs_row);
        if p.w.campaign.over() {
            break;
        }
    }
    out.ending = p.w.campaign.ending.clone();
    out.warchest = p.w.campaign.warchest;
    out.log = std::mem::take(&mut p.out.log);
    out.seconds = started.elapsed().as_secs_f32();
    (out, p.w)
}
