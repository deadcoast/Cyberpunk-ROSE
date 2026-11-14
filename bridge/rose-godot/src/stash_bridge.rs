//! `RoseItems` (the item table for the pickers) and `RoseStash`: the city's one Stash, its ledger and its Bench as
//! the Stash, Bench and Market screens read and change them. A view onto the World `RoseSim` owns.

use super::*;

use items::{ItemInstance, ItemTable, PriceContext};
use inventory::Stash;


pub(crate) fn table() -> ItemTable {
    data().items.clone()
}

fn def_dict(t: &ItemTable, idx: usize) -> VarDictionary {
    let d = &t.defs[idx];
    let mut out = VarDictionary::new();
    out.set("index", idx as i64);
    out.set("id", d.id.as_str());
    out.set("name", d.name.as_str());
    out.set("category", format!("{:?}", d.category).to_lowercase().as_str());
    out.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
    out.set("w", d.size[0] as i64);
    out.set("h", d.size[1] as i64);
    out.set("weight", d.weight as f64);
    out.set("value", d.value as i64);
    out.set("feed", d.feed as f64);
    out.set("codex", d.codex.as_str());
    let tags = PackedStringArray::from(d.tags.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice());
    out.set("tags", &tags);
    // gear: slot ("hand", "rig", "head", ...), kind ("melee", "ranged", "armour", "tool", "implant"), the numbers
    if let Some(g) = &d.gear {
        out.set("gear_slot", g.slot.as_str());
        out.set("gear_kind", g.kind.as_str());
        out.set("dmg", g.dmg as i64);
        out.set("armour", g.armour as i64);
        out.set("light", g.light as i64);
        out.set("gear_noise", g.noise as i64);
    }
    out
}

/// The item table, read-only.
#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseItems {
    base: Base<RefCounted>,
}

#[godot_api]
impl RoseItems {
    /// The process's item table (`game::data`).
    fn t(&self) -> &'static ItemTable {
        &data().items
    }

    #[func]
    fn count(&mut self) -> i64 {
        self.t().len() as i64
    }

    #[func]
    fn get(&mut self, index: i64) -> VarDictionary {
        let t = self.t();
        if index < 0 || index as usize >= t.len() {
            return VarDictionary::new();
        }
        def_dict(t, index as usize)
    }

    #[func]
    fn find(&mut self, id: GString) -> i64 {
        self.t().id_of(&id.to_string()).map_or(-1, |i| i.0 as i64)
    }

    /// Indices of every item in a category ("chrome", "gear", ...).
    #[func]
    fn by_category(&mut self, category: GString) -> PackedInt32Array {
        let t = self.t();
        let want = category.to_string();
        let v: Vec<i32> = (0..t.len()).filter(|&i| format!("{:?}", t.defs[i].category).to_lowercase() == want).map(|i| i as i32).collect();
        PackedInt32Array::from(v.as_slice())
    }
}

/// A Stash backed by the inventory crate; the Stash screen is a view of this.
#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseStash {
    base: Base<RefCounted>,
    /// The sim whose World holds the city's one Stash, its ledger and its Bench (Phase G, PAC-101): this object is
    /// a view onto them for the Stash, Bench and Market screens, and holds no item of its own (`attach`).
    pub(crate) sim: Option<Gd<RoseSim>>,
}

/// One item as the Stash screen and the pickers read it (no placement: the caller adds x, y, w, h).
pub(crate) fn item_dict(t: &ItemTable, h: u32, item: &ItemInstance) -> VarDictionary {
    let d = t.def(item.def);
    let mut e = VarDictionary::new();
    e.set("handle", h as i64);
    e.set("index", item.def.0 as i64);
    e.set("id", d.id.as_str());
    e.set("name", d.name.as_str());
    e.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
    e.set("condition", item.condition as i64);
    e.set("identified", item.ident == items::Ident::Full);
    e.set("pinned", item.pinned);
    // a recover contract's item (Economy): sent from the Market screen, it counts where it lands
    e.set("bound", item.bound.map_or(-1, |c| c as i64));
    e.set("sell", items::price(d, item, PriceContext::default()) as i64);
    e.set("feed", items::feed_value(d, item).0 as f64);
    e.set("category", format!("{:?}", d.category).to_lowercase().as_str());
    e.set("weight", d.weight as f64);
    e.set("found", item.found_in_run);
    e.set("spored", item.spored);
    if let Some(c) = &item.crafted {
        e.set("crafted_by", format!("{} on day {}", c.by, c.day).as_str());
        e.set("crafted_from", &PackedStringArray::from(c.parts.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
    }
    if let Some(st) = item.gear_stats(d) {
        let mut sd = VarDictionary::new();
        sd.set("dmg", st.dmg as i64);
        sd.set("armour", st.armour as i64);
        sd.set("noise", st.noise as i64);
        sd.set("light", st.light as i64);
        sd.set("mag", st.mag as i64);
        e.set("stats", &sd);
    }
    e.set("tags", &PackedStringArray::from(d.tags.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
    e.set("codex", d.codex.as_str());
    e
}

/// The Stash's side of a closed shift as the report the screens read.
pub(crate) fn closed_dict(w: &game::World, c: &game::store::Closed) -> VarDictionary {
    let mut d = report_dict(&c.report);
    d.set("crafted", c.placed as i64);
    d.set("crafted_names", &PackedStringArray::from(c.crafted.iter().map(|n| GString::from(n.as_str())).collect::<Vec<_>>().as_slice()));
    d.set("waiting", c.waiting as i64);
    d.set("eddies", w.ledger.eddies as i64);
    d.set("held", w.stash.floor.len() as i64);
    d
}

fn report_dict(r: &inventory::ShiftReport) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("sold", r.sold as i64);
    d.set("earned", r.earned as i64);
    d.set("fed", r.fed as i64);
    d.set("stored", r.stored as i64);
    d.set("home", r.home as i64);
    d.set("spilled", r.spilled as i64);
    d.set("cut", r.cut as i64);
    d.set("left", r.left as i64);
    let mut dl = VarArray::new();
    for de in &r.deliveries {
        let mut e = VarDictionary::new();
        e.set("pit", de.pit as i64);
        e.set("nutrient", de.nutrient as f64);
        e.set("drift", &PackedFloat32Array::from(de.drift.as_slice()));
        dl.push(&e.to_variant());
    }
    d.set("deliveries", &dl);
    d
}

fn parts_for(stash: &Stash, handles: &PackedInt32Array) -> Option<Vec<ItemInstance>> {
    let mut parts = Vec::new();
    for h in handles.as_slice() {
        parts.push(stash.get_any(*h as u32)?.clone());
    }
    Some(parts)
}

fn outcome_dict(t: &ItemTable, o: &bench::Outcome) -> VarDictionary {
    let d = t.def(o.product.def);
    let mut out = VarDictionary::new();
    out.set("ok", true);
    out.set("product_id", d.id.as_str());
    out.set("product_name", d.name.as_str());
    out.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
    out.set("condition", o.product.condition as i64);
    out.set("count", o.count as i64);
    out.set("shifts", o.shifts as i64);
    out.set("spored", o.product.spored);
    out.set("identified", o.product.ident == items::Ident::Full);
    let stat_dict = |g: &items::GearStats| {
        let mut sd = VarDictionary::new();
        sd.set("dmg", g.dmg as i64);
        sd.set("armour", g.armour as i64);
        sd.set("noise", g.noise as i64);
        sd.set("light", g.light as i64);
        sd.set("mag", g.mag as i64);
        sd
    };
    if let Some(b) = &o.base {
        out.set("base", &stat_dict(b));
    }
    if let Some(st) = &o.stats {
        out.set("stats", &stat_dict(st));
    }
    if let Some(c) = &o.product.crafted {
        out.set("by", c.by.as_str());
        out.set("parts", &PackedStringArray::from(c.parts.iter().map(|p| GString::from(p.as_str())).collect::<Vec<_>>().as_slice()));
    }
    out
}

#[godot_api]
impl RoseStash {
    /// The sim whose World this view reads and changes: the city's one Stash, its ledger and its Bench.
    #[func]
    fn attach(&mut self, sim: Gd<RoseSim>) {
        self.sim = Some(sim);
    }

    // ------------------------------------------------------------ the Bench

    /// Every recipe: [{id, name, tier, product, product_name, count, shifts, open, slots: [{tag, drives}]}].
    #[func]
    fn recipes(&mut self) -> VarArray {
        let Some(mut sim) = self.sim.clone() else { return VarArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarArray::new() };
        let t = &w.table;
        let f = &w.recipes;
        let mut out = VarArray::new();
        for r in &f.recipe {
            let mut d = VarDictionary::new();
            d.set("id", r.id.as_str());
            d.set("name", r.name.as_str());
            d.set("tier", r.tier as i64);
            d.set("product", r.product.as_str());
            let pname = if let Some(job) = r.product.strip_prefix('@') {
                match job {
                    "repair" => "the gear, +40 condition".to_string(),
                    "identify" => "the item, identified".to_string(),
                    _ => "the gear, Spored".to_string(),
                }
            } else {
                t.get(&r.product).map(|d| d.name.clone()).unwrap_or_default()
            };
            d.set("product_name", pname.as_str());
            d.set("count", r.count as i64);
            d.set("shifts", f.shifts(r) as i64);
            d.set("open", w.bench.is_open(r));
            let mut slots = VarArray::new();
            for sl in &r.slots {
                let mut e = VarDictionary::new();
                e.set("tag", sl.tag.as_str());
                e.set("drives", sl.drives.clone().unwrap_or_default().as_str());
                slots.push(&e.to_variant());
            }
            d.set("slots", &slots);
            out.push(&d.to_variant());
        }
        out
    }

    /// Grid handles that fill a slot with this tag, best tier first.
    #[func]
    fn candidates(&mut self, tag: GString) -> PackedInt32Array {
        let Some(mut sim) = self.sim.clone() else { return PackedInt32Array::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return PackedInt32Array::new() };
        let t = &w.table;
        let slot = bench::Slot { tag: tag.to_string(), drives: None };
        let mut v: Vec<(i32, u32)> = (&w.stash)
            .grid
            .placed()
            .into_iter()
            .filter(|(_, p)| bench::accepts(&slot, t.def(p.item.def), &p.item))
            .map(|(h, p)| (h as i32, t.def(p.item.def).tier as u32))
            .collect();
        v.sort_by_key(|(h, tier)| (std::cmp::Reverse(*tier), *h));
        PackedInt32Array::from(v.iter().map(|(h, _)| *h).collect::<Vec<i32>>().as_slice())
    }



    /// The outcome of a recipe on these grid handles (one per slot), without crafting: {ok, error?, product_id,
    /// product_name, tier, condition, count, shifts, base {dmg, armour, noise, light, mag}, stats {...}, by, parts}.
    #[func]
    fn preview(&mut self, recipe: GString, handles: PackedInt32Array, by: GString, day: i64) -> VarDictionary {
        let Some(mut sim) = self.sim.clone() else { return VarDictionary::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarDictionary::new() };
        let mut err = VarDictionary::new();
        err.set("ok", false);
        let Some(r) = (&w.recipes).get(&recipe.to_string()).cloned() else {
            err.set("error", "no such recipe");
            return err;
        };
        let Some(parts) = parts_for(&w.stash, &handles) else {
            err.set("error", "a part is missing from the Stash");
            return err;
        };
        let t = &w.table;
        match bench::outcome(t, &w.recipes, &r, &parts, &by.to_string(), day.max(0) as u32) {
            Ok(o) => outcome_dict(&w.table, &o),
            Err(e) => {
                let msg = match e {
                    bench::CraftError::WrongPartCount { want, got } => format!("{want} slots, {got} parts"),
                    bench::CraftError::SlotRefuses { slot, tag, item } => format!("slot {} ({}) refuses {}", slot + 1, tag, t.get(&item).map(|d| d.name.as_str()).unwrap_or(&item)),
                    bench::CraftError::UnknownProduct(p) => format!("no such product {p}"),
                    bench::CraftError::IdentifyNeedsUnidentified => "that item is already identified".to_string(),
                };
                err.set("error", msg.as_str());
                err
            }
        }
    }

    /// Queue the craft: the parts leave the grid. False when the recipe is closed or the slotting is wrong.
    #[func]
    fn craft(&mut self, recipe: GString, handles: PackedInt32Array, by: GString, day: i64) -> bool {
        let Some(mut sim) = self.sim.clone() else { return false };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return false };
        let Some(r) = (&w.recipes).get(&recipe.to_string()).cloned() else { return false };
        if !w.bench.is_open(&r) {
            return false;
        }
        let Some(parts) = parts_for(&w.stash, &handles) else { return false };
        let t = (&w.table).clone();
        if bench::outcome(&t, &w.recipes, &r, &parts, &by.to_string(), day.max(0) as u32).is_err() {
            return false;
        }
        let stash = &mut w.stash;
        for h in handles.as_slice() {
            stash.take(*h as u32);
        }
        let f = &w.recipes;
        w.bench.queue(f, &r, parts, &by.to_string(), day.max(0) as u32);
        true
    }

    /// The Bench queue: [{index, recipe, name, shifts_left, minutes_left, parts: [names]}], then whatever waits for
    /// Stash room.
    #[func]
    fn bench_jobs(&mut self) -> VarArray {
        let Some(mut sim) = self.sim.clone() else { return VarArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarArray::new() };
        let t = &w.table;
        let f = &w.recipes;
        let mut out = VarArray::new();
        for (i, j) in w.bench.jobs.iter().enumerate() {
            let mut d = VarDictionary::new();
            d.set("index", i as i64);
            d.set("recipe", j.recipe.as_str());
            d.set("name", f.get(&j.recipe).map(|r| r.name.as_str()).unwrap_or("?"));
            d.set("shifts_left", j.shifts_left as i64);
            // the minutes of a Tweaker's slot it still asks (Full Design v3.21: a shift of its time is ten minutes)
            d.set("minutes_left", (j.shifts_left as i64 * bench::MINUTES_A_SHIFT as i64 - j.minutes as i64).max(0));
            d.set("parts", &PackedStringArray::from(j.parts.iter().map(|p| GString::from(t.def(p.def).name.as_str())).collect::<Vec<_>>().as_slice()));
            out.push(&d.to_variant());
        }
        out
    }

    /// The best open recipe this grid item can go into (highest product value), for the Stash card's third number:
    /// {id, name, tier, product_name, slot_tag} or empty.
    #[func]
    fn best_recipe(&mut self, handle: i64) -> VarDictionary {
        let Some(mut sim) = self.sim.clone() else { return VarDictionary::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarDictionary::new() };
        let mut out = VarDictionary::new();
        let t = &w.table;
        let Some(item) = (&w.stash).get_any(handle as u32) else { return out };
        let d = t.def(item.def);
        let mut best: Option<(&bench::Recipe, &bench::Slot, u32)> = None;
        for r in &(&w.recipes).recipe {
            if !w.bench.is_open(r) {
                continue;
            }
            let Some(slot) = r.slots.iter().find(|s| bench::accepts(s, d, item)) else { continue };
            let value = t.get(&r.product).map(|pd| pd.value).unwrap_or(d.value);
            if best.is_none_or(|(_, _, v)| value > v) {
                best = Some((r, slot, value));
            }
        }
        if let Some((r, slot, _)) = best {
            out.set("id", r.id.as_str());
            out.set("name", r.name.as_str());
            out.set("tier", r.tier as i64);
            out.set("product_name", t.get(&r.product).map(|pd| pd.name.as_str()).unwrap_or(r.name.as_str()));
            out.set("slot_tag", slot.tag.as_str());
        }
        out
    }

    #[func]
    fn bench_waiting(&self) -> i64 {
        let Some(sim) = self.sim.clone() else { return 0 };
        let g = sim.bind();
        let Some(w) = g.world.as_ref() else { return 0 };
        w.bench.waiting.len() as i64
    }

    #[func]
    fn tier_open(&self) -> i64 {
        let Some(sim) = self.sim.clone() else { return 0 };
        let g = sim.bind();
        let Some(w) = g.world.as_ref() else { return 0 };
        w.bench.tier_open.max(1) as i64
    }

    /// Cancel a job: its parts come back to the grid (those that fit; the rest wait at the Bench).
    #[func]
    fn cancel_job(&mut self, index: i64) -> bool {
        let Some(mut sim) = self.sim.clone() else { return false };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return false };
        let parts = w.bench.cancel(index.max(0) as usize);
        if parts.is_empty() {
            return false;
        }
        let t = (&w.table).clone();
        let stash = &mut w.stash;
        for p in parts {
            if stash.grid.auto_place(t.def(p.def), p.clone()).is_err() {
                w.bench.waiting.push(p);
            }
        }
        true
    }

    // ------------------------------------------------------------ the four verbs and the ledger

    /// Queue an item to sell at a market (index among the colony's market sites). It leaves the grid.
    #[func]
    fn queue_sell(&mut self, handle: i64, market: i64, day: i64) -> bool {
        let Some(mut sim) = self.sim.clone() else { return false };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return false };
        let stash = &mut w.stash;
        w.ledger.queue(stash, handle as u32, inventory::Verb::Sell { market: market.clamp(0, 255) as u8 }, day.max(0) as u32)
    }

    /// Queue an item to feed a Pit (0 is the Heart). Only items with a feed value go; returns false otherwise.
    #[func]
    fn queue_feed(&mut self, handle: i64, pit: i64, day: i64) -> bool {
        let Some(mut sim) = self.sim.clone() else { return false };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return false };
        let t = &w.table;
        let stash = &mut w.stash;
        let Some(item) = stash.get_any(handle as u32) else { return false };
        if t.def(item.def).feed <= 0.0 {
            return false;
        }
        w.ledger.queue(stash, handle as u32, inventory::Verb::Feed { pit: pit.clamp(0, 255) as u8 }, day.max(0) as u32)
    }

    /// Pull a queued item back into the grid; -1 when it is missing or there is no room.
    #[func]
    fn unqueue(&mut self, index: i64) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        let t = (&w.table).clone();
        let stash = &mut w.stash;
        w.ledger.unqueue(stash, &t, index.max(0) as usize).map_or(-1, |h| h as i64)
    }

    /// The queue: [{index, id, name, tier, identified, verb: "sell"|"feed", target, quote, nutrient, day}]
    #[func]
    fn queued(&mut self) -> VarArray {
        let Some(mut sim) = self.sim.clone() else { return VarArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarArray::new() };
        let t = &w.table;
        let mut out = VarArray::new();
        for (i, q) in w.ledger.queue.iter().enumerate() {
            let d = t.def(q.item.def);
            let mut e = VarDictionary::new();
            e.set("index", i as i64);
            e.set("id", d.id.as_str());
            e.set("name", d.name.as_str());
            e.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
            e.set("identified", q.item.ident == items::Ident::Full);
            e.set("w", d.size[0] as i64);
            e.set("h", d.size[1] as i64);
            match q.verb {
                inventory::Verb::Sell { market } => {
                    e.set("verb", "sell");
                    e.set("target", market as i64);
                    // the price it will fetch given what of its category is queued ahead of it at that market; -1 when
                    // no stall there buys it
                    let cat = d.category;
                    let ahead = w.ledger.queue[..i].iter().filter(|p| p.verb == q.verb && t.def(p.item.def).category == cat).count() as u32;
                    let glut = w.ledger.glut_units(market, cat as usize) + ahead;
                    e.set("quote", w.ledger.best(t, &q.item, market, glut).map_or(-1, |(p, _)| p as i64));
                    e.set("nutrient", 0.0);
                }
                inventory::Verb::Deliver { market, contract } => {
                    e.set("verb", "deliver");
                    e.set("target", market as i64);
                    e.set("contract", contract as i64);
                    e.set("quote", 0i64);
                    e.set("nutrient", 0.0);
                }
                inventory::Verb::Feed { pit } => {
                    e.set("verb", "feed");
                    e.set("target", pit as i64);
                    e.set("quote", 0i64);
                    e.set("nutrient", items::feed_value(d, &q.item).0 as f64);
                }
            }
            e.set("day", q.day as i64);
            out.push(&e.to_variant());
        }
        out
    }

    /// What an item in the grid would fetch at a market now (glut and the queue ahead included); -1 when no stall
    /// there buys it (a Relic; Blackline outside the Corpo remnants' tier 3; a faction closed to the city).
    #[func]
    fn quote(&mut self, handle: i64, market: i64) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        let t = &w.table;
        let stash = &w.stash;
        stash.get_any(handle as u32).map_or(0, |it| w.ledger.quote(t, it, market.clamp(0, 255) as u8).map_or(-1, |p| p as i64))
    }

    #[func]
    fn eddies(&self) -> i64 {
        let Some(sim) = self.sim.clone() else { return 0 };
        let g = sim.bind();
        let Some(w) = g.world.as_ref() else { return 0 };
        w.ledger.eddies as i64
    }

    /// Re-pack the grid by "tier", "value", "weight", "condition" or "category".
    #[func]
    fn sort_by(&mut self, key: GString) {
        let Some(mut sim) = self.sim.clone() else { return };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return };
        let t = (&w.table).clone();
        (&mut w.stash).sort_by(&t, &key.to_string());
    }

    #[func]
    fn queue_count(&self) -> i64 {
        let Some(sim) = self.sim.clone() else { return 0 };
        let g = sim.bind();
        let Some(w) = g.world.as_ref() else { return 0 };
        w.ledger.queue.len() as i64
    }

    /// Unpinned items in the grid that match the filters: category ("" any), tier ("" any), condition below
    /// `max_condition` (101 any), unidentified only. Pinned items never match.
    #[func]
    fn bulk_handles(&mut self, category: GString, tier: GString, max_condition: i64, unidentified: bool) -> PackedInt32Array {
        let Some(mut sim) = self.sim.clone() else { return PackedInt32Array::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return PackedInt32Array::new() };
        let t = &w.table;
        let stash = &w.stash;
        let (cat, tr) = (category.to_string(), tier.to_string());
        let mut v: Vec<i32> = stash
            .grid
            .placed()
            .into_iter()
            .filter(|(_, p)| !p.item.pinned)
            .filter(|(_, p)| {
                let d = t.def(p.item.def);
                (cat.is_empty() || format!("{:?}", d.category).to_lowercase() == cat)
                    && (tr.is_empty() || format!("{:?}", d.tier).to_lowercase() == tr)
                    && (p.item.condition as i64) < max_condition
                    && (!unidentified || p.item.ident != items::Ident::Full)
            })
            .map(|(h, _)| h as i32)
            .collect();
        v.sort_unstable();
        PackedInt32Array::from(v.as_slice())
    }

    /// What has landed since the last shift end (the report so far) and what the Stash has on the Mat:
    /// {sold, earned, fed, stored, home, spilled, cut, left, deliveries, riding, held}.
    #[func]
    fn landed(&self) -> VarDictionary {
        let Some(sim) = self.sim.clone() else { return VarDictionary::new() };
        let g = sim.bind();
        let Some(w) = g.world.as_ref() else { return VarDictionary::new() };
        let mut d = report_dict(&w.ledger.landed);
        d.set("riding", w.ledger.in_flight.len() as i64);
        d.set("held", Some(&w.stash).map_or(0, |s| s.floor.len()) as i64);
        d
    }

    #[func]
    fn width(&mut self) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        (&w.stash).grid.w as i64
    }

    #[func]
    fn height(&mut self) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        (&w.stash).grid.h as i64
    }

    #[func]
    fn tier(&mut self) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        (&w.stash).tier as i64
    }

    /// Auto-place one item by table index; returns the handle or -1.
    #[func]
    fn add(&mut self, index: i64, condition: i64, found_in_run: bool) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        let t = &w.table;
        if index < 0 || index as usize >= t.len() {
            return -1;
        }
        let def = &t.defs[index as usize];
        let mut inst = ItemInstance::new(items::ItemId(index as u16));
        inst.condition = condition.clamp(0, 100) as u8;
        inst.found_in_run = found_in_run;
        if found_in_run {
            inst.ident = items::Ident::Unknown;
        }
        match (&mut w.stash).grid.auto_place(def, inst) {
            Ok(h) => h as i64,
            Err(_) => -1,
        }
    }

    #[func]
    fn remove(&mut self, handle: i64) -> bool {
        let Some(mut sim) = self.sim.clone() else { return false };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return false };
        (&mut w.stash).take(handle as u32).is_some()
    }

    #[func]
    fn pin(&mut self, handle: i64, on: bool) {
        let Some(mut sim) = self.sim.clone() else { return };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return };
        if let Some(it) = (&mut w.stash).get_any_mut(handle as u32) {
            it.pinned = on;
        }
    }

    /// Every placed item: handle, x, y, w, h, rotated, index, id, name, tier, condition, identified, pinned, sell, feed_cells.
    #[func]
    fn items(&mut self) -> VarArray {
        let Some(mut sim) = self.sim.clone() else { return VarArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarArray::new() };
        let t = w.table.clone();
        let placed: Vec<(u32, inventory::Placed)> = (&w.stash).grid.placed().into_iter().map(|(h, p)| (h, p.clone())).collect();
        let mut out = VarArray::new();
        for (h, p) in placed {
            let mut e = item_dict(&t, h, &p.item);
            e.set("x", p.x as i64);
            e.set("y", p.y as i64);
            e.set("w", p.w as i64);
            e.set("h", p.h as i64);
            e.set("rotated", p.rotated);
            out.push(&e.to_variant());
        }
        out
    }

    #[func]
    fn count(&mut self) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        (&w.stash).count() as i64
    }

    #[func]
    fn free_cells(&mut self) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        (&w.stash).grid.free_cells() as i64
    }

    /// Take a Runner's rig (RoseRun.rig_export) into the Stash grid, best value first. Returns how many did not fit.
    #[func]
    fn import_rig(&mut self, bytes: PackedByteArray) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        let Ok(rig) = postcard::from_bytes::<inventory::Rig>(bytes.to_vec().as_slice()) else { return -1 };
        let t = w.table.clone();
        let items = game::store::rig_items(&rig, &t);
        // what the grid cannot hold lies on the Heart's floor (game::store::take_haul)
        let (_, held) = game::store::take_haul(&mut w.stash, &t, items);
        held as i64
    }

    /// Take loose items (postcard Vec<ItemInstance>: a Mat haul, regrown gear) into the Stash; what the grid cannot
    /// hold lies on the Heart's floor (game::store::take_haul). Returns how many.
    #[func]
    fn import_items(&mut self, bytes: PackedByteArray) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        let Ok(items) = postcard::from_bytes::<Vec<ItemInstance>>(bytes.to_vec().as_slice()) else { return -1 };
        let t = w.table.clone();
        let (_, held) = game::store::take_haul(&mut w.stash, &t, items);
        held as i64
    }

    /// Pack a Runner's rig from the Stash: the hand weapon by handle (-1 for none), up to four quick slots and any
    /// number of backpack items by handle. Packed items leave the Stash; a handle that is missing or does not fit is
    /// skipped. Returns the rig serialised (postcard) for `RoseRun.begin_run_rig`, or empty when nothing packed.
    #[func]
    fn pack_rig(&mut self, hand: i64, quick: PackedInt32Array, pack: PackedInt32Array, rows: i64, carry_kg: f64) -> PackedByteArray {
        let Some(mut sim) = self.sim.clone() else { return PackedByteArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return PackedByteArray::new() };
        let t = (&w.table).clone();
        let stash = &mut w.stash;
        let mut rig = inventory::Rig::new(8, rows.clamp(3, 8) as u8, 4, carry_kg.max(1.0) as f32);
        let mut any = false;
        if hand >= 0 {
            if let Some(it) = stash.grid.remove(hand as u32) {
                rig.gear.hands[0] = Some(it);
                any = true;
            }
        }
        for (i, h) in quick.as_slice().iter().take(4).enumerate() {
            if *h < 0 {
                continue;
            }
            let Some(p) = stash.grid.get(*h as u32) else { continue };
            // quick slots take items of one or two cells
            if (p.w as u32) * (p.h as u32) > 2 {
                continue;
            }
            if let Some(it) = stash.grid.remove(*h as u32) {
                rig.quick[i] = Some(it);
                any = true;
            }
        }
        for h in pack.as_slice() {
            if *h < 0 {
                continue;
            }
            let Some(it) = stash.grid.remove(*h as u32) else { continue };
            if let Err(_) = rig.backpack.auto_place(t.def(it.def), it.clone()) {
                // no room: back into the Stash
                let _ = stash.grid.auto_place(t.def(it.def), it);
            } else {
                any = true;
            }
        }
        if !any {
            return PackedByteArray::new();
        }
        PackedByteArray::from(postcard::to_allocvec(&rig).unwrap_or_default().as_slice())
    }

    /// Serialise through the save crate and back; returns true when the round trip reproduces the Stash.
    #[func]
    fn save_round_trip(&mut self) -> bool {
        let Some(mut sim) = self.sim.clone() else { return false };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return false };
        let mut file = save::SaveFile::new(1, 1);
        file.stash = (&w.stash).clone();
        file.ledger = w.ledger.clone();
        file.bench = w.bench.clone();
        let bytes = file.encode();
        match save::SaveFile::decode(&bytes) {
            Ok(back) => back.stash == file.stash && back.ledger == file.ledger && back.bench == file.bench,
            Err(_) => false,
        }
    }
}

// ---------------------------------------------------------------- the run
