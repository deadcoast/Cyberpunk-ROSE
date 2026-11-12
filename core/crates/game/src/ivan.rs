//! Ivan on the map (Full Design v3.19, Narrative and Ivan: On the map): he walks the Mat only, a tile every two ticks,
//! and stands at the Heart where nothing is ordered; the squads find him among the city's slimos; his four verbs, each
//! once a shift; and his voice — the barks, chosen for his Authority state, the strain's dominant class and the Rite.

use colony::{Colony, StructureFile};
use inventory::Stash;
use items::ItemTable;
use logistics::City;
use mat::sim::is_hyphal;
use people::{Need, Roster};
use story::barks::Ctx;
use story::campaign::{Campaign, IVAN_HP, IVAN_ID};
use threat::Threats;

use crate::log::{CityLog, Class};

/// Rally's reach, Parley's at a grate, Feed the Heart's at a Pit, Ooze-time's from a hostile (tiles).
pub const RALLY_REACH: i32 = 8;
pub const PARLEY_REACH: i32 = 3;
pub const FEED_REACH: i32 = 2;
pub const OOZE_REACH: i32 = 12;
/// Rally's Mood.
pub const RALLY_MOOD: i32 = 10;
/// Feed the Heart's weight on the feed's bias.
pub const FEED_WEIGHT: f32 = 3.0;
/// Ooze-time: the clock's scale and the player's seconds (the view runs it).
pub const OOZE_SCALE: f32 = 0.2;
pub const OOZE_SECONDS: f32 = 6.0;

fn cheb(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// Ivan at the founding: at the Heart.
pub fn found(c: &mut Campaign, sim: &mat::MatSim) {
    if let Some((x, y)) = sim.heart {
        c.ivan.x = x as u16;
        c.ivan.y = y as u16;
    }
}

/// Send him to a tile of the Mat; refused off it.
pub fn order(c: &mut Campaign, sim: &mat::MatSim, x: u16, y: u16) -> Result<(), String> {
    if c.ivan.fell {
        return Err("Ivan has fallen".into());
    }
    let i = y as usize * sim.w + x as usize;
    if i >= sim.hy.len() || !is_hyphal(sim.hy[i]) {
        return Err("Ivan walks the Mat only".into());
    }
    c.ivan.goal = Some((x, y));
    Ok(())
}

/// One city tick: a step toward his goal along the Mat every two ticks; the squads' harm taken; a fall.
pub fn tick(c: &mut Campaign, city: &City, threats: &mut Threats, log: &mut CityLog) {
    if c.ivan.fell {
        threats.others.clear();
        return;
    }
    let sim = &city.sim;
    let w = sim.w;
    // the harm the squads did him last tick
    if let Some(h) = threats.hurt.remove(&IVAN_ID) {
        c.ivan.hp -= h;
        if c.ivan.hp <= 0.0 {
            c.ivan.fell = true;
            c.ivan_fell = true;
            c.barker.silent = true;
            threats.others.clear();
            log.push(city.tick, Class::Loss, format!("Ivan fell at {},{}: the city goes on without his voice", c.ivan.x, c.ivan.y));
            return;
        }
    }
    // the Mat under him died: back toward the Heart
    let here = c.ivan.y as usize * w + c.ivan.x as usize;
    if here >= sim.hy.len() || !is_hyphal(sim.hy[here]) {
        if let Some((hx, hy)) = sim.heart {
            c.ivan.goal = Some((hx as u16, hy as u16));
        }
    }
    if city.tick.is_multiple_of(logistics::city::WALK_EVERY) {
        if let Some((gx, gy)) = c.ivan.goal {
            let goal = gy as usize * w + gx as usize;
            if goal == here {
                c.ivan.goal = None;
            } else if let Some(n) = next_step(sim, here, goal) {
                c.ivan.x = (n % w) as u16;
                c.ivan.y = (n / w) as u16;
            } else {
                c.ivan.goal = None;
            }
        }
    }
    threats.others = vec![(IVAN_ID, (c.ivan.x, c.ivan.y))];
}

/// The next tile from `from` toward `goal` along the Mat (a search over its tissue from the goal).
fn next_step(sim: &mat::MatSim, from: usize, goal: usize) -> Option<usize> {
    let w = sim.w;
    if goal >= sim.hy.len() || !is_hyphal(sim.hy[goal]) {
        return None;
    }
    let mut dist: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
    let mut q = std::collections::VecDeque::from([goal]);
    dist.insert(goal, 0);
    let nbs = |i: usize| {
        let (x, y) = (i % w, i / w);
        [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)].into_iter().filter(|&(nx, ny)| nx < w && ny < sim.h).map(move |(nx, ny)| ny * w + nx)
    };
    while let Some(i) = q.pop_front() {
        if i == from {
            break;
        }
        let d = dist[&i];
        for j in nbs(i) {
            if !dist.contains_key(&j) && (is_hyphal(sim.hy[j]) || j == from) {
                dist.insert(j, d + 1);
                q.push_back(j);
            }
        }
    }
    let d0 = *dist.get(&from)?;
    nbs(from).filter(|j| dist.get(j).is_some_and(|&d| d + 1 == d0)).min()
}

/// A shift's end: his HP mended and his verbs ready (Full Design v3.19: a shift's end mends him to full).
pub fn shift_end(c: &mut Campaign, city: &mut City) {
    if !c.ivan.fell {
        c.ivan.hp = IVAN_HP;
    }
    city.rallied.clear();
}

fn once(last: u32, day: u32, verb: &str) -> Result<(), String> {
    if last == day {
        return Err(format!("{verb} is spent this shift"));
    }
    Ok(())
}

/// Rally: slimos within 8 tiles walk half again as fast and gain Mood 10 for the rest of the shift.
pub fn rally(c: &mut Campaign, city: &mut City, roster: &mut Roster, day: u32, log: &mut CityLog) -> Result<usize, String> {
    once(c.ivan.rally, day, "Rally")?;
    if c.ivan.fell {
        return Err("Ivan has fallen".into());
    }
    let w = city.sim.w;
    let me = (c.ivan.x as i32, c.ivan.y as i32);
    let near: Vec<u32> = city.ids.iter().zip(city.pos.iter()).filter(|(_, &p)| cheb(((p as usize % w) as i32, (p as usize / w) as i32), me) <= RALLY_REACH).map(|(&id, _)| id).collect();
    for &id in &near {
        if let Some(r) = roster.get_mut(id) {
            let m = Need::Mood as usize;
            r.needs[m] = (r.needs[m] + RALLY_MOOD).min(100);
        }
        if !city.rallied.contains(&id) {
            city.rallied.push(id);
        }
    }
    c.ivan.rally = day;
    log.push(city.tick, Class::Decision, format!("Ivan rallied {} slimo{}", near.len(), if near.len() == 1 { "" } else { "s" }));
    Ok(near.len())
}

/// The market Ivan stands at: the one whose grate is within 3 tiles of him.
pub fn market_here(c: &Campaign, city: &City) -> Option<usize> {
    let w = city.sim.w;
    let me = (c.ivan.x as i32, c.ivan.y as i32);
    city.markets.iter().position(|&g| cheb(((g % w) as i32, (g / w) as i32), me) <= PARLEY_REACH)
}

/// Parley at the market he stands at: 1 Authority, once a shift; what it opens grows with his Authority state; in
/// chapter 6, at reputation +1 or more, it also signs the alliance.
pub fn parley(c: &mut Campaign, city: &City, colony: &mut Colony, econ: &mut economy::Economy, day: u32, log: &mut CityLog) -> Result<String, String> {
    once(c.ivan.parley, day, "Parley")?;
    if c.ivan.fell {
        return Err("Ivan has fallen".into());
    }
    let m = market_here(c, city).ok_or("Ivan stands at no Seep Market's grate")?;
    let g = city.markets[m];
    let f = econ.faction_at(g).or_else(|| econ.s.accord_grate.filter(|&a| a as usize == g).and_then(|_| econ.file.factions.iter().position(|d| d.accord).map(|k| k as u8))).ok_or("no faction works that grate")?;
    if colony.charter.authority < 1 {
        return Err("Parley needs 1 Authority".into());
    }
    colony.charter.authority -= 1;
    let state = colony.charter.ivan_state();
    let mut did = econ.parley(f, state);
    if c.chapter == 6 && econ.s.ally.is_none() && econ.s.standing[f as usize].rep >= 1 && !econ.file.factions[f as usize].accord {
        econ.ally_with(f)?;
        did.push_str(", the alliance signed");
    }
    c.ivan.parley = day;
    let line = format!("Ivan's Parley with the {} at grate {}: {did}", econ.name(f), city.market_grate(m));
    log.push(city.tick, Class::Decision, line.clone());
    Ok(line)
}

/// The Pit Ivan stands at: (structure, its cargo cell), within 2 tiles of him.
pub fn pit_here(c: &Campaign, city: &City, colony: &Colony, defs: &StructureFile) -> Option<(u32, usize)> {
    let w = city.sim.w;
    let me = (c.ivan.x as i32, c.ivan.y as i32);
    colony.pits(defs).into_iter().filter_map(|sid| colony.cargo_cell(defs, &city.sim, sid).map(|cell| (sid, cell))).find(|&(_, cell)| cheb(((cell % w) as i32, (cell / w) as i32), me) <= FEED_REACH)
}

/// Feed the Heart: a Stash item into the Pit he stands at, its bias at 3× weight — the one feed that steers on purpose.
#[allow(clippy::too_many_arguments)]
pub fn feed(c: &mut Campaign, city: &mut City, colony: &mut Colony, defs: &StructureFile, stash: &mut Stash, table: &ItemTable, handle: u32, day: u32, log: &mut CityLog) -> Result<String, String> {
    once(c.ivan.feed, day, "Feed the Heart")?;
    if c.ivan.fell {
        return Err("Ivan has fallen".into());
    }
    let (_, cell) = pit_here(c, city, colony, defs).ok_or("Ivan stands at no Pit")?;
    let h = stash.handles().into_iter().map(|(h, _, _)| h).find(|&h| h == handle).ok_or("not in the Stash")?;
    let it = stash.take(h).ok_or("not in the Stash")?;
    let def = table.def(it.def);
    let (n, bias, d) = items::feed_value(def, &it);
    let drift: [f32; 10] = std::array::from_fn(|g| bias[g] * d * FEED_WEIGHT);
    let w = city.sim.w;
    city.sim.feed(cell % w, cell / w, n);
    crate::grow::fed(city, colony, table, it.def, drift);
    c.ivan.feed = day;
    let line = format!("Ivan fed {} to the Pit by hand: its bias at three times", def.name);
    log.push(city.tick, Class::Decision, line.clone());
    Ok(line)
}

/// Ooze-time: an Oathkeeper, raider or creature within 12 tiles of him; the view slows the clock. Once a shift.
pub fn ooze(c: &mut Campaign, threats: &Threats, day: u32) -> Result<(), String> {
    once(c.ivan.ooze, day, "Ooze-time")?;
    if c.ivan.fell {
        return Err("Ivan has fallen".into());
    }
    let me = (c.ivan.x as i32, c.ivan.y as i32);
    if !threats.bodies.iter().any(|b| !b.dead && !b.gone && b.role.hostile() && cheb((b.x as i32, b.y as i32), me) <= OOZE_REACH) {
        return Err("nothing hostile within 12 tiles".into());
    }
    c.ivan.ooze = day;
    Ok(())
}

/// The moment a bark is chosen for: his Authority state, the strain's dominant class (the held one whose genes stand
/// furthest over its line), the Rite.
pub fn ctx(colony: &Colony, sim: &mat::MatSim) -> Ctx {
    let f = progress::StrainFile::shipped();
    let g = &sim.genome.0;
    let class = f
        .class
        .iter()
        .filter(|k| colony.progress.strain.holds(&k.id))
        .map(|k| {
            let v: f32 = k.genes.iter().map(|x| progress::strain::gene_index(x).map_or(0.0, |i| g[i])).sum::<f32>() / 2.0;
            (k.id.clone(), v - k.line)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|k| k.0);
    let rite = colony.charter.law("dissolution_rite").is_some_and(|l| l.repeal_reach.is_none());
    Ctx { state: colony.charter.ivan_state(), class, rite, beat: 0 }
}

/// The events of the shift since the last call, said: the log's barks (a source marks them) and the threats' sent.
pub fn voice(c: &mut Campaign, colony: &Colony, sim: &mat::MatSim, threats: &mut Threats, log: &mut CityLog, tick: u64) {
    let ctx = ctx(colony, sim);
    for s in std::mem::take(&mut threats.sent) {
        let ev = match s.as_str() {
            "patrol" | "sweep" | "cleansing" | "bloom" => s,
            _ => continue,
        };
        c.barker.say(&ev, &ctx, &[], tick);
    }
    for (ev, extra) in std::mem::take(&mut log.barks) {
        let ex: Vec<(&str, &str)> = extra.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let mut x = ctx.clone();
        if let Some(b) = ex.iter().find(|e| e.0 == "beat").and_then(|e| e.1.parse().ok()) {
            x.beat = b;
        }
        c.barker.say(&ev, &x, &ex, tick);
    }
    c.barker.tick(tick);
}

#[cfg(test)]
mod tests {
    use super::*;
    use items::{ItemTable, ITEMS_TOML};

    #[test]
    fn ivan_walks_the_mat_to_his_goal_and_refuses_ground_off_it() {
        let mut w = crate::World::found(11, ItemTable::from_toml(ITEMS_TOML).unwrap());
        let (hx, hy) = w.city.sim.heart.unwrap();
        assert_eq!((w.campaign.ivan.x as usize, w.campaign.ivan.y as usize), (hx, hy), "at the Heart");
        let g = w.city.market_cells[0].unwrap();
        let sw = w.city.sim.w;
        order(&mut w.campaign, &w.city.sim, (g % sw) as u16, (g / sw) as u16).unwrap();
        let rock = (0..w.city.sim.sub.len()).find(|&i| w.city.sim.sub[i] == mat::sim::sub::ROCK).unwrap();
        assert!(order(&mut w.campaign, &w.city.sim, (rock % sw) as u16, (rock / sw) as u16).is_err());
        for _ in 0..800 {
            w.tick();
            if w.campaign.ivan.goal.is_none() {
                break;
            }
        }
        assert_eq!((w.campaign.ivan.x as usize, w.campaign.ivan.y as usize), (g % sw, g / sw), "at the market's cell along the Mat");
        let i = w.campaign.ivan.y as usize * sw + w.campaign.ivan.x as usize;
        assert!(is_hyphal(w.city.sim.hy[i]));
        // at the grate: a Parley, once a shift, a point of Authority
        w.colony.charter.earn(2);
        let before = w.economy.s.standing[0].favour + w.economy.s.standing.iter().map(|s| s.favour).sum::<i32>();
        let line = parley(&mut w.campaign, &w.city, &mut w.colony, &mut w.economy, w.day, &mut w.log).unwrap();
        assert!(line.contains("favour +10"), "{line}");
        assert_eq!(w.colony.charter.authority, 1);
        assert!(parley(&mut w.campaign, &w.city, &mut w.colony, &mut w.economy, w.day, &mut w.log).is_err(), "once a shift");
        assert!(w.economy.s.standing.iter().map(|s| s.favour).sum::<i32>() + w.economy.s.standing[0].favour > before);
    }

    #[test]
    fn the_city_s_events_reach_ivan_s_voice_for_his_state_and_the_strain_s_class() {
        let mut w = crate::World::found(11, ItemTable::from_toml(ITEMS_TOML).unwrap());
        // a Runner's death on the log: his next line is a death line with the name in it
        w.log.bark("runner_dead", &[("name", "Kerr")]);
        w.tick();
        let line = w.campaign.barker.current().expect("Ivan speaks").to_string();
        let f = story::barks::BarkFile::shipped();
        assert!(f.bark.iter().any(|b| b.event == "runner_dead" && story::strings::fill(&b.text, &[("name", "Kerr")]) == line), "{line}");
        // the moment: a Hive strain held reads the Hive's lines first
        w.colony.progress.strain.held.push(("hive".into(), 1));
        let ctx = ctx(&w.colony, &w.city.sim);
        assert_eq!(ctx.class.as_deref(), Some("hive"));
        assert_eq!(ctx.state, 1);
        // a stage sent is a line too, after the gap
        w.threats.sent.push("patrol".into());
        for _ in 0..story::barks::BARK_GAP + 1 {
            w.tick();
        }
        assert_eq!(w.campaign.barker.said, 2);
    }

    #[test]
    fn rally_feed_and_a_fall() {
        let mut w = crate::World::found(11, ItemTable::from_toml(ITEMS_TOML).unwrap());
        // Rally at the Heart: the founders near it gain Mood and walk faster
        let mood = |w: &crate::World| w.roster.records.iter().map(|r| r.needs[Need::Mood as usize]).sum::<i32>();
        let before = mood(&w);
        let n = rally(&mut w.campaign, &mut w.city, &mut w.roster, w.day, &mut w.log).unwrap();
        assert!(n > 0 && mood(&w) > before && !w.city.rallied.is_empty());
        assert!(rally(&mut w.campaign, &mut w.city, &mut w.roster, w.day, &mut w.log).is_err());
        // Feed the Heart by hand: a biased item at three times its drift
        let lume = progress::strain::gene_index("lume").unwrap();
        let (h, id) = w.stash.handles().iter().find(|(_, _, it)| w.table.def(it.def).bias.contains_key("lume")).map(|(h, _, it)| (*h, it.def)).expect("a Lume-biased item in the starting haul");
        let g0 = w.city.sim.genome.0[lume];
        let (_, bias, d) = items::feed_value(w.table.def(id), &items::ItemInstance::new(id));
        feed(&mut w.campaign, &mut w.city, &mut w.colony, &w.defs, &mut w.stash, &w.table, h, w.day, &mut w.log).unwrap();
        assert!((w.city.sim.genome.0[lume] - (g0 + bias[lume] * d * 3.0)).abs() < 1e-4, "three times the drift");
        // a fall: the squads' harm past his HP silences him and the ending will be Scattered
        w.threats.hurt.insert(IVAN_ID, IVAN_HP + 1.0);
        tick(&mut w.campaign, &w.city, &mut w.threats, &mut w.log);
        assert!(w.campaign.ivan.fell && w.campaign.ivan_fell && w.campaign.barker.silent);
        assert!(w.log.events.iter().any(|e| e.what.starts_with("Ivan fell")));
    }
}
