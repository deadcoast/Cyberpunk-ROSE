//! The Market screen's reads and verbs (Full Design v3.16, UI and screens: Market; Economy). Every rule is in the
//! economy crate and `game::trade`; this file shapes their answers for Godot.

use godot::prelude::*;

use crate::{RosePeople, RoseSim};
use economy::Kind;
use items::{ItemTable, Tier};

/// The categories a stall buys, in the bars' order (Keys and Relics never sell).
const BARS: [items::Category; 7] = [items::Category::Chrome, items::Category::Part, items::Category::Organic, items::Category::Tech, items::Category::Gear, items::Category::Contraband, items::Category::Curio];

fn tier_name(t: Tier) -> &'static str {
    match t {
        Tier::Common => "common",
        Tier::Uncommon => "uncommon",
        Tier::Rare => "rare",
        Tier::Blackline => "blackline",
        Tier::Relic => "relic",
    }
}

fn kind_name(k: Kind) -> &'static str {
    match k {
        Kind::Deliver => "deliver",
        Kind::ClearRaiders => "clear_raiders",
        Kind::ClearBloom => "clear_bloom",
        Kind::Recover => "recover",
        Kind::Host => "host",
    }
}

/// What a link asks, in the panel's words.
fn ask_line(c: &economy::Contract, table: &ItemTable) -> String {
    match c.kind {
        Kind::Deliver => {
            let thing = match (c.ask.item, c.ask.category) {
                (Some(i), _) => table.def(i).name.clone(),
                (None, Some(k)) => format!("{}{}", c.ask.tier.map(|t| format!("{} ", tier_name(t))).unwrap_or_default(), economy::category_name(k)),
                _ => "goods".into(),
            };
            let cond = if c.ask.condition > 0 { format!(" at {}+", c.ask.condition) } else { String::new() };
            format!("deliver {} {thing}{cond}", c.ask.n)
        }
        Kind::ClearRaiders => format!("kill {} raiders on runs", c.ask.n),
        Kind::ClearBloom => "clear the Bloom from the city".into(),
        Kind::Recover => format!("recover {}'s {}", c.name, c.ask.item.map(|i| table.def(i).name.clone()).unwrap_or_default()),
        Kind::Host => format!("shelter {} five shifts", c.name),
    }
}

#[godot_api(secondary)]
impl RoseSim {
    /// The Market screen for one faction (an index into the factions file):
    /// {factions: [{index, name, grates: "G2 G7", reached}], events: [String], eddies, day,
    ///  faction, name, market (-1 when none in reach), market_name, why (when no market),
    ///  rep, favour, step, tier, next: {tier, rep, volume} or {}, traded, lever, rival, closed,
    ///  services: {intel, timings, protection, convoy},
    ///  stock: [{entry, tier, open, item, name, service, units, price, item_tier}],
    ///  demand: [{category, today, base, uncommon, rare, glut}],
    ///  offers: [{id, links, kind, ask, eddies, until}],
    ///  contracts: [{id, kind, link, links, got, n, due, eddies, ask, where}],
    ///  queue: [{index, name, verb, quote}], orders: [{name, units, each}]}
    #[func]
    fn market(&mut self, people: Gd<RosePeople>, faction: i64) -> VarDictionary {
        let defs = self.defs().clone();
        let mut d = VarDictionary::new();
        let (Some(e), Some(city), Some(st)) = (self.economy.as_ref(), self.city.as_ref(), self.stash.as_ref()) else { return d };
        let p = people.bind();
        let mut st = st.clone();
        let mut st = st.bind_mut();
        st.ensure();
        let st = &*st;
        let table = st.table.as_ref().unwrap();
        let f = (faction.max(0) as u8).min(e.file.factions.len() as u8 - 1);
        let open_grates = game::trade::open_grates(&self.colony);
        // the tabs
        let mut tabs = VarArray::new();
        for (k, def) in e.file.factions.iter().enumerate() {
            if def.chapter > game::trade::CHAPTER {
                continue;
            }
            let grates: Vec<String> = e.s.streets.iter().filter(|s| s.faction as usize == k).map(|s| format!("G{}", s.number)).collect();
            let mut t = VarDictionary::new();
            t.set("index", k as i64);
            t.set("name", def.name.as_str());
            t.set("grates", grates.join(" ").as_str());
            t.set("reached", game::trade::market_of(e, city, &self.colony, k as u8).is_some());
            tabs.push(&t.to_variant());
        }
        d.set("factions", &tabs);
        d.set("events", &PackedStringArray::from(e.s.events.iter().map(|&(k, days)| GString::from(format!("{} {} day{}", e.file.events[k as usize].name, days, if days == 1 { "" } else { "s" }).as_str())).collect::<Vec<_>>().as_slice()));
        d.set("eddies", st.ledger.eddies as i64);
        d.set("day", e.s.day as i64);
        // the faction and where the city deals with it
        let def = &e.file.factions[f as usize];
        d.set("faction", f as i64);
        d.set("name", def.name.as_str());
        let market = game::trade::market_of(e, city, &self.colony, f);
        d.set("market", market.map_or(-1, |m| m as i64));
        d.set("market_name", market.map_or(String::new(), |m| format!("grate {}", city.market_grate(m as usize))).as_str());
        let streets: Vec<String> = e.s.streets.iter().filter(|s| s.faction == f).map(|s| format!("G{}", s.number)).collect();
        d.set("why", if market.is_some() { String::new() } else if streets.is_empty() { format!("no {} street on this stratum", def.name) } else { format!("the Mat reaches none of {}", streets.join(" ")) }.as_str());
        // the standing line
        let stn = &e.s.standing[f as usize];
        let tier = e.tier_open(f);
        d.set("rep", stn.rep as i64);
        d.set("favour", stn.favour as i64);
        d.set("step", e.file.rules.favour as i64);
        d.set("tier", tier as i64);
        d.set("traded", stn.traded as i64);
        d.set("closed", tier == 0);
        let mut next = VarDictionary::new();
        if (1..4).contains(&tier) {
            next.set("tier", (tier + 1) as i64);
            next.set("rep", e.file.rules.tier_rep[tier as usize] as i64);
            next.set("volume", e.file.rules.tier_volume[tier as usize] as i64);
        }
        d.set("next", &next);
        let lever = match def.lever.as_str() {
            "volume" => format!("+1 favour a shift you sell them {}", def.lever_at as u32),
            "quiet" => format!("+1 favour a shift their grates stay under {}", def.lever_at as u32),
            "protection" => "+2 favour a shift you buy protection".into(),
            "intel" => "+2 favour a shift you buy intel".into(),
            _ => String::new(),
        };
        d.set("lever", lever.as_str());
        d.set("rival", e.file.index(&def.rival).map_or("", |r| e.file.factions[r as usize].name.as_str()));
        let mut sv = VarDictionary::new();
        sv.set("intel", e.s.intel as i64);
        sv.set("timings", e.s.timings as i64);
        sv.set("protection", e.s.protection as i64);
        sv.set("convoy", !e.s.convoy.is_empty());
        d.set("services", &sv);
        // the stall's haggling and Past where the city deals with it
        let staff = game::trade::stalls(city, &self.colony, &defs, &p.roster, &self.plan);
        let (trade, pasts) = market.and_then(|m| staff.get(m as usize).cloned()).unwrap_or_default();
        let past = pasts.contains(&def.past);
        // the stock: today's entries, every tier, the closed ones dimmed
        let mut stock = VarArray::new();
        for &(entry, units) in &e.s.stock[f as usize] {
            let s = &def.stock[entry as usize];
            let mut x = VarDictionary::new();
            x.set("entry", entry as i64);
            x.set("tier", s.tier as i64);
            x.set("open", s.tier <= tier);
            x.set("units", units as i64);
            x.set("price", e.price_of(f, entry, table, trade, past, open_grates).map_or(-1, |v| v as i64));
            x.set("service", s.service.as_str());
            match table.id_of(&s.item) {
                Some(i) => {
                    let id = table.def(i);
                    x.set("item", id.id.as_str());
                    x.set("name", id.name.as_str());
                    x.set("item_tier", tier_name(id.tier));
                    x.set("w", id.size[0] as i64);
                    x.set("h", id.size[1] as i64);
                }
                None => {
                    x.set("item", "");
                    x.set("name", s.service.as_str());
                    x.set("item_tier", "common");
                }
            }
            stock.push(&x.to_variant());
        }
        d.set("stock", &stock);
        // demand and glut
        let mut bars = VarArray::new();
        for c in BARS {
            let mut b = VarDictionary::new();
            b.set("category", economy::category_name(c));
            b.set("today", e.demand(f, c, Tier::Common) as f64);
            b.set("uncommon", e.demand(f, c, Tier::Uncommon) as f64);
            b.set("rare", e.demand(f, c, Tier::Rare) as f64);
            b.set("base", def.base(c) as f64);
            let glut = market.map_or(0, |m| st.ledger.glut_units(m, c as usize));
            b.set("glut", (glut * 5).min(75) as i64);
            bars.push(&b.to_variant());
        }
        d.set("demand", &bars);
        // offers and links in hand
        let mut offers = VarArray::new();
        for o in e.s.offers.iter().filter(|o| o.faction == f) {
            let c = e.first_link(o, e.s.day * 3);
            let mut x = VarDictionary::new();
            x.set("id", o.id as i64);
            x.set("links", o.links as i64);
            x.set("kind", kind_name(c.kind));
            x.set("ask", ask_line(&c, table).as_str());
            x.set("eddies", c.eddies as i64);
            x.set("shifts", (c.due - c.from) as i64);
            x.set("until", o.until as i64);
            offers.push(&x.to_variant());
        }
        d.set("offers", &offers);
        let mut held = VarArray::new();
        for c in e.s.contracts.iter().filter(|c| c.faction == f) {
            let mut x = VarDictionary::new();
            x.set("id", c.id as i64);
            x.set("kind", kind_name(c.kind));
            x.set("link", c.link as i64);
            x.set("links", c.links as i64);
            x.set("got", c.got as i64);
            x.set("n", c.ask.n as i64);
            x.set("due", c.due as i64);
            x.set("eddies", c.eddies as i64);
            x.set("ask", ask_line(c, table).as_str());
            let at = match (c.kind, c.route) {
                (Kind::Recover, Some((s, _))) => format!("the body lies at the far end of the S{s} route"),
                (Kind::Host, Some(_)) | (Kind::Host, None) => c.host.and_then(|h| p.roster.get(h).map(|r| format!("{} is in the city", r.name)).or_else(|| p.roster.waiting.iter().find(|(r, _)| r.id == h).map(|(r, _)| format!("{} waits at a grate for a bed", r.name)))).unwrap_or_default(),
                _ => String::new(),
            };
            x.set("where", at.as_str());
            x.set("sendable", matches!(c.kind, Kind::Deliver | Kind::Recover));
            held.push(&x.to_variant());
        }
        d.set("contracts", &held);
        // the market's queue and this faction's orders
        let mine: Vec<u32> = e.s.contracts.iter().filter(|c| c.faction == f).map(|c| c.id).collect();
        let mut queue = VarArray::new();
        for (i, q) in st.ledger.queue.iter().enumerate() {
            let (verb, quote) = match q.verb {
                inventory::Verb::Sell { market: m } if Some(m) == market => {
                    let cat = table.def(q.item.def).category;
                    let ahead = st.ledger.queue[..i].iter().filter(|x| x.verb == q.verb && table.def(x.item.def).category == cat).count() as u32;
                    ("sell", st.ledger.best(table, &q.item, m, st.ledger.glut_units(m, cat as usize) + ahead).map_or(-1, |(v, _)| v as i64))
                }
                inventory::Verb::Deliver { contract, .. } if mine.contains(&contract) => ("deliver", 0),
                _ => continue,
            };
            let mut x = VarDictionary::new();
            x.set("index", i as i64);
            x.set("name", table.def(q.item.def).name.as_str());
            x.set("verb", verb);
            x.set("quote", quote);
            queue.push(&x.to_variant());
        }
        d.set("queue", &queue);
        let mut orders = VarArray::new();
        for o in st.ledger.orders.iter().filter(|o| o.faction == f) {
            let s = &def.stock[o.entry as usize];
            let mut x = VarDictionary::new();
            x.set("name", table.id_of(&s.item).map_or(s.service.clone(), |i| table.def(i).name.clone()).as_str());
            x.set("units", o.units as i64);
            x.set("each", o.each as i64);
            orders.push(&x.to_variant());
        }
        d.set("orders", &orders);
        d
    }

    /// Order `units` of a stock entry from a faction where the city deals with it: paid at the shift's end. Returns
    /// "" or why not.
    #[func]
    fn market_order(&mut self, people: Gd<RosePeople>, faction: i64, entry: i64, units: i64) -> GString {
        let defs = self.defs().clone();
        let (Some(e), Some(city), Some(st)) = (self.economy.as_mut(), self.city.as_ref(), self.stash.as_mut()) else { return "no city".into() };
        let p = people.bind();
        let mut st = st.bind_mut();
        st.ensure();
        let st = &mut *st;
        let f = faction.clamp(0, e.file.factions.len() as i64 - 1) as u8;
        let Some(m) = game::trade::market_of(e, city, &self.colony, f) else { return format!("the Mat reaches no street of the {}", e.name(f)).as_str().into() };
        match game::trade::order(e, &mut st.ledger, st.table.as_ref().unwrap(), city, &self.colony, &defs, &p.roster, &self.plan, m, f, entry.clamp(0, u16::MAX as i64) as u16, units.clamp(1, 99) as u16) {
            Ok(each) => {
                self.log.push(city.tick, game::Class::Decision, format!("ordered {} from the {} at {each} e", units.max(1), e.name(f)));
                GString::new()
            }
            Err(why) => why.as_str().into(),
        }
    }

    /// Take an offer. A recover's body is laid on its route as it is taken: {line, why, laid: [{contract, stratum,
    /// route, name, x, y, chunk, day, rig: bytes}]}.
    #[func]
    fn market_accept(&mut self, people: Gd<RosePeople>, offer: i64, day: i64) -> VarDictionary {
        let mut d = VarDictionary::new();
        let (Some(e), Some(city), Some(st)) = (self.economy.as_mut(), self.city.as_ref(), self.stash.as_mut()) else { return d };
        let mut people = people;
        let mut p = people.bind_mut();
        p.ensure();
        let p = &mut *p;
        let traits = p.traits.clone().unwrap();
        let names = p.names.clone().unwrap();
        let mut st = st.bind_mut();
        st.ensure();
        let table = st.table.as_ref().unwrap();
        let lib = run::ChunkLib::from_toml(run::CHUNKS_TOML).expect("chunks.toml");
        let board = game::trade::Board { lib: &lib, city_seed: e.s.seed, strata: self.strata.max(1) };
        match game::trade::accept(e, (city, &self.colony), &mut p.roster, &traits, &names, table, Some(&board), offer.max(0) as u32, day.max(1) as u32, &mut self.log) {
            Ok((line, laid)) => {
                d.set("line", line.as_str());
                d.set("why", "");
                d.set("laid", &laid_array(&laid));
            }
            Err(why) => {
                d.set("line", "");
                d.set("why", why.as_str());
                d.set("laid", &VarArray::new());
            }
        }
        d
    }

    /// Queue what the Stash holds that fits a deliver or recover link in hand, up to what it still asks and is not
    /// riding, to the faction's market. Returns how many went, or -1 when the faction has no market in reach.
    #[func]
    fn market_send(&mut self, contract: i64, day: i64) -> i64 {
        let (Some(e), Some(city), Some(st)) = (self.economy.as_ref(), self.city.as_ref(), self.stash.as_mut()) else { return -1 };
        let Some(c) = e.s.contracts.iter().find(|c| c.id as i64 == contract).cloned() else { return 0 };
        let Some(m) = game::trade::market_of(e, city, &self.colony, c.faction) else { return -1 };
        let mut st = st.bind_mut();
        st.ensure();
        let st = &mut *st;
        let table = st.table.as_ref().unwrap();
        let stash = st.stash.as_mut().unwrap();
        let riding = st.ledger.in_flight.iter().filter(|s| s.contract == Some(c.id)).count() + st.ledger.queue.iter().filter(|q| matches!(q.verb, inventory::Verb::Deliver { contract, .. } if contract == c.id)).count();
        let need = (c.ask.n as usize).saturating_sub(c.got as usize + riding);
        let mut fit: Vec<(u32, u32)> = stash.handles().into_iter().filter(|(_, _, it)| !it.pinned && e.fits(&c, it, table)).map(|(h, _, it)| (table.def(it.def).value, h)).collect();
        fit.sort_unstable();
        let mut sent = 0;
        for (_, h) in fit.into_iter().take(need) {
            if st.ledger.queue(stash, h, inventory::Verb::Deliver { market: m, contract: c.id }, day.max(0) as u32) {
                sent += 1;
            }
        }
        sent
    }

    /// The Board's route seed of a stratum for the save's seed (run::route_seed).
    #[func]
    fn route_seed(&self, city_seed: i64, stratum: i64) -> i64 {
        run::route_seed(city_seed as u64, stratum.clamp(1, 255) as u8) as i64
    }
}

/// Bodies laid for recover links, as the Board's corpses take them.
pub(crate) fn laid_array(laid: &[game::trade::Laid]) -> VarArray {
    let mut a = VarArray::new();
    for l in laid {
        let mut x = VarDictionary::new();
        x.set("contract", l.contract as i64);
        x.set("stratum", l.stratum as i64);
        x.set("route", l.route as i64);
        x.set("name", l.name.as_str());
        x.set("x", l.corpse.x as i64);
        x.set("y", l.corpse.y as i64);
        x.set("chunk", l.corpse.chunk.as_str());
        x.set("day", l.corpse.day as i64);
        x.set("rig", &PackedByteArray::from(postcard::to_allocvec(&l.corpse.rig).unwrap_or_default().as_slice()));
        a.push(&x.to_variant());
    }
    a
}
