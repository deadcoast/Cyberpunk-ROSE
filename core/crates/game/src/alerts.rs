//! Alerts (Full Design v3.12, Alerts): the crates raise them, the HUD shows them. Red is loss in progress, yellow a
//! decision, white the Oathkeepers (Phase D). Each carries the tile the camera jumps to and a key that stays the
//! same while its cause stands, so the queue does not repeat itself.

use colony::{Colony, ShiftOutcome, StructureFile};
use inventory::{Ledger, Stash};
use logistics::{City, Kind};
use people::{Need, Roster, Status};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Red,
    Yellow,
    White,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    pub level: Level,
    pub key: String,
    pub text: String,
    pub cell: Option<usize>,
}

/// A jam that has stood this long is a decision (grow a parallel cord): a minute.
pub const JAM_ALERT_TICKS: u64 = 120;

/// The alerts standing now. `last` is the last shift's outcome (strikes, defection, ghosts waiting).
#[allow(clippy::too_many_arguments)]
pub fn current(colony: &Colony, defs: &StructureFile, city: &City, roster: &Roster, stash: &Stash, ledger: &Ledger, last: Option<&ShiftOutcome>, jam_since: Option<u64>) -> Vec<Alert> {
    let mut v = Vec::new();
    let w = city.sim.w;
    let hub_cell = |d: &colony::District| Some(d.hub.1 as usize * w + d.hub.0 as usize);
    // red: loss in progress
    for d in colony.districts.iter().filter(|d| d.strike) {
        v.push(Alert { level: Level::Red, key: format!("strike {}", d.id), text: format!("{} is on strike", d.name), cell: hub_cell(d) });
    }
    if let Some(o) = last {
        for did in &o.ready_to_defect {
            if let Some(d) = colony.districts.iter().find(|d| d.id == *did) {
                v.push(Alert { level: Level::Red, key: format!("defect {}", d.id), text: format!("{}: ready to defect", d.name), cell: hub_cell(d) });
            }
        }
    }
    for s in colony.structures.iter().filter(|s| s.built) {
        let d = &defs.structure[s.def];
        let i = s.y as usize * w + s.x as usize;
        if d.sigma > 0.0 && i < city.sim.sig.len() && city.sim.sig[i] < d.sigma {
            v.push(Alert { level: Level::Red, key: format!("signal {}", s.id), text: format!("{} has lost its signal", d.name), cell: Some(i) });
        }
    }
    for r in roster.records.iter().filter(|r| r.is_up() && r.need(Need::Fed) < 10) {
        let cell = colony.homes.get(&r.id).and_then(|h| colony.cargo_cell(defs, &city.sim, *h)).or(city.heart_cell());
        v.push(Alert { level: Level::Red, key: format!("starving {}", r.id), text: format!("{} is starving", r.full_name()), cell });
    }
    if ledger.landed.spilled > 0 {
        v.push(Alert { level: Level::Red, key: "spilled".into(), text: format!("{} cargo spilled off the Mat this shift", ledger.landed.spilled), cell: None });
    }
    for r in roster.records.iter().filter(|r| r.status == Status::Stranded) {
        v.push(Alert { level: Level::Red, key: format!("stranded {}", r.id), text: format!("{} is stranded on a route", r.full_name()), cell: None });
    }
    // yellow: decisions
    for d in colony.districts.iter() {
        if let Some(p) = &d.petition {
            v.push(Alert { level: Level::Yellow, key: format!("petition {}", d.id), text: format!("{} petitions: {}", d.name, p.need.name().to_lowercase()), cell: hub_cell(d) });
        }
    }
    if let Some(o) = last {
        for (sid, cost) in &o.waiting {
            if let Some(s) = colony.get(*sid) {
                let d = &defs.structure[s.def];
                v.push(Alert { level: Level::Yellow, key: format!("waiting {sid}"), text: format!("{} waits for {}", d.name, cost.text()), cell: Some(s.y as usize * w + s.x as usize) });
            }
        }
    }
    for (k, sid) in colony.pits(defs).into_iter().enumerate() {
        if colony.quota_of(defs, sid).is_some_and(|q| q.0 > 0.0) && city.from_heart(Kind::Pit, k as u32).is_none() {
            let cell = colony.cargo_cell(defs, &city.sim, sid);
            v.push(Alert { level: Level::Yellow, key: format!("unreached {sid}"), text: "a Pit's quota: the Mat no longer reaches it".into(), cell });
        }
    }
    if let Some(since) = jam_since {
        if city.tick.saturating_sub(since) >= JAM_ALERT_TICKS {
            let cell = city.jam.iter().position(|&j| j);
            v.push(Alert { level: Level::Yellow, key: "jam".into(), text: "a jam has stood a minute: grow a parallel cord".into(), cell });
        }
    }
    if !roster.waiting.is_empty() {
        let n = roster.waiting.len();
        let cell = city.market_cells.iter().flatten().next().copied();
        let why = if cell.is_some() { "a free bed shelters them" } else { "the Mat reaches no grate" };
        v.push(Alert { level: Level::Yellow, key: "newcomers".into(), text: format!("{n} newcomer{} wait{} at a grate: {why}", if n == 1 { "" } else { "s" }, if n == 1 { "s" } else { "" }), cell });
    }
    if !stash.floor.is_empty() {
        v.push(Alert { level: Level::Yellow, key: "floor".into(), text: format!("{} on the Heart's floor: no room in the grid", stash.floor.len()), cell: city.heart_cell() });
    }
    v.sort_by(|a, b| a.level.cmp(&b.level).then(a.key.cmp(&b.key)));
    v
}
