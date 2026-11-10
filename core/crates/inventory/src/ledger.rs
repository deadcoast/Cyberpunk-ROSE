//! The ledger: the Stash's sell and feed queues, the eddies, per-market glut, and the cargo the Stash has out on the
//! Mat (Full Design v3.11, Stash: The four verbs; Colony mode: Cargo; v3.15, Economy). At shift end the queue and the
//! quotas' pulls leave the Heart as shipments; each lands when its packet does. A sale pays what the market's stall
//! pays — its faction's book (`items::Book`: demand, reputation, haggling) at that market's glut for the item's
//! category at that moment (−5 % per unit of the category already sold there this shift, floor 25 %, the price
//! recovering 20 points a shift), the best of its books when it has more than one (Open Grates, the convoy); a delivery to a contract counts
//! instead of being paid; a feed delivers the item's nutrient (halved below condition 20) and a genome drift of bias ×
//! the tier's drift to its Pit; a Part waits in its Still's store; a product, or what was bought at a stall, rides home
//! to the grid.

use items::{feed_value, price, sell_at, Book, ItemInstance, ItemTable, PriceContext, Tier};
use serde::{Deserialize, Serialize};

use crate::Stash;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verb {
    /// to the market with this index among the colony's market sites
    Sell { market: u8 },
    /// to the Pit with this index among the colony's Pits (0 is the Heart)
    Feed { pit: u8 },
    /// to a contract, at a market of its faction (v13, Economy)
    Deliver { market: u8, contract: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Queued {
    pub item: ItemInstance,
    pub verb: Verb,
    /// the colony day it was queued on
    pub day: u32,
    /// the stockpile it was queued from (None: the Heart's grid); it rides from there (v10)
    #[serde(default)]
    pub from: Option<u32>,
}

/// Where a shipment is bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dest {
    /// a Pit by index among the colony's Pits (0 is the Heart)
    Pit(u8),
    /// a market by index among the market sites
    Market(u8),
    /// a Still by structure id: into its store
    Still(u32),
    /// home to the Stash grid
    Stash,
    /// into a stockpile (its zone id): the Stash's settling (v10)
    Room(u32),
}

/// An item on the Mat.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shipment {
    pub id: u32,
    pub item: ItemInstance,
    pub to: Dest,
    /// a delivery to a contract (v13)
    #[serde(default)]
    pub contract: Option<u32>,
}

/// An order at a stall (v13, Economy: Buying): what the stall's faction sells, by its stock entry, paid at the shift's
/// end and sent down the cords to the Heart.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub market: u8,
    pub faction: u8,
    /// the stock entry in the faction's list
    pub entry: u16,
    pub units: u16,
    /// the price a unit when it was ordered
    pub each: u32,
}

/// What landed since the last shift end, and what has just left.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShiftReport {
    pub sold: u32,
    pub earned: u32,
    /// what each faction bought: (faction, units, eddies) (v13)
    #[serde(default)]
    pub to_faction: Vec<(u8, u32, u32)>,
    /// paid at the stalls this shift end (v13)
    #[serde(default)]
    pub bought: u32,
    pub fed: u32,
    pub deliveries: Vec<Delivery>,
    /// Parts into Stills, products home to the grid, cargo lost off the Mat
    pub stored: u32,
    pub home: u32,
    pub spilled: u32,
    /// taken by a foreman under the Foreman's Cut
    pub cut: u32,
    /// shipments that left at this shift end
    pub left: u32,
}

/// Nutrient and genome drift for one Pit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Delivery {
    pub pit: u8,
    pub nutrient: f32,
    /// summed bias × drift per gene; the colony adds it to the genome and clamps
    pub drift: [f32; 10],
}

/// What a shipment did when it landed.
#[derive(Clone, Debug, PartialEq)]
pub enum Landing {
    Sold { market: u8, eddies: u32, faction: u8, blackline: bool },
    /// counted toward a contract (Economy): the economy reads the item
    Delivered { market: u8, contract: u32, item: ItemInstance },
    /// no stall at the market would buy it: it rides home
    Refused { market: u8, item: ItemInstance },
    Fed { pit: u8, nutrient: f32, drift: [f32; 10] },
    Stored { still: u32 },
    /// into the grid, or onto the Heart's floor while the grid is full
    Home { placed: bool },
    /// onto a stockpile; into the grid or onto the Heart's floor when it had no room left
    Shelved { zone: u32, placed: bool },
}

/// A standing feed or Part policy (Full Design v3.11, Quotas), handed to `pull` by the colony's side.
#[derive(Clone, Debug, PartialEq)]
pub struct Quota {
    pub to: Dest,
    pub want: Want,
    /// weighted tiles from the Heart; None when the Mat does not reach it
    pub distance: Option<f32>,
    /// a dissolved district's structure: served after every other
    pub low: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Want {
    /// keep this much nutrient in the Pit (its stock now given; what is riding to it is added here)
    Nutrient { target: f32, stock: f32, ceiling: Tier },
    /// keep this many Parts with the tag in the Still's store
    Tagged { tag: String, target: u32, stock: u32 },
}

pub const GLUT_PER_UNIT: f32 = 0.05;
pub const GLUT_FLOOR: f32 = 0.25;
pub const GLUT_RECOVERY: f32 = 0.20;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    pub eddies: u32,
    pub queue: Vec<Queued>,
    /// v12's glut, one number a market (read as nothing since v13)
    #[serde(default)]
    pub glut: Vec<f32>,
    /// units sold per market and per category this shift (fractional after recovery) (v13)
    #[serde(default)]
    pub glut_by: Vec<[f32; 9]>,
    /// each market's stalls' books, set by the economy each day and as standing moves; empty: the 1.0 book (v13)
    #[serde(default)]
    pub books: Vec<Vec<Book>>,
    /// orders at the stalls, paid and sent at the shift's end (v13)
    #[serde(default)]
    pub orders: Vec<Order>,
    pub lifetime_earned: u32,
    /// what the Stash has on the Mat (v9)
    #[serde(default)]
    pub in_flight: Vec<Shipment>,
    #[serde(default)]
    pub next_ship: u32,
    /// Parts landed at a Still, by structure id (v9)
    #[serde(default)]
    pub stores: Vec<(u32, Vec<ItemInstance>)>,
    /// the report so far this shift (v9)
    #[serde(default)]
    pub landed: ShiftReport,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger::default()
    }

    /// Units of a category sold at a market this shift, after recovery.
    pub fn glut_units(&self, market: u8, category: usize) -> u32 {
        self.glut_by.get(market as usize).map_or(0.0, |g| g[category]).round() as u32
    }

    /// The best a market's stalls pay for an item with `glut` units of its category ahead of it: (eddies, faction).
    /// None when no stall there will buy it.
    pub fn best(&self, table: &ItemTable, item: &ItemInstance, market: u8, glut: u32) -> Option<(u32, u8)> {
        let def = table.def(item.def);
        let fallback = [Book::default()];
        let books = self.books.get(market as usize).filter(|b| !b.is_empty()).map_or(&fallback[..], |b| &b[..]);
        books.iter().filter_map(|b| sell_at(def, item, b, glut).map(|p| (p, b.faction))).max_by_key(|&(p, f)| (p, std::cmp::Reverse(f)))
    }

    /// The price an item would fetch at a market now, with that market's glut for its category and what of that
    /// category is queued and riding there ahead of it. None when no stall there buys it.
    pub fn quote(&self, table: &ItemTable, item: &ItemInstance, market: u8) -> Option<u32> {
        let cat = table.def(item.def).category as usize;
        let same = |it: &ItemInstance| table.def(it.def).category as usize == cat;
        let ahead = self.queue.iter().filter(|q| q.verb == Verb::Sell { market } && same(&q.item)).count() + self.in_flight.iter().filter(|s| s.to == Dest::Market(market) && s.contract.is_none() && same(&s.item)).count();
        self.best(table, item, market, self.glut_units(market, cat) + ahead as u32).map(|(p, _)| p)
    }

    /// Move an item out of whichever room of the Stash holds it into the queue. False when the handle is missing.
    pub fn queue(&mut self, stash: &mut Stash, handle: crate::Handle, verb: Verb, day: u32) -> bool {
        let from = stash.room_of(handle);
        let Some(item) = stash.take(handle) else { return false };
        self.queue.push(Queued { item, verb, day, from });
        true
    }

    /// Pull a queued item back to its room (or the grid); with no room left it stays queued. Returns the new handle.
    pub fn unqueue(&mut self, stash: &mut Stash, table: &ItemTable, index: usize) -> Option<crate::Handle> {
        if index >= self.queue.len() {
            return None;
        }
        let q = self.queue[index].clone();
        match stash.put_back(table, q.item, q.from) {
            Ok(h) => {
                self.queue.remove(index);
                Some(h)
            }
            Err(_) => None,
        }
    }

    /// Put an item on the Mat bound somewhere; returns its shipment id.
    pub fn ship(&mut self, item: ItemInstance, to: Dest) -> u32 {
        let id = self.next_ship;
        self.next_ship = self.next_ship.wrapping_add(1);
        self.in_flight.push(Shipment { id, item, to, contract: None });
        self.landed.left += 1;
        id
    }

    /// Shift end: the queue leaves the Heart, or the stockpile each item lay in. Returns the shipments for the city
    /// to dispatch as (id, where to, the room it leaves from).
    pub fn dispatch_queue(&mut self) -> Vec<(u32, Dest, Option<u32>)> {
        let mut out = Vec::new();
        for q in std::mem::take(&mut self.queue) {
            let (to, contract) = match q.verb {
                Verb::Sell { market } => (Dest::Market(market), None),
                Verb::Feed { pit } => (Dest::Pit(pit), None),
                Verb::Deliver { market, contract } => (Dest::Market(market), Some(contract)),
            };
            let id = self.ship(q.item, to);
            if let Some(s) = self.in_flight.last_mut() {
                s.contract = contract;
            }
            out.push((id, to, q.from));
        }
        out
    }

    /// Nutrient riding to a Pit, or how many shipments ride to a Still.
    pub fn riding(&self, table: &ItemTable, to: Dest) -> (f32, u32) {
        let mut n = 0.0;
        let mut k = 0;
        for s in self.in_flight.iter().filter(|s| s.to == to) {
            n += feed_value(table.def(s.item.def), &s.item).0;
            k += 1;
        }
        (n, k)
    }

    /// The quotas' pull at shift end (Full Design v3.11, Quotas): item by item to the quota with the highest demand
    /// (the share of it not met, counting what rides to it) per tile of distance from the Heart, low priority last,
    /// ties toward the Heart; each takes the lowest tier first and the cheapest within it, never a pinned item.
    pub fn pull(&mut self, stash: &mut Stash, table: &ItemTable, quotas: &[Quota]) -> Vec<(u32, Dest, Option<u32>)> {
        let mut have: Vec<f32> = quotas
            .iter()
            .map(|q| match &q.want {
                Want::Nutrient { stock, .. } => stock + self.riding(table, q.to).0,
                Want::Tagged { stock, .. } => (*stock + self.riding(table, q.to).1) as f32,
            })
            .collect();
        let target = |q: &Quota| match &q.want {
            Want::Nutrient { target, .. } => *target,
            Want::Tagged { target, .. } => *target as f32,
        };
        let eligible = |q: &Quota, it: &ItemInstance| -> bool {
            let d = table.def(it.def);
            !it.pinned
                && match &q.want {
                    Want::Nutrient { ceiling, .. } => d.feed > 0.0 && d.tier <= *ceiling,
                    Want::Tagged { tag, .. } => d.category == items::Category::Part && d.tags.iter().any(|t| t == tag),
                }
        };
        let mut out = Vec::new();
        loop {
            // the quota to serve next
            let mut best: Option<(usize, bool, f32, f32)> = None;
            for (k, q) in quotas.iter().enumerate() {
                let Some(dist) = q.distance else { continue };
                let t = target(q);
                if t <= 0.0 || have[k] >= t - 1e-6 {
                    continue;
                }
                if !stash.handles().iter().any(|(_, _, it)| eligible(q, it)) {
                    continue;
                }
                let score = (t - have[k]) / t / (1.0 + dist);
                let better = match best {
                    None => true,
                    Some((_, low, s, d)) => {
                        if q.low != low {
                            !q.low
                        } else if (score - s).abs() > 1e-6 {
                            score > s
                        } else {
                            dist < d
                        }
                    }
                };
                if better {
                    best = Some((k, q.low, score, dist));
                }
            }
            let Some((k, ..)) = best else { break };
            let q = &quotas[k];
            // the lowest tier, then the cheapest, from any room (ties: the Heart's grid first, then the lowest handle)
            let pick = stash
                .handles()
                .into_iter()
                .filter(|(_, _, it)| eligible(q, it))
                .map(|(h, room, it)| (table.def(it.def).tier, price(table.def(it.def), it, PriceContext::default()), room.is_some(), h, room))
                .min_by(|a, b| (a.0, a.1, a.2, a.3).cmp(&(b.0, b.1, b.2, b.3)));
            let Some((_, _, _, h, room)) = pick else { break };
            let Some(item) = stash.take(h) else { break };
            have[k] += match &q.want {
                Want::Nutrient { .. } => feed_value(table.def(item.def), &item).0,
                Want::Tagged { .. } => 1.0,
            };
            out.push((self.ship(item, q.to), q.to, room));
        }
        out
    }

    /// A shipment reached `at` (where the city's routing took it; a re-routed one lands where it arrived).
    pub fn land(&mut self, stash: &mut Stash, table: &ItemTable, id: u32, at: Dest) -> Option<Landing> {
        let k = self.in_flight.iter().position(|s| s.id == id)?;
        let s = self.in_flight.remove(k);
        Some(match at {
            Dest::Market(market) => {
                if let Some(contract) = s.contract {
                    return Some(Landing::Delivered { market, contract, item: s.item });
                }
                let m = market as usize;
                if self.glut_by.len() <= m {
                    self.glut_by.resize(m + 1, [0.0; 9]);
                }
                let def = table.def(s.item.def);
                let cat = def.category as usize;
                let Some((paid, faction)) = self.best(table, &s.item, market, self.glut_by[m][cat].round() as u32) else {
                    return Some(Landing::Refused { market, item: s.item });
                };
                self.glut_by[m][cat] += 1.0;
                self.eddies += paid;
                self.lifetime_earned += paid;
                self.landed.sold += 1;
                self.landed.earned += paid;
                match self.landed.to_faction.iter_mut().find(|t| t.0 == faction) {
                    Some(t) => {
                        t.1 += 1;
                        t.2 += paid;
                    }
                    None => self.landed.to_faction.push((faction, 1, paid)),
                }
                Landing::Sold { market, eddies: paid, faction, blackline: def.tier == Tier::Blackline }
            }
            Dest::Pit(pit) => {
                let (nutrient, bias, drift) = feed_value(table.def(s.item.def), &s.item);
                let mut d = [0.0f32; 10];
                for g in 0..10 {
                    d[g] = bias[g] * drift;
                }
                match self.landed.deliveries.iter_mut().find(|x| x.pit == pit) {
                    Some(x) => {
                        x.nutrient += nutrient;
                        for g in 0..10 {
                            x.drift[g] += d[g];
                        }
                    }
                    None => self.landed.deliveries.push(Delivery { pit, nutrient, drift: d }),
                }
                self.landed.fed += 1;
                Landing::Fed { pit, nutrient, drift: d }
            }
            Dest::Still(still) => {
                match self.stores.iter_mut().find(|(k, _)| *k == still) {
                    Some((_, v)) => v.push(s.item),
                    None => self.stores.push((still, vec![s.item])),
                }
                self.landed.stored += 1;
                Landing::Stored { still }
            }
            Dest::Room(zone) => match stash.shelve(zone, s.item) {
                Ok(_) => Landing::Shelved { zone, placed: true },
                Err(item) => {
                    // the stockpile filled while it rode: the grid takes it, else the Heart's floor
                    stash.place_or_floor(table, item);
                    Landing::Shelved { zone, placed: false }
                }
            },
            Dest::Stash => {
                self.landed.home += 1;
                Landing::Home { placed: stash.place_or_floor(table, s.item) }
            }
        })
    }

    /// A shipment fell off the Mat.
    pub fn spill(&mut self, id: u32) -> Option<ItemInstance> {
        let k = self.in_flight.iter().position(|s| s.id == id)?;
        self.landed.spilled += 1;
        Some(self.in_flight.remove(k).item)
    }

    /// A foreman took it (the Foreman's Cut): it leaves the economy.
    pub fn divert(&mut self, id: u32) -> Option<ItemInstance> {
        let k = self.in_flight.iter().position(|s| s.id == id)?;
        self.landed.cut += 1;
        Some(self.in_flight.remove(k).item)
    }

    /// How many Parts a Still holds.
    pub fn store_count(&self, still: u32) -> u32 {
        self.stores.iter().find(|(k, _)| *k == still).map_or(0, |(_, v)| v.len() as u32)
    }

    /// A Still's slot takes one Part with the tag from its store.
    pub fn take_from_store(&mut self, table: &ItemTable, still: u32, tag: &str) -> Option<ItemInstance> {
        let (_, v) = self.stores.iter_mut().find(|(k, _)| *k == still)?;
        let i = v.iter().position(|it| table.def(it.def).tags.iter().any(|t| t == tag))?;
        Some(v.remove(i))
    }

    /// Shift end, after the queue and the pulls have left: the report of what landed since the last shift end and
    /// what just left; the Heart's floor tries the grid again; each market's glut recovers 20 points of price.
    pub fn close_shift(&mut self, stash: &mut Stash, table: &ItemTable) -> ShiftReport {
        self.close_shift_by(stash, table, &[])
    }

    /// As `close_shift`, each market's glut recovering faster by its Seep Market's bonus (`faster[m]`, points as a share).
    pub fn close_shift_by(&mut self, stash: &mut Stash, table: &ItemTable, faster: &[f32]) -> ShiftReport {
        stash.sweep_floor(table);
        self.glut.clear();
        for (m, g) in self.glut_by.iter_mut().enumerate() {
            // the price recovers 20 points a shift (Pass 1), four units' worth, and the Seep Market's bonus more
            let back = (GLUT_RECOVERY + faster.get(m).copied().unwrap_or(0.0)) / GLUT_PER_UNIT;
            for u in g.iter_mut() {
                *u = (*u - back).max(0.0);
            }
        }
        std::mem::take(&mut self.landed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance;

    const DATA: &str = include_str!("../../../../data/items.toml");

    fn put(table: &ItemTable, stash: &mut Stash, id: &str, condition: u8) -> crate::Handle {
        let (_, mut it) = instance(table, id).unwrap();
        it.condition = condition;
        stash.grid.auto_place(table.def(it.def), it).unwrap()
    }

    #[test]
    fn a_sale_pays_when_it_lands_at_the_glut_it_finds_and_recovers() {
        let table = ItemTable::from_toml(DATA).unwrap();
        let mut stash = Stash::new();
        let mut ledger = Ledger::new();
        let handles: Vec<_> = (0..6).map(|_| put(&table, &mut stash, "chr_street_eye", 100)).collect();
        let first = ledger.quote(&table, &stash.grid.get(handles[0]).unwrap().item, 0).unwrap();
        for h in &handles {
            assert!(ledger.queue(&mut stash, *h, Verb::Sell { market: 0 }, 1));
        }
        let ships = ledger.dispatch_queue();
        assert_eq!((ships.len(), ledger.in_flight.len(), ledger.queue.len()), (6, 6, 0));
        // nothing is paid on the Mat; the quote of a seventh counts the six riding ahead
        assert_eq!(ledger.eddies, 0);
        let h7 = put(&table, &mut stash, "chr_street_eye", 100);
        assert!(ledger.quote(&table, &stash.grid.get(h7).unwrap().item, 0).unwrap() < first);
        // the glut is the category's: a Part at the same market is quoted as if nothing rode there
        let part = put(&table, &mut stash, "prt_pump", 100);
        assert_eq!(ledger.quote(&table, &stash.grid.get(part).unwrap().item, 0), Some(10));
        for (id, to, _) in ships {
            assert!(matches!(ledger.land(&mut stash, &table, id, to), Some(Landing::Sold { market: 0, .. })));
        }
        let r = ledger.close_shift(&mut stash, &table);
        assert_eq!((r.sold, r.left), (6, 6));
        // 1 + .95 + .9 + .85 + .8 + .75 = 5.25 units of the first price, with rounding
        assert!(r.earned < first * 6 && r.earned >= (first as f32 * 5.0) as u32, "earned {} for first {}", r.earned, first);
        assert_eq!(ledger.eddies, r.earned);
        let g = ledger.glut_by[0][items::Category::Chrome as usize];
        assert!((g - 2.0).abs() < 1e-4, "glut {g}");
    }

    #[test]
    fn a_feed_delivers_nutrient_and_drift_where_it_lands() {
        let table = ItemTable::from_toml(DATA).unwrap();
        let mut stash = Stash::new();
        let mut ledger = Ledger::new();
        let hk = put(&table, &mut stash, "chr_street_eye", 100);
        let item = stash.grid.get(hk).unwrap().item.clone();
        ledger.queue(&mut stash, hk, Verb::Feed { pit: 2 }, 1);
        let ships = ledger.dispatch_queue();
        // re-routed on the way: it lands at Pit 1, and Pit 1 is fed
        let got = ledger.land(&mut stash, &table, ships[0].0, Dest::Pit(1)).unwrap();
        let (n, bias, drift) = feed_value(table.def(item.def), &item);
        let g = bias.iter().position(|b| *b != 0.0).expect("chrome carries a bias");
        match got {
            Landing::Fed { pit, nutrient, drift: d } => {
                assert_eq!(pit, 1);
                assert!((nutrient - n).abs() < 1e-6);
                assert!((d[g] - bias[g] * drift).abs() < 1e-6);
            }
            other => panic!("{other:?}"),
        }
        assert!(ledger.land(&mut stash, &table, ships[0].0, Dest::Pit(1)).is_none(), "it lands once");
        let r = ledger.close_shift(&mut stash, &table);
        assert_eq!((r.fed, r.deliveries.len(), r.deliveries[0].pit), (1, 1, 1));
    }

    #[test]
    fn a_quota_pulls_the_cheapest_lowest_tier_counts_what_rides_and_never_takes_a_pinned_item() {
        let table = ItemTable::from_toml(DATA).unwrap();
        let mut stash = Stash::new();
        let mut ledger = Ledger::new();
        let pinned = put(&table, &mut stash, "chr_street_eye", 100);
        stash.grid.get_mut(pinned).unwrap().item.pinned = true;
        for _ in 0..4 {
            put(&table, &mut stash, "chr_street_eye", 100);
        }
        put(&table, &mut stash, "prt_pipe", 100); // no feed value
        let eye = table.def(table.id_of("chr_street_eye").unwrap()).feed;
        // keep 2.5 × an eye's nutrient at Pit 0: three leave (the third crosses the line)
        let q = vec![Quota { to: Dest::Pit(0), want: Want::Nutrient { target: eye * 2.5, stock: 0.0, ceiling: Tier::Uncommon }, distance: Some(10.0), low: false }];
        let ships = ledger.pull(&mut stash, &table, &q);
        assert_eq!(ships.len(), 3);
        assert!(stash.grid.get(pinned).is_some(), "the pinned one stays");
        // the next shift's pull sees the three riding and sends nothing
        assert!(ledger.pull(&mut stash, &table, &q).is_empty());
        // a Blackline ceiling below the item's tier takes nothing; an unreached Pit takes nothing
        let mut ledger2 = Ledger::new();
        let none = vec![Quota { to: Dest::Pit(1), want: Want::Nutrient { target: 5.0, stock: 0.0, ceiling: Tier::Common }, distance: Some(5.0), low: false }, Quota { to: Dest::Pit(2), want: Want::Nutrient { target: 5.0, stock: 0.0, ceiling: Tier::Relic }, distance: None, low: false }];
        let tier = table.def(table.id_of("chr_street_eye").unwrap()).tier;
        let n = ledger2.pull(&mut stash, &table, &none).len();
        assert_eq!(n, if tier <= Tier::Common { 1 } else { 0 });
    }

    #[test]
    fn the_pull_serves_demand_per_tile_and_low_priority_last() {
        let table = ItemTable::from_toml(DATA).unwrap();
        let mut stash = Stash::new();
        let mut ledger = Ledger::new();
        put(&table, &mut stash, "chr_street_eye", 100);
        let eye = table.def(table.id_of("chr_street_eye").unwrap()).feed;
        let want = |target: f32| Want::Nutrient { target, stock: 0.0, ceiling: Tier::Relic };
        // one item, three Pits wanting: near and low-priority, far, nearer — the nearer normal one gets it
        let qs = vec![
            Quota { to: Dest::Pit(0), want: want(eye), distance: Some(2.0), low: true },
            Quota { to: Dest::Pit(1), want: want(eye), distance: Some(40.0), low: false },
            Quota { to: Dest::Pit(2), want: want(eye), distance: Some(12.0), low: false },
        ];
        let ships = ledger.pull(&mut stash, &table, &qs);
        assert_eq!(ships.iter().map(|s| s.1).collect::<Vec<_>>(), vec![Dest::Pit(2)]);
    }

    #[test]
    fn a_still_keeps_its_parts_and_a_product_rides_home_or_waits_at_the_heart() {
        let table = ItemTable::from_toml(DATA).unwrap();
        let mut stash = Stash::new();
        let mut ledger = Ledger::new();
        let enzyme = table.defs.iter().find(|d| d.category == items::Category::Part && d.tags.iter().any(|t| t == "enzyme")).map(|d| d.id.clone()).expect("an enzyme part");
        put(&table, &mut stash, &enzyme, 100);
        put(&table, &mut stash, &enzyme, 100);
        let q = vec![Quota { to: Dest::Still(9), want: Want::Tagged { tag: "enzyme".into(), target: 1, stock: 0 }, distance: Some(6.0), low: false }];
        let ships = ledger.pull(&mut stash, &table, &q);
        assert_eq!(ships.len(), 1, "keep one");
        ledger.land(&mut stash, &table, ships[0].0, Dest::Still(9));
        assert_eq!(ledger.store_count(9), 1);
        assert!(ledger.take_from_store(&table, 9, "enzyme").is_some());
        assert_eq!(ledger.store_count(9), 0);
        // a dose rides home; the grid takes it
        let (_, dose) = instance(&table, "org_downer").unwrap();
        let id = ledger.ship(dose, Dest::Stash);
        let before = stash.count();
        assert_eq!(ledger.land(&mut stash, &table, id, Dest::Stash), Some(Landing::Home { placed: true }));
        assert_eq!(stash.count(), before + 1);
        // a spill is lost and counted
        let (_, eye) = instance(&table, "chr_street_eye").unwrap();
        let id = ledger.ship(eye, Dest::Pit(0));
        assert!(ledger.spill(id).is_some());
        assert_eq!(ledger.close_shift(&mut stash, &table).spilled, 1);
    }
}
