//! The city's shift in core (Full Design v3.12): stockpiles, haulers, the steering tools, orders, alerts, the log.

use colony::{Filter, ZoneKind};
use inventory::{Dest, SHELF};
use items::{ItemInstance, ItemTable, ITEMS_TOML};
use mat::sim::hy;

use crate::{Level, World};

/// A player's shift: ten minutes of the city's clock (the desk work is the minute's, v3.21), then its end.
fn play_shift(w: &mut World) {
    for _ in 0..(colony::MINUTES_A_SHIFT as u64 * crate::shift::WORK_TICKS) {
        w.tick();
    }
    w.end_shift();
}

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
    assert!(w.log.events.iter().any(|e| e.what == format!("the Gangs held off the Patrol at grate {}", w.threats.grate_label(&w.city.sim, zone))));
    assert_eq!(w.economy.s.protection, 0);
    // none bought: the next one comes
    w.threats.zones[zone].attention = 40.0;
    w.end_shift();
    assert!(w.threats.due.iter().any(|d| d.zone as usize == zone));
}

fn order(w: &mut World, f: u8, service: &str) -> Result<u32, String> {
    let e = w.economy.service_entry(f, service).unwrap();
    let plan = w.plan.clone();
    crate::trade::order(&mut w.economy, &mut w.ledger, &w.table, &w.city, &w.colony, &w.defs, &w.roster, &plan, 0, f, e, 1)
}

#[test]
fn the_services_run_from_the_shift_after_they_are_paid_and_do_what_they_are_for() {
    let mut w = world();
    let ix = |w: &World, id: &str| w.economy.file.index(id).unwrap();
    let (nomads, corpo, badges) = (ix(&w, "nomads"), ix(&w, "corpo"), ix(&w, "badges"));
    w.economy.s.standing[nomads as usize].opened = 4;
    w.ledger.eddies = 2000;
    // the Nomads' convoy and a hire at their stall over grate 5; the Corpo remnants have no street in reach
    order(&mut w, nomads, "convoy").unwrap();
    order(&mut w, nomads, "hire").unwrap();
    assert!(order(&mut w, corpo, "intel").unwrap_err().contains("do not trade"));
    let waiting = w.roster.waiting.len();
    w.end_shift();
    // the convoy covers this shift's landings: every trading faction buys at the Nomads' stall, the others less 10 %
    assert_eq!(w.economy.s.convoy, vec![0]);
    let books = &w.ledger.books[0];
    assert_eq!(books.len(), 4, "the Nomads, the Gangs, the Corpo remnants and the Badges");
    let own = books.iter().find(|b| b.faction == nomads).unwrap().rep;
    assert!(books.iter().filter(|b| b.faction != nomads).all(|b| (b.rep - 0.9).abs() < 1e-4) && (own - 1.0).abs() < 1e-4);
    // the hire: a Nomad at a grate, their Past's skills two over the roll
    let hired = w.roster.waiting.iter().map(|(r, _)| r).find(|r| r.past == people::Past::Nomad && w.log.events.iter().any(|e| e.what.starts_with(&r.full_name()) && e.what.contains("hired through the Nomads")));
    assert!(hired.is_some() && w.roster.waiting.len() > waiting);
    w.end_shift();
    assert!(w.economy.s.convoy.is_empty() && w.ledger.books[0].len() == 1, "the convoy ran one shift");
    // Open Grates: the Corpo remnants' intel and the Badges' timings at the same stall
    let law = w.laws.get("open_grates").unwrap().clone();
    w.colony.charter.authority = 1;
    w.colony.sign(&w.defs, &w.city.sim, &law).unwrap();
    w.economy.s.standing[badges as usize].opened = 4;
    order(&mut w, corpo, "intel").unwrap();
    order(&mut w, badges, "timings").unwrap();
    w.end_shift();
    assert_eq!((w.economy.s.intel, w.economy.s.timings), (3, 3));
    // a zone a shift from the Sweep line: intel says so, and the timings say it of every line
    let zone = w.threats.zones.iter().position(|z| z.grate == w.city.markets[0]).unwrap();
    let sweep = w.threats.rules.stage.iter().find(|s| s.id == "sweep").unwrap().at;
    w.threats.zones[zone].attention = sweep - 4.0;
    w.threats.zones[zone].gained = 8.0;
    let alerts: Vec<String> = w.alerts().into_iter().map(|a| a.text).collect();
    assert!(alerts.iter().any(|a| a.starts_with("the Badges' timings: grate") && a.contains("Sweep")), "{alerts:?}");
    w.economy.s.timings = 0;
    let alerts: Vec<String> = w.alerts().into_iter().map(|a| a.text).collect();
    assert!(alerts.iter().any(|a| a.starts_with("Corpo intel: grate") && a.contains("Sweep")), "{alerts:?}");
    for _ in 0..3 {
        w.end_shift();
    }
    assert_eq!(w.economy.s.intel, 0, "intel ran three shifts");
}

#[test]
fn a_recover_body_is_laid_at_the_far_end_of_a_route_and_its_tag_counts_where_it_lands() {
    let mut w = world();
    let gangs = w.economy.file.index("gangs").unwrap();
    let t = w.economy.file.factions[gangs as usize].contracts.iter().position(|c| c.kind == "recover").unwrap() as u16;
    w.economy.s.offers.push(economy::Offer { id: 77, faction: gangs, template: t, links: 3, until: 9, name: "Hull".into() });
    let lib = run::ChunkLib::from_toml(run::CHUNKS_TOML).unwrap();
    let board = crate::trade::Board { lib: &lib, city_seed: 11, strata: 3 };
    // the Gangs' street is not in reach: under Open Grates the market nearest the Heart deals with them
    assert!(crate::trade::accept(&mut w.economy, (&w.city, &w.colony), &mut w.roster, &w.traits, &w.names, &w.table, Some(&board), 77, 1, &mut w.log).unwrap_err().contains("reaches no street"));
    let law = w.laws.get("open_grates").unwrap().clone();
    w.colony.charter.authority = 1;
    w.colony.sign(&w.defs, &w.city.sim, &law).unwrap();
    assert_eq!(crate::trade::market_of(&w.economy, &w.city, &w.colony, gangs), Some(0));
    let (line, laid) = crate::trade::accept(&mut w.economy, (&w.city, &w.colony), &mut w.roster, &w.traits, &w.names, &w.table, Some(&board), 77, 1, &mut w.log).unwrap();
    assert!(line.contains("recover Hull"), "{line}");
    assert_eq!(laid.len(), 1);
    let body = &laid[0];
    // the floor farthest from the Mat edge on the Board's route of its stratum, the tag in its pack, bound
    let map = run::RouteMap::compile(&lib, &run::route(&lib, run::route_seed(11, body.stratum), body.stratum).unwrap());
    assert_eq!(run::creature::far_floor(&map).map(|f| (f.0 .0 as u16, f.0 .1 as u16)), Some((body.corpse.x, body.corpse.y)));
    let tag = body.corpse.rig.backpack.placed().into_iter().map(|(_, p)| p.item.clone()).next().unwrap();
    assert_eq!((w.table.def(tag.def).id.as_str(), tag.bound), ("org_corpse_tag", Some(77)));
    assert_eq!(w.economy.s.contracts.iter().find(|c| c.id == 77).unwrap().route, Some((body.stratum, body.route)));
    // an unbound tag does not count; the body's does
    for it in [ItemInstance::new(tag.def), tag.clone()] {
        let h = w.stash.grid.auto_place(w.table.def(tag.def), it).unwrap();
        w.ledger.queue(&mut w.stash, h, inventory::Verb::Deliver { market: 0, contract: 77 }, 1);
    }
    w.end_shift();
    tick_n(&mut w, 400);
    assert_eq!(w.economy.s.contracts.iter().find(|c| c.id == 77).map(|c| (c.link, c.got)), Some((1, 1)));
    assert!(w.log.events.iter().any(|e| e.what.contains("not taken at grate")), "the unbound tag rode home");
    w.end_shift();
    assert!(w.log.events.iter().any(|e| e.what == "the Gangs' link 1/3 done: 50 e"));
}

#[test]
fn a_drowned_structure_loses_a_quarter_at_the_shift_end_and_the_builders_mend_it_the_next() {
    let mut w = world();
    for r in w.roster.records.iter_mut() {
        r.priorities[people::Job::Build.index()] = 4;
    }
    let bench = w.colony.structures.iter().find(|s| w.defs.structure[s.def].id == "bench").unwrap().id;
    w.colony.get_mut(bench).unwrap().drowned = true;
    w.end_shift();
    assert_eq!(w.colony.get(bench).unwrap().hp, 75, "the Flood took a quarter");
    // the next shift's plan puts builders on it, and its end mends 2 x Hands each
    let on: Vec<i32> = w.plan.iter().filter(|p| p.1 == people::Job::Build && p.2 == bench).map(|p| w.roster.get(p.0).unwrap().stat(people::Stat::Hands)).collect();
    assert_eq!(on.len(), 2);
    w.end_shift();
    assert_eq!(w.colony.get(bench).unwrap().hp, (75 + colony::MEND_PER_HANDS * on.iter().sum::<i32>()).min(100));
    assert!(w.log.events.iter().any(|e| e.what.starts_with("Tweaker Bench at") && e.what.contains("mended")));
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

#[test]
fn a_blueprint_studied_opens_its_tier_and_the_node_researched_lets_its_structure_stand() {
    let mut w = world();
    // one D3 researches; nobody else works
    let d3 = w.roster.records.iter().find(|r| r.depth == people::Depth::D3).unwrap().id;
    for r in w.roster.records.iter_mut() {
        r.priorities = [0; 9];
    }
    w.roster.get_mut(d3).unwrap().priorities[people::Job::Research.index()] = 4;
    let tweak = w.roster.get(d3).unwrap().skill(people::Skill::Tweak) as u32;
    assert!(tweak >= 2, "a D3 is born at Tweak 2");
    // the Gate waits on its node
    assert!(matches!(w.colony.can_place(&w.defs, &w.city.sim, "gate", 0, 0), Err(colony::PlaceError::Locked(_))));
    // a Blueprint I in the Stash: picked as the project, spent at once
    let bp = w.table.id_of("tec_blueprint_chrome1").unwrap();
    w.stash.place_or_floor(&w.table, ItemInstance::new(bp));
    crate::grow::pick_project(&mut w.colony, &mut w.stash, &mut w.ledger, &w.table, progress::Project::Study("tec_blueprint_chrome1".into())).unwrap();
    assert!(!w.stash.all_items().any(|it| it.def == bp), "the blueprint is spent");
    let mut shifts = 0;
    while w.colony.progress.tech.open[0] < 2 && shifts < 10 {
        play_shift(&mut w);
        shifts += 1;
    }
    assert_eq!(w.colony.progress.tech.open[0], 2, "Chrome tier 2 is open");
    assert_eq!(shifts as u32, 8u32.div_ceil(tweak), "8 points at the researcher's Tweak a shift");
    assert!(w.log.events.iter().any(|e| e.what == "Chrome tier 2 is open"));
    assert!(w.roster.get(d3).unwrap().xp[people::Skill::Tweak.index()] > 0 || w.roster.get(d3).unwrap().skill(people::Skill::Tweak) as u32 > tweak, "Research trains Tweak");
    // the node: its Parts and Chrome paid at the pick, then 24 points
    for _ in 0..6 {
        let p = w.table.id_of("prt_pipe").unwrap();
        w.stash.place_or_floor(&w.table, ItemInstance::new(p));
    }
    for _ in 0..2 {
        let c = w.table.id_of("chr_optic_slag").unwrap();
        w.stash.place_or_floor(&w.table, ItemInstance::new(c));
    }
    crate::grow::pick_project(&mut w.colony, &mut w.stash, &mut w.ledger, &w.table, progress::Project::Node("c2_gate".into())).unwrap();
    while !w.colony.progress.tech.has("c2_gate") && shifts < 40 {
        play_shift(&mut w);
        shifts += 1;
    }
    assert!(w.colony.progress.tech.has("c2_gate"));
    assert!(!matches!(w.colony.can_place(&w.defs, &w.city.sim, "gate", 0, 0), Err(colony::PlaceError::Locked(_))), "the Gate can be placed");
    assert_eq!(w.bench.tier_open, 1, "a tier-1 Bench caps the recipes at 1 though Chrome's tier 2 is open");
}

#[test]
fn a_gene_over_its_line_grows_its_organ_the_mat_stands_its_own_and_the_class_follows() {
    let mut w = world();
    // Lume and Thrift over the Garden's line: a Lume Bulb grows, a Filter Lung where water is near, the class holds
    w.city.sim.genome.0[mat::genome::LUME] = 0.7;
    w.city.sim.genome.0[mat::genome::THRIFT] = 0.7;
    w.city.sim.genome.0[mat::genome::BRANCHING] = 0.7;
    w.end_shift();
    let f = progress::StrainFile::shipped();
    let bulb = f.organ_index("bulb").unwrap();
    let knot = f.organ_index("knot").unwrap();
    assert_eq!(w.city.sim.organs_of(bulb).count(), 1, "one a shift");
    assert!(w.colony.progress.strain.holds("garden"));
    assert!(w.colony.progress.strain.has_card("bulb") && w.colony.progress.strain.has_card("garden"));
    // a Bulb lights its tiles without a housing
    let b = w.city.sim.organs_of(bulb).next().unwrap().i as usize;
    let lit = w.colony.lit_cells(&w.defs, &w.city.sim, true);
    assert!(lit[b]);
    // a Brood Knot stands as the Mat's own structure, two beds
    if let Some(k) = w.city.sim.organs_of(knot).next().map(|o| o.i as usize) {
        let (x, y) = ((k % w.city.sim.w) as u16, (k / w.city.sim.w) as u16);
        let sid = w.colony.occupied(&w.defs, x, y).expect("the Knot stands on its organ");
        assert_eq!(w.defs.structure[w.colony.get(sid).unwrap().def].id, "knot");
        assert!(w.colony.can_place(&w.defs, &w.city.sim, "knot", x, y).is_err(), "the Mat's own is never placed");
    }
    // the cap: 1 + Nodes / 30 of a kind, one more a shift until it is met
    let cap = 1 + w.city.sim.node_count() / f.organ_per_nodes;
    for _ in 0..cap + 2 {
        w.end_shift();
    }
    assert!(w.city.sim.organs_of(bulb).count() <= 1 + w.city.sim.node_count() / f.organ_per_nodes);
    // a Lume Farm stands only on a Bulb, once its node stands
    w.colony.progress.tech = progress::Tech::every_structure();
    let b = w.city.sim.organs_of(bulb).next().unwrap().i as usize;
    let (bx, by) = ((b % w.city.sim.w) as u16, (b / w.city.sim.w) as u16);
    let off = (0..w.city.sim.hy.len()).find(|&i| w.city.sim.hy[i] == hy::NODE && w.city.sim.organ_at(i).is_none()).unwrap();
    assert!(matches!(w.colony.can_place(&w.defs, &w.city.sim, "lume_farm", (off % w.city.sim.w) as u16, (off / w.city.sim.w) as u16), Err(colony::PlaceError::NeedsOrgan(_)) | Err(_)));
    let on = w.colony.can_place(&w.defs, &w.city.sim, "lume_farm", bx, by);
    assert!(!matches!(on, Err(colony::PlaceError::NeedsOrgan(_))), "{on:?}");
    // the genes fall back: the class goes, the organs stay
    w.city.sim.genome.0[mat::genome::LUME] = 0.5;
    w.end_shift();
    assert!(!w.colony.progress.strain.holds("garden"));
    assert!(w.colony.progress.strain.has_card("class_lost"));
    assert!(w.city.sim.organs_of(bulb).count() >= 1);
}

#[test]
fn a_feed_past_the_ceiling_reflects_and_a_repelled_patrol_pays_authority() {
    let mut w = world();
    w.city.sim.genome.0[mat::genome::LUME] = 0.95;
    let mut d = [0.0f32; 10];
    d[mat::genome::LUME] = 0.13;
    let bounced = w.city.sim.drift_genome(d);
    assert_eq!(bounced, vec![(mat::genome::LUME, true)]);
    assert!((w.city.sim.genome.0[mat::genome::LUME] - 0.92).abs() < 1e-5, "1.08 lands at 0.92");
    // the deeds: a Patrol repelled +1, a Sweep +2, a Cleansing +3, and the trophies the threats' shift hands over
    let before = w.colony.charter.authority;
    let ended = threat::Ended { repelled: vec!["patrol".into(), "cleansing".into()], killed: vec!["flamer".into()], lived: vec!["flood".into()], ..Default::default() };
    crate::pressure::deeds(&mut w.colony, &ended, 3, 0, &mut w.log);
    assert_eq!(w.colony.charter.authority, before + 4);
    assert!(w.colony.progress.collections.has_trophy("kill_flamer") && w.colony.progress.collections.has_trophy("lived_flood"));
}

#[test]
fn a_shrines_nerve_rules_read_at_its_relay_flip_a_quota_and_sever_once() {
    let mut w = world();
    w.colony.progress.tech = progress::Tech::all();
    // a Relay raised by the founding: a Shrine on it
    let (rx, ry) = w.city.sim.relays.first().copied().unwrap_or_else(|| {
        let (hx, hy_) = w.city.sim.heart.unwrap();
        (hx + 2, hy_)
    });
    if !w.city.sim.relays.contains(&(rx, ry)) {
        w.city.sim.place_relay(rx, ry);
    }
    w.city.sim.compute_signal();
    // the founding's Pit on the Relay goes, so the Shrine can stand
    if let Some(p) = w.colony.occupied(&w.defs, rx as u16, ry as u16) {
        w.colony.ruin(&w.defs, p);
    }
    let shrine = w.colony.place_built(&w.defs, &w.city.sim, "shrine", rx as u16, ry as u16, 1).unwrap();
    // before Nerve logic stands the card refuses
    let mut t = progress::Tech::every_structure();
    std::mem::swap(&mut t, &mut w.colony.progress.tech);
    let rule = colony::NerveRule { reading: colony::Reading::Signal, above: true, value: 0.0, action: colony::NerveAction::FlipQuota, held: false };
    assert!(w.colony.set_rule(&w.defs, shrine, 0, Some(rule)).is_err());
    std::mem::swap(&mut t, &mut w.colony.progress.tech);
    w.colony.set_rule(&w.defs, shrine, 0, Some(rule)).unwrap();
    w.colony.set_rule(&w.defs, shrine, 1, Some(colony::NerveRule { action: colony::NerveAction::Sever, ..rule })).unwrap();
    let heart = w.colony.pits(&w.defs)[0];
    w.colony.set_quota(&w.defs, heart, Some(colony::QuotaSet { target: 2.0, ceiling: 2 }));
    let cuts = w.colony.cuts.len();
    crate::nerve::beat(&w.city, &mut w.colony, &w.defs, Some(&w.threats), &mut w.stash, &mut w.ledger, &w.table, &mut w.log);
    assert!(w.colony.rules_of(shrine).iter().all(|r| r.held), "signal over 0 at the Relay");
    let near = w.colony.get(heart).map(|h| (h.x as i32 - rx as i32).abs().max((h.y as i32 - ry as i32).abs()) <= colony::NERVE_REACH).unwrap();
    assert_eq!(w.colony.quota_of(&w.defs, heart).unwrap().0 == 0.0, near, "a Pit within 12 pulls nothing while it holds");
    assert_eq!(w.colony.cuts.len(), cuts + 1, "sever here marks the outward cord");
    // the second beat: still held, no second cut
    crate::nerve::beat(&w.city, &mut w.colony, &w.defs, Some(&w.threats), &mut w.stash, &mut w.ledger, &w.table, &mut w.log);
    assert_eq!(w.colony.cuts.len(), cuts + 1, "the once-actions fire on the rising edge");
    // cleared, the quota is the player's again
    w.colony.set_rule(&w.defs, shrine, 0, None).unwrap();
    crate::nerve::beat(&w.city, &mut w.colony, &w.defs, Some(&w.threats), &mut w.stash, &mut w.ledger, &w.table, &mut w.log);
    assert_eq!(w.colony.quota_of(&w.defs, heart).unwrap().0, 2.0);
}

#[test]
fn a_route_come_home_from_twice_loops_with_a_slimo_every_shift_and_decays() {
    let mut w = world();
    let slimo = w.roster.eligible()[0].id;
    // not before Route automation, nor before two runs home
    assert!(w.colony.set_loop(1, Some(slimo)).is_err());
    w.colony.progress.tech = progress::Tech::all();
    assert_eq!(w.colony.set_loop(1, Some(slimo)), Err("come home from this route 2 times first".into()));
    w.colony.home_from(1);
    w.colony.home_from(1);
    w.colony.set_loop(1, Some(slimo)).unwrap();
    let before = w.stash.all_items().count();
    w.end_shift();
    let looped = w.log.events.iter().any(|e| e.what.contains("loop on the S1 route came home") || e.what.contains("died on the S1 route's loop"));
    assert!(looped, "the loop ran at the shift's end");
    let alive = w.roster.get(slimo).is_some();
    if alive {
        assert_eq!(w.colony.loops[0].repeats, 1);
        assert!(w.stash.all_items().count() + w.stash.floor.len() >= before.saturating_sub(5), "the kit went out and the haul came home");
        // a run by hand on the route starts the decay over
        w.colony.home_from(1);
        assert_eq!(w.colony.loops[0].repeats, 0);
    } else {
        assert!(w.colony.loops.is_empty(), "a loop whose slimo died ends");
    }
}

#[test]
fn a_bred_strain_proposes_a_bypass_round_a_standing_jam_and_lays_it_when_confirmed() {
    let mut w = world();
    let (sw, sh) = (w.city.sim.w, w.city.sim.h);
    // a jam on one tile of a strand: try strands until one has open ground round it
    let mut found = None;
    for c in sw..sw * (sh - 1) {
        let h = |i: usize| hy::CORD == w.city.sim.hy[i] || hy::NODE == w.city.sim.hy[i];
        if !h(c) || w.city.sim.sig[c] <= 0.0 || c % sw == 0 || c % sw + 1 == sw {
            continue;
        }
        if !(h(c - 1) && h(c + 1)) {
            continue;
        }
        w.city.jam.iter_mut().for_each(|j| *j = false);
        w.city.jam[c] = true;
        if crate::handoff::bypass(&w.city).is_some() {
            found = Some(c);
            break;
        }
    }
    let c = found.expect("a strand with ground round it");
    // not bred: nothing proposed
    crate::handoff::tick(&mut w.city, &mut w.colony, Some(0), 1, &mut w.log);
    assert!(w.colony.proposal.is_empty());
    w.city.sim.genome.0[mat::genome::PLASTICITY] = 0.75;
    w.city.sim.genome.0[mat::genome::CONDUCTANCE] = 0.75;
    // a jam that stood less than a minute: nothing yet
    let t = w.city.tick;
    crate::handoff::tick(&mut w.city, &mut w.colony, Some(t), 1, &mut w.log);
    assert!(w.colony.proposal.is_empty());
    w.city.tick += crate::alerts::JAM_ALERT_TICKS;
    crate::handoff::tick(&mut w.city, &mut w.colony, Some(t), 1, &mut w.log);
    let path = w.colony.proposal.clone();
    assert!(!path.is_empty() && path.len() <= crate::handoff::MOST_TILES);
    assert!(!path.contains(&(c as u32)), "it keeps off the jam");
    assert!(w.colony.progress.strain.has_card("handoff"));
    assert!(w.alerts().iter().any(|a| a.key == "handoff"));
    // confirmed: a tile a Mat step, paid from the Heart
    crate::handoff::answer(&mut w.colony, true);
    let (hx, hy_) = w.city.sim.heart.unwrap();
    for _ in 0..path.len() * 2 + 2 {
        let hi = w.city.sim.idx(hx, hy_);
        w.city.sim.n[hi] = 1.0;
        w.city.tick += w.city.mat_every.max(1) - w.city.tick % w.city.mat_every.max(1);
        crate::handoff::tick(&mut w.city, &mut w.colony, None, 1, &mut w.log);
    }
    assert!(w.colony.proposal.is_empty() && !w.colony.proposal_confirmed);
    assert!(path.iter().all(|&i| w.city.sim.hy[i as usize] != hy::EMPTY), "the bypass is laid");
    // severed, a proposal is dropped
    w.colony.proposal = vec![1, 2];
    crate::handoff::answer(&mut w.colony, false);
    assert!(w.colony.proposal.is_empty());
}
