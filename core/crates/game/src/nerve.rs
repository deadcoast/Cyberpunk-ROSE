//! Nerve logic (Full Design v3.18, Nerve logic; Pass 2, carried in full): each Relay Shrine's four rules, read at its
//! tile every 20 ticks (the cargo field's beat). What holds lands in the colony's `NerveState`, which the threats
//! (gates, the militia), the cargo (quotas) and the light (dims) read; a Sever marks a cut and a Dump sends a Blackline
//! to a Pit, each once.

use colony::{Colony, NerveAction, NerveState, Reading, StructureFile, NERVE_REACH};
use inventory::{Dest, Ledger, Stash};
use items::{ItemTable, Tier};
use logistics::City;
use mat::sim::{hy, is_hyphal};
use threat::Threats;

use crate::log::{CityLog, Class};

fn cheb(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// What a reading reads at a tile now.
pub fn read(r: Reading, city: &City, traffic: &[u16], threats: Option<&Threats>, x: u16, y: u16) -> f32 {
    let sim = &city.sim;
    let (w, h) = (sim.w as i32, sim.h as i32);
    let i = y as usize * sim.w + x as usize;
    let around = |rad: i32, f: &dyn Fn(usize) -> f32| -> f32 {
        let mut v = 0.0;
        for dy in -rad..=rad {
            for dx in -rad..=rad {
                let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                if xx >= 0 && yy >= 0 && xx < w && yy < h {
                    v += f(yy as usize * sim.w + xx as usize);
                }
            }
        }
        v
    };
    match r {
        Reading::Signal => sim.sig[i],
        Reading::Nutrient => around(1, &|j| sim.n[j]) / 9.0,
        Reading::Attention => threats.and_then(|t| t.zone_at(sim, x, y).map(|k| t.zones[k].attention)).unwrap_or(0.0),
        Reading::Water => threats.map_or(0.0, |t| (t.flood.level + t.flood.sluice) as f32),
        Reading::Traffic => around(6, &|j| if is_hyphal(sim.hy[j]) { traffic.get(j).copied().unwrap_or(0) as f32 } else { 0.0 }),
        Reading::Light => threats
            .and_then(|t| t.bodies.iter().filter(|b| !b.dead && !b.gone && t.body_def(b.role).lamp > 0).map(|b| cheb((b.x as i32, b.y as i32), (x as i32, y as i32))).filter(|&d| d <= 30).min())
            .map_or(99.0, |d| d as f32),
        Reading::Strain => around(12, &|j| if sim.is_feral(j) { 1.0 } else { 0.0 }),
    }
}

/// One beat of every Shrine's rules. Returns the lines for the log.
#[allow(clippy::too_many_arguments)]
pub fn beat(city: &City, colony: &mut Colony, defs: &StructureFile, threats: Option<&Threats>, stash: &mut Stash, ledger: &mut Ledger, table: &ItemTable, log: &mut CityLog) {
    if colony.nerves.is_empty() || !colony.progress.tech.flag("nerve_logic") {
        colony.nerve = NerveState { dumped: colony.nerve.dumped, ..Default::default() };
        return;
    }
    let traffic = city.traffic();
    let mut st = NerveState { dumped: colony.nerve.dumped, ..Default::default() };
    let tick = city.tick;
    let w = city.sim.w;
    let bulb = progress::StrainFile::shipped().organ_index("bulb");
    let shrines: Vec<(u32, u16, u16, Option<u32>)> = colony.nerves.iter().filter_map(|(id, _)| colony.get(*id).filter(|s| s.built).map(|s| (s.id, s.x, s.y, s.district))).collect();
    for (sid, x, y, district) in shrines {
        let near = |sx: u16, sy: u16| cheb((sx as i32, sy as i32), (x as i32, y as i32)) <= NERVE_REACH;
        let rules = colony.nerves.iter().find(|n| n.0 == sid).map(|n| n.1.clone()).unwrap_or_default();
        let mut after = rules.clone();
        for (k, r) in rules.iter().enumerate() {
            let v = read(r.reading, city, &traffic, threats, x, y);
            let holds = if r.above { v > r.value } else { v < r.value };
            after[k].held = holds;
            if !holds {
                continue;
            }
            match r.action {
                NerveAction::ShutGates | NerveAction::OpenGates => {
                    for g in colony.structures.iter().filter(|g| g.built && defs.structure[g.def].shuts && near(g.x, g.y)) {
                        if r.action == NerveAction::ShutGates {
                            st.shut.insert(g.id);
                        } else {
                            st.open.insert(g.id);
                        }
                    }
                }
                NerveAction::FlipQuota => {
                    for p in colony.structures.iter().filter(|p| p.built && matches!(defs.structure[p.def].id.as_str(), "pit" | "heart") && near(p.x, p.y)) {
                        st.quota_off.insert(p.id);
                    }
                }
                NerveAction::Alarm => {
                    if let Some(d) = district.and_then(|d| colony.districts.iter().find(|x| x.id == d)) {
                        st.alarm.extend(d.members.iter().copied());
                    }
                }
                NerveAction::Dim => {
                    for f in colony.structures.iter().filter(|f| f.built && defs.structure[f.def].id == "lume_farm" && near(f.x, f.y)) {
                        st.dim.insert(f.id);
                    }
                    if let Some(b) = bulb {
                        for o in city.sim.organs_of(b) {
                            if near((o.i as usize % w) as u16, (o.i as usize / w) as u16) {
                                st.dark_bulbs.insert(o.i);
                            }
                        }
                    }
                }
                NerveAction::Sever if !r.held => {
                    // the outward cord beside the Relay: the hyphal neighbour with the least signal
                    let i = y as usize * w + x as usize;
                    let out = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
                        .iter()
                        .map(|(dx, dy)| (x as i32 + dx, y as i32 + dy))
                        .filter(|&(xx, yy)| xx >= 0 && yy >= 0 && (xx as usize) < w && (yy as usize) < city.sim.h)
                        .map(|(xx, yy)| yy as usize * w + xx as usize)
                        .filter(|&j| matches!(city.sim.hy[j], hy::CORD | hy::NODE | hy::TIP) && city.sim.sig[j] < city.sim.sig[i])
                        .min_by(|&a, &b| city.sim.sig[a].total_cmp(&city.sim.sig[b]).then(a.cmp(&b)));
                    if let Some(j) = out {
                        let (cx, cy) = ((j % w) as u16, (j / w) as u16);
                        if colony.mark_cut(&city.sim, cx, cy).is_ok() {
                            log.push(tick, Class::Decision, format!("a Nerve rule at {x},{y} marked {cx},{cy} to be cut"));
                        }
                    }
                }
                NerveAction::Dump if st.dumped != colony.shift => {
                    let vault = colony.structures.iter().any(|v| v.built && defs.structure[v.def].id == "vault" && near(v.x, v.y));
                    let pits = colony.pits(defs);
                    let pit = pits.iter().enumerate().filter_map(|(k, &p)| colony.get(p).map(|s| (k, cheb((s.x as i32, s.y as i32), (x as i32, y as i32))))).min_by_key(|&(k, d)| (d, k)).map(|(k, _)| k);
                    let item = stash.handles().into_iter().find(|(_, _, it)| !it.pinned && table.def(it.def).tier == Tier::Blackline).map(|(h, _, _)| h);
                    if let (true, Some(k), Some(h)) = (vault, pit, item) {
                        if let Some(it) = stash.take(h) {
                            let name = table.def(it.def).name.clone();
                            ledger.ship(it, Dest::Pit(k as u8));
                            st.dumped = colony.shift;
                            log.push(tick, Class::Decision, format!("a Nerve rule at {x},{y} dumped {name} into Pit {k}"));
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(n) = colony.nerves.iter_mut().find(|n| n.0 == sid) {
            n.1 = after;
        }
    }
    colony.nerve = st;
}
