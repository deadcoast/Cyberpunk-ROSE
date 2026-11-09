//! The ledger: the Stash's sell and feed queues, the eddies, per-market glut, and the shift-end resolution
//! (Full Design v3.4, Stash: The four verbs, Shift end). Sell pays base × condition × demand × glut × blind; glut
//! is −5 % per unit already sold at that market this shift, floor 25 %, recovering 20 % per shift. Feed delivers
//! the item's nutrient (halved below condition 20) and a genome drift of bias × the tier's drift to a Pit; the
//! colony applies the deliveries, the ledger only computes them.

use items::{feed_value, price, ItemInstance, ItemTable, PriceContext};
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

/// What a shift's resolution hands the colony.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShiftReport {
    pub sold: u32,
    pub earned: u32,
    pub fed: u32,
    pub deliveries: Vec<Delivery>,
}

/// Nutrient and genome drift for one Pit.
#[derive(Clone, Debug, PartialEq)]
pub struct Delivery {
    pub pit: u8,
    pub nutrient: f32,
    /// summed bias × drift per gene; the colony adds it to the genome and clamps
    pub drift: [f32; 10],
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
}

impl Ledger {
    pub fn new() -> Self {
        Ledger::default()
    }

    fn glut_units(&self, market: u8) -> u32 {
        self.glut.get(market as usize).copied().unwrap_or(0.0).round() as u32
    }

    /// The price an item would fetch at a market now, with that market's glut and the queue ahead of it.
    pub fn quote(&self, table: &ItemTable, item: &ItemInstance, market: u8) -> u32 {
        let ahead = self.queue.iter().filter(|q| q.verb == Verb::Sell { market }).count() as u32;
        let ctx = PriceContext { glut_units: self.glut_units(market) + ahead, ..PriceContext::default() };
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

    /// Shift end: sell everything queued to sell (glut rising per unit as it goes), hand every feed to its Pit as a
    /// delivery, recover last shift's glut by 20 %.
    pub fn resolve(&mut self, table: &ItemTable) -> ShiftReport {
        let mut report = ShiftReport::default();
        let mut deliveries: Vec<Delivery> = Vec::new();
        for q in std::mem::take(&mut self.queue) {
            match q.verb {
                Verb::Sell { market } => {
                    let m = market as usize;
                    if self.glut.len() <= m {
                        self.glut.resize(m + 1, 0.0);
                    }
                    let ctx = PriceContext { glut_units: self.glut[m].round() as u32, ..PriceContext::default() };
                    let paid = price(table.def(q.item.def), &q.item, ctx);
                    self.glut[m] += 1.0;
                    self.eddies += paid;
                    self.lifetime_earned += paid;
                    report.sold += 1;
                    report.earned += paid;
                }
                Verb::Feed { pit } => {
                    let (nutrient, bias, drift) = feed_value(table.def(q.item.def), &q.item);
                    let d = deliveries.iter_mut().find(|d| d.pit == pit);
                    let d = match d {
                        Some(d) => d,
                        None => {
                            deliveries.push(Delivery { pit, nutrient: 0.0, drift: [0.0; 10] });
                            deliveries.last_mut().unwrap()
                        }
                    };
                    d.nutrient += nutrient;
                    for g in 0..10 {
                        d.drift[g] += bias[g] * drift;
                    }
                    report.fed += 1;
                }
            }
        }
        for g in &mut self.glut {
            *g = (*g * (1.0 - GLUT_RECOVERY)).max(0.0);
            if *g < 0.05 {
                *g = 0.0;
            }
        }
        report.deliveries = deliveries;
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance;

    const DATA: &str = include_str!("../../../../data/items.toml");

    #[test]
    fn selling_the_same_thing_at_one_market_pays_less_each_unit_and_recovers() {
        let table = ItemTable::from_toml(DATA).unwrap();
        let mut stash = Stash::new();
        let mut ledger = Ledger::new();
        let mut handles = Vec::new();
        for _ in 0..6 {
            let (_, mut it) = instance(&table, "chr_street_eye").unwrap();
            it.condition = 100;
            handles.push(stash.grid.auto_place(table.def(it.def), it).unwrap());
        }
        let first = ledger.quote(&table, stash.grid.get(handles[0]).map(|p| &p.item).unwrap(), 0);
        for h in &handles {
            assert!(ledger.queue(&mut stash, *h, Verb::Sell { market: 0 }, 1));
        }
        assert_eq!(stash.count(), 0);
        let r = ledger.resolve(&table);
        assert_eq!(r.sold, 6);
        // 1 + .95 + .9 + .85 + .8 + .75 = 5.25 units of the first price, with rounding
        assert!(r.earned < first * 6 && r.earned >= (first as f32 * 5.0) as u32, "earned {} for first {}", r.earned, first);
        assert_eq!(ledger.eddies, r.earned);
        // next shift: glut has recovered a fifth
        assert!((ledger.glut[0] - 4.8).abs() < 1e-4, "glut {}", ledger.glut[0]);
    }

    #[test]
    fn unidentified_sells_blind_and_feed_delivers_nutrient_and_drift() {
        let table = ItemTable::from_toml(DATA).unwrap();
        let mut stash = Stash::new();
        let mut ledger = Ledger::new();
        let (_, mut blind) = instance(&table, "chr_street_eye").unwrap();
        blind.condition = 100;
        blind.ident = items::Ident::Unknown;
        let (_, mut known) = instance(&table, "chr_street_eye").unwrap();
        known.condition = 100;
        let hb = stash.grid.auto_place(table.def(blind.def), blind).unwrap();
        let hk = stash.grid.auto_place(table.def(known.def), known).unwrap();
        let qb = ledger.quote(&table, &stash.grid.get(hb).unwrap().item, 0);
        let qk = ledger.quote(&table, &stash.grid.get(hk).unwrap().item, 0);
        assert_eq!(qb, (qk as f32 * 0.4).round() as u32);
        // feed the known one to the Heart; pull the blind one back
        ledger.queue(&mut stash, hk, Verb::Feed { pit: 0 }, 1);
        ledger.queue(&mut stash, hb, Verb::Sell { market: 1 }, 1);
        assert!(ledger.unqueue(&mut stash, &table, 1).is_some());
        assert_eq!(stash.count(), 1);
        let r = ledger.resolve(&table);
        assert_eq!((r.sold, r.fed), (0, 1));
        let d = &r.deliveries[0];
        let (n, bias, drift) = feed_value(table.def(table.id_of("chr_street_eye").unwrap()), &stash.grid.items().next().unwrap().clone());
        assert_eq!(d.pit, 0);
        assert!((d.nutrient - n).abs() < 1e-6);
        let g = bias.iter().position(|b| *b != 0.0).expect("chrome carries a bias");
        assert!((d.drift[g] - bias[g] * drift).abs() < 1e-6);
    }
}
