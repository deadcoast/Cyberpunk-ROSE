//! The quota card (Full Design v3.11, Quotas) as the Colony screen reads it. The cargo's rules are in `game::cargo`.

use colony::{Colony, QuotaFrom, StructureFile};
use game::cargo::{pit_stock, TIERS};
use inventory::Dest;
use logistics::{City, Kind};

use crate::RoseStash;

/// A Pit's or Still's quota for its card: kind, target, ceiling, whose, the stock now, what rides to it, the cap
/// (a Pit's stock; a Still has none, -1).
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
            e.set("cap", -1.0);
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
