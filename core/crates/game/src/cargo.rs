//! The cargo between the colony, the city and the Stash (Full Design v3.11, Cargo; v3.12, Stockpiles): the consumers
//! the colony publishes, the shift's dispatch (the queue, the quotas' pull, what the Stills and Salt Works make), the
//! landings as packets arrive, and the roster's movers.

use colony::{Colony, ShiftOutcome, StructureFile};
use inventory::{Dest, Landing, Ledger, Quota, Stash, Want};
use items::{ItemInstance, ItemTable, Tier};
use logistics::{City, Consumer, Kind, Mover};
use people::{Job, Roster, Status};

use crate::log::{CityLog, Class};

pub const TIERS: [Tier; 5] = [Tier::Common, Tier::Uncommon, Tier::Rare, Tier::Blackline, Tier::Relic];

pub fn pit_stock(city: &City, cell: usize) -> f32 {
    let w = city.sim.w;
    city.sim.pits.get(&(cell % w, cell / w)).copied().unwrap_or(0.0)
}

/// The Pits' and Stills' demand: the share of the quota not met, counting what rides to it (0 with no quota).
fn demand(colony: &Colony, defs: &StructureFile, city: &City, ledger: &Ledger, table: &ItemTable, sid: u32, to: Dest, cell: usize) -> f32 {
    let Some((target, _, _)) = colony.quota_of(defs, sid) else { return 0.0 };
    if target <= 0.0 {
        return 0.0;
    }
    let (n, k) = ledger.riding(table, to);
    let have = match to {
        Dest::Pit(_) => pit_stock(city, cell) + n,
        Dest::Still(s) => (ledger.store_count(s) + k) as f32,
        _ => 0.0,
    };
    ((target - have) / target).clamp(0.0, 1.0)
}

/// Every Pit (the Heart first), Still and stockpile as a consumer; the markets the Mat reaches the city adds itself.
pub fn consumers(colony: &Colony, defs: &StructureFile, city: &City, stash: &Stash, ledger: &Ledger, table: &ItemTable) -> Vec<Consumer> {
    let mut out = Vec::new();
    for (k, sid) in colony.pits(defs).into_iter().enumerate() {
        let Some(cell) = colony.cargo_cell(defs, &city.sim, sid) else { continue };
        out.push(Consumer { kind: Kind::Pit, key: k as u32, cell, demand: demand(colony, defs, city, ledger, table, sid, Dest::Pit(k as u8), cell), low: colony.low_priority.contains(&sid) });
    }
    for sid in colony.stills(defs) {
        let Some(cell) = colony.cargo_cell(defs, &city.sim, sid) else { continue };
        out.push(Consumer { kind: Kind::Still, key: sid, cell, demand: demand(colony, defs, city, ledger, table, sid, Dest::Still(sid), cell), low: colony.low_priority.contains(&sid) });
    }
    for r in &stash.rooms {
        let Some(cell) = colony.cargo_cell(defs, &city.sim, r.anchor) else { continue };
        out.push(Consumer { kind: Kind::Room, key: r.zone, cell, demand: 0.0, low: colony.low_priority.contains(&r.anchor) });
    }
    out
}

/// Refresh the published demands in place (every field rebuild), without rebuilding the fields.
pub fn refresh_demands(colony: &Colony, defs: &StructureFile, city: &mut City, stash: &Stash, ledger: &Ledger, table: &ItemTable) {
    let fresh = consumers(colony, defs, city, stash, ledger, table);
    for c in city.consumers.iter_mut().filter(|c| c.kind != Kind::Market) {
        if let Some(f) = fresh.iter().find(|f| f.kind == c.kind && f.key == c.key) {
            c.demand = f.demand;
            c.low = f.low;
        }
    }
}

pub fn dest_of(kind: Kind, key: u32) -> Dest {
    match kind {
        Kind::Pit => Dest::Pit(key as u8),
        Kind::Market => Dest::Market(key as u8),
        Kind::Still => Dest::Still(key),
        Kind::Heart => Dest::Stash,
        Kind::Room => Dest::Room(key),
    }
}

pub fn kind_of(to: Dest) -> (Kind, u32) {
    match to {
        Dest::Pit(p) => (Kind::Pit, p as u32),
        Dest::Market(m) => (Kind::Market, m as u32),
        Dest::Still(s) => (Kind::Still, s),
        Dest::Stash => (Kind::Heart, 0),
        Dest::Room(z) => (Kind::Room, z),
    }
}

/// Where a shipment leaves from: the Heart, or the store a stockpile hangs on.
pub fn source_cell(colony: &Colony, defs: &StructureFile, city: &City, stash: &Stash, from: Option<u32>) -> Option<usize> {
    match from.and_then(|z| stash.rooms.iter().find(|r| r.zone == z)) {
        Some(r) => colony.cargo_cell(defs, &city.sim, r.anchor).or_else(|| city.heart_cell()),
        None => city.heart_cell(),
    }
}

/// Put a shipment on the Mat from where it lies.
pub fn send(city: &mut City, colony: &Colony, defs: &StructureFile, stash: &Stash, id: u32, to: Dest, from: Option<u32>) {
    if let Some(at) = source_cell(colony, defs, city, stash, from) {
        let (k, key) = kind_of(to);
        city.dispatch(at, id, k, key);
    }
}

/// Packets that reached their consumer: the ledger lands them (a Pit's nutrient into the Mat and the genome's
/// drift here), the Foreman's Cut takes its share, and what fell off the Mat is spilled. Each is a line of the
/// city's log. Returns the sales, (market, eddies), for the grates' Attention.
#[allow(clippy::too_many_arguments)]
pub fn land(city: &mut City, colony: &mut Colony, defs: &StructureFile, stash: &mut Stash, ledger: &mut Ledger, table: &ItemTable, log: &mut CityLog) -> Vec<(u8, u32)> {
    let mut sold = Vec::new();
    let arrivals = city.take_arrivals();
    let spills = city.take_spills();
    if arrivals.is_empty() && spills.is_empty() {
        return sold;
    }
    let pits = colony.pits(defs);
    let tick = city.tick;
    for a in arrivals {
        let name = ledger.in_flight.iter().find(|s| s.id == a.cargo).map(|s| table.def(s.item.def).name.clone()).unwrap_or_default();
        let sent_to = ledger.in_flight.iter().find(|s| s.id == a.cargo).map(|s| s.to);
        // district cargo: the structure it landed at
        let sid = match a.kind {
            Kind::Pit => pits.get(a.key as usize).copied(),
            Kind::Still => Some(a.key),
            _ => None,
        };
        if let Some(sid) = sid {
            if colony.cut_takes(&city.sim, sid) {
                ledger.divert(a.cargo);
                log.push(tick, Class::Loss, format!("the foreman's cut took {name}"));
                continue;
            }
        }
        match ledger.land(stash, table, a.cargo, dest_of(a.kind, a.key)) {
            Some(Landing::Fed { pit, nutrient, drift }) => {
                if let Some(cell) = pits.get(pit as usize).and_then(|sid| colony.cargo_cell(defs, &city.sim, *sid)) {
                    let w = city.sim.w;
                    city.sim.feed(cell % w, cell / w, nutrient);
                }
                city.sim.drift_genome(drift);
                log.push(tick, Class::Landing, format!("{name} fed to {}", if pit == 0 { "the Heart".to_string() } else { format!("Pit {pit}") }));
            }
            Some(Landing::Sold { market, eddies }) => {
                sold.push((market, eddies));
                log.push(tick, Class::Landing, format!("{name} sold for {eddies} e at grate {}", city.market_grate(market as usize)))
            }
            Some(Landing::Stored { .. }) => log.push(tick, Class::Landing, format!("{name} into the Still's store")),
            Some(Landing::Home { placed }) => {
                let back = if a.kind == Kind::Heart && sent_to != Some(Dest::Stash) { ", back: nothing of its kind in reach" } else { "" };
                log.push(tick, Class::Landing, format!("{name} home{back}{}", if placed { "" } else { ": held at the Heart, no room" }))
            }
            Some(Landing::Shelved { placed, .. }) => log.push(tick, Class::Landing, format!("{name} {}", if placed { "shelved" } else { "back to the Heart: the stockpile filled" })),
            None => {}
        }
    }
    for id in spills {
        if let Some(it) = ledger.spill(id) {
            log.push(tick, Class::Loss, format!("{} spilled off the Mat", table.def(it.def).name));
        }
    }
    sold
}

/// The quotas standing on the Pits and Stills for this shift's pull.
fn quotas(colony: &Colony, defs: &StructureFile, city: &City, ledger: &Ledger) -> Vec<Quota> {
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
        out.push(Quota { to: Dest::Still(sid), want: Want::Tagged { tag: "enzyme".into(), target: target as u32, stock: ledger.store_count(sid) }, distance: city.from_heart(Kind::Still, sid), low: colony.low_priority.contains(&sid) });
    }
    out
}

/// Shift end: what the Stills and Salt Works made rides home, the Stash's queue leaves the Heart (or its
/// stockpile), the quotas pull. Returns the lines for the report.
#[allow(clippy::too_many_arguments)]
pub fn dispatch_shift(city: &mut City, colony: &Colony, defs: &StructureFile, stash: &mut Stash, ledger: &mut Ledger, table: &ItemTable, o: &ShiftOutcome) -> Vec<String> {
    let mut lines = Vec::new();
    if city.heart_cell().is_none() {
        return lines;
    }
    city.set_consumers(consumers(colony, defs, city, stash, ledger, table));
    // products ride home from where they were made
    let mut made = 0;
    let mut brine = o.brine;
    for &still in &o.stills {
        if brine == 0 {
            break;
        }
        if ledger.take_from_store(table, still, "enzyme").is_none() {
            continue;
        }
        brine -= 1;
        if let (Some(idx), Some(cell)) = (table.id_of("org_downer"), colony.cargo_cell(defs, &city.sim, still)) {
            let id = ledger.ship(ItemInstance::new(idx), Dest::Stash);
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
            let id = ledger.ship(salt, Dest::Stash);
            city.dispatch(cell, id, Kind::Heart, 0);
        }
        if n > 0 {
            lines.push(format!("the Salt Works made {n} salt: on the cords home"));
        }
    }
    // the queue, then the quotas' pull, each from where its item lies
    for (id, to, from) in ledger.dispatch_queue() {
        send(city, colony, defs, stash, id, to, from);
    }
    let qs = quotas(colony, defs, city, ledger);
    let pulled = ledger.pull(stash, table, &qs);
    if !pulled.is_empty() {
        lines.push(format!("the quotas pulled {} item{} from the Stash", pulled.len(), if pulled.len() == 1 { "" } else { "s" }));
    }
    for (id, to, from) in pulled {
        send(city, colony, defs, stash, id, to, from);
    }
    lines
}

/// The roster on the map: everyone in the city at their bed, or at the structure the shift's plan gives them.
pub fn movers(colony: &Colony, defs: &StructureFile, city: &City, roster: &Roster, plan: &[(u32, Job, u32)]) -> Vec<Mover> {
    let heart = city.heart_cell().unwrap_or(0);
    roster
        .records
        .iter()
        .filter(|r| !matches!(r.status, Status::OnRun | Status::Stranded | Status::Dead))
        .map(|r| {
            let home = colony.homes.get(&r.id).and_then(|h| colony.cargo_cell(defs, &city.sim, *h)).unwrap_or(heart);
            let work = plan.iter().find(|(s, _, _)| *s == r.id).and_then(|(_, _, sid)| {
                // a cut is worked where it is marked
                colony.cargo_cell(defs, &city.sim, *sid).or_else(|| colony.cuts.iter().find(|c| c.id == *sid).map(|c| c.y as usize * city.sim.w + c.x as usize))
            });
            Mover { id: r.id, home, work }
        })
        .collect()
}
