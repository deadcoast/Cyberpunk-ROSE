//! The Cleansing proof (Full Design v3.14, Pressure: The proof; the Build plan's Phase D): a Cleansing sent down the
//! first grate of a founded city its steward has played for some shifts, twice from the same state — once as it stands,
//! once with two Sphincter Gates and a Scrap Turret on the squads' way from the grate and a Guard on each.

use std::collections::VecDeque;

use people::Skill;

use crate::city;
use crate::log::Class;
use crate::World;

/// The city counted before and after.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub cells: usize,
    pub structures: usize,
    pub slimos: usize,
    pub heart_hp: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Siege {
    pub defended: bool,
    pub defence: Vec<String>,
    /// the Gates close every way from the grate to the Heart
    pub closed: bool,
    pub before: Tally,
    pub after: Tally,
    pub heart_lost: bool,
    pub sent: u32,
    pub fallen: u32,
    pub burned: u32,
    pub ruined: u32,
    pub dead: usize,
    pub bleeding: usize,
    /// city seconds from the sluice to the last body gone
    pub seconds: f32,
    pub blackline: u32,
    pub log: String,
}

fn tally(w: &World) -> Tally {
    Tally {
        cells: w.city.sim.hyphal_count(),
        structures: w.colony.structures.iter().filter(|s| s.built).count(),
        slimos: w.roster.records.len(),
        heart_hp: w.colony.heart(&w.defs).map_or(0, |h| h.hp),
    }
}

/// Steps from the Heart over the ground bodies walk, Gates closed or not.
fn heart_field(w: &World, gates_closed: bool) -> Vec<u16> {
    let sim = &w.city.sim;
    let gate = w.defs.index("gate");
    let gates: Vec<usize> = w.colony.structures.iter().filter(|s| s.built && Some(s.def) == gate).flat_map(|s| s.tiles(&w.defs.structure[s.def])).map(|(x, y)| y as usize * sim.w + x as usize).collect();
    let mut d = vec![u16::MAX; sim.w * sim.h];
    let Some((hx, hy)) = sim.heart else { return d };
    let start = hy * sim.w + hx;
    d[start] = 0;
    let mut q = VecDeque::from([start]);
    while let Some(i) = q.pop_front() {
        let (x, y) = (i % sim.w, i / sim.w);
        for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
            if nx < sim.w && ny < sim.h {
                let j = ny * sim.w + nx;
                if d[j] == u16::MAX && threat::walkable(sim, j) && !(gates_closed && gates.contains(&j)) {
                    d[j] = d[i] + 1;
                    q.push_back(j);
                }
            }
        }
    }
    d
}

/// The proof's defence: two Sphincter Gates across the narrowest crossing of the squads' way down from the grate that
/// the Mat spans — a run of walkable ground at most two wide, wall on both sides, every tile of it Cord or Node a Gate
/// can stand on — the first such crossing down from the grate (else the first two placeable tiles of the way past 8
/// and 14 steps); a Scrap Turret within 3 tiles of the way on the Heart's side of the Gates, the nearest them along
/// the way that its signal allows; a Guard ordered to each, the best Fight first.
fn defend(w: &mut World, gx: usize) -> Vec<String> {
    let mut out = Vec::new();
    let f = heart_field(w, false);
    let sw = w.city.sim.w;
    let sh = w.city.sim.h;
    // the way: from the grate down the field to the Heart
    let mut way = vec![gx];
    let mut at = gx;
    while f[at] > 0 && f[at] != u16::MAX {
        let (x, y) = (at % sw, at / sw);
        let next = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)]
            .into_iter()
            .filter(|&(nx, ny)| nx < sw && ny < sh)
            .map(|(nx, ny)| ny * sw + nx)
            .filter(|&j| f[j] < f[at])
            .min_by_key(|&j| (f[j], j));
        let Some(n) = next else { break };
        way.push(n);
        at = n;
    }
    let walk = |i: usize| threat::walkable(&w.city.sim, i);
    // a crossing through a way tile: the walkable run across it, row-wise or column-wise, wall at both ends, at most
    // four wide (two Gates)
    let crossing = |i: usize| -> Vec<Vec<usize>> {
        let (x, y) = (i % sw, i / sw);
        let mut runs = Vec::new();
        for horizontal in [true, false] {
            let mut run = vec![i];
            let mut open = false;
            for dir in [-1i32, 1] {
                let mut k = 1i32;
                loop {
                    let (nx, ny) = if horizontal { (x as i32 + dir * k, y as i32) } else { (x as i32, y as i32 + dir * k) };
                    if nx < 0 || ny < 0 || nx >= sw as i32 || ny >= sh as i32 {
                        break;
                    }
                    let j = ny as usize * sw + nx as usize;
                    if !walk(j) {
                        break;
                    }
                    run.push(j);
                    k += 1;
                    if run.len() > 4 {
                        open = true;
                        break;
                    }
                }
            }
            if !open {
                run.sort_unstable();
                runs.push((horizontal, run));
            }
        }
        runs.into_iter().map(|(_, r)| r).collect()
    };
    // a Gate spans two tiles (anchored on a cord, turned for a shaft); the first crossing down from the grate that
    // two Gates at most can close, every Gate's anchor on the Mat
    let mut gates: Vec<u32> = Vec::new();
    'find: for (k, &i) in way.iter().enumerate().skip(1) {
        for run in crossing(i) {
            let turned = run[0] / sw != run[run.len() - 1] / sw;
            let step = if turned { sw } else { 1 };
            let anchors: Vec<usize> = run.iter().step_by(2).copied().collect();
            if anchors.len() + gates.len() > 2 {
                continue;
            }
            let mut ids = Vec::new();
            for &a in &anchors {
                let (ax, ay) = ((a % sw) as u16, (a / sw) as u16);
                let _ = step;
                match w.colony.place_r(&w.defs, &w.city.sim, "gate", ax, ay, turned) {
                    Ok(id) => ids.push(id),
                    Err(_) => break,
                }
            }
            let closes = ids.len() == anchors.len() && run.iter().all(|&j| w.colony.occupied(&w.defs, (j % sw) as u16, (j / sw) as u16).is_some_and(|o| ids.contains(&o)));
            if !closes {
                for id in ids {
                    w.colony.cancel(&w.defs, id);
                }
                continue;
            }
            for id in &ids {
                if let Some(st) = w.colony.get_mut(*id) {
                    st.built = true;
                    st.paid = true;
                    st.hp = w.defs.structure[st.def].hp_at(1);
                    out.push(format!("Sphincter Gate at {},{}{}, across the way {k} steps down from the grate", st.x, st.y, if turned { " turned" } else { "" }));
                }
            }
            gates.extend(ids);
            if gates.len() >= 2 {
                break 'find;
            }
        }
    }
    if gates.is_empty() {
        let mut last = 0usize;
        for (k, &i) in way.iter().enumerate() {
            let need = if gates.is_empty() { 8 } else { last + 6 };
            if gates.len() == 2 || k < need {
                continue;
            }
            if let Ok(id) = w.colony.place_built(&w.defs, &w.city.sim, "gate", (i % sw) as u16, (i / sw) as u16, 1) {
                gates.push(id);
                last = k;
                out.push(format!("Sphincter Gate at {},{}, on the way {k} steps down from the grate (no crossing the Mat spans)", i % sw, i / sw));
            }
        }
    }
    // the Turret: where it may stand (σ 0.5) within 3 tiles of the way, on the Heart's side of the first Gate, the
    // nearest the Gate along the way
    let mut posts = gates.clone();
    let gate_k = gates.first().and_then(|&g| w.colony.get(g)).and_then(|g| way.iter().position(|&i| i == g.y as usize * sw + g.x as usize)).unwrap_or(0);
    let mut spot = None;
    'turret: for &i in way.iter().skip(gate_k + 1) {
        let (wx, wy) = ((i % sw) as i32, (i / sw) as i32);
        let mut near: Vec<(i32, usize)> = Vec::new();
        for dy in -3i32..=3 {
            for dx in -3i32..=3 {
                let (x, y) = (wx + dx, wy + dy);
                if x >= 0 && y >= 0 && (x as usize) < sw && (y as usize) < sh && dx.abs() + dy.abs() <= 3 {
                    near.push((dx.abs() + dy.abs(), y as usize * sw + x as usize));
                }
            }
        }
        near.sort_unstable();
        for (_, j) in near {
            if w.colony.can_place(&w.defs, &w.city.sim, "turret", (j % sw) as u16, (j / sw) as u16).is_ok() {
                spot = Some(j);
                break 'turret;
            }
        }
    }
    if let Some(i) = spot {
        if let Ok(id) = w.colony.place_built(&w.defs, &w.city.sim, "turret", (i % sw) as u16, (i / sw) as u16, 1) {
            posts.push(id);
            out.push(format!("Scrap Turret at {},{}", i % sw, i / sw));
        }
    }
    let mut fighters: Vec<(u8, u32)> = w.roster.records.iter().filter(|r| r.is_up()).map(|r| (r.skill(Skill::Fight), r.id)).collect();
    fighters.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (sid, (_, slimo)) in posts.iter().zip(fighters) {
        if w.colony.order(&w.defs, slimo, *sid).is_ok() {
            let name = w.roster.get(slimo).map(|r| r.full_name()).unwrap_or_default();
            out.push(format!("{name} on Guard at {}", w.defs.structure[w.colony.get(*sid).unwrap().def].name));
        }
    }
    out
}

/// Play `shifts` of the city from `seed`, then send a Cleansing down its first grate, the city defended or not.
pub fn cleansing(seed: u64, shifts: u32, defended: bool) -> Siege {
    let (_, mut w) = city::play_world(seed, shifts, &mut |_, _| {});
    let mut out = Siege { defended, before: tally(&w), ..Default::default() };
    let gx = city::first_grate(&w).map(|g| g.0 as usize).unwrap_or(0);
    if defended {
        out.defence = defend(&mut w, gx);
        out.closed = heart_field(&w, true)[gx] == u16::MAX;
        w.publish();
    }
    w.movers();
    let zone = w.threats.zone_at(&w.city.sim, gx as u16, 0).unwrap_or(0);
    let stage = w.threats.rules.stage_index("cleansing").expect("a Cleansing in threats.toml");
    let log_from = w.log.events.len();
    let start = w.city.tick;
    let notes = w.threats.send(&w.city, &w.colony, &w.defs, stage, zone);
    w.note(notes);
    let lasts = w.threats.rules.stage[stage].lasts + 1200;
    let id = w.threats.events.last().map(|e| e.id).unwrap_or(0);
    for _ in 0..lasts {
        w.tick();
        if let Some(e) = w.threats.events.iter().find(|e| e.id == id) {
            out.sent = e.sent;
            out.fallen = e.fallen;
            out.burned = e.burned;
            out.ruined = e.ruined;
            if e.phase == threat::Phase::Done {
                break;
            }
        }
    }
    out.seconds = (w.city.tick - start) as f32 / 2.0;
    out.heart_lost = w.city.sim.heart_lost;
    let roster_before = w.roster.records.len();
    let bleeding_before = w.roster.records.iter().filter(|r| r.injuries.iter().any(|(i, _)| *i == people::Injury::Bleeding)).count();
    let owed = w.threats.owed.0;
    w.end_shift();
    out.blackline = owed;
    out.dead = roster_before.saturating_sub(w.roster.records.len());
    out.bleeding = w.roster.records.iter().filter(|r| r.injuries.iter().any(|(i, _)| *i == people::Injury::Bleeding)).count().saturating_sub(bleeding_before);
    out.after = tally(&w);
    for e in &w.log.events[log_from..] {
        if matches!(e.class, Class::Loss | Class::Landing) && !e.what.contains(" fed to ") && !e.what.contains(" home") {
            out.log.push_str(&format!("{:>6.1} s  {}  {}\n", (e.tick.saturating_sub(start)) as f32 / 2.0, e.class.name(), e.what));
        }
    }
    out
}
