//! The Stash's side of a shift (Full Design: Stash; v3.12, Stockpiles): paying for a ghost, the Appraisal Table,
//! the rooms mirrored from the stockpile zones, the settling by priority, the haulers, spoiling, the Bench's slots
//! and the ledger's close.

use bench::{BenchQueue, RecipeFile};
use colony::{Colony, Cost, StructureFile, ZoneKind};
use inventory::{Dest, Ledger, Room, ShiftReport, Stash, FLOOR, GRID_PRIORITY};
use items::{price, Category, Ident, ItemInstance, ItemTable, PriceContext};
use logistics::City;

use crate::cargo;

/// Can the Stash pay this cost: unpinned Parts and Chrome items by count from any room, eddies from the ledger.
/// Pays when it can (the cheapest items first) and says so.
pub fn take_cost(stash: &mut Stash, ledger: &mut Ledger, table: &ItemTable, cost: Cost) -> bool {
    let pick = |stash: &Stash, cat: Category| -> Vec<(u32, u32)> {
        let mut v: Vec<(u32, u32)> = stash
            .handles()
            .into_iter()
            .filter(|(_, _, it)| !it.pinned && table.def(it.def).category == cat)
            .map(|(h, _, it)| (price(table.def(it.def), it, PriceContext::default()), h))
            .collect();
        v.sort();
        v
    };
    let parts = pick(stash, Category::Part);
    let chrome = pick(stash, Category::Chrome);
    if (parts.len() as u32) < cost.parts || (chrome.len() as u32) < cost.chrome || ledger.eddies < cost.eddies {
        return false;
    }
    for (_, h) in parts.iter().take(cost.parts as usize).chain(chrome.iter().take(cost.chrome as usize)) {
        stash.take(*h);
    }
    ledger.eddies -= cost.eddies;
    true
}

/// Identify up to `n` unidentified items, where they lie (the Appraisal Table's shift). Returns what it named.
pub fn identify_n(stash: &mut Stash, n: usize) -> Vec<items::ItemId> {
    let handles: Vec<u32> = stash.handles().into_iter().filter(|(_, _, it)| it.ident != Ident::Full).map(|(h, _, _)| h).take(n).collect();
    let mut done = Vec::new();
    for h in handles {
        if let Some(it) = stash.get_any_mut(h) {
            it.ident = Ident::Full;
            done.push(it.def);
        }
    }
    done
}

/// What the Stash can pay with now: unpinned Parts and Chrome by count (any room), eddies (the Build menu's have).
pub fn holds(stash: &Stash, ledger: &Ledger, table: &ItemTable) -> Cost {
    let count = |cat: Category| stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && table.def(it.def).category == cat).count() as u32;
    Cost { parts: count(Category::Part), chrome: count(Category::Chrome), eddies: ledger.eddies }
}

/// Take the bait (Full Design v3.12, The steering tools): the cheapest unpinned Common Chrome with a feed value, from
/// any room. Returns its nutrient, to be smeared over the 3 x 3 around the click.
pub fn take_bait(stash: &mut Stash, table: &ItemTable) -> Option<f32> {
    let (_, h) = stash
        .handles()
        .into_iter()
        .filter(|(_, _, it)| {
            let d = table.def(it.def);
            !it.pinned && d.category == Category::Chrome && d.tier == items::Tier::Common && d.feed > 0.0
        })
        .map(|(h, _, it)| (price(table.def(it.def), it, PriceContext::default()), h))
        .min()?;
    let item = stash.take(h)?;
    Some(items::feed_value(table.def(item.def), &item).0)
}

/// A rig home from a run as items, the most valuable first (the order the grid takes them in).
pub fn rig_items(rig: &inventory::Rig, table: &ItemTable) -> Vec<ItemInstance> {
    // the backpack in its grid's order (its own map has none), so a haul lands the same way every time
    let mut items: Vec<ItemInstance> = rig.backpack.placed().into_iter().map(|(_, p)| p.item.clone()).collect();
    items.extend(rig.quick.iter().flatten().cloned());
    let g = &rig.gear;
    items.extend([&g.head, &g.body, &g.feet, &g.back].into_iter().flatten().cloned());
    items.extend(g.hands.iter().flatten().cloned());
    items.sort_by_key(|it| std::cmp::Reverse(table.def(it.def).value));
    items
}

/// Take a bag of road salt from the Stash (the cheapest, from any room).
pub fn take_salt_bag(stash: &mut Stash, table: &ItemTable) -> bool {
    let pick = stash
        .handles()
        .into_iter()
        .filter(|(_, _, it)| !it.pinned && table.def(it.def).tags.iter().any(|t| t == "salt"))
        .map(|(h, _, it)| (it.condition, h))
        .min();
    match pick {
        Some((_, h)) => stash.take(h).is_some(),
        None => false,
    }
}

/// The rooms mirror the stockpile zones whose store stands: cap, cells on and off the Mat, footprint, priority,
/// filter. A zone gone or its store not built leaves its room's items where they are until a store stands again.
pub fn sync_rooms(stash: &mut Stash, colony: &Colony, defs: &StructureFile, city: &City) {
    let mat = &city.sim;
    for z in colony.zones.iter().filter(|z| z.kind == ZoneKind::Stockpile) {
        let Some(a) = z.anchor.and_then(|a| colony.get(a)) else { continue };
        if !a.built {
            continue;
        }
        let d = &defs.structure[a.def];
        let cap = (d.cells_at(a.tier).max(0) as usize).min(z.cells.len()) as u16;
        let (mut on, mut off) = (Vec::new(), Vec::new());
        for &c in &z.cells {
            if (c as usize) < mat.hy.len() && (mat.hy[c as usize] == mat::sim::hy::CORD || mat.hy[c as usize] == mat::sim::hy::NODE) {
                on.push(c);
            } else {
                off.push(c);
            }
        }
        let (w, h) = a.dims(d);
        let footprint = w * h;
        let name = format!("{} {}", d.name.to_uppercase(), a.id);
        match stash.rooms.iter_mut().find(|r| r.zone == z.id) {
            Some(r) => {
                r.cap = cap;
                r.priority = z.priority;
                r.categories = z.filter.categories.clone();
                r.min_tier = z.filter.min_tier;
                r.min_condition = z.filter.min_condition;
                r.footprint = footprint;
                r.name = name;
                r.reshape(on, off);
            }
            None => stash.rooms.push(Room {
                zone: z.id,
                anchor: a.id,
                name,
                cap,
                priority: z.priority,
                cold: d.id == "cold_store",
                categories: z.filter.categories.clone(),
                min_tier: z.filter.min_tier,
                min_condition: z.filter.min_condition,
                mat_cells: on,
                off_cells: off,
                footprint,
                items: Vec::new(),
            }),
        }
    }
}

/// The Stash settles itself at shift end (Stockpiles): a room at 4–3 draws what it accepts out of the Heart's grid
/// and out of rooms below it; one at 2–1 takes what the grid cannot hold (what waits at the Heart), 2 before 1.
/// Each move is a shipment to the room from where the item lies. Returns how many moved.
#[allow(clippy::too_many_arguments)]
pub fn settle(city: &mut City, colony: &Colony, defs: &StructureFile, stash: &mut Stash, ledger: &mut Ledger, table: &ItemTable) -> usize {
    let mut order: Vec<(u8, u32)> = stash.rooms.iter().map(|r| (r.priority, r.zone)).collect();
    // the highest priority first, ties by zone
    order.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut moved = 0;
    for (prio, zone) in order {
        let riding = ledger.in_flight.iter().filter(|s| s.to == Dest::Room(zone)).count();
        let Some(room) = stash.rooms.iter().find(|r| r.zone == zone).cloned() else { continue };
        let free = room.room_for().saturating_sub(riding);
        if free == 0 {
            continue;
        }
        if (prio as f32) > GRID_PRIORITY {
            // draw: from the grid and from rooms with a lower priority, unpinned
            let lower: Vec<u32> = stash.rooms.iter().filter(|r| r.priority < prio).map(|r| r.zone).collect();
            let take: Vec<(u32, Option<u32>)> = stash
                .handles()
                .into_iter()
                .filter(|(_, from, it)| !it.pinned && from.is_none_or(|z| lower.contains(&z)) && room.accepts(table.def(it.def), it))
                .map(|(h, from, _)| (h, from))
                .take(free)
                .collect();
            for (h, from) in take {
                if let Some(it) = stash.take(h) {
                    let id = ledger.ship(it, Dest::Room(zone));
                    cargo::send(city, colony, defs, stash, id, Dest::Room(zone), from);
                    moved += 1;
                }
            }
        } else {
            // overflow: what lies on the Heart's floor for want of room in the grid, oldest first
            let take: Vec<u32> = stash.floor.iter().filter(|s| !s.item.pinned && room.accepts(table.def(s.item.def), &s.item)).map(|s| FLOOR + s.id).take(free).collect();
            for h in take {
                if let Some(it) = stash.take(h) {
                    let id = ledger.ship(it, Dest::Room(zone));
                    cargo::send(city, colony, defs, stash, id, Dest::Room(zone), None);
                    moved += 1;
                }
            }
        }
    }
    moved
}

/// The haulers' shift: each shelves up to its Body in items from the store's footprint onto the zone's cells off the
/// Mat. Returns (slimo, items) for the Scav XP.
pub fn haul(stash: &mut Stash, haulers: &[(u32, u32, i32)]) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for &(slimo, sid, body) in haulers {
        if let Some(r) = stash.rooms.iter_mut().find(|r| r.anchor == sid) {
            let n = r.haul(body.max(0) as usize);
            if n > 0 {
                out.push((slimo, n as u32));
            }
        }
    }
    out
}

/// What the Stash's side of the shift close gives back.
#[derive(Clone, Debug, Default)]
pub struct Closed {
    pub report: ShiftReport,
    pub crafted: Vec<String>,
    pub placed: u32,
    pub waiting: usize,
}

/// Shift end on the Stash's side, after the cargo left: what the Bench made in the shift's minutes and what waited for
/// room come into the grid (or wait on), `bench_slots` jobs lose a shift, and the ledger closes the shift, each
/// market's glut recovering faster by `faster` (its Seep Market's bonus).
pub fn close(stash: &mut Stash, ledger: &mut Ledger, bench: &mut BenchQueue, table: &ItemTable, recipes: &RecipeFile, bench_slots: usize, faster: &[f32]) -> Closed {
    let products = bench.end_shift_slots(table, recipes, bench_slots);
    // what the minutes made and placed is the shift's too (Full Design v3.21, The living city)
    let mut c = Closed { crafted: std::mem::take(&mut bench.made), ..Default::default() };
    c.placed = c.crafted.len() as u32;
    for p in products {
        c.crafted.push(table.def(p.def).name.clone());
        match stash.grid.auto_place(table.def(p.def), p.clone()) {
            Ok(_) => c.placed += 1,
            Err(_) => bench.waiting.push(p),
        }
    }
    c.report = ledger.close_shift_by(stash, table, faster);
    c.waiting = bench.waiting.len();
    c
}

/// The run's rig into the Stash before the shift ends (Full Design: the rig is handed to the Stash whole on
/// extraction): what the grid cannot hold lies on the Heart's floor. Returns (into the grid, onto the floor).
pub fn take_haul(stash: &mut Stash, table: &ItemTable, items: Vec<ItemInstance>) -> (usize, usize) {
    let (mut placed, mut floor) = (0, 0);
    for it in items {
        if stash.place_or_floor(table, it) {
            placed += 1;
        } else {
            floor += 1;
        }
    }
    (placed, floor)
}
