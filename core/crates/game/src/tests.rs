//! The city's shift in core (Full Design v3.12): stockpiles, haulers, the steering tools, orders, alerts, the log.

use colony::{Filter, ZoneKind};
use inventory::{Dest, SHELF};
use items::{ItemInstance, ItemTable, ITEMS_TOML};
use mat::sim::hy;

use crate::{Level, World};

fn world() -> World {
    World::found(11, ItemTable::from_toml(ITEMS_TOML).unwrap())
}

/// A built structure of a def at the nearest anchor to the Heart that takes it.
fn stand(w: &mut World, def: &str) -> u32 {
    let (hx, hy_) = w.city.sim.heart.unwrap();
    let mut spots: Vec<(u32, u16, u16)> = Vec::new();
    for y in 0..w.city.sim.h as u16 {
        for x in 0..w.city.sim.w as u16 {
            if w.colony.can_place(&w.defs, &w.city.sim, def, x, y).is_ok() {
                spots.push(((x as i32 - hx as i32).unsigned_abs() + (y as i32 - hy_ as i32).unsigned_abs(), x, y));
            }
        }
    }
    spots.sort_unstable();
    let (_, x, y) = spots[0];
    w.colony.place_built(&w.defs, &w.city.sim, def, x, y, 1).unwrap()
}

fn tick_n(w: &mut World, n: u32) {
    for _ in 0..n {
        w.tick();
    }
}

#[test]
fn a_rack_catches_the_overflow_and_a_cold_store_draws_the_food_out_of_the_heart() {
    let mut w = world();
    // a Rack and its stockpile around it: priority 3, it moves nothing the grid can hold
    let rack = stand(&mut w, "rack");
    let r = w.colony.get(rack).unwrap().clone();
    let cells: Vec<u32> = (-3i32..=4).flat_map(|dx| (-1i32..=1).map(move |dy| (dx, dy))).map(|(dx, dy)| ((r.y as i32 + dy) as u32) * w.city.sim.w as u32 + (r.x as i32 + dx) as u32).collect();
    let zone = w.colony.paint_stockpile(&w.defs, &w.city.sim, rack, &cells).unwrap();
    assert_eq!(w.colony.zones.iter().find(|z| z.id == zone).unwrap().priority, colony::RACK_PRIORITY);
    w.end_shift();
    tick_n(&mut w, 200);
    assert_eq!(w.stash.rooms.len(), 1, "the stockpile is a room of the Stash");
    assert!(w.stash.rooms[0].items.is_empty(), "an overflow store draws nothing the grid holds");
    assert!(w.ledger.in_flight.iter().all(|s| !matches!(s.to, Dest::Room(_))));
    // overflow: what waits at the Heart goes to the Rack and lands there
    let pipe = w.table.id_of("prt_pipe").unwrap();
    for _ in 0..3 {
        w.stash.to_floor(ItemInstance::new(pipe));
    }
    w.end_shift();
    tick_n(&mut w, 400);
    let room = &w.stash.rooms[0];
    assert_eq!((room.items.len(), w.stash.floor.len()), (3, 0), "the overflow rode to the Rack");
    // a stockpile item is the Stash's: its handle answers, a Sell rides from the Rack
    let h = SHELF + room.items[0].id;
    assert!(w.stash.get_any(h).is_some() && w.stash.room_of(h) == Some(zone));
    assert!(w.ledger.queue(&mut w.stash, h, inventory::Verb::Sell { market: 0 }, w.day));
    assert_eq!(w.ledger.queue[0].from, Some(zone));
    // a Cold Store draws the Organics out of the Heart (priority 2, Organic filter)
    let cold = stand(&mut w, "cold_store");
    let c = w.colony.get(cold).unwrap().clone();
    let cells: Vec<u32> = (0..2u32).flat_map(|dy| (0..2u32).map(move |dx| (dx, dy))).map(|(dx, dy)| (c.y as u32 + dy) * w.city.sim.w as u32 + c.x as u32 + dx).collect();
    let cz = w.colony.paint_stockpile(&w.defs, &w.city.sim, cold, &cells).unwrap();
    let organics = |w: &World| w.stash.grid.items().filter(|i| w.table.def(i.def).category == items::Category::Organic).count();
    let in_grid = organics(&w);
    assert!(in_grid > 0);
    w.end_shift();
    tick_n(&mut w, 400);
    let cold_room = w.stash.rooms.iter().find(|r| r.zone == cz).unwrap();
    assert!(cold_room.cold && cold_room.items.len() >= 1, "the Cold Store drew Organics: {}", cold_room.items.len());
    assert!(organics(&w) < in_grid);
    // in the cold they keep; in the grid they spoil
    let kept: Vec<u8> = cold_room.items.iter().map(|s| s.item.condition).collect();
    w.end_shift();
    let after: Vec<u8> = w.stash.rooms.iter().find(|r| r.zone == cz).unwrap().items.iter().take(kept.len()).map(|s| s.item.condition).collect();
    assert_eq!(kept, after, "a Cold Store keeps its Organics");
}

#[test]
fn haulers_shelve_off_the_mat_what_waits_on_the_store() {
    let mut w = world();
    let rack = stand(&mut w, "rack");
    let r = w.colony.get(rack).unwrap().clone();
    // a zone of floor cells beside the Rack that are not Mat
    let mut cells = Vec::new();
    for dy in -3i32..=3 {
        for dx in -4i32..=5 {
            let (x, y) = (r.x as i32 + dx, r.y as i32 + dy);
            let i = y as usize * w.city.sim.w + x as usize;
            if !mat::sim::is_hyphal(w.city.sim.hy[i]) && colony::Colony::zone_cell_ok(&w.city.sim, i) {
                cells.push(i as u32);
            }
        }
    }
    let zone = w.colony.paint_stockpile(&w.defs, &w.city.sim, rack, &cells).unwrap();
    w.end_shift();
    let room = w.stash.rooms.iter().find(|r| r.zone == zone).unwrap();
    assert!(room.mat_cells.is_empty() && !room.off_cells.is_empty(), "the zone is off the Mat");
    // land four: the Rack's footprint holds two, the rest has no room until a hauler shelves them
    let pipe = w.table.id_of("prt_pipe").unwrap();
    let landed = (0..4).filter(|_| w.stash.shelve(zone, ItemInstance::new(pipe)).is_ok()).count();
    assert_eq!(landed, 2, "two wait on a Rack's footprint");
    let moved = crate::store::haul(&mut w.stash, &[(101, rack, 5)]);
    assert_eq!(moved, vec![(101, 2)]);
    let room = w.stash.rooms.iter().find(|r| r.zone == zone).unwrap();
    assert!(room.items.iter().all(|s| s.cell.is_some()), "shelved onto the zone");
    assert_eq!(room.room_for(), 2, "the footprint is free again");
}

#[test]
fn salt_takes_a_bag_for_eight_and_a_cut_is_build_work_and_a_relay_raised_is_a_nerve() {
    let mut w = world();
    let (hx, hy_) = w.city.sim.heart.unwrap();
    // salt: the starting haul has one bag; eight tiles from it, the ninth needs another
    let mut laid = 0;
    let mut refused = None;
    for x in 0..12u16 {
        let (st, t) = (&mut w.stash, &w.table);
        match w.colony.lay_salt(&mut w.city.sim, 10 + x, 2, &mut || crate::store::take_salt_bag(st, t)) {
            Ok(_) => laid += 1,
            Err(e) => {
                refused = Some(e);
                break;
            }
        }
    }
    assert_eq!(laid, 8);
    assert!(refused.unwrap().contains("road salt"));
    assert!(w.city.sim.salt[2 * w.city.sim.w + 10]);
    // a cut: marked now, done at shift end by Build at 8 points
    let mut cut = None;
    'find: for y in 0..w.city.sim.h {
        for x in 0..w.city.sim.w {
            let i = y * w.city.sim.w + x;
            let by_heart = x.abs_diff(hx) <= 2 && y.abs_diff(hy_) <= 2;
            if !by_heart && w.city.sim.hy[i] == hy::CORD && w.colony.occupied(&w.defs, x as u16, y as u16).is_none() {
                cut = Some((x as u16, y as u16));
                break 'find;
            }
        }
    }
    let (cx, cy) = cut.unwrap();
    assert!(w.colony.mark_cut(&w.city.sim, cx, cy).is_ok());
    assert!(w.colony.mark_cut(&w.city.sim, hx as u16, hy_ as u16).is_err(), "never the Heart");
    let mut shifts = 0;
    while !w.colony.cuts.is_empty() && shifts < 4 {
        w.end_shift();
        shifts += 1;
    }
    assert!(w.colony.cuts.is_empty(), "nobody cut it in four shifts");
    assert!(w.city.sim.hy[cy as usize * w.city.sim.w + cx as usize] != hy::CORD, "the cord is severed");
    // a Relay: a structure on signalled Mat, built by Build; standing, it is a Mat relay
    let relays = w.city.sim.relays.len();
    let mut placed = None;
    'n: for y in 0..w.city.sim.h as u16 {
        for x in 0..w.city.sim.w as u16 {
            if w.colony.can_place(&w.defs, &w.city.sim, "relay", x, y).is_ok() {
                placed = w.colony.place(&w.defs, &w.city.sim, "relay", x, y).ok();
                break 'n;
            }
        }
    }
    let id = placed.expect("signalled Mat takes a Relay");
    let chrome = w.table.id_of("chr_street_eye").unwrap();
    for _ in 0..6 {
        let _ = w.stash.grid.auto_place(w.table.def(chrome), ItemInstance::new(chrome));
    }
    for _ in 0..6 {
        if w.colony.get(id).unwrap().built {
            break;
        }
        w.end_shift();
    }
    assert!(w.colony.get(id).unwrap().built, "the Relay stands");
    assert_eq!(w.city.sim.relays.len(), relays + 1, "and it is a nerve");
}

#[test]
fn an_order_puts_a_slimo_at_a_structure_for_the_next_shift_and_is_spent() {
    let mut w = world();
    let bench = w.colony.structures.iter().find(|s| w.defs.structure[s.def].id == "bench").unwrap().id;
    let who = w.roster.records.iter().map(|r| r.id).max().unwrap();
    // the slimo's own Tweak priority is off: the order stands over it
    w.roster.get_mut(who).unwrap().priorities = [0; 9];
    w.roster.get_mut(who).unwrap().schedule = [people::Duty::Work; 3];
    w.colony.order(&w.defs, who, bench).unwrap();
    let s = w.end_shift();
    assert!(s.outcome.assignments.iter().any(|(sl, _, sid)| *sl == who && *sid == bench), "the order was worked");
    assert!(w.colony.orders.is_empty(), "spent with the shift");
    assert!(w.colony.order(&w.defs, who, 999_999).is_err());
}

#[test]
fn alerts_name_a_waiting_ghost_a_petition_and_cargo_held_at_the_heart() {
    let mut w = world();
    // a ghost the Stash cannot pay for
    let mut ghost = None;
    'g: for y in 0..w.city.sim.h as u16 {
        for x in 0..w.city.sim.w as u16 {
            if w.colony.can_place(&w.defs, &w.city.sim, "bunkhouse", x, y).is_ok() {
                ghost = w.colony.place(&w.defs, &w.city.sim, "bunkhouse", x, y).ok();
                break 'g;
            }
        }
    }
    let ghost = ghost.unwrap();
    w.stash = inventory::Stash::new();
    w.end_shift();
    let pipe = w.table.id_of("prt_pipe").unwrap();
    w.stash.to_floor(ItemInstance::new(pipe));
    let a = w.alerts();
    assert!(a.iter().any(|x| x.level == Level::Yellow && x.key == format!("waiting {ghost}")), "{a:?}");
    assert!(a.iter().any(|x| x.key == "floor"));
    // red sorts before yellow
    assert!(a.windows(2).all(|p| p[0].level <= p[1].level));
}

#[test]
fn a_delivery_counts_at_its_grate_a_relic_rides_home_and_blackline_sold_is_loud() {
    let mut w = world();
    let nomads = w.economy.file.index("nomads").unwrap();
    assert_eq!(w.economy.faction_at(w.city.markets[0]), Some(nomads), "the founded grate is the Nomads' street");
    // a deliver link of two Parts in hand; a Relic, which no stall buys, sent to sell at the same market
    let t = w.economy.file.factions[nomads as usize].contracts.iter().position(|c| c.kind == "deliver" && c.category == "part").unwrap() as u16;
    let mut c = w.economy.first_link(&economy::Offer { id: 99, faction: nomads, template: t, links: 3, until: 9, name: "N".into() }, 1);
    (c.ask.n, c.ask.condition) = (2, 0);
    let id = c.id;
    w.economy.s.contracts.push(c);
    let part = w.table.id_of("prt_pipe").unwrap();
    for _ in 0..2 {
        let h = w.stash.grid.auto_place(w.table.def(part), ItemInstance::new(part)).unwrap();
        w.ledger.queue(&mut w.stash, h, inventory::Verb::Deliver { market: 0, contract: id }, 1);
    }
    let relic = (0..w.table.len()).find(|&k| w.table.defs[k].tier == items::Tier::Relic).unwrap();
    let h = w.stash.grid.auto_place(&w.table.defs[relic], ItemInstance::new(items::ItemId(relic as u16))).unwrap();
    w.ledger.queue(&mut w.stash, h, inventory::Verb::Sell { market: 0 }, 1);
    let zone = w.threats.zones.iter().position(|z| z.grate == w.city.markets[0]).unwrap();
    w.end_shift();
    tick_n(&mut w, 400);
    let got = w.economy.s.contracts.iter().find(|c| c.id == id).map(|c| c.got);
    assert_eq!(got, Some(2), "{:?}", w.log.events.iter().map(|e| e.what.clone()).collect::<Vec<_>>());
    assert!(w.log.events.iter().any(|e| e.what.starts_with("no stall at grate") && e.what.contains(&w.table.defs[relic].name)));
    assert!(w.stash.handles().iter().any(|(_, _, it)| it.def.0 as usize == relic), "the Relic came home");
    // Blackline sold at a grate counts five Attention there at the shift's end (only the Corpo remnants fence it)
    let landed = crate::cargo::Landed { blackline: vec![0], ..Default::default() };
    let before = w.threats.zones[zone].attention;
    crate::pressure::tick(&mut w.threats, &mut w.city, &mut w.colony, &w.defs, &w.posts, &mut w.log, &landed);
    w.end_shift();
    assert!(w.threats.zones[zone].gained >= 5.0, "Attention at the grate {before} gained {}", w.threats.zones[zone].gained);
}

#[test]
fn protection_holds_off_the_patrol_due_at_a_gang_grate() {
    let mut w = world();
    let gangs = w.economy.file.index("gangs").unwrap();
    let zone = w.threats.zones.iter().position(|z| w.economy.faction_at(z.grate) == Some(gangs)).unwrap();
    w.threats.zones[zone].attention = 40.0;
    w.economy.s.protection = 1;
    w.end_shift();
    assert!(!w.threats.due.iter().any(|d| d.zone as usize == zone), "{:?}", w.threats.due);
    assert!(w.log.events.iter().any(|e| e.what == format!("the Gangs held off the Patrol at grate {}", zone + 1)));
    assert_eq!(w.economy.s.protection, 0);
    // none bought: the next one comes
    w.threats.zones[zone].attention = 40.0;
    w.end_shift();
    assert!(w.threats.due.iter().any(|d| d.zone as usize == zone));
}

#[test]
fn the_city_log_has_the_cargo_landing_and_the_structure_built() {
    let mut w = world();
    let heart = w.colony.pits(&w.defs)[0];
    // above the founding Pit's six Junk Chrome, so the Heart asks for a feed
    w.colony.set_quota(&w.defs, heart, Some(colony::QuotaSet { target: 2.0, ceiling: 4 }));
    for it in w.stash.handles().into_iter().filter(|(_, _, it)| w.table.def(it.def).category == items::Category::Curio).map(|(h, _, _)| h).collect::<Vec<_>>() {
        w.ledger.queue(&mut w.stash, it, inventory::Verb::Sell { market: 0 }, 1);
    }
    w.end_shift();
    tick_n(&mut w, 300);
    let landings: Vec<&str> = w.log.events.iter().filter(|e| e.class == crate::Class::Landing).map(|e| e.what.as_str()).collect();
    assert!(landings.iter().any(|l| l.contains("sold for")), "{landings:?}");
    assert!(landings.iter().any(|l| l.contains("fed to the Heart")), "{landings:?}");
    assert!(w.ledger.in_flight.iter().all(|s| s.to != Dest::Market(0)));
}

#[test]
fn a_stockpile_filter_and_priority_are_the_zone_s() {
    let mut w = world();
    let rack = stand(&mut w, "rack");
    let r = w.colony.get(rack).unwrap().clone();
    let cells: Vec<u32> = vec![r.y as u32 * w.city.sim.w as u32 + r.x as u32 + 2];
    let zone = w.colony.paint_stockpile(&w.defs, &w.city.sim, rack, &cells).unwrap_or_else(|_| w.colony.paint_zone(&w.city.sim, ZoneKind::Stockpile, &cells, Filter::default(), Some(rack)).unwrap());
    assert!(w.colony.set_zone_priority(zone, 1));
    assert!(w.colony.set_zone_filter(zone, Filter { categories: vec!["part".into()], min_tier: 0, min_condition: 50 }));
    w.end_shift();
    let room = w.stash.rooms.iter().find(|r| r.zone == zone).unwrap();
    assert_eq!((room.priority, room.categories.clone(), room.min_condition), (1, vec!["part".to_string()], 50));
}

#[test]
fn the_city_replays_bit_exact_and_its_cargo_lands_within_the_next_shift() {
    // two shifts of `rose city`, played twice: the twenty-shift proof is the CLI's (release), this is its replay
    let c = crate::city::replays(11, 2, &mut |_, _| {}).expect("the city replays bit-exact");
    assert_eq!(c.shifts.len(), 2);
    assert!(c.shifts.iter().all(|r| r.late == 0 && r.starving == 0), "{:?}", c.failures);
    assert!(c.log.contains("city LAND"), "the city landed nothing");
}
