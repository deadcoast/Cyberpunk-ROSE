//! GDExtension: exposes the ROSE core to Godot as `RoseSim`.
//! Godot owns the clock and the picture; this class owns every number.

use godot::prelude::*;
use logistics::city::state;
use mat::genome::Genome;
use mat::sim::{hy, sub};
use logistics::City;
use mat::MatSim;

struct RoseExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RoseExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseSim {
    base: Base<RefCounted>,
    city: Option<City>,
    last_step_us: i64,
}

#[godot_api]
impl RoseSim {
    /// Build the reference scale scenario: grow a stratum for `grow_ticks`, then populate it with `agents` slimos.
    #[func]
    fn start_scale(&mut self, seed: i64, agents: i64, grow_ticks: i64) {
        self.city = Some(logistics::scale_scenario(seed as u64, agents.max(0) as usize, grow_ticks.max(0) as u64));
    }

    /// Fresh stratum with a Heart at (x, y) and no city; feed it to make it grow.
    #[func]
    fn start_empty(&mut self, seed: i64, w: i64, h: i64, heart_x: i64, heart_y: i64) {
        let mut sim = MatSim::new(w.max(16) as usize, h.max(8) as usize, seed as u64, Genome::default());
        sim.seed_heart(heart_x as usize, heart_y as usize);
        self.city = Some(City::new(sim, 0, &[], seed as u64));
    }

    #[func]
    fn step(&mut self) -> i64 {
        let t0 = std::time::Instant::now();
        let changed = match self.city.as_mut() {
            Some(c) => {
                c.step();
                c.sim.changed.len() as i64
            }
            None => 0,
        };
        self.last_step_us = t0.elapsed().as_micros() as i64;
        changed
    }

    #[func]
    fn width(&self) -> i64 {
        self.city.as_ref().map_or(0, |c| c.sim.w as i64)
    }

    #[func]
    fn height(&self) -> i64 {
        self.city.as_ref().map_or(0, |c| c.sim.h as i64)
    }

    #[func]
    fn tick(&self) -> i64 {
        self.city.as_ref().map_or(0, |c| c.tick as i64)
    }

    /// Substrate per cell (see matsim::sim::sub), row-major.
    #[func]
    fn substrate(&self) -> PackedByteArray {
        self.city.as_ref().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.sim.sub.as_slice()))
    }

    /// Hyphal state per cell (0 empty, 1 tip, 2 cord, 3 node, 4 necrotic).
    #[func]
    fn hyphae(&self) -> PackedByteArray {
        self.city.as_ref().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.sim.hy.as_slice()))
    }

    /// Signal 0..1 per cell.
    #[func]
    fn signal(&self) -> PackedFloat32Array {
        self.city.as_ref().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.sig.as_slice()))
    }

    /// Nutrient 0..1 per cell.
    #[func]
    fn nutrient(&self) -> PackedFloat32Array {
        self.city.as_ref().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.n.as_slice()))
    }

    /// Cell indices whose hyphal state changed on the last step.
    #[func]
    fn changed_cells(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.sim.changed.iter().map(|&i| i as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Slimo positions as cell indices.
    #[func]
    fn agent_positions(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.pos.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Slimo states (0 home, 1 to market, 2 carrying to pit, 3 going home).
    #[func]
    fn agent_states(&self) -> PackedByteArray {
        self.city.as_ref().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.state.as_slice()))
    }

    /// Cargo packet positions as cell indices.
    #[func]
    fn packet_positions(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.pk_pos.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Traffic per cell (slimos + 2 x packets), what the Stack view draws as light.
    #[func]
    fn traffic(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.traffic().iter().map(|&t| t as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    #[func]
    fn sites(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        if let Some(c) = &self.city {
            let to_arr = |v: &Vec<usize>| PackedInt32Array::from(v.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice());
            let (m, p, h) = (to_arr(&c.markets), to_arr(&c.pits), to_arr(&c.pods));
            d.set("markets", &m);
            d.set("pits", &p);
            d.set("pods", &h);
        }
        d
    }

    // ------------------------------------------------------------ player verbs
    #[func]
    fn feed(&mut self, x: i64, y: i64, amount: f64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.feed(x as usize, y as usize, amount as f32);
            }
        }
    }

    #[func]
    fn bait(&mut self, x: i64, y: i64, amount: f64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.bait(x as usize, y as usize, amount as f32);
            }
        }
    }

    #[func]
    fn sever(&mut self, x: i64, y: i64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.sever(x as usize, y as usize);
            }
        }
    }

    #[func]
    fn salt(&mut self, x: i64, y: i64, on: bool) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.salt_tile(x as usize, y as usize, on);
            }
        }
    }

    #[func]
    fn place_relay(&mut self, x: i64, y: i64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.place_relay(x as usize, y as usize);
                c.pits.push(y as usize * c.sim.w + x as usize);
                c.rebuild_fields();
            }
        }
    }

    #[func]
    fn stats(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        if let Some(c) = &self.city {
            let s = c.sim.stats();
            let cs = c.stats();
            d.set("tick", s.tick as i64);
            d.set("tips", s.tips as i64);
            d.set("cords", s.cords as i64);
            d.set("nodes", s.nodes as i64);
            d.set("necrotic", s.necro as i64);
            d.set("cells", (s.tips + s.cords + s.nodes) as i64);
            d.set("reach", s.reach as i64);
            d.set("nutrient", s.nutrient as f64);
            d.set("agents", cs.agents as i64);
            d.set("carrying", cs.carrying as i64);
            d.set("packets", cs.packets as i64);
            d.set("deliveries", cs.deliveries as i64);
            d.set("hauls", cs.hauls as i64);
            d.set("jammed", cs.jammed_tiles as i64);
            d.set("step_us", self.last_step_us);
            let _ = (hy::EMPTY, sub::ROCK, state::HOME);
        }
        d
    }

    fn inside(c: &City, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < c.sim.w && (y as usize) < c.sim.h
    }
}

// ======================================================================= items and the Stash
use godot::classes::ProjectSettings;
use items::{ItemInstance, ItemTable, PriceContext};
use inventory::Stash;

const ITEMS_TOML: &str = include_str!("../../../data/items.toml");

fn table() -> ItemTable {
    // data/items.toml is compiled in so a built game carries its table; in debug the file beside the project wins
    let path = ProjectSettings::singleton().globalize_path("res://../data/items.toml").to_string();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| ITEMS_TOML.to_string());
    ItemTable::from_toml(&text).unwrap_or_else(|_| ItemTable::from_toml(ITEMS_TOML).expect("built-in item table"))
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
    out
}

/// The item table, read-only.
#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseItems {
    base: Base<RefCounted>,
    table: Option<ItemTable>,
}

#[godot_api]
impl RoseItems {
    fn t(&mut self) -> &ItemTable {
        if self.table.is_none() {
            self.table = Some(table());
        }
        self.table.as_ref().unwrap()
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
    table: Option<ItemTable>,
    stash: Option<Stash>,
}

#[godot_api]
impl RoseStash {
    fn ensure(&mut self) {
        if self.table.is_none() {
            self.table = Some(table());
        }
        if self.stash.is_none() {
            self.stash = Some(Stash::new());
        }
    }

    #[func]
    fn width(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().grid.w as i64
    }

    #[func]
    fn height(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().grid.h as i64
    }

    #[func]
    fn tier(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().tier as i64
    }

    #[func]
    fn upgrade(&mut self) -> bool {
        self.ensure();
        self.stash.as_mut().unwrap().upgrade()
    }

    /// Auto-place one item by table index; returns the handle or -1.
    #[func]
    fn add(&mut self, index: i64, condition: i64, found_in_run: bool) -> i64 {
        self.ensure();
        let t = self.table.as_ref().unwrap();
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
        match self.stash.as_mut().unwrap().grid.auto_place(def, inst) {
            Ok(h) => h as i64,
            Err(_) => -1,
        }
    }

    #[func]
    fn remove(&mut self, handle: i64) -> bool {
        self.ensure();
        self.stash.as_mut().unwrap().grid.remove(handle as u32).is_some()
    }

    #[func]
    fn pin(&mut self, handle: i64, on: bool) {
        self.ensure();
        if let Some(p) = self.stash.as_mut().unwrap().grid.get_mut(handle as u32) {
            p.item.pinned = on;
        }
    }

    /// Every placed item: handle, x, y, w, h, rotated, index, id, name, tier, condition, identified, pinned, sell, feed_cells.
    #[func]
    fn items(&mut self) -> VarArray {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let s = self.stash.as_ref().unwrap();
        let mut out = VarArray::new();
        let mut placed: Vec<_> = s.grid.iter().collect();
        placed.sort_by_key(|(h, _)| **h);
        for (h, p) in placed {
            let d = t.def(p.item.def);
            let mut e = VarDictionary::new();
            e.set("handle", *h as i64);
            e.set("x", p.x as i64);
            e.set("y", p.y as i64);
            e.set("w", p.w as i64);
            e.set("h", p.h as i64);
            e.set("rotated", p.rotated);
            e.set("index", p.item.def.0 as i64);
            e.set("id", d.id.as_str());
            e.set("name", d.name.as_str());
            e.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
            e.set("condition", p.item.condition as i64);
            e.set("identified", p.item.ident == items::Ident::Full);
            e.set("pinned", p.item.pinned);
            e.set("sell", items::price(d, &p.item, PriceContext::default()) as i64);
            e.set("feed", items::feed_value(d, &p.item).0 as f64);
            out.push(&e.to_variant());
        }
        out
    }

    #[func]
    fn count(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().count() as i64
    }

    #[func]
    fn free_cells(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().grid.free_cells() as i64
    }

    /// Serialise through the save crate and back; returns true when the round trip reproduces the Stash.
    #[func]
    fn save_round_trip(&mut self) -> bool {
        self.ensure();
        let mut file = save::SaveFile::new(1, 1);
        file.stash = self.stash.as_ref().unwrap().clone();
        let bytes = file.encode();
        match save::SaveFile::decode(&bytes) {
            Ok(back) => back.stash == file.stash,
            Err(_) => false,
        }
    }
}
