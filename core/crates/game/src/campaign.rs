//! The campaign on the city (Full Design v3.19, Narrative and Ivan): each chapter's goal read off the city at a shift's
//! end, its close (Authority +2 and its panels) and the next one's opening with what it sets in motion — the envoy and
//! the accord, the chapter's Bloom, the council and the alliance, the Great Cleansing — the Tithe, and the ending with
//! its warchest. The state is `story::Campaign`; the bridge and the headless world both call `shift_end`.

use colony::{Colony, StructureFile};
use inventory::Stash;
use items::{ItemTable, Tier};
use logistics::City;
use people::{Record, Roster, Status};
use story::campaign::{Accord, Campaign, Mode, ACCORD_CALM, BEAT_AUTHORITY, BLOOM_TILES, ENVOY_WAITS, GREAT_GRATES, GREAT_IN, SEEPAGE_SLIMOS, TAMED_GOAL};
use story::chapters::{opens_at, ChapterFile};
use threat::{Due, Role, Threats};

use crate::log::{CityLog, Class};

/// Chapter 2's close: the tweakers' plans (Full Design v3.20).
const CHAPTER_2_PLANS: &str = "tec_blueprint_chrome1";

/// The Empire's card: 2,000 slimos (Full Design v3.18, the city tiers).
const EMPIRE: u32 = 2000;

/// What the campaign reads and moves at a shift's end.
pub struct Parts<'a> {
    pub city: &'a mut City,
    pub colony: &'a mut Colony,
    pub defs: &'a StructureFile,
    pub threats: &'a mut Threats,
    pub economy: &'a mut economy::Economy,
    pub roster: &'a mut Roster,
    pub died: &'a mut Vec<Record>,
    pub stash: &'a mut Stash,
    pub table: &'a ItemTable,
    pub log: &'a mut CityLog,
    /// slimos standing Guard in the shift's plan (the warchest)
    pub guards: usize,
}

/// A new run's campaign on its threats and economy: the difficulty's Attention decay, corpses, Spored gear and Floods,
/// the chapter the surface reads.
pub fn found(c: &Campaign, threats: &mut Threats, economy: &mut economy::Economy) {
    threats.campaign.decay = c.difficulty.decay();
    threats.campaign.raid_corpses = c.difficulty.raid_corpses();
    threats.campaign.spored_first_only = c.difficulty.spored_first_only();
    threats.campaign.dawn_flood = c.difficulty.dawn_flood();
    economy.s.chapter = c.chapter.max(1);
}

/// Every flush's batch and every zone's gain multiplied (Full Design v3.19): Endless Sump grows both with the Mat —
/// 1 + its cells / 2,000 — and from chapter 6 the occupation doubles the flush.
pub fn mults(c: &Campaign, sim: &mat::MatSim) -> (f32, f32) {
    match c.mode {
        Mode::Endless => {
            let cells = sim.hy.iter().filter(|&&h| mat::sim::is_hyphal(h)).count() as f32;
            let m = 1.0 + cells / 2000.0;
            (m, m)
        }
        Mode::Campaign => (if (6..=7).contains(&c.chapter) { 2.0 } else { 1.0 }, 1.0),
    }
}

/// A run over (the bridge's and the headless game's): Ivan's line on it, and the campaign's note — chapter 5 reads a
/// Vault run's best item (the rig home, backpack and quick slots).
#[allow(clippy::too_many_arguments)]
pub fn run_home(c: &mut Campaign, log: &mut CityLog, table: &ItemTable, stratum: u8, name: &str, extracted: bool, died: bool, rig: Option<&inventory::Rig>) {
    let best = rig.and_then(|r| r.backpack.items().chain(r.quick.iter().flatten()).map(|it| table.def(it.def).tier).max());
    if died {
        log.bark("runner_dead", &[("name", name)]);
    } else if extracted && rig.is_some() {
        log.bark("haul", &[("name", name)]);
    }
    note_run(c, stratum, extracted, best);
}

/// A Runner home from a run: chapter 5 reads the Vault's.
pub fn note_run(c: &mut Campaign, stratum: u8, extracted: bool, best: Option<Tier>) {
    if extracted && stratum == 5 && best.is_some_and(|t| t >= Tier::Rare) {
        c.vault_home = true;
    }
}

fn built(colony: &Colony, defs: &StructureFile, id: &str) -> bool {
    colony.structures.iter().any(|s| s.built && defs.structure[s.def].id == id)
}

fn cleansing_on(threats: &Threats) -> bool {
    threats.events.iter().any(|e| threats.rules.stage[e.stage].id == "cleansing" && e.phase != threat::Phase::Done)
}

/// Whether the open chapter's goal holds now (the chapters' table).
fn goal_met(c: &Campaign, p: &Parts, day: u32) -> bool {
    let (colony, defs) = (&*p.colony, p.defs);
    match c.chapter {
        1 => {
            let (beds, _) = colony.bed_count(defs);
            p.roster.records.len() >= SEEPAGE_SLIMOS && beds as usize >= p.roster.records.len()
        }
        2 => built(colony, defs, "forge") && built(colony, defs, "market") && p.economy.s.standing.iter().any(|s| s.traded > 0),
        3 => matches!(c.accord, Some(Accord::Accepted { .. } | Accord::Refused { .. })) && c.cleansing_sent && c.cleansing_at.is_some_and(|at| day >= at) && !cleansing_on(p.threats),
        4 => c.bloom && (!p.city.sim.wild.iter().any(|&w| w) || p.threats.bodies.iter().filter(|b| b.role == Role::Tamed && !b.dead && !b.gone).count() >= TAMED_GOAL),
        5 => c.vault_home && built(colony, defs, "still"),
        6 => p.economy.s.ally.is_some() && p.threats.stratum >= 6,
        7 => c.cleansing_sent && c.great_at.is_some_and(|at| day >= at) && !cleansing_on(p.threats),
        _ => false,
    }
}

/// The reached grate nearest the Heart: the envoy's (chapter 3).
fn envoy_zone(p: &Parts) -> Option<usize> {
    let (hx, hy) = p.city.sim.heart?;
    let w = p.city.sim.w;
    p.threats
        .zones
        .iter()
        .enumerate()
        .filter(|(_, z)| p.city.markets.contains(&z.grate))
        .min_by_key(|(k, z)| (((z.grate % w) as i64 - hx as i64).abs() + ((z.grate / w) as i64 - hy as i64).abs(), *k))
        .map(|(k, _)| k)
}

/// Chapter 4's Bloom: 40 tiles at the Storm Drains' shaft farthest from where the Mat came into them.
fn grow_bloom(p: &mut Parts) -> usize {
    let sim = &mut p.city.sim;
    let w = sim.w;
    let joined = logistics::joined_to_heart(sim);
    let ours: Vec<usize> = (0..joined.len()).filter(|&i| joined[i] && sim.stratum_at(i) == 3).collect();
    let entry = if ours.is_empty() { w / 2 } else { ours.iter().map(|&i| i % w).sum::<usize>() / ours.len() };
    let Some(far) = sim.grates_of(3).into_iter().max_by_key(|&g| ((g % w).abs_diff(entry), g)) else { return 0 };
    let (fx, mid) = (far % w, sim.ceiling(3) + sim.strata_rows / 2);
    // the nearest empty ground to the shaft's middle
    let start = (0..sim.hy.len())
        .filter(|&i| sim.stratum_at(i) == 3 && sim.hy[i] == mat::sim::hy::EMPTY && mat::sim::sub::GROWABLE[sim.sub[i] as usize] && !joined[i])
        .min_by_key(|&i| ((i % w).abs_diff(fx) + (i / w).abs_diff(mid), i));
    start.map_or(0, |s| sim.seed_wild(s, BLOOM_TILES))
}

/// Chapter 4's beat: the earliest flushed slimo still on the roster and home leaves to the Bloom.
fn feral_newcomer(p: &mut Parts) -> String {
    let pick = p.roster.records.iter().filter(|r| r.joined_day > 1 && !matches!(r.status, Status::OnRun | Status::Stranded)).min_by_key(|r| (r.joined_day, r.id)).map(|r| r.id);
    let Some(id) = pick else { return String::new() };
    let i = p.roster.records.iter().position(|r| r.id == id).unwrap();
    let mut rec = p.roster.records.remove(i);
    for other in p.roster.records.iter_mut() {
        other.bonds.retain(|b| b.with != id);
    }
    p.colony.homes.remove(&id);
    rec.status = Status::Feral;
    let name = rec.full_name();
    p.log.push(p.city.tick, Class::Loss, format!("{name} went feral in the Storm Drains, in a drain the city never reached"));
    p.died.push(rec);
    name
}

/// The Tithe at the shift's end: a tenth of the Junk the Stash gained over the shift goes up the envoy's grate.
fn tithe(c: &mut Campaign, p: &mut Parts) {
    let junk = |st: &Stash| st.handles().into_iter().filter(|(_, _, it)| p.table.def(it.def).tier == Tier::Common).count() as u32;
    let now = junk(p.stash);
    if p.colony.charter.law("tithe").is_some_and(|l| l.repeal_reach.is_none()) {
        let owed = now.saturating_sub(c.junk_last) / 10;
        let mut gone = 0;
        for _ in 0..owed {
            let h = p.stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && p.table.def(it.def).tier == Tier::Common).min_by_key(|(h, _, it)| (p.table.def(it.def).value, *h)).map(|(h, _, _)| h);
            if let Some(h) = h {
                p.stash.take(h);
                gone += 1;
            }
        }
        if gone > 0 {
            p.log.push(p.city.tick, Class::Loss, format!("the Tithe: {gone} Junk up the envoy's grate"));
        }
    }
    c.junk_last = junk(p.stash);
}

/// The warchest (Pass 1's "biomass, armoured cords, tamed sewerats, militia, Still output and alliance reputation").
pub fn warchest(p: &Parts) -> u32 {
    let sim = &p.city.sim;
    let joined = logistics::joined_to_heart(sim);
    let cells = joined.iter().filter(|&&j| j).count() as u32;
    let armoured = (0..joined.len()).filter(|&i| joined[i] && sim.hy[i] == mat::sim::hy::CORD && sim.armour_at(i) >= 5.0).count() as u32;
    let tamed = p.threats.bodies.iter().filter(|b| b.role == Role::Tamed && !b.dead && !b.gone).count() as u32;
    let militia = p.threats.bodies.iter().filter(|b| b.role == Role::Militia && !b.dead && !b.gone).count() as u32;
    cells + 2 * armoured + 10 * tamed + 10 * (militia + p.guards as u32) + p.colony.downer_made + 100 * p.economy.ally_rep().max(0) as u32
}

fn end(c: &mut Campaign, p: &mut Parts, day: u32, lines: &mut Vec<String>) {
    let f = ChapterFile::shipped();
    let e = f.ending_of(&p.city.sim.genome.0, p.economy.ally_rep(), p.city.sim.heart_lost, c.ivan_fell);
    let chest = warchest(p);
    c.ending = Some(e.id.clone());
    c.warchest = Some(chest);
    c.ended_at = Some(day);
    if e.id != "scattered" && c.chapter == 7 {
        c.panels.push("ch7.close".into());
    }
    c.panels.push(format!("end.{}", e.id));
    c.chapter = 8;
    let line = format!("the ending: {} — the warchest {chest}", e.name);
    p.log.push(p.city.tick, Class::Landing, line.clone());
    lines.push(line);
}

/// Send the scripted Cleansing next shift down `zones`, with `share` of its squads.
fn schedule(p: &mut Parts, zones: &[u16], share: f32, day: u32) {
    let Some(st) = p.threats.rules.stage_index("cleansing") else { return };
    let arrive = p.threats.rules.stage[st].arrive_in.max(1);
    for (k, &z) in zones.iter().enumerate() {
        let in_ticks = (mat::rng::Pcg32::new(p.threats.seed ^ ((day as u64) << 8) ^ k as u64 ^ 0xC1EA).below(arrive)) / 4;
        p.threats.due.retain(|d| d.zone != z);
        p.threats.due.push(Due { stage: st, zone: z, in_ticks, share });
    }
}

/// The Undercroft grates the Great Cleansing comes down: up to three the Mat reaches, nearest the Heart's column (or
/// the nearest at all when it reaches none).
fn great_zones(p: &Parts) -> Vec<u16> {
    let w = p.city.sim.w;
    let hx = p.city.sim.heart.map_or(w / 2, |h| h.0);
    let top = p.city.sim.strata;
    let mut s6: Vec<(usize, &threat::Zone)> = p.threats.zones.iter().enumerate().filter(|(_, z)| p.city.sim.stratum_at(z.grate) == top).collect();
    let reached: Vec<(usize, &threat::Zone)> = s6.iter().copied().filter(|(_, z)| p.city.markets.contains(&z.grate)).collect();
    if !reached.is_empty() {
        s6 = reached;
    }
    s6.sort_by_key(|(k, z)| ((z.grate % w).abs_diff(hx), *k));
    s6.into_iter().take(GREAT_GRATES).map(|(k, _)| k as u16).collect()
}

/// The envoy's answer (chapter 3): accepted, the Tithe signed at no Authority, the Oathkeepers trading at its grate,
/// two shifts of calm, then half a Cleansing for the Heart; refused, the whole Cleansing next shift.
pub fn answer_accord(c: &mut Campaign, p: &mut Parts, accept: bool, day: u32) -> Result<(), String> {
    if !matches!(c.accord, Some(Accord::Waiting { .. })) {
        return Err("no envoy is waiting".into());
    }
    let zone = c.envoy_zone.ok_or("no envoy")? as usize;
    p.threats.envoy_leaves();
    if accept {
        c.accord = Some(Accord::Accepted { on: day });
        if let Some(law) = colony::LawFile::shipped().get("tithe") {
            if !p.colony.charter.offered.iter().any(|l| l == "tithe") {
                p.colony.charter.offered.push("tithe".into());
            }
            p.colony.sign(p.defs, &p.city.sim, law)?;
        }
        p.economy.s.accord_grate = Some(p.threats.zones[zone].grate as u16);
        p.threats.campaign.mapped = true;
        p.threats.campaign.calm_until = day + ACCORD_CALM;
        c.cleansing_at = Some(day + ACCORD_CALM + 1);
        c.cleansing_share = 0.5;
        p.log.push(p.city.tick, Class::Decision, "the accord accepted: the Tithe is signed, and the Oathkeepers trade at the envoy's grate".into());
    } else {
        c.accord = Some(Accord::Refused { on: day });
        c.cleansing_at = Some(day + 1);
        c.cleansing_share = 1.0;
        p.log.push(p.city.tick, Class::Decision, "the accord refused: the envoy goes back up the grate".into());
    }
    c.cleansing_zones = vec![zone as u16];
    Ok(())
}

/// The alliance (chapter 6's council): Ivan's Parley at the faction's market at reputation +1 or more.
pub fn ally(c: &Campaign, p: &mut Parts, faction: u8) -> Result<(), String> {
    if c.chapter != 6 {
        return Err("the council sits in chapter 6".into());
    }
    if p.economy.s.standing[faction as usize].rep < 1 {
        return Err("an alliance needs reputation +1".into());
    }
    p.economy.ally_with(faction)?;
    let name = p.economy.name(faction).to_string();
    p.log.push(p.city.tick, Class::Decision, format!("the alliance signed with the {name}"));
    Ok(())
}

/// The shift's end. Returns the lines for the report.
pub fn shift_end(c: &mut Campaign, p: &mut Parts, day: u32) -> Vec<String> {
    let mut lines = Vec::new();
    tithe(c, p);
    if c.over() {
        return lines;
    }
    // Endless Sump: no chapters; ranked by the shift the Empire's card landed on, over when the Heart burns
    if c.mode == Mode::Endless {
        if c.empire_at.is_none() && p.colony.progress.collections.titles.contains(&EMPIRE) {
            c.empire_at = Some(day);
        }
        if p.city.sim.heart_lost {
            c.ending = Some("burned".into());
            c.warchest = Some(warchest(p));
            c.ended_at = Some(day);
            let line = format!("the Heart burned on shift {day}{}", c.empire_at.map_or(String::new(), |at| format!("; the Empire stood from shift {at}")));
            p.log.push(p.city.tick, Class::Loss, line.clone());
            lines.push(line);
        }
        return lines;
    }
    // a burned Heart ends the campaign Scattered, then or at the Great Cleansing
    if p.city.sim.heart_lost {
        end(c, p, day, &mut lines);
        return lines;
    }
    // the envoy's offer stands two shifts, then counts as refused
    if let Some(Accord::Waiting { since }) = c.accord {
        if day >= since + ENVOY_WAITS {
            let _ = answer_accord(c, p, false, day);
            lines.push("the envoy's offer went unanswered: refused".into());
        }
    }
    // the scripted Cleansing comes next shift
    if !c.cleansing_sent && c.cleansing_at.is_some_and(|at| day + 1 >= at) {
        let zones = c.cleansing_zones.clone();
        schedule(p, &zones, c.cleansing_share, day);
        c.cleansing_sent = true;
        lines.push("the white light is coming down the envoy's grate".into());
    }
    // the open chapter's goal
    let f = ChapterFile::shipped();
    if goal_met(c, p, day) {
        let n = c.chapter;
        c.closed.push((n, day));
        p.colony.charter.earn(BEAT_AUTHORITY as u32);
        let beat = n.to_string();
        let feral = if c.feral_name.is_empty() { "one of ours".to_string() } else { c.feral_name.clone() };
        p.log.bark("beat", &[("beat", &beat), ("name", &feral)]);
        let name = f.get(n).map(|d| d.name.clone()).unwrap_or_default();
        let line = format!("chapter {n}, {name}, closes: Authority +{BEAT_AUTHORITY}");
        p.log.push(p.city.tick, Class::Landing, line.clone());
        lines.push(line);
        if n == 7 {
            end(c, p, day, &mut lines);
            return lines;
        }
        // the Tweakers' plans: Chrome's Blueprint I, unless its tier 2 is open (Full Design v3.20)
        if n == 2 && p.colony.progress.tech.open[0] < 2 {
            if let Some(id) = p.table.id_of(CHAPTER_2_PLANS) {
                p.stash.place_or_floor(p.table, items::ItemInstance::new(id));
                let line = format!("the tweakers' plans: {} into the Stash", p.table.def(id).name);
                p.log.push(p.city.tick, Class::Landing, line.clone());
                lines.push(line);
            }
        }
        c.panels.push(format!("ch{n}.close"));
        c.chapter = n + 1;
        c.open_since = 0;
        c.cleansing_sent = false;
        c.cleansing_at = None;
    }
    // the next chapter opens when its stratum is reached
    if c.open_since == 0 && p.threats.stratum >= opens_at(c.chapter) {
        let n = c.chapter;
        c.open_since = day;
        c.panels.push(format!("ch{n}.open"));
        let name = f.get(n).map(|d| d.name.clone()).unwrap_or_default();
        let line = format!("chapter {n}, {name}, opens");
        p.log.push(p.city.tick, Class::Landing, line.clone());
        lines.push(line);
        match n {
            3 => {
                if let Some(z) = envoy_zone(p) {
                    p.threats.send_envoy(&p.city.sim, z);
                    c.envoy_zone = Some(z as u16);
                    c.accord = Some(Accord::Waiting { since: day });
                    let g = p.threats.grate_label(&p.city.sim, z);
                    p.log.push(p.city.tick, Class::Decision, format!("an Oathkeeper envoy waits at grate {g} with an offer"));
                }
            }
            4 => {
                let n = grow_bloom(p);
                c.bloom = n > 0;
                c.feral_name = feral_newcomer(p);
                p.log.push(p.city.tick, Class::Loss, format!("a cut-off strain of {n} tiles blooms in the Storm Drains"));
            }
            5 => {
                c.teller = p.roster.records.iter().filter(|r| r.past == people::Past::Corpo).min_by_key(|r| r.id).map(|r| r.full_name()).unwrap_or_default();
            }
            7 => {
                c.great_at = Some(day + GREAT_IN);
                p.log.push(p.city.tick, Class::Decision, format!("the Great Cleansing comes in {GREAT_IN} shifts"));
            }
            _ => {}
        }
    }
    // the Great Cleansing next shift: every squad for the Heart, the ally's militia at it
    if c.chapter == 7 && !c.cleansing_sent && c.great_at.is_some_and(|at| day + 1 >= at) {
        let zones = great_zones(p);
        schedule(p, &zones, 1.0, day);
        p.threats.campaign.mapped = true;
        let militia = 2 * p.economy.ally_rep().max(0) as u32;
        p.threats.send_militia(&p.city.sim, militia);
        c.cleansing_sent = true;
        let line = format!("the Great Cleansing comes down {} Undercroft grate{}; the ally's {militia} militia stand at the Heart", zones.len(), if zones.len() == 1 { "" } else { "s" });
        p.log.push(p.city.tick, Class::Loss, line.clone());
        lines.push(line);
    }
    p.economy.s.chapter = c.chapter.clamp(1, 7);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use items::{ItemTable, ITEMS_TOML};

    fn world() -> crate::World {
        crate::World::found(11, ItemTable::from_toml(ITEMS_TOML).unwrap())
    }

    fn end(w: &mut crate::World, day: u32) -> Vec<String> {
        let (c, mut p) = w.campaign_parts();
        shift_end(c, &mut p, day)
    }

    #[test]
    fn a_chapter_closes_on_its_goal_pays_its_beat_and_the_next_opens_with_its_stratum() {
        let mut w = world();
        assert_eq!(w.campaign.chapter, 1);
        assert_eq!(w.campaign.panels, vec!["ch1.open".to_string()]);
        let before = w.colony.charter.earned;
        // not yet: the founders are fewer than eight
        end(&mut w, 1);
        assert_eq!(w.campaign.chapter, 1);
        // eight with beds
        let (t, n) = (people::TraitFile::shipped(), people::NameFile::shipped());
        let mut rng = mat::rng::Pcg32::new(5);
        while w.roster.records.len() < 8 {
            w.roster.create(&t, &n, &mut rng, 2, false);
        }
        w.colony.progress.tech = progress::Tech::every_structure();
        while (w.colony.bed_count(&w.defs).0 as usize) < w.roster.records.len() {
            let (hx, hy) = w.city.sim.heart.unwrap();
            let spot = (0..w.city.sim.hy.len()).filter(|&i| w.colony.can_place(&w.defs, &w.city.sim, "bunkhouse", (i % w.city.sim.w) as u16, (i / w.city.sim.w) as u16).is_ok()).min_by_key(|&i| (i % w.city.sim.w).abs_diff(hx) + (i / w.city.sim.w).abs_diff(hy)).expect("room for a Bunkhouse");
            w.colony.place_built(&w.defs, &w.city.sim, "bunkhouse", (spot % w.city.sim.w) as u16, (spot / w.city.sim.w) as u16, 1).unwrap();
        }
        let lines = end(&mut w, 2);
        assert!(lines.iter().any(|l| l.contains("chapter 1, Seepage, closes")), "{lines:?}");
        assert_eq!(w.colony.charter.earned, before + 2, "the beat's Authority");
        assert_eq!(w.campaign.chapter, 2, "Tweakers opens at once: it needs no stratum");
        assert!(w.campaign.panels.ends_with(&["ch1.close".to_string(), "ch2.open".to_string()]));
        // chapter 3 waits for the Culverts
        w.campaign.closed.push((2, 3));
        w.campaign.chapter = 3;
        w.campaign.open_since = 0;
        end(&mut w, 3);
        assert_eq!(w.campaign.open_since, 0, "not before the Culverts");
        w.threats.stratum = 2;
        end(&mut w, 4);
        assert_eq!(w.campaign.open_since, 4);
        assert!(matches!(w.campaign.accord, Some(Accord::Waiting { since: 4 })));
        assert!(w.threats.bodies.iter().any(|b| b.role == Role::Envoy), "the envoy waits at its grate");
    }

    #[test]
    fn the_accord_accepted_signs_the_tithe_maps_the_heart_and_sends_half_after_two_calm_shifts() {
        let mut w = world();
        w.campaign.chapter = 3;
        w.campaign.open_since = 0;
        w.threats.stratum = 2;
        end(&mut w, 5);
        {
            let (c, mut p) = w.campaign_parts();
            answer_accord(c, &mut p, true, 5).unwrap();
        }
        assert!(w.colony.charter.law("tithe").is_some(), "the Tithe signed at no Authority");
        assert!(w.threats.campaign.mapped);
        assert_eq!(w.threats.campaign.calm_until, 7);
        assert!(w.economy.s.accord_grate.is_some(), "the Oathkeepers trade at the envoy's grate");
        assert_eq!(w.campaign.cleansing_at, Some(8));
        end(&mut w, 6);
        assert!(!w.campaign.cleansing_sent, "two calm shifts");
        end(&mut w, 7);
        let st = w.threats.rules.stage_index("cleansing").unwrap();
        let due = w.threats.due.iter().find(|d| d.stage == st).expect("the accord's Cleansing is due");
        assert_eq!(due.share, 0.5, "half its bodies");
        // the refused envoy: the whole Cleansing the next shift
        let mut v = world();
        v.campaign.chapter = 3;
        v.campaign.open_since = 0;
        v.threats.stratum = 2;
        end(&mut v, 5);
        end(&mut v, 6);
        end(&mut v, 7);
        assert!(matches!(v.campaign.accord, Some(Accord::Refused { on: 7 })), "unanswered two shifts: refused");
        assert!(v.threats.due.iter().any(|d| d.stage == st && d.share == 1.0), "the whole Cleansing next shift");
        assert!(!v.threats.campaign.mapped);
    }

    #[test]
    fn the_tithe_takes_a_tenth_of_the_junk_the_stash_gained() {
        let mut w = world();
        w.colony.charter.offered.push("tithe".into());
        let laws = colony::LawFile::shipped();
        w.colony.sign(&w.defs, &w.city.sim, laws.get("tithe").unwrap()).unwrap();
        end(&mut w, 1);
        let junk = |w: &crate::World| w.stash.handles().iter().filter(|(_, _, it)| w.table.def(it.def).tier == Tier::Common).count();
        let before = junk(&w);
        let id = w.table.id_of("chr_optic_slag").unwrap();
        assert_eq!(w.table.def(id).tier, Tier::Common);
        for _ in 0..20 {
            w.stash.place_or_floor(&w.table, items::ItemInstance::new(id));
        }
        end(&mut w, 2);
        assert_eq!(junk(&w), before + 20 - 2, "a tenth of the twenty gained");
    }

    #[test]
    fn chapter_four_grows_its_bloom_in_the_storm_drains_and_takes_a_newcomer() {
        let mut w = world();
        let (t, n) = (people::TraitFile::shipped(), people::NameFile::shipped());
        let mut rng = mat::rng::Pcg32::new(9);
        let id = w.roster.create(&t, &n, &mut rng, 3, false);
        w.campaign.chapter = 4;
        w.campaign.open_since = 0;
        w.threats.stratum = 3;
        end(&mut w, 6);
        assert!(w.campaign.bloom);
        let wild: Vec<usize> = (0..w.city.sim.wild.len()).filter(|&i| w.city.sim.wild[i]).collect();
        assert_eq!(wild.len(), BLOOM_TILES);
        assert!(wild.iter().all(|&i| w.city.sim.stratum_at(i) == 3 && w.city.sim.is_bloom(i)));
        assert!(w.roster.get(id).is_none() && w.died.iter().any(|r| r.id == id), "the newcomer gone feral, on the Memorial");
        assert!(!w.campaign.feral_name.is_empty());
    }

    #[test]
    fn the_great_cleansing_comes_six_shifts_on_with_the_militia_and_the_genome_names_the_ending() {
        let mut w = world();
        let nomads = w.economy.file.index("nomads").unwrap();
        w.economy.s.standing[nomads as usize].rep = 2;
        w.campaign.chapter = 6;
        w.campaign.open_since = 3;
        w.threats.stratum = 6;
        {
            let (c, mut p) = w.campaign_parts();
            ally(c, &mut p, nomads).unwrap();
        }
        end(&mut w, 10);
        assert_eq!(w.campaign.chapter, 7);
        assert_eq!(w.campaign.great_at, Some(16));
        for d in 11..15 {
            end(&mut w, d);
        }
        assert!(!w.campaign.cleansing_sent);
        end(&mut w, 15);
        assert!(w.campaign.cleansing_sent);
        let st = w.threats.rules.stage_index("cleansing").unwrap();
        assert!(!w.threats.due.iter().filter(|d| d.stage == st).collect::<Vec<_>>().is_empty());
        assert_eq!(w.threats.bodies.iter().filter(|b| b.role == Role::Militia).count(), 4, "two a step of the ally's +2");
        // the Cleansing gone and the Heart standing: the genome reads Garden
        w.threats.due.clear();
        let lume = progress::strain::gene_index("lume").unwrap();
        let thrift = progress::strain::gene_index("thrift").unwrap();
        w.city.sim.genome.0[lume] = 0.8;
        w.city.sim.genome.0[thrift] = 0.75;
        end(&mut w, 16);
        assert_eq!(w.campaign.ending.as_deref(), Some("garden"));
        assert!(w.campaign.warchest.unwrap() > 200, "the ally's +2 alone is 200");
        assert!(w.campaign.panels.ends_with(&["ch7.close".to_string(), "end.garden".to_string()]));
    }
}
