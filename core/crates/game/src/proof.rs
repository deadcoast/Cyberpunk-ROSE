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

/// The ending a scripted campaign plays for: its genes (the pair the ending scores) and whether it defends the city.
#[derive(Clone, Copy, Debug)]
pub struct Aim {
    pub ending: &'static str,
    pub genes: [usize; 2],
    /// raises the defences, and refuses the accord once a Gate and a Turret stand; Scattered's script raises none and
    /// refuses it, and the chapter's Cleansing, whole the next shift, burns the Heart
    pub defend: bool,
}

/// The five scripts (the genes are Full Design v3.19's Endings; Scattered refuses the accord with no defence raised, and its chapter-3 Cleansing burns the Heart).
pub fn aims() -> [Aim; 5] {
    use mat::genome::*;
    [
        Aim { ending: "garden", genes: [LUME, THRIFT], defend: true },
        Aim { ending: "hive", genes: [CHROME, CONDUCTANCE], defend: true },
        Aim { ending: "world_eater", genes: [APPETITE, FERAL], defend: true },
        Aim { ending: "accord", genes: [THRIFT, TOXICITY], defend: true },
        Aim { ending: "scattered", genes: [LUME, THRIFT], defend: false },
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
    /// the loudest reached zone's Attention and what it gained at the shift's end
    pub attention: (f32, f32),
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
    /// the Pit at the climb's frontier, the one the climb feeds (a Pit left behind by the frontier pulls nothing)
    front_pit: Option<u32>,
    /// the grates whose way down it has tried to shut (where the ground is wider than two Gates, once is enough)
    gates_tried: Vec<usize>,
    /// chapter 4's Bloom stands farther than a Pen can be raised from: the frontier grows toward it, not up
    to_bloom: bool,
}

/// Defences the steward raises before a Cleansing (this design's: the `rose cleansing` proof held the Heart with two
/// Gates and a manned Turret).
/// Chrome the steward keeps in the Stash for research and Relays (a tier-3 node's eight and a Relay's six), and the
/// eddies it never spends on it.
const CHROME_WANTED: usize = 14;
/// Chrome the frontier's Pits leave in the Stash: a Relay's six (research takes its node's when it starts one).
const FRONT_RESERVE: usize = 6;
const EDDY_RESERVE: u32 = 100;
/// Parts the steward buys up to with the eddies past the reserve: a Bunkhouse's ten and a tier-3 node's ten.
const PARTS_WANTED: usize = 20;
/// Tiles a chapter-4 Pen stands off the nearest crawler's home, and the farthest it may (a Bloom's crawler goes for a
/// structure within its territory's 16 of its Node, and comes within the Pen's 4 on the way).
const PEN_OFF: i32 = 9;
const PEN_REACH: u16 = 16;
/// The gauge a grate's sales stop under (the Sweep's line, 60, less a shift's trade at a busy grate).
const GAUGE_LINE: f32 = 45.0;
/// Scrap Turrets by the Heart.
const TURRETS: usize = 4;
/// Parts a home leaves in the Stash past every ghost's bill (a city that houses every flush has none for its Vaults).
const BED_RESERVE: i32 = 20;
/// Beds kept ahead of the slimos and the waiting: a Bunkhouse's six (a flush is 1–6 a stratum of its grate).
const SPARE_BEDS: i32 = 6;
/// Below these cells joined to the Heart the Heart's Pit feeds whatever research wants (a Cleansing burns the Mat
/// back to the Heart's few dozen; the founding leaves 107–290).
const HUNGRY: usize = 100;
/// Joined tissue a bed asks of the Mat: a slimo eats the surplus about its home or two eddies of paste, and a city of
/// seven hundred ate every eddy it made and bought no Parts (this script's).
const CELLS_A_MOUTH: usize = 80;
/// A Vault's Parts, and the Parts a Vault leaves for the frontier's Pit (the climb before the Blackline's shelter).
const VAULT_PARTS: i32 = 12;
const FRONT_PARTS: i32 = 4;
/// Beds the city keeps however small the Mat (the founding's eight and the first flush).
const MOUTHS_LEAST: i32 = 20;
/// The Heart's quota while the Mat is burned back to it, Rare at most: a deep run's Chrome is Rare, and a Heart kept on
/// Common and Uncommon alone stood at two tiles for fifty shifts (Blackline stays the contraband's to place: its drift
/// is the widest, and the only Chrome with a Feral Drive in its bias is Blackline).
const HEART_HUNGRY: QuotaSet = QuotaSet { target: 6.0, ceiling: 2 };
/// A frontier Pit's quota: nutrient kept, Rare at most (the Chrome a run past the Storm Drains brings is Rare, and a
/// trunk kept on Uncommon stood starving a shaft's length under the climb with a hundred Chrome in the Stash).
const FRONT_KEEP: QuotaSet = QuotaSet { target: 4.0, ceiling: 2 };

/// Parts the Stash holds past every unpaid ghost's bill.
fn spare_parts(w: &World) -> i32 {
    let parts = w.stash.handles().iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == Category::Part).count() as i32;
    let owed: i32 = w.colony.structures.iter().filter(|s| !s.built && !s.paid).map(|s| w.defs.structure[s.def].cost.parts as i32).sum();
    parts - owed
}

fn count(w: &World, def: &str, built: bool) -> usize {
    w.colony.structures.iter().filter(|s| w.defs.structure[s.def].id == def && (s.built || !built)).count()
}

/// The Feral Drive over which a strain's feral tissue is a Bloom (Pressure: Blooms).
fn threat_bloom_line(w: &World) -> f32 {
    w.threats.rules.bloom.feral_drive
}

fn heart_xy(w: &World) -> (u16, u16) {
    w.city.sim.heart.map(|(x, y)| (x as u16, y as u16)).unwrap_or((0, 0))
}

/// A home's anchor: joined to the Heart, five tiles or more from it (a meal is taken from the tissue about a home,
/// and homes round the Heart starve the Heart's own tissue to nothing) and ten from the grates (the squads' way down),
/// where Deposits feed the tissue about it, then where it holds the most nutrient; nearest the Heart on a tie.
fn richest_spot(w: &World, def: &str) -> Option<(u16, u16)> {
    let sim = &w.city.sim;
    let joined = logistics::joined_to_heart(sim);
    let (hx, hy) = (heart_xy(w).0 as i32, heart_xy(w).1 as i32);
    let food = |i: usize| -> f32 {
        let (x, y) = ((i % sim.w) as i32, (i / sim.w) as i32);
        let mut t = 0.0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && (nx as usize) < sim.w && (ny as usize) < sim.h {
                    let j = ny as usize * sim.w + nx as usize;
                    t += sim.n[j] + if sim.sub[j] == mat::sim::sub::DEPOSIT { 1.0 } else { 0.0 };
                }
            }
        }
        t
    };
    let far = |i: usize| ((i % sim.w) as i32 - hx).abs().max(((i / sim.w) as i32 - hy).abs());
    // and off the way the squads come down: ten tiles or more from every grate the Mat reaches
    let grates: Vec<(i32, i32)> = w.city.markets.iter().map(|&g| ((g % sim.w) as i32, (g / sim.w) as i32)).collect();
    let off_way = |i: usize| grates.iter().all(|&(gx, gy)| ((i % sim.w) as i32 - gx).abs().max(((i / sim.w) as i32 - gy).abs()) >= 10);
    (0..joined.len())
        .filter(|&i| joined[i] && far(i) >= HOME_FROM_HEART && off_way(i) && w.colony.can_place(&w.defs, sim, def, (i % sim.w) as u16, (i / sim.w) as u16).is_ok())
        .map(|i| (food(i), far(i), i))
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(b.1.cmp(&a.1)).then(b.2.cmp(&a.2)))
        .map(|(_, _, i)| ((i % sim.w) as u16, (i / sim.w) as u16))
}

/// Homes stand this far from the Heart (Chebyshev): its 3 x 3 and the ring the Heart's Pit feeds are its own.
const HOME_FROM_HEART: i32 = 5;

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
        self.chapter_builds(w, &mut out);
        self.buy(w, &mut out);
        self.climb(w, &mut out);
        self.beds(w, &mut out);
        self.loops(w, &mut out);
        self.steer(w, &mut out);
        self.ivan(w, &mut out);
        self.keep_lead(w, &mut out);
        self.contraband(w, &mut out);
        w.publish();
        // onto the log what the verbs did not put there themselves
        let tick = w.city.tick;
        for o in &out {
            if !w.log.events.iter().rev().take_while(|e| e.tick == tick).any(|e| e.what == *o) {
                w.log.push(tick, crate::log::Class::Decision, o.clone());
            }
        }
        out
    }

    /// The stratum the run goes to: the Vault while chapter 5 waits on a Runner home from it, else the lowest the Board
    /// offers that has no loop yet.
    pub fn run_on(&self, w: &World) -> u8 {
        let board: Vec<u8> = logistics::board_strata(w.threats.stratum, w.city.sim.strata).collect();
        if w.campaign.chapter == 5 && !w.campaign.vault_home && board.contains(&5) {
            return 5;
        }
        // a seasoned Runner takes the lowest route not yet on a loop, so each earns its two runs home and a crew of its
        // own; a green one the lowest (the deep routes kill a Runner of Scav 2 every time)
        let seasoned = w.roster.eligible().first().is_some_and(|r| r.skill(people::Skill::Scav) >= SEASONED);
        let open = board.iter().copied().find(|&s| !w.colony.loops.iter().any(|l| l.stratum == s));
        open.filter(|_| seasoned).or_else(|| board.first().copied()).unwrap_or(1)
    }

    /// Route loops (Full Design v3.18): every Board route the Runners have come home from twice gets a crew of one,
    /// the best Scav not posted elsewhere — a loop is a haul a shift, and the haul's Chrome is the climb's food.
    fn loops(&mut self, w: &mut World, out: &mut Vec<String>) {
        if !w.colony.progress.tech.flag("route_loops") {
            return;
        }
        let board: Vec<u8> = logistics::board_strata(w.threats.stratum, w.city.sim.strata).collect();
        let (research, guard) = (people::Job::Research, people::Job::Guard);
        for s in board {
            if w.colony.loops.iter().any(|l| l.stratum == s) || w.colony.homes_from(s) < colony::LOOP_AFTER {
                continue;
            }
            let looping: Vec<u32> = w.colony.loops.iter().map(|l| l.slimo).collect();
            let mut crew: Vec<(u8, u32, String)> = w.roster.eligible().into_iter().filter(|r| !looping.contains(&r.id) && r.priorities[research.index()] != 4 && r.priorities[guard.index()] != 4).map(|r| (r.skill(people::Skill::Scav), r.id, r.full_name())).collect();
            crew.sort_by_key(|c| (std::cmp::Reverse(c.0), c.1));
            // the best stays the Runner
            if let Some((_, id, name)) = crew.get(1).cloned() {
                if w.colony.set_loop(s, Some(id)).is_ok() {
                    out.push(format!("{name} loops the S{s} route"));
                }
            }
        }
    }

    fn answer(&mut self, w: &mut World, out: &mut Vec<String>) {
        // refused when the city can meet the whole Cleansing the next shift — a Gate and a Turret standing, as `rose
        // cleansing`'s defended city — for an accepted accord maps the Heart, and every squad after down the Heart's zone walks for it
        // (Narrative and Ivan: the accord); taken when it cannot. Scattered refuses with nothing raised.
        let defended = count(w, "gate", true) > 0 && count(w, "turret", true) > 0;
        let accept = self.aim.is_some_and(|a| a.defend && !defended);
        if matches!(w.campaign.accord, Some(Accord::Waiting { .. })) {
            let day = w.day;
            let (c, mut p) = w.campaign_parts();
            if crate::campaign::answer_accord(c, &mut p, accept, day).is_ok() {
                out.push(format!("the envoy answered: {}", if accept { "accepted" } else { "refused" }));
            }
        }
    }

    /// The Job grid: while a project stands, the slimo of best Tweak that can research (Tweak 2) puts Research
    /// first; before a Cleansing (chapter 3 on, and the Great one) Conscription and the Quarantine Protocol are signed
    /// when Authority allows.
    fn jobs(&mut self, w: &mut World, out: &mut Vec<String>) {
        // two researchers for every Research slot the Benches open (one a Bench below tier 3): a slimo rests one
        // shift of the day's three, so two keep the slot worked
        let research = people::Job::Research;
        let benches = 2 * count(w, "bench", true);
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
        // before a Sweep or a Cleansing (due this shift, down, or the chapter's own coming), a slimo ordered to every
        // Turret and Gate seat, the best Fight first (Full Design v3.12, Orders: over priorities and Any — a Guard's
        // priority loses a shift to a foreman's 4 on a job the slimo is better at)
        let heavy = |st: usize| w.threats.rules.stage[st].id != "patrol";
        let coming = w.threats.due.iter().any(|d| heavy(d.stage)) || w.threats.events.iter().any(|e| e.phase != threat::Phase::Done && heavy(e.stage)) || w.campaign.cleansing_at.is_some_and(|at| w.day + 1 >= at);
        if coming {
            let seats: Vec<u32> = w.colony.structures.iter().filter(|s| s.built && matches!(w.defs.structure[s.def].id.as_str(), "turret" | "gate")).map(|s| s.id).collect();
            let mut crew: Vec<(u8, u32)> = w.roster.records.iter().filter(|r| r.is_up() && r.priorities[research.index()] != 4).map(|r| (r.skill(people::Skill::Fight), r.id)).collect();
            crew.sort_by_key(|c| (std::cmp::Reverse(c.0), c.1));
            let mut ordered = 0;
            for (sid, (_, slimo)) in seats.into_iter().zip(crew) {
                if w.colony.order(&w.defs, slimo, sid).is_ok() {
                    ordered += 1;
                }
            }
            if ordered > 0 {
                out.push(format!("{ordered} ordered to the Turrets and Gates: the white light is coming"));
            }
        }
        // while ghosts wait, Build first for the rest (the Job grid's column): the defences and the climb are built
        // by hands, and mending takes them first
        let ghosts = w.colony.structures.iter().any(|s| !s.built);
        let build = people::Job::Build;
        for r in w.roster.records.iter_mut().filter(|r| r.priorities[build.index()] != 0) {
            // a researcher or a Guard builds last: its post comes first
            let posted = r.priorities[research.index()] == 4 || r.priorities[guard.index()] == 4;
            r.priorities[build.index()] = if posted { 1 } else if ghosts { 4 } else { 2 };
        }
        let threatened = w.campaign.chapter >= 3 && self.aim.is_none_or(|a| a.defend);
        // Conscription, and for a strain that blooms (Feral Drive over 0.5) the Quarantine Protocol: a Cleansing burns
        // the Mat back to the Heart, and what it cuts off goes feral in a shift or three and blooms against the city
        // unless it is severed (a strain that does not bloom keeps what it cut off, to join again)
        let blooms = w.city.sim.genome.0[mat::genome::FERAL] > threat_bloom_line(w);
        let laws = [("conscription", "Conscription signed: every D2 is militia while the white light is down", true), ("quarantine", "the Quarantine Protocol signed: what loses signal for a shift is severed before it blooms", blooms)];
        for (id, line, want) in laws {
            if threatened && want && w.colony.charter.law(id).is_none() {
                if let Some(law) = colony::LawFile::shipped().get(id).cloned() {
                    if w.colony.sign(&w.defs, &w.city.sim, &law).is_ok() {
                        out.push(line.into());
                    }
                }
            }
        }
    }

    /// Beds for the slimos, the newcomers waiting at a grate and a Bunkhouse's worth ahead (Recruitment: a batch not
    /// sheltered within two shifts goes feral), every ghost placed this shift, each where the Mat is richest.
    fn beds(&mut self, w: &mut World, out: &mut Vec<String>) {
        let def = if w.colony.progress.tech.structure_locked("bunkhouse").is_none() { "bunkhouse" } else { "pod" };
        // no more mouths than the Mat joined to the Heart feeds: every slimo eats from the tissue about its home before
        // paste (Homes, meals), and a city past it eats every nutrient the frontier would grow with
        let fed = (logistics::joined_to_heart(&w.city.sim).iter().filter(|&&j| j).count() / CELLS_A_MOUTH) as i32;
        let want = ((w.roster.records.len() + w.roster.waiting.len()) as i32 + SPARE_BEDS).min(fed.max(MOUTHS_LEAST));
        let ghost_beds = |w: &World| -> i32 { w.colony.structures.iter().filter(|s| !s.built).filter_map(|s| w.defs.structure[s.def].beds.first().copied()).sum() };
        // the Parts a ghost of the defences, a Vault, a Relay or a node waits on come first: a home is placed only
        // while the Stash keeps a reserve past every ghost's bill
        let parts = |w: &World| w.stash.handles().iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == Category::Part).count() as i32;
        let owed = |w: &World| -> i32 { w.colony.structures.iter().filter(|s| !s.built && !s.paid).map(|s| w.defs.structure[s.def].cost.parts as i32).sum() };
        for _ in 0..8 {
            let (beds, _) = w.colony.bed_count(&w.defs);
            if beds + ghost_beds(w) >= want || parts(w) - owed(w) < BED_RESERVE {
                break;
            }
            // a slimo with a bed eats from the tissue about its home (Homes, meals): away from the Heart, on the
            // Deposits, else where the Mat holds most; with no such spot, nearest the Heart as before
            let Some((x, y)) = richest_spot(w, def).or_else(|| nearest_spot(w, def, heart_xy(w))) else { break };
            if w.colony.place(&w.defs, &w.city.sim, def, x, y).is_err() {
                break;
            }
            out.push(format!("{} placed at {x},{y} for the slimos and the newcomers without beds", w.defs.get(def).map_or(def.to_string(), |d| d.name.clone())));
        }
    }

    /// The chapters' structures: the forge and the market (2), the defences before a Cleansing (3, 7), a Pen by the
    /// Bloom (4), the Still (5).
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
        let defend = self.aim.is_none_or(|a| a.defend);
        if defend && count(w, "gate", false) == 0 && w.colony.progress.tech.structure_locked("gate").is_none() {
            if let Some(g) = crate::city::first_grate(w) {
                out.extend(crate::siege::gates(w, g));
            }
        }
        // once Sweeps are a stratum off, every reached grate's way down to the Heart shut, one a shift: a Patrol or a
        // Sweep turns back at a Gate (Full Design v3.20, Bodies and defence), and one let walk to the Heart bleaches the
        // tissue round it dead however fast the Turrets drop it
        if defend && w.threats.stratum >= 3 && w.colony.progress.tech.structure_locked("gate").is_none() && !pending(w, "gate") {
            // the Heart's stratum first, deepest first: the ways nearest the Heart
            let sw = w.city.sim.w;
            let mut open: Vec<usize> = w.city.markets.iter().copied().filter(|g| !self.gates_tried.contains(g) && !crate::siege::gated(w, *g)).collect();
            open.sort_by_key(|&g| (std::cmp::Reverse(g / sw), g % sw));
            if let Some(&g) = open.first() {
                self.gates_tried.push(g);
                out.extend(crate::siege::gates(w, g));
            }
        }
        // a Turret covers every side of the Heart's walls (within three of its middle: six reaches round the 3 x 3)
        // from the side the squads come down, so it fires on the way in as well as at the walls
        if defend && count(w, "turret", false) < TURRETS && !pending(w, "turret") {
            if let Some(g) = crate::city::first_grate(w) {
                if let Some((x, y)) = crate::siege::turret_spot(w, g) {
                    if w.colony.place(&w.defs, &w.city.sim, "turret", x, y).is_ok() {
                        out.push(format!("Scrap Turret placed at {x},{y}, by the Heart on the grate's side"));
                    }
                }
            }
        }
        // a Vault for the Blackline and Relics a Cleansing leaves, once more lie in the open than the Vaults hold
        // as many as the hot goods past the Vaults' room ask, four a shift (a deep run brings home five and more)
        if w.colony.progress.tech.structure_locked("vault").is_none() {
            for _ in 0..4 {
                let hot = w.stash.exposed_contraband(&w.table);
                let room: usize = w.colony.structures.iter().filter(|s| w.defs.structure[s.def].id == "vault").map(|s| w.defs.structure[s.def].value_at("capacity", s.tier).max(0) as usize).sum();
                if hot <= room || spare_parts(w) < VAULT_PARTS + FRONT_PARTS || place_near(w, "vault", heart, " for the Blackline in the open", out).is_none() {
                    break;
                }
            }
        }
        // chapter 4: a Pen on the city's Mat nine off the Node a crawler stands for, within its territory's sixteen,
        // and a slimo ordered to Tend it (a crawler that comes within 4 is tamed; two tamed close the chapter); with
        // none in reach the frontier grows toward the Bloom, and Strain grafting takes its tissue back where it touches
        if ch == 4 && w.campaign.open_since > 0 && w.campaign.bloom {
            let sim = &w.city.sim;
            let sw = sim.w;
            // tiles from the nearest Node a crawler stands for (its home: it hunts a slimo within 12 of where it is
            // and goes for a structure within 16 of it), as the crow flies; with none standing, from every Bloom Node
            let mut far = vec![u16::MAX; sim.hy.len()];
            let mut q = std::collections::VecDeque::new();
            let homes: Vec<usize> = w.threats.bodies.iter().filter(|b| b.role == threat::Role::Crawler && !b.dead && sim.is_bloom(b.home)).map(|b| b.home).collect();
            let seeds: Vec<usize> = if homes.is_empty() { (0..sim.hy.len()).filter(|&i| sim.is_bloom(i) && sim.hy[i] == mat::sim::hy::NODE).collect() } else { homes };
            for i in seeds {
                far[i] = 0;
                q.push_back(i);
            }
            while let Some(i) = q.pop_front() {
                let (x, y) = ((i % sw) as i32, (i / sw) as i32);
                for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx >= 0 && ny >= 0 && (nx as usize) < sw && (ny as usize) < sim.h {
                        let j = ny as usize * sw + nx as usize;
                        if far[j] == u16::MAX && far[i] < PEN_REACH {
                            far[j] = far[i] + 1;
                            q.push_back(j);
                        }
                    }
                }
            }
            let tended = w.colony.structures.iter().filter(|s| w.defs.structure[s.def].id == "pen").any(|s| far[s.y as usize * sw + s.x as usize] <= PEN_REACH);
            self.to_bloom = false;
            if !tended {
                // anywhere the city's signal lets a Pen stand: the frontier past the Relays counts; nine off the
                // crawlers' homes, where they come at the keeper a few at a time (a Pen three off a Bloom of twenty
                // crawlers tamed its two and was chewed down by the rest in seconds)
                let spot = (0..sim.hy.len()).filter(|&i| far[i] <= PEN_REACH && mat::sim::is_hyphal(sim.hy[i]) && !sim.is_bloom(i)).filter(|&i| w.colony.can_place(&w.defs, sim, "pen", (i % sw) as u16, (i / sw) as u16).is_ok()).min_by_key(|&i| ((far[i] as i32 - PEN_OFF).abs(), i));
                if let Some(i) = spot {
                    let (x, y) = ((i % sw) as u16, (i / sw) as u16);
                    if w.colony.place(&w.defs, &w.city.sim, "pen", x, y).is_ok() {
                        out.push(format!("Sewerat Pen placed at {x},{y}, {} from the Bloom's nearest Node (chapter 4)", far[i]));
                    }
                }
                self.to_bloom = spot.is_none();
            }
            let tend = people::Job::Tend;
            let pens: Vec<u32> = w.colony.structures.iter().filter(|s| s.built && w.defs.structure[s.def].id == "pen" && far[s.y as usize * sw + s.x as usize] <= PEN_REACH).map(|s| s.id).collect();
            for sid in pens {
                let keeper = w.roster.records.iter().filter(|r| r.is_up() && r.utility_at(&w.traits, tend, 4).is_some() && !w.colony.orders.contains_key(&r.id)).max_by_key(|r| (r.skill(tend.skill()), std::cmp::Reverse(r.id))).map(|r| r.id);
                if let Some(id) = keeper {
                    let _ = w.colony.order(&w.defs, id, sid);
                }
            }
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
        // the climb cut from the Heart (a Sweep's bleach, a Cleansing's light): the city's tissue its Relays keep alive
        // above the Heart's, which the frontier grows back to before it climbs on — cargo rides only the cords
        // joined to the Heart, so the Pits up there go hungry until it does
        let sim = &w.city.sim;
        let top = (0..joined.len()).filter(|&i| joined[i]).map(|i| i / sw).min().unwrap_or(sim.h);
        let cut: Vec<usize> = (0..joined.len()).filter(|&i| !joined[i] && mat::sim::is_hyphal(sim.hy[i]) && sim.sig[i] > 0.0 && !sim.is_bloom(i) && i / sw + REJOIN_ABOVE < top).collect();
        if cut.len() >= REJOIN_LEAST {
            let way = ground_from(sim, &cut);
            if let Some(d) = (0..way.len()).filter(|&i| joined[i] && way[i] != usize::MAX).map(|i| way[i]).min() {
                self.grow_frontier(w, out, &joined, &way, d, "the cut climb");
                return;
            }
        }
        // chapter 4's Bloom out of a Pen's reach, or Strain grafting held: the frontier grows toward its tissue (the
        // chapter waits on it, and the next one on the stratum the city already holds); grafted, the city's signal
        // takes the Bloom back as far as its Relays reach
        let grafting = w.colony.progress.tech.flag("grafting");
        if (self.to_bloom || grafting) && w.campaign.chapter == 4 && w.campaign.bloom {
            let seeds: Vec<usize> = (0..sim.hy.len()).filter(|&i| sim.wild[i] && mat::sim::is_hyphal(sim.hy[i])).collect();
            let way = ground_from(sim, &seeds);
            if let Some(d) = (0..way.len()).filter(|&i| joined[i] && way[i] != usize::MAX).map(|i| way[i]).min() {
                self.grow_frontier(w, out, &joined, &way, d, "the chapter's Bloom");
                return;
            }
        }
        // the crossing nearest the Mat along the ground, by the floor under its slab: nearest the tissue strong enough
        // to carry a Relay first (thin tissue under one slab is no nearer than a strong cord under the next), then any
        let need = w.defs.get("relay").map_or(0.0, |d| w.colony.sigma_need(d));
        let mut best: Option<((usize, usize), usize, Vec<usize>)> = None;
        for c in w.city.sim.crossings_of(s) {
            let probe = if w.city.sim.sealed(c) { c + sw } else { c };
            let way = logistics::ground_distance(&w.city.sim, probe);
            let reach = |strong: bool| (0..way.len()).filter(|&i| joined[i] && way[i] != usize::MAX && (!strong || w.city.sim.sig[i] >= need)).map(|i| way[i]).min();
            if let Some(d) = reach(false) {
                let key = (reach(true).unwrap_or(usize::MAX), d);
                if best.as_ref().is_none_or(|b| key < b.0) {
                    best = Some((key, c, way));
                }
            }
        }
        let Some(((_, d), c, way)) = best else { return };
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
        let target = format!("the crossing at {cx},{cy}");
        self.grow_frontier(w, out, &joined, &way, d, &target);
    }

    /// The frontier toward `way`'s source: a Relay at the joined tissue nearest it when none stands within six, and a
    /// Pit there fed by quota when the frontier has stood a shift.
    fn grow_frontier(&mut self, w: &mut World, out: &mut Vec<String>, joined: &[bool], way: &[usize], d: usize, target: &str) {
        let sw = w.city.sim.w;
        // the frontier: has it stood?
        self.stood = if d >= self.last_way { self.stood + 1 } else { 0 };
        self.last_way = d;
        let anchor = |w: &World, def: &str| (0..joined.len()).filter(|&i| joined[i] && way[i] != usize::MAX).filter(|&i| w.colony.can_place(&w.defs, &w.city.sim, def, (i % sw) as u16, (i / sw) as u16).is_ok()).min_by_key(|&i| (way[i], i));
        let within = |w: &World, def: &str, x: u16, y: u16, r: i32| w.colony.structures.iter().any(|st| w.defs.structure[st.def].id == def && (st.x as i32 - x as i32).abs().max((st.y as i32 - y as i32).abs()) <= r);
        // a frontier ghost being built holds the next; one the Stash cannot pay for holds nothing (a Pit's Parts gone to
        // the Vaults stood the climb still for a hundred shifts)
        let pending = w.colony.structures.iter().any(|st| !st.built && st.paid && matches!(w.defs.structure[st.def].id.as_str(), "relay" | "pit"));
        if pending {
            return;
        }
        // a Relay where none stands within six; and where the frontier has stood a shift, at its edge whatever stands
        // beyond it: a Relay raised sprouts its neighbours (Full Design v3.20, The Mat's growth), which walks the Mat
        // up a bare shaft its food does not reach and closes a seam no strand's end borders
        if let Some(i) = anchor(w, "relay") {
            let (x, y) = ((i % sw) as u16, (i / sw) as u16);
            let room = !within(w, "relay", x, y, 6) || (self.stood >= 1 && !within(w, "relay", x, y, 1));
            if room && w.colony.place(&w.defs, &w.city.sim, "relay", x, y).is_ok() {
                out.push(format!("Relay placed at {x},{y}, {d} tiles from {target}"));
                return;
            }
        }
        if self.stood >= 1 {
            if let Some(i) = anchor(w, "pit") {
                let (x, y) = ((i % sw) as u16, (i / sw) as u16);
                if !within(w, "pit", x, y, 3) {
                    if spare_parts(w) < FRONT_PARTS {
                        return;
                    }
                    if let Ok(id) = w.colony.place(&w.defs, &w.city.sim, "pit", x, y) {
                        w.colony.set_quota(&w.defs, id, Some(FRONT_KEEP));
                        self.front_pit = Some(id);
                        out.push(format!("a Pit at the frontier {x},{y}, {d} tiles from {target}, keeping {:.0}", FRONT_KEEP.target));
                    }
                }
            }
        }
    }

    /// Chrome and Parts bought with the eddies past a reserve, the cheapest a unit at any stall in reach, while the Stash
    /// holds less than a research node's and a Relay's Chrome, or a Bunkhouse's and a node's Parts (the beds a big flush
    /// asks for would otherwise take every Part research waits on).
    fn buy(&mut self, w: &mut World, out: &mut Vec<String>) {
        // never Chrome the steering would keep off the Pits
        let hold = self.aim.map(|a| held(w, a)).unwrap_or_default();
        for (cat, want, what) in [(Category::Chrome, CHROME_WANTED, "Chrome"), (Category::Part, PARTS_WANTED, "Parts")] {
            let have = w.stash.handles().iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == cat).count();
            let riding: u16 = w.ledger.orders.iter().filter(|o| w.economy.stock_item(o.faction, o.entry).is_some_and(|i| w.table.def(i).category == cat)).map(|o| o.units).sum();
            if have + riding as usize >= want || w.ledger.eddies < EDDY_RESERVE {
                continue;
            }
            let mut best: Option<(u32, u8, u8, u16, u16)> = None;
            for m in crate::city::Steward::in_reach(w) {
                let g = w.city.markets[m];
                let Some(f) = w.economy.faction_at(g) else { continue };
                for (entry, units, each) in w.economy.stock(f, &w.table, 0, false, false) {
                    let is = w.economy.stock_item(f, entry).is_some_and(|i| w.table.def(i).category == cat && !hold.iter().any(|&k| w.table.def(i).bias_vector()[k] > 0.0));
                    if is && units > 0 && best.is_none_or(|b| each < b.0) {
                        best = Some((each, m as u8, f, entry, units));
                    }
                }
            }
            let Some((each, m, f, entry, units)) = best else { continue };
            let afford = ((w.ledger.eddies - EDDY_RESERVE) / each.max(1)) as u16;
            let n = units.min(afford).min((want - have) as u16);
            if n == 0 {
                continue;
            }
            let plan = w.plan.clone();
            if crate::trade::order(&mut w.economy, &mut w.ledger, &w.table, &w.city, &w.colony, &w.defs, &w.roster, &plan, m, f, entry, n).is_ok() {
                out.push(format!("{n} {what} ordered from the {} at {each} e each", w.economy.name(f)));
            }
        }
    }

    /// The feed: the Heart's quota takes Common Chrome only (a hundredth of a bias a feed), the frontier's up to Rare
    /// (the climb is the Chrome's first use; a stratum's 48 rows are about ten nutrient), past a Relay's Chrome; the Heart's past what research and the Relays want (the Heart's Pit is nearest, so it
    /// is served first when both pull) — unless the Mat is burned back to the Heart, when it eats first; Ivan's hand feeds the item that widens the aim's lead most at three times its bias.
    fn steer(&mut self, w: &mut World, out: &mut Vec<String>) {
        let chrome = w.stash.handles().iter().filter(|(_, _, it)| !it.pinned && w.table.def(it.def).category == Category::Chrome).count();
        // a Mat burned back to the Heart eats first, up to Rare: what it lost goes feral unless it regrows to it, and
        // the frontier waits
        let hungry = logistics::joined_to_heart(&w.city.sim).iter().filter(|&&j| j).count() < HUNGRY;
        let feeding = chrome >= CHROME_WANTED || hungry;
        // the frontier's Pit feeds on what a Relay leaves (the climb stands without it); a Pit the frontier left behind
        // pulls nothing while Chrome is short, or the nearest of them, nearest the Heart, would take the climb's every
        // Chrome — with the Stash past what research wants, they all feed the trunk the climb hangs by
        let frontier = chrome >= FRONT_RESERVE && !hungry;
        let trunk = chrome >= CHROME_WANTED && !hungry;
        let pits: Vec<(u32, bool)> = w.colony.pits(&w.defs).into_iter().filter_map(|id| w.colony.get(id).map(|s| (id, w.defs.structure[s.def].id == "heart"))).collect();
        for (id, heart) in pits {
            let q = if heart && hungry {
                HEART_HUNGRY
            } else if heart {
                QuotaSet { target: if feeding { crate::city::HEART_KEEP.target } else { 0.0 }, ceiling: 0 }
            } else {
                QuotaSet { target: if trunk || (frontier && self.front_pit == Some(id)) { FRONT_KEEP.target } else { 0.0 }, ceiling: FRONT_KEEP.ceiling }
            };
            if w.colony.get(id).and_then(|s| s.quota) != Some(q) {
                w.colony.set_quota(&w.defs, id, Some(q));
            }
        }
        if feeding != self.feeding {
            self.feeding = feeding;
            out.push(if feeding { "the Heart's Pit feeds: the Mat is burned back, or the Stash holds the Chrome research wants".into() } else { "the Heart's Pit stops feeding: research, the Relays and the climb want the Chrome".into() });
        }
    }

    /// The steering. The Stash keeps (pins: no quota takes it) the Chrome and Relics whose bias raises a gene the
    /// script holds (`held`): genes only rise with a feed, so a rival held under one of its pair is held for good. In
    /// the last chapter it keeps every Chrome and Relic whose drift would narrow the lead and feeds the Heart by hand
    /// every one whose drift widens it, so the genome the Great Cleansing reads is the one the script steered.
    fn keep_lead(&mut self, w: &mut World, out: &mut Vec<String>) {
        let Some(aim) = self.aim else { return };
        let hold = held(w, aim);
        let g = self.projected(w);
        let (ch, rep) = (w.campaign.chapter, ally_step(w));
        let base = lead(aim, &g, ch, rep);
        let feeds: Vec<(f32, u32)> = w.stash.handles().into_iter().filter(|(_, _, it)| matches!(w.table.def(it.def).category, Category::Chrome | Category::Relic)).map(|(h, _, it)| (if hold.iter().any(|&k| w.table.def(it.def).bias_vector()[k] > 0.0) { f32::MAX } else { base - lead(aim, &after(w, g, w.table.def(it.def), 1.0), ch, rep) }, h)).collect();
        let kept: Vec<u32> = feeds.iter().filter(|&&(d, _)| d == f32::MAX || (ch == 7 && d > 0.0)).map(|&(_, h)| h).collect();
        let mut newly = 0;
        for (_, h) in feeds {
            if let Some(it) = w.stash.get_any_mut(h) {
                let keep = kept.contains(&h);
                newly += (keep && !it.pinned) as usize;
                it.pinned = keep;
            }
        }
        if newly > 0 {
            out.push(format!("{newly} kept off the Pits: their drift would narrow the {} lead", aim.ending));
        }
        // and in the last chapter the Feed verb itself: every item whose drift widens the lead goes to the Heart, the
        // widest first, an overshoot off a rival's ceiling as good as a pull toward the script's own
        if ch != 7 {
            return;
        }
        let mut g = g;
        let mut fed = 0;
        loop {
            let now = lead(aim, &g, ch, rep);
            let best = w.stash.handles().into_iter().filter(|(_, _, it)| matches!(w.table.def(it.def).category, Category::Chrome | Category::Relic) && w.table.def(it.def).feed > 0.0).map(|(h, _, it)| (lead(aim, &after(w, g, w.table.def(it.def), 1.0), ch, rep) - now, h, it.def)).filter(|&(d, _, _)| d > 1e-4).max_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(b.1.cmp(&a.1)));
            let Some((_, h, def)) = best else { break };
            if let Some(it) = w.stash.get_any_mut(h) {
                it.pinned = false;
            }
            if !w.ledger.queue(&mut w.stash, h, inventory::Verb::Feed { pit: 0 }, w.day) {
                break;
            }
            g = after(w, g, w.table.def(def), 1.0);
            fed += 1;
        }
        if fed > 0 {
            out.push(format!("{fed} fed to the Heart by hand: their drift widens the {} lead", aim.ending));
        }
    }

    /// The genome once what rides to the Pits lands: the feeds queued and on the cords.
    fn projected(&self, w: &World) -> [f32; 10] {
        let mut g = w.city.sim.genome.0;
        let riding = w.ledger.queue.iter().filter(|q| matches!(q.verb, inventory::Verb::Feed { .. })).map(|q| &q.item).chain(w.ledger.in_flight.iter().filter(|s| matches!(s.to, inventory::Dest::Pit(_))).map(|s| &s.item));
        for it in riding {
            g = after(w, g, w.table.def(it.def), 1.0);
        }
        g
    }

    /// Blackline and Relics outside a Vault cost the Heart's zone 5 Attention a shift each (Pressure), and a
    /// Cleansing's salvage is 8–15 of them: the steward feeds the Chrome and Relics that pull toward its aim to the
    /// Heart while its drift widens the aim's lead (Pass 2: Blackline is the richest feed and the biggest drift), keeps
    /// what the Vaults hold, sells the rest where a stall buys them, at the quietest grate, and holds the keys and what
    /// nobody takes.
    fn contraband(&mut self, w: &mut World, out: &mut Vec<String>) {
        let Some(aim) = self.aim else { return };
        if w.colony.progress.tech.flag("stash_vaulted") {
            return;
        }
        // what the built Vaults hold stays: the steward disposes of the rest
        let room: usize = w.colony.structures.iter().filter(|s| s.built && w.defs.structure[s.def].id == "vault").map(|s| w.defs.structure[s.def].value_at("capacity", s.tier).max(0) as usize).sum();
        let reach = crate::city::Steward::in_reach(w);
        let quiet = |w: &World, k: usize| w.threats.zones.iter().find(|z| z.grate == w.city.markets[k]).map_or(0.0, |z| z.attention);
        let hot: Vec<(u32, items::ItemInstance)> = w.stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && matches!(w.table.def(it.def).tier, Tier::Blackline | Tier::Relic)).map(|(h, _, it)| (h, it.clone())).collect();
        let (mut fed, mut sold) = (Vec::new(), 0);
        let mut genome = self.projected(w);
        for (h, it) in hot {
            let d = w.table.def(it.def);
            if d.category == Category::Key {
                continue;
            }
            let then = after(w, genome, d, 1.0);
            // fed when it widens the lead; past what the Vaults hold, fed while the lead stays over its floor, the
            // margin spent on getting the hot goods out of the light (each is 5 a shift at the grate)
            let over = w.stash.exposed_contraband(&w.table) > room;
            let feeds = d.feed > 0.0 && (lead(aim, &then, w.campaign.chapter, ally_step(w)) >= lead(aim, &genome, w.campaign.chapter, ally_step(w)) || (over && lead(aim, &then, w.campaign.chapter, ally_step(w)) >= LEAD_FLOOR));
            if !feeds && !over {
                continue;
            }
            if feeds {
                genome = then;
                let name = d.name.clone();
                if w.ledger.queue(&mut w.stash, h, inventory::Verb::Feed { pit: 0 }, w.day) {
                    fed.push(name);
                }
                continue;
            }
            let best = reach.iter().copied().filter(|&k| w.ledger.best(&w.table, &it, k as u8, w.ledger.glut_units(k as u8, d.category as usize)).is_some()).min_by(|&a, &b| quiet(w, a).partial_cmp(&quiet(w, b)).unwrap().then(a.cmp(&b)));
            if let Some(k) = best {
                if w.ledger.queue(&mut w.stash, h, inventory::Verb::Sell { market: k as u8 }, w.day) {
                    sold += 1;
                }
            }
        }
        if !fed.is_empty() {
            out.push(format!("{} to the Heart out of the light: {}", fed.len(), fed.join(", ")));
        }
        if sold > 0 {
            out.push(format!("{sold} Blackline to sell at the quietest grate"));
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
            // walk to the market of the faction with the best standing; once one stands at +1, the ally the script's
            // ending wants: the Garden the warmest (it adds 0.05 a step of the ally's reputation), the rest the coolest
            // that will sign (the Garden's step is a rival's, and the World-Eater's own loss)
            let stands: Vec<(i32, i32, u8)> = (0..w.economy.s.standing.len() as u8).filter(|&f| !w.economy.file.factions[f as usize].accord).filter_map(|f| crate::trade::market_of(&w.economy, &w.city, &w.colony, f).map(|m| (w.economy.s.standing[f as usize].rep, w.economy.s.standing[f as usize].favour, m))).collect();
            let warm = self.aim.is_none_or(|a| a.ending == "garden");
            let signs = stands.iter().copied().filter(|s| s.0 >= 1);
            let pick = if warm { signs.max_by_key(|&(r, fav, m)| (r, fav, std::cmp::Reverse(m))) } else { signs.min_by_key(|&(r, fav, m)| (r, std::cmp::Reverse(fav), m)) };
            let best = pick.or_else(|| stands.iter().copied().max_by_key(|&(r, fav, m)| (r, fav, std::cmp::Reverse(m))));
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
        // the item whose drift at three times widens the aim's lead most
        let now = self.projected(w);
        let (ch, rep) = (w.campaign.chapter, ally_step(w));
        let base = lead(aim, &now, ch, rep);
        // the kept too: what a quota must not take at its bias once may be the overshoot Ivan's three times reflects
        let best = w.stash.handles().into_iter().filter(|(_, _, it)| matches!(w.table.def(it.def).category, Category::Chrome | Category::Relic)).map(|(h, _, it)| (lead(aim, &after(w, now, w.table.def(it.def), 3.0), ch, rep) - base, h)).filter(|&(s, _)| s > 0.0).max_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(b.1.cmp(&a.1)));
        if let Some((_, h)) = best {
            if let Ok(line) = crate::ivan::feed(&mut w.campaign, &mut w.city, &mut w.colony, &w.defs, &mut w.stash, &w.table, h, day, &mut w.log) {
                out.push(line);
            }
        }
    }
}

/// The genome after an item lands in a Pit `times` over (Ivan's hand is three): its bias by its tier's drift, half
/// again with Steered drift, each gene reflecting at 0 and 1 as `MatSim::drift_genome` does.
fn after(w: &World, genome: [f32; 10], d: &items::ItemDef, times: f32) -> [f32; 10] {
    let steered = if w.colony.progress.tech.flag("steered_drift") { 1.5 } else { 1.0 };
    let b = d.bias_vector();
    let mut g = genome;
    for k in 0..10 {
        let mut v = g[k] + b[k] * d.tier.drift() * steered * times;
        if v > 1.0 {
            v = 2.0 - v;
        } else if v < 0.0 {
            v = -v;
        }
        g[k] = v.clamp(0.0, 1.0);
    }
    g
}

/// The aim's ending's score less the best other strain ending's (Full Design v3.19, Endings, the ally's reputation counted), and
/// for every aim but the World-Eater's a strain kept off the Bloom line: past Feral Drive 0.5 everything a Cleansing
/// cuts off blooms against the city (Pressure: Blooms), so a feed that carries it over counts as a lost ending.
fn lead(aim: Aim, genome: &[f32; 10], chapter: u8, ally_rep: i32) -> f32 {
    let own = if aim.ending == "scattered" { "garden" } else { aim.ending };
    let scores = story::ChapterFile::shipped().scores(genome, ally_rep);
    let mine = scores.iter().find(|s| s.0 == own).map_or(0.0, |s| s.1);
    let rival = scores.iter().filter(|s| s.0 != own).map(|s| s.1).fold(0.0, f32::max);
    // a strain past the bloom line turns every cut of its climb into a Bloom: the World-Eater lets it loose only from
    // the alliance's chapter, when the stack is climbed and the Great Cleansing is what is left
    let blooms = (own != "world_eater" || chapter < 6) && genome[mat::genome::FERAL] > BLOOM_LINE;
    mine - rival - if blooms { 1.0 } else { 0.0 }
}

/// The genes a script holds where they stand: one of every rival ending's pair outside the script's own — a gene two
/// rivals share first, then the lowest — from chapter 4, and before it only a rival that shares a gene with the
/// script's pair (the shared gene rises for both with every feed, so that race is the other gene). Holding every
/// rival's every gene kept nearly all the Chrome off the Pits, and the eddies bought more to be kept: the World-Eater's
/// climb, cut, never grew back.
fn held(w: &World, aim: Aim) -> Vec<usize> {
    let own = if aim.ending == "scattered" { "garden" } else { aim.ending };
    let g = w.city.sim.genome.0;
    let mut rivals: Vec<Vec<usize>> = story::ChapterFile::shipped()
        .ending
        .iter()
        .filter(|e| e.id != own && e.genes.len() == 2)
        .map(|e| e.genes.iter().filter_map(|k| progress::strain::gene_index(k)).filter(|k| !aim.genes.contains(k)).collect::<Vec<usize>>())
        .filter(|gs| w.campaign.chapter >= 4 || gs.len() == 1)
        .collect();
    let mut held = Vec::new();
    while let Some(k) = (0..10).filter(|k| rivals.iter().any(|r| r.contains(k))).max_by(|&a, &b| {
        let covers = |k: usize| rivals.iter().filter(|r| r.contains(&k)).count();
        covers(a).cmp(&covers(b)).then(g[b].total_cmp(&g[a])).then(b.cmp(&a))
    }) {
        held.push(k);
        rivals.retain(|r| !r.contains(&k));
    }
    held
}

/// The ally's reputation the ending will read: the alliance chapter 6 asks for is a step at least, so a script counts
/// one from the founding (the ending reads none of it, so the lead is the genes' until the friendship goes past it).
fn ally_step(w: &World) -> i32 {
    w.economy.ally_rep().max(1)
}

/// Tiles along the ground from the nearest of `from` (`logistics::ground_distance` from many).
fn ground_from(sim: &mat::MatSim, from: &[usize]) -> Vec<usize> {
    let (w, h) = (sim.w, sim.h);
    let mut d = vec![usize::MAX; w * h];
    let mut q = std::collections::VecDeque::new();
    for &i in from {
        d[i] = 0;
        q.push_back(i);
    }
    while let Some(i) = q.pop_front() {
        let (x, y) = (i % w, i / w);
        for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
            if nx < w && ny < h {
                let j = ny * w + nx;
                if d[j] == usize::MAX && mat::sim::sub::GROWABLE[sim.sub[j] as usize] {
                    d[j] = d[i] + 1;
                    q.push_back(j);
                }
            }
        }
    }
    d
}

/// A cut climb the frontier grows back to first: this many rows above the Heart's Mat, this many tiles of it.
const REJOIN_ABOVE: usize = 4;
const REJOIN_LEAST: usize = 40;

/// A Runner's Scav at which the steward sends them up the Board.
const SEASONED: u8 = 5;

/// The lead over the next ending the steward keeps when it feeds Blackline it cannot hide (Endings: the higher pair
/// wins; a tenth and a half is about six Blackline fed against the aim).
const LEAD_FLOOR: f32 = 0.15;

/// Feral Drive over which a strain's cut-off tissue blooms (`data/threats.toml`'s bloom.feral_drive, Pass 1).
const BLOOM_LINE: f32 = 0.5;

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
    // the Turret first: the flamers end at the Heart's walls, where it stands
    p.steward.wanted = vec!["c1_forge", "c2_turret", "c2_gate", "c2_vault", "m2_routes", "f1_pens", "f2_still", "m3_graft", "c4_relic"];
    p.steward.only_wanted = true;
    let mut cs = CampaignSteward { aim: Some(aim), ..Default::default() };
    let mut out = CampaignRun { seed, aim: aim.ending.to_string(), ..Default::default() };
    for _ in 0..max {
        // the gauge minded once Sweeps are a stratum off (a Patrol maps and leaves)
        p.steward.gauge_line = (p.w.threats.stratum >= 3).then_some(GAUGE_LINE);
        // no contract from the ally once signed: a chain missed costs a step the ending and the militia read (the
        // Garden's Badges stood +3 at the alliance and 0 at the Great Cleansing on missed recovers), and a chain done
        // is a step the Garden's ending gains over every other script's
        p.steward.cool = p.w.economy.s.ally;
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
            attention: w.threats.zones.iter().filter(|z| w.city.markets.contains(&z.grate)).map(|z| (z.attention, z.gained)).max_by(|a, b| a.0.partial_cmp(&b.0).unwrap()).unwrap_or((0.0, 0.0)),
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
