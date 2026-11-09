//! The ledger: the Stash's sell and feed queues, the eddies, per-market glut, and the cargo the Stash has out on the
//! Mat (Full Design v3.11, Stash: The four verbs; Colony mode: Cargo). At shift end the queue and the quotas' pulls
//! leave the Heart as shipments; each lands when its packet does. A sale pays base × condition × demand × glut × blind
//! at that market's glut at that moment (−5 % per unit already sold there this shift, floor 25 %, recovering 20 % a
//! shift); a feed delivers the item's nutrient (halved below condition 20) and a genome drift of bias × the tier's
//! drift to its Pit; a Part waits in its Still's store; a product rides home to the grid.

use items::{feed_value, price, ItemInstance, ItemTable, PriceContext, Tier};
use serde::{Deserialize, Serialize};

use crate::Stash;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verb {
    /// to the market with this index among the colony's market sites
    Sell { market: u8 },
    /// to the Pit with this index among the colony's Pits (0 is the Heart)
    Feed { pit: u8 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Queued {
    pub item: ItemInstance,
    pub verb: Verb,
    /// the colony day it was queued on
    pub day: u32,
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
}

/// An item on the Mat.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shipment {
    pub id: u32,
    pub item: ItemInstance,
    pub to: Dest,
}

/// What landed since the last shift end, and what has just left.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShiftReport {
    pub sold: u32,
    pub earned: u32,
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
    Sold { market: u8, eddies: u32 },
    Fed { pit: u8, nutrient: f32, drift: [f32; 10] },
    Stored { still: u32 },
    /// into the grid, or held at the Heart while the grid is full
    Home { placed: bool },
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
    /// units sold per market this shift (fractional after recovery)
    pub glut: Vec<f32>,
    pub lifetime_earned: u32,
    /// what the Stash has on the Mat (v9)
    #[serde(default)]
    pub in_flight: Vec<Shipment>,
    #[serde(default)]
    pub next_ship: u32,
    /// Parts landed at a Still, by structure id (v9)
    #[serde(default)]
    pub stores: Vec<(u32, Vec<ItemInstance>)>,
    /// products at the Heart waiting for room in the grid (v9)
    #[serde(default)]
    pub held: Vec<ItemInstance>,
    /// the report so far this shift (v9)
    #[serde(default)]
    pub landed: ShiftReport,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger::default()
    }

    fn glut_units(&self, market: u8) -> u32 {
        self.glut.get(market as usize).copied().unwrap_or(0.0).round() as u32
    }

    /// The price an item would fetch at a market now, with that market's glut and what is queued and riding there
    /// ahead of it.
    pub fn quote(&self, table: &ItemTable, item: &ItemInstance, market: u8) -> u32 {
        let ahead = self.queue.iter().filter(|q| q.verb == Verb::Sell { market }).count() + self.in_flight.iter().filter(|s| s.to == Dest::Market(market)).count();
        let ctx = PriceContext { glut_units: self.glut_units(market) + ahead as u32, ..PriceContext::default() };
        price(table.def(item.def), item, ctx)
    }

    /// Move an item out of the Stash grid into the queue. Returns false when the handle is missing.
    pub fn queue(&mut self, stash: &mut Stash, handle: crate::Handle, verb: Verb, day: u32) -> bool {
        let Some(item) = stash.grid.remove(handle) else { return false };
        self.queue.push(Queued { item, verb, day });
        true
    }

    /// Pull a queued item back into the grid; if the grid has no room it stays queued. Returns the new handle.
    pub fn unqueue(&mut self, stash: &mut Stash, table: &ItemTable, index: usize) -> Option<crate::Handle> {
        if index >= self.queue.len() {
            return None;
        }
        let item = self.queue[index].item.clone();
        match stash.grid.auto_place(table.def(item.def), item) {
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
        self.in_flight.push(Shipment { id, item, to });
        self.landed.left += 1;
        id
    }

    /// Shift end: the queue leaves the Heart. Returns the shipments for the city to dispatch.
    pub fn dispatch_queue(&mut self) -> Vec<(u32, Dest)> {
        let mut out = Vec::new();
        for q in std::mem::take(&mut self.queue) {
            let to = match q.verb {
                Verb::Sell { market } => Dest::Market(market),
                Verb::Feed { pit } => Dest::Pit(pit),
            };
            out.push((self.ship(q.item, to), to));
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
    pub fn pull(&mut self, stash: &mut Stash, table: &ItemTable, quotas: &[Quota]) -> Vec<(u32, Dest)> {
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
                if !stash.grid.items().any(|it| eligible(q, it)) {
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
            // the lowest tier, then the cheapest
            let pick = stash
                .grid
                .placed()
                .into_iter()
                .filter(|(_, p)| eligible(q, &p.item))
                .map(|(h, p)| (table.def(p.item.def).tier, price(table.def(p.item.def), &p.item, PriceContext::default()), h))
                .min();
            let Some((_, _, h)) = pick else { break };
            let Some(item) = stash.grid.remove(h) else { break };
            have[k] += match &q.want {
                Want::Nutrient { .. } => feed_value(table.def(item.def), &item).0,
                Want::Tagged { .. } => 1.0,
            };
            out.push((self.ship(item, q.to), q.to));
        }
        out
    }

    /// A shipment reached `at` (where the city's routing took it; a re-routed one lands where it arrived).
    pub fn land(&mut self, stash: &mut Stash, table: &ItemTable, id: u32, at: Dest) -> Option<Landing> {
        let k = self.in_flight.iter().position(|s| s.id == id)?;
        let s = self.in_flight.remove(k);
        Some(match at {
            Dest::Market(market) => {
                let m = market as usize;
                if self.glut.len() <= m {
                    self.glut.resize(m + 1, 0.0);
                }
                let ctx = PriceContext { glut_units: self.glut[m].round() as u32, ..PriceContext::default() };
                let paid = price(table.def(s.item.def), &s.item, ctx);
                self.glut[m] += 1.0;
                self.eddies += paid;
                self.lifetime_earned += paid;
                self.landed.sold += 1;
                self.landed.earned += paid;
                Landing::Sold { market, eddies: paid }
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
            Dest::Stash => {
                self.landed.home += 1;
                match stash.grid.auto_place(table.def(s.item.def), s.item.clone()) {
                    Ok(_) => Landing::Home { placed: true },
                    Err(_) => {
                        self.held.push(s.item);
                        Landing::Home { placed: false }
                    }
                }
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
    /// what just left; products held at the Heart try the grid again; this shift's glut recovers by 20 %.
    pub fn close_shift(&mut self, stash: &mut Stash, table: &ItemTable) -> ShiftReport {
        let mut keep = Vec::new();
        for it in std::mem::take(&mut self.held) {
            if let Err(_) = stash.grid.auto_place(table.def(it.def), it.clone()) {
                keep.push(it);
            }
        }
        self.held = keep;
        for g in &mut self.glut {
            *g = (*g * (1.0 - GLUT_RECOVERY)).max(0.0);
            if *g < 0.05 {
                *g = 0.0;
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
        let first = ledger.quote(&table, &stash.grid.get(handles[0]).unwrap().item, 0);
        for h in &handles {
            assert!(ledger.queue(&mut stash, *h, Verb::Sell { market: 0 }, 1));
        }
        let ships = ledger.dispatch_queue();
        assert_eq!((ships.len(), ledger.in_flight.len(), ledger.queue.len()), (6, 6, 0));
        // nothing is paid on the Mat; the quote of a seventh counts the six riding ahead
        assert_eq!(ledger.eddies, 0);
        let h7 = put(&table, &mut stash, "chr_street_eye", 100);
        assert!(ledger.quote(&table, &stash.grid.get(h7).unwrap().item, 0) < first);
        for (id, to) in ships {
            assert!(matches!(ledger.land(&mut stash, &table, id, to), Some(Landing::Sold { market: 0, .. })));
        }
        let r = ledger.close_shift(&mut stash, &table);
        assert_eq!((r.sold, r.left), (6, 6));
        // 1 + .95 + .9 + .85 + .8 + .75 = 5.25 units of the first price, with rounding
        assert!(r.earned < first * 6 && r.earned >= (first as f32 * 5.0) as u32, "earned {} for first {}", r.earned, first);
        assert_eq!(ledger.eddies, r.earned);
        assert!((ledger.glut[0] - 4.8).abs() < 1e-4, "glut {}", ledger.glut[0]);
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
