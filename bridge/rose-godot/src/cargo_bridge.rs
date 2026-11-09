//! The cargo between the colony, the city and the Stash (Full Design v3.11, Colony mode: Cargo): the consumers the
//! colony publishes, the shift's dispatch (the queue, the quotas' pull, what the Stills and Salt Works make), the
//! landings as packets arrive, and the roster's movers. No rules live here that the crates do not state.

use colony::{Colony, QuotaFrom, StructureFile};
use inventory::{Dest, Landing, Quota, Want};
use items::{ItemInstance, Tier};
use logistics::{City, Consumer, Kind, Mover};
use people::{Roster, Status};

use crate::RoseStash;

pub(crate) const TIERS: [Tier; 5] = [Tier::Common, Tier::Uncommon, Tier::Rare, Tier::Blackline, Tier::Relic];

fn pit_stock(city: &City, cell: usize) -> f32 {
    let w = city.sim.w;
    city.sim.pits.get(&(cell % w, cell / w)).copied().unwrap_or(0.0)
}

/// The Pits' and Stills' demand: the share of the quota not met, counting what rides to it (0 with no quota).
fn demand(colony: &Colony, defs: &StructureFile, city: &City, stash: &RoseStash, sid: u32, to: Dest, cell: usize) -> f32 {
    let Some((target, _, _)) = colony.quota_of(defs, sid) else { return 0.0 };
    if target <= 0.0 {
        return 0.0;
    }
    let t = stash.table.as_ref().unwrap();
    let (n, k) = stash.ledger.riding(t, to);
    let have = match to {
        Dest::Pit(_) => pit_stock(city, cell) + n,
        _ => (stash.ledger.store_count(match to {
            Dest::Still(s) => s,
            _ => 0,
        }) + k) as f32,
    };
    ((target - have) / target).clamp(0.0, 1.0)
}

/// Every Pit (the Heart first) and Still as a consumer; the markets the Mat reaches the city adds itself.
pub(crate) fn consumers(colony: &Colony, defs: &StructureFile, city: &City, stash: &RoseStash) -> Vec<Consumer> {
    let mut out = Vec::new();
    for (k, sid) in colony.pits(defs).into_iter().enumerate() {
        let Some(cell) = colony.cargo_cell(defs, &city.sim, sid) else { continue };
        out.push(Consumer { kind: Kind::Pit, key: k as u32, cell, demand: demand(colony, defs, city, stash, sid, Dest::Pit(k as u8), cell), low: colony.low_priority.contains(&sid) });
    }
    for sid in colony.stills(defs) {
        let Some(cell) = colony.cargo_cell(defs, &city.sim, sid) else { continue };
        out.push(Consumer { kind: Kind::Still, key: sid, cell, demand: demand(colony, defs, city, stash, sid, Dest::Still(sid), cell), low: colony.low_priority.contains(&sid) });
    }
    out
}

/// Refresh the published demands in place (every field rebuild), without rebuilding the fields.
pub(crate) fn refresh_demands(colony: &Colony, defs: &StructureFile, city: &mut City, stash: &RoseStash) {
    let fresh = consumers(colony, defs, city, stash);
    for c in city.consumers.iter_mut().filter(|c| c.kind != Kind::Market) {
        if let Some(f) = fresh.iter().find(|f| f.kind == c.kind && f.key == c.key) {
            c.demand = f.demand;
            c.low = f.low;
        }
    }
}

pub(crate) fn dest_of(kind: Kind, key: u32) -> Dest {
    match kind {
        Kind::Pit => Dest::Pit(key as u8),
        Kind::Market => Dest::Market(key as u8),
        Kind::Still => Dest::Still(key),
        Kind::Heart => Dest::Stash,
    }
}

pub(crate) fn kind_of(to: Dest) -> (Kind, u32) {
    match to {
        Dest::Pit(p) => (Kind::Pit, p as u32),
        Dest::Market(m) => (Kind::Market, m as u32),
        Dest::Still(s) => (Kind::Still, s),
        Dest::Stash => (Kind::Heart, 0),
    }
}

/// Packets that reached their consumer: the ledger lands them (a Pit's nutrient into the Mat and the genome's
/// drift here), the Foreman's Cut takes its share, and what fell off the Mat is spilled.
pub(crate) fn land(city: &mut City, colony: &mut Colony, defs: &StructureFile, stash: &mut RoseStash) {
    let arrivals = city.take_arrivals();
    let spills = city.take_spills();
    if arrivals.is_empty() && spills.is_empty() {
        return;
    }
    stash.ensure();
    let table = stash.table.clone().unwrap();
    let pits = colony.pits(defs);
    for a in arrivals {
        // district cargo: the structure it landed at
        let sid = match a.kind {
            Kind::Pit => pits.get(a.key as usize).copied(),
            Kind::Still => Some(a.key),
            _ => None,
        };
        if let Some(sid) = sid {
            if colony.cut_takes(&city.sim, sid) {
                stash.ledger.divert(a.cargo);
                continue;
            }
        }
        let st = stash.stash.as_mut().unwrap();
        if let Some(Landing::Fed { pit, nutrient, drift }) = stash.ledger.land(st, &table, a.cargo, dest_of(a.kind, a.key)) {
            if let Some(cell) = pits.get(pit as usize).and_then(|sid| colony.cargo_cell(defs, &city.sim, *sid)) {
                let w = city.sim.w;
                city.sim.feed(cell % w, cell / w, nutrient);
            }
            city.sim.drift_genome(drift);
        }
    }
    for id in spills {
        stash.ledger.spill(id);
    }
}

/// The quotas standing on the Pits and Stills for this shift's pull.
fn quotas(colony: &Colony, defs: &StructureFile, city: &City, stash: &RoseStash) -> Vec<Quota> {
    let mut out = Vec::new();
    for (k, sid) in colony.pits(defs).into_iter().enumerate() {
        let Some((target, ceiling, _)) = colony.quota_of(defs, sid) else { continue };
        let Some(cell) = colony.cargo_cell(defs, &city.sim, sid) else { continue };
        if target <= 0.0 {
            continue;
        }
        out.push(Quota { to: Dest::Pit(k as u8), want: Want::Nutrient { target, stock: pit_stock(city, cell), ceiling: TIERS[ceiling.min(4) as usize] }, distance: city.from_heart(Kind::Pit, k as u32), low: colony.low_priority.contains(&sid) });
    }
    for sid in colony.stills(defs) {
        let Some((target, _, _)) = colony.quota_of(defs, sid) else { continue };
        if target <= 0.0 {
            continue;
        }
        out.push(Quota { to: Dest::Still(sid), want: Want::Tagged { tag: "enzyme".into(), target: target as u32, stock: stash.ledger.store_count(sid) }, distance: city.from_heart(Kind::Still, sid), low: colony.low_priority.contains(&sid) });
    }
    out
}

/// Shift end: what the Stills and Salt Works made rides home, the Stash's queue leaves the Heart, the quotas pull.
/// Returns the lines for the report.
pub(crate) fn dispatch_shift(city: &mut City, colony: &Colony, defs: &StructureFile, stash: &mut RoseStash, o: &colony::ShiftOutcome) -> Vec<String> {
    let mut lines = Vec::new();
    stash.ensure();
    let table = stash.table.clone().unwrap();
    let Some(heart) = city.heart_cell() else { return lines };
    city.set_consumers(consumers(colony, defs, city, stash));
    // products ride home from where they were made
    let mut made = 0;
    let mut brine = o.brine;
    for &still in &o.stills {
        if brine == 0 {
            break;
        }
        if stash.ledger.take_from_store(&table, still, "enzyme").is_none() {
            continue;
        }
        brine -= 1;
        if let (Some(idx), Some(cell)) = (table.id_of("org_downer"), colony.cargo_cell(defs, &city.sim, still)) {
            let mut dose = ItemInstance::new(idx);
            dose.condition = 100;
            let id = stash.ledger.ship(dose, Dest::Stash);
            city.dispatch(cell, id, Kind::Heart, 0);
            made += 1;
        }
    }
    if !o.stills.is_empty() {
        lines.push(if made > 0 { format!("the Still made {made} dose{}: on the cords home", if made == 1 { "" } else { "s" }) } else { "the Still made nothing: no brine or no Enzyme Part in its store".into() });
    }
    for &(works, n) in &o.salt_from {
        let (Some(idx), Some(cell)) = (table.id_of("prt_salt"), colony.cargo_cell(defs, &city.sim, works)) else { continue };
        for _ in 0..n {
            let mut salt = ItemInstance::new(idx);
            salt.condition = 90;
            let id = stash.ledger.ship(salt, Dest::Stash);
            city.dispatch(cell, id, Kind::Heart, 0);
        }
        if n > 0 {
            lines.push(format!("the Salt Works made {n} salt: on the cords home"));
        }
    }
    // the queue, then the quotas' pull, from the Heart
    for (id, to) in stash.ledger.dispatch_queue() {
        let (k, key) = kind_of(to);
        city.dispatch(heart, id, k, key);
    }
    let qs = quotas(colony, defs, city, stash);
    let st = stash.stash.as_mut().unwrap();
    let pulled = stash.ledger.pull(st, &table, &qs);
    if !pulled.is_empty() {
        lines.push(format!("the quotas pulled {} item{} from the Stash", pulled.len(), if pulled.len() == 1 { "" } else { "s" }));
    }
    for (id, to) in pulled {
        let (k, key) = kind_of(to);
        city.dispatch(heart, id, k, key);
    }
    lines
}

/// The roster on the map: everyone in the city at their bed, or at the structure the shift's plan gives them.
pub(crate) fn movers(colony: &Colony, defs: &StructureFile, city: &City, roster: &Roster, plan: &[(u32, people::Job, u32)]) -> Vec<Mover> {
    let heart = city.heart_cell().unwrap_or(0);
    roster
        .records
        .iter()
        .filter(|r| !matches!(r.status, Status::OnRun | Status::Stranded | Status::Dead))
        .map(|r| {
            let home = colony.homes.get(&r.id).and_then(|h| colony.cargo_cell(defs, &city.sim, *h)).unwrap_or(heart);
            let work = plan.iter().find(|(s, _, _)| *s == r.id).and_then(|(_, _, sid)| colony.cargo_cell(defs, &city.sim, *sid));
            Mover { id: r.id, home, work }
        })
        .collect()
}

/// A Pit's or Still's quota for its card: kind, target, ceiling, whose, the stock now, what rides to it, the cap.
pub(crate) fn quota_view(colony: &Colony, defs: &StructureFile, city: &City, stash: &RoseStash, sid: u32) -> Option<godot::prelude::VarDictionary> {
    use godot::prelude::*;
    let s = colony.get(sid)?;
    let d = &defs.structure[s.def];
    let (target, ceiling, from) = colony.quota_of(defs, sid).unwrap_or((0.0, colony::QUOTA_CEILING, QuotaFrom::Set));
    let table = stash.table.as_ref()?;
    let mut e = VarDictionary::new();
    e.set("structure", sid as i64);
    e.set("name", d.name.as_str());
    e.set("target", target as f64);
    e.set("ceiling", ceiling as i64);
    e.set("ceiling_name", format!("{:?}", TIERS[ceiling.min(4) as usize]).to_uppercase().as_str());
    e.set("from", match (s.quota.is_some(), from) {
        (true, _) => "yours",
        (false, QuotaFrom::Foreman) => "the foreman's",
        (false, QuotaFrom::Slots) => "its slots",
        (false, QuotaFrom::Set) => "off",
    });
    let cell = colony.cargo_cell(defs, &city.sim, sid)?;
    match d.id.as_str() {
        "still" => {
            e.set("kind", "still");
            e.set("stock", stash.ledger.store_count(sid) as f64);
            e.set("riding", stash.ledger.riding(table, Dest::Still(sid)).1 as f64);
            e.set("cap", 12.0);
            e.set("reached", city.from_heart(Kind::Still, sid).is_some());
        }
        _ => {
            let k = colony.pits(defs).iter().position(|p| *p == sid)? as u32;
            e.set("kind", "pit");
            e.set("pit", k as i64);
            e.set("stock", pit_stock(city, cell) as f64);
            e.set("riding", stash.ledger.riding(table, Dest::Pit(k as u8)).0 as f64);
            e.set("cap", d.stock_at(s.tier).max(0) as f64);
            e.set("reached", city.from_heart(Kind::Pit, k).is_some());
        }
    }
    Some(e)
}
