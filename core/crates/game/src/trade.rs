//! The economy in the shift (Full Design v3.15, Economy): what the World and the game's bridge both call at a shift's
//! start, as deliveries land and at a shift's end, so the surface trades the same whoever holds the city. The rules
//! are the `economy` crate's; this is where they meet the Stash, the cords, the roster and the threats.

use colony::{Colony, StructureFile};
use economy::{Bought, Economy, Kind, Note, ShiftFacts};
use inventory::{Dest, Ledger, Stash};
use items::{ItemInstance, ItemTable};
use logistics::City;
use people::{Job, NameFile, Past, Roster, TraitFile};
use threat::Threats;

use crate::log::{CityLog, Class};

/// Chapters are Phase E: the city is in the first.
pub const CHAPTER: u8 = 1;

/// The surface's day of a shift (three shifts a day).
pub fn day_of(shift: u32) -> u32 {
    Economy::day_of(shift)
}

/// Whether raiders walk the routes of a Board open to `strata` strata (data/creatures.toml's raid chance by stratum).
pub fn raiders_walk(strata: u8) -> bool {
    let chance = run::CreatureFile::shipped().raid.chance;
    (1..=strata.max(1)).any(|s| chance[(s as usize - 1).min(chance.len() - 1)] > 0.0)
}

/// Lines from the economy into the city's log.
pub fn note(log: &mut CityLog, tick: u64, notes: Vec<Note>) {
    for n in notes {
        match n {
            Note::Gain(s) => log.push(tick, Class::Landing, s),
            Note::Loss(s) => log.push(tick, Class::Loss, s),
            Note::Event(s) => log.push(tick, Class::Decision, format!("topside: {s}")),
        }
    }
}

/// The Seep Market at each market: a built one whose footprint stands within two columns of the market's grate.
pub fn seep_markets(city: &City, colony: &Colony, defs: &StructureFile) -> Vec<Option<u32>> {
    city.markets
        .iter()
        .map(|&g| {
            colony
                .structures
                .iter()
                .filter(|s| s.built && !s.dark && defs.structure[s.def].id == "market")
                .filter(|s| s.tiles(&defs.structure[s.def]).iter().any(|&(x, _)| (x as usize).abs_diff(g) <= 2))
                .map(|s| s.id)
                .next()
        })
        .collect()
}

/// Each market's stall this shift: the best Trade among the Haul slimos the plan puts at its Seep Market, and their
/// Pasts.
pub fn stalls(city: &City, colony: &Colony, defs: &StructureFile, roster: &Roster, plan: &[(u32, Job, u32)]) -> Vec<(u8, Vec<String>)> {
    seep_markets(city, colony, defs)
        .into_iter()
        .map(|seep| {
            let Some(sid) = seep else { return (0, Vec::new()) };
            let at: Vec<&people::Record> = plan.iter().filter(|(_, j, s)| *j == Job::Haul && *s == sid).filter_map(|(id, _, _)| roster.get(*id)).collect();
            (at.iter().map(|r| r.skill(people::Skill::Trade)).max().unwrap_or(0), at.iter().map(|r| r.past.name().to_string()).collect())
        })
        .collect()
}

/// How much faster each market's glut recovers: its Seep Market's +5 / 10 / 15 % a tier.
pub fn faster(city: &City, colony: &Colony, defs: &StructureFile) -> Vec<f32> {
    seep_markets(city, colony, defs).into_iter().map(|seep| seep.and_then(|sid| colony.get(sid)).map_or(0.0, |st| defs.structure[st.def].value_at("glut_bonus", st.tier) as f32 / 100.0)).collect()
}

/// Open Grates (the Charter): every market trades with every faction.
pub fn open_grates(colony: &Colony) -> bool {
    colony.charter.flag_at("any_faction_plus_1", 0)
}

/// The books the ledger prices by, as the day's demand, the standing and the staff have them now.
pub fn set_books(econ: &Economy, ledger: &mut Ledger, city: &City, colony: &Colony, defs: &StructureFile, roster: &Roster, plan: &[(u32, Job, u32)]) {
    let staff = stalls(city, colony, defs, roster, plan);
    ledger.books = econ.books(&city.markets, &staff, open_grates(colony), CHAPTER);
}

/// The grates the Mat reaches from the Heart (columns).
pub fn reached(city: &City) -> Vec<u16> {
    (0..city.markets.len()).filter(|&k| city.from_heart(logistics::Kind::Market, k as u32).is_some()).map(|k| city.markets[k] as u16).collect()
}

fn facts(city: &City, threats: Option<&Threats>, roster: &Roster, raiders: bool) -> ShiftFacts {
    let bloom = (0..city.sim.hy.len()).any(|i| city.sim.is_bloom(i));
    let attention = threats.map(|t| t.zones.iter().map(|z| (z.grate as u16, z.attention)).collect()).unwrap_or_default();
    let mut alive: Vec<u32> = roster.records.iter().map(|r| r.id).collect();
    let here = alive.clone();
    alive.extend(roster.waiting.iter().map(|(r, _)| r.id));
    ShiftFacts { sold: Vec::new(), bloom, attention, hosts_here: here, hosts_alive: alive, reached: reached(city), raiders, road_favour: false }
}

/// A shift begins: on the first shift of a day the surface's clock moves (demand walks, events roll, the stock
/// rotates), the factions in reach refill their offers, and the ledger takes the books.
#[allow(clippy::too_many_arguments)]
pub fn start_shift(econ: &mut Economy, ledger: &mut Ledger, city: &City, colony: &Colony, defs: &StructureFile, roster: &Roster, names: &NameFile, plan: &[(u32, Job, u32)], threats: Option<&Threats>, shift: u32, raiders: bool, log: &mut CityLog) {
    let day = Economy::day_of(shift);
    if day != econ.s.day {
        let notes = econ.begin_day(day, CHAPTER);
        note(log, city.tick, notes);
    }
    let f = facts(city, threats, roster, raiders);
    let first = names.first.clone();
    econ.refresh_offers(&f, CHAPTER, &mut |s| first[(s % first.len().max(1) as u64) as usize].clone());
    set_books(econ, ledger, city, colony, defs, roster, plan);
}

/// The Route Board a recover link's body is laid on: the chunks, the save's seed (its routes' seeds) and the strata it
/// offers.
pub struct Board<'a> {
    pub lib: &'a run::ChunkLib,
    pub city_seed: u64,
    pub strata: u8,
}

/// A recover link's body as laid: the Board's route of `stratum` (seed `route`), for the routes' corpses.
#[derive(Clone, Debug)]
pub struct Laid {
    pub contract: u32,
    pub stratum: u8,
    pub route: u64,
    pub name: String,
    pub corpse: run::Corpse,
}

/// Take an offer: a host's newcomer comes down a grate, a recover's body is laid on its route. Returns the log line
/// and the bodies laid.
#[allow(clippy::too_many_arguments)]
pub fn accept(econ: &mut Economy, town: (&City, &Colony), roster: &mut Roster, traits: &TraitFile, names: &NameFile, table: &ItemTable, board: Option<&Board>, offer: u32, shift: u32, log: &mut CityLog) -> Result<(String, Vec<Laid>), String> {
    let (city, colony) = town;
    let f = econ.s.offers.iter().find(|o| o.id == offer).map(|o| o.faction).ok_or("no such offer")?;
    if market_of(econ, city, colony, f).is_none() {
        return Err(format!("the Mat reaches no street of the {}", econ.name(f)));
    }
    let tick = city.tick;
    let c = econ.accept(offer, shift).ok_or(format!("the {}' stalls are closed to the city", econ.name(f)))?;
    hosts(econ, roster, traits, names, shift, tick, log);
    let laid = bodies(econ, table, board, shift, tick, log);
    let line = format!("took {}", econ.s.contracts.iter().find(|x| x.id == c.id).map(|x| x.line(&econ.file, table)).unwrap_or_default());
    log.push(tick, Class::Decision, line.clone());
    Ok((line, laid))
}

/// A recover link in hand without its body: the named item, bound to the contract, in a body at the far end of the
/// Board's route of a stratum seeded among those the Board offers.
fn bodies(econ: &mut Economy, table: &ItemTable, board: Option<&Board>, shift: u32, tick: u64, log: &mut CityLog) -> Vec<Laid> {
    let mut out = Vec::new();
    let Some(b) = board else { return out };
    for k in 0..econ.s.contracts.len() {
        let c = &econ.s.contracts[k];
        if c.kind != Kind::Recover || c.route.is_some() {
            continue;
        }
        let Some(item) = c.ask.item else { continue };
        let stratum = 1 + mat::rng::Pcg32::new(econ.s.seed ^ 0x0B0D_0000 ^ ((c.id as u64) << 8) ^ c.link as u64).below(b.strata.max(1) as u32) as u8;
        let route = run::route_seed(b.city_seed, stratum);
        let Ok(lay) = run::route(b.lib, route, stratum) else { continue };
        let map = run::RouteMap::compile(b.lib, &lay);
        let mut it = ItemInstance::new(item);
        it.bound = Some(c.id);
        let Some(corpse) = run::laid_body(&map, shift, it, table) else { continue };
        let (id, name) = (c.id, c.name.clone());
        log.push(tick, Class::Decision, format!("{name}'s body lies at the far end of the S{stratum} route ({})", econ.name(c.faction)));
        econ.s.contracts[k].route = Some((stratum, route));
        out.push(Laid { contract: id, stratum, route, name, corpse });
    }
    out
}

/// A host link in hand without its newcomer: one of the faction's Past joins the waiting list at a grate (the chain's
/// named one on its first link).
fn hosts(econ: &mut Economy, roster: &mut Roster, traits: &TraitFile, names: &NameFile, shift: u32, tick: u64, log: &mut CityLog) {
    for k in 0..econ.s.contracts.len() {
        let c = &econ.s.contracts[k];
        if c.kind != Kind::Host || c.host.is_some() {
            continue;
        }
        let past = Past::all().into_iter().find(|p| p.name() == econ.file.factions[c.faction as usize].past).unwrap_or(Past::Civ);
        let mut rng = mat::rng::Pcg32::new(econ.s.seed ^ 0x4057_0000 ^ ((c.id as u64) << 8) ^ c.link as u64);
        let name = (c.link == 1).then(|| c.name.clone());
        let id = roster.newcomer(traits, names, &mut rng, shift, past, 0, name.as_deref());
        let who = roster.waiting.iter().find(|(r, _)| r.id == id).map(|(r, _)| r.full_name()).unwrap_or_default();
        log.push(tick, Class::Landing, format!("{who} came down a grate for the {}: shelter them five shifts", econ.name(c.faction)));
        econ.s.contracts[k].host = Some(id);
    }
}

/// Where the city deals with a faction: its own street nearest the Heart when the Mat reaches one, else under Open
/// Grates the market nearest the Heart (Full Design v3.16, Economy: Buying).
pub fn market_of(econ: &Economy, city: &City, colony: &Colony, faction: u8) -> Option<u8> {
    let near = |own: bool| {
        (0..city.markets.len())
            .filter(|&k| !own || econ.faction_at(city.markets[k]) == Some(faction))
            .filter_map(|k| city.from_heart(logistics::Kind::Market, k as u32).map(|d| (d, k)))
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_, k)| k as u8)
    };
    near(true).or_else(|| if open_grates(colony) { near(false) } else { None })
}

/// An order at a market's stall: the faction must trade there (its street, or Open Grates). Returns the price a unit.
#[allow(clippy::too_many_arguments)]
pub fn order(econ: &mut Economy, ledger: &mut Ledger, table: &ItemTable, city: &City, colony: &Colony, defs: &StructureFile, roster: &Roster, plan: &[(u32, Job, u32)], market: u8, faction: u8, entry: u16, units: u16) -> Result<u32, String> {
    let open = open_grates(colony);
    let g = *city.markets.get(market as usize).ok_or("no such market")?;
    if !open && econ.faction_at(g) != Some(faction) && !econ.s.convoy.contains(&market) {
        return Err(format!("the {} do not trade at grate {}", econ.name(faction), city.market_grate(market as usize)));
    }
    let (trade, pasts) = stalls(city, colony, defs, roster, plan).get(market as usize).cloned().unwrap_or_default();
    let past = pasts.iter().any(|p| *p == econ.file.factions[faction as usize].past);
    let each = econ.order(faction, entry, units, table, trade, past, open)?;
    ledger.orders.push(inventory::Order { market, faction, entry, units, each });
    Ok(each)
}

/// An item sent to a contract landed at its market: it counts, or (the link gone or the item short of the ask) it
/// rides home.
#[allow(clippy::too_many_arguments)]
pub fn delivered(econ: &mut Economy, ledger: &mut Ledger, city: &mut City, table: &ItemTable, market: u8, contract: u32, item: ItemInstance, log: &mut CityLog) {
    let name = table.def(item.def).name.clone();
    if econ.delivered(contract, &item, table) {
        let c = econ.s.contracts.iter().find(|c| c.id == contract).unwrap();
        log.push(city.tick, Class::Landing, format!("{name} delivered to the {}: {}/{}", econ.name(c.faction), c.got, c.ask.n));
    } else {
        log.push(city.tick, Class::Loss, format!("{name} not taken at grate {}: back home", city.market_grate(market as usize)));
        home_from(ledger, city, market, item);
    }
}

/// An item rides home from a market: on the cords from where the market takes its cargo.
pub fn home_from(ledger: &mut Ledger, city: &mut City, market: u8, item: ItemInstance) {
    let id = ledger.ship(item, Dest::Stash);
    if let Some(cell) = city.market_cells.get(market as usize).copied().flatten() {
        city.dispatch(cell, id, logistics::Kind::Heart, 0);
    } else if let Some(h) = city.heart_cell() {
        city.dispatch(h, id, logistics::Kind::Heart, 0);
    }
}

/// The shift's end at the stalls, after the threats' (their next Patrols are known): last shift's services run
/// down, orders are paid and their goods leave the stalls on the cords, a hire joins the waiting list, contracts finish or lapse, the levers move
/// favour, a chain's unique item lands in the Stash, protection holds off a Patrol at a Gang grate, and on a clinic
/// day a double flush comes down; a recover link begun now has its body laid. Returns the lines for the report and
/// the bodies laid.
#[allow(clippy::too_many_arguments)]
pub fn end_shift(econ: &mut Economy, ledger: &mut Ledger, stash: &mut Stash, table: &ItemTable, city: &mut City, colony: &mut colony::Colony, roster: &mut Roster, traits: &TraitFile, names: &NameFile, threats: Option<&mut Threats>, board: Option<&Board>, sold: &[(u8, u32, u32)], shift: u32, raiders: bool, log: &mut CityLog) -> (Vec<String>, Vec<Laid>) {
    let tick = city.tick;
    let mut lines = Vec::new();
    // the services bought last shift run down; then the orders
    econ.services_tick();
    let (bought, paid) = econ.settle(&mut ledger.orders, &mut ledger.eddies);
    for b in bought {
        match b {
            Bought::Goods { market, item, units } => {
                for _ in 0..units {
                    home_from(ledger, city, market, ItemInstance::new(item));
                }
                log.push(tick, Class::Landing, format!("{units} {} bought at grate {}: on the cords home", table.def(item).name, city.market_grate(market as usize)));
            }
            Bought::Hire { faction } => {
                let past = Past::all().into_iter().find(|p| p.name() == econ.file.factions[faction as usize].past).unwrap_or(Past::Civ);
                let mut rng = mat::rng::Pcg32::new(econ.s.seed ^ 0x41E0_0000 ^ ((shift as u64) << 8) ^ faction as u64);
                let id = roster.newcomer(traits, names, &mut rng, shift, past, 2, None);
                let who = roster.waiting.iter().find(|(r, _)| r.id == id).map(|(r, _)| r.full_name()).unwrap_or_default();
                log.push(tick, Class::Landing, format!("{who} hired through the {}: waiting at a grate", econ.name(faction)));
            }
            Bought::Service { faction, name } => log.push(tick, Class::Landing, format!("{name} bought from the {}", econ.name(faction))),
        }
    }
    if paid > 0 {
        lines.push(format!("{paid} e paid at the stalls"));
    }
    if !ledger.orders.is_empty() {
        lines.push(format!("{} order{} wait for eddies", ledger.orders.len(), if ledger.orders.len() == 1 { "" } else { "s" }));
    }
    // contracts, levers, services
    let mut f = facts(city, threats.as_deref(), roster, raiders);
    f.sold = sold.to_vec();
    f.road_favour = colony.progress.collections.set_flag("road_favour");
    let ended = econ.end_shift(shift, &f, &mut ledger.eddies, table);
    ledger.lifetime_earned += ended.paid;
    // a chain finished is a deed (Full Design v3.18, Authority)
    if ended.chains_done > 0 {
        colony.charter.earn(ended.chains_done);
        log.push(tick, Class::Landing, format!("a contract chain finished: Authority +{}", ended.chains_done));
    }
    for it in ended.uniques {
        let name = table.def(it.def).name.clone();
        stash.place_or_floor(table, it);
        lines.push(format!("{name} into the Stash"));
    }
    note(log, tick, ended.notes);
    hosts(econ, roster, traits, names, shift, tick, log);
    let laid = bodies(econ, table, board, shift, tick, log);
    // protection: the next Patrol due in a zone of the Gangs' grates holds off a shift
    if let (Some(t), Some(gangs)) = (threats, econ.file.index("gangs")) {
        if let Some(k) = t.due.iter().position(|d| d.stage == 0 && econ.faction_at(t.zones[d.zone as usize].grate) == Some(gangs)) {
            if econ.take_protection() {
                let d = t.due.remove(k);
                log.push(tick, Class::Landing, format!("the Gangs held off the Patrol at grate {}", t.grate_label(&city.sim, d.zone as usize)));
            }
        }
    }
    // a clinic day's flush, twice the size
    if econ.s.flush {
        econ.s.flush = false;
        let mut rng = mat::rng::Pcg32::new(econ.s.seed ^ 0xC1_1C00 ^ shift as u64);
        let came = roster.clinic_flush(traits, names, &mut rng, shift);
        log.push(tick, Class::Landing, format!("clinic day topside: {came} newcomers came down a grate"));
    }
    (lines, laid)
}
