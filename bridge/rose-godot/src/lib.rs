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

    /// The colony strain's gene value (0..1) by index (mat::genome constants); the Run reads Lume (4).
    #[func]
    fn gene(&self, index: i64) -> f64 {
        self.city.as_ref().map_or(0.5, |c| c.sim.genome.0.get(index as usize).copied().unwrap_or(0.5) as f64)
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

    /// Take a Runner's rig (RoseRun.rig_export) into the Stash grid, best value first. Returns how many did not fit.
    #[func]
    fn import_rig(&mut self, bytes: PackedByteArray) -> i64 {
        self.ensure();
        let Ok(rig) = postcard::from_bytes::<inventory::Rig>(bytes.to_vec().as_slice()) else { return -1 };
        let t = self.table.as_ref().unwrap();
        let mut items: Vec<ItemInstance> = rig.backpack.items().cloned().collect();
        items.extend(rig.quick.iter().flatten().cloned());
        let g = &rig.gear;
        items.extend([&g.head, &g.body, &g.feet, &g.back].into_iter().flatten().cloned());
        items.extend(g.hands.iter().flatten().cloned());
        items.sort_by_key(|it| std::cmp::Reverse(t.def(it.def).value));
        let stash = self.stash.as_mut().unwrap();
        let mut left = 0i64;
        for it in items {
            let def = t.def(it.def);
            if stash.grid.auto_place(def, it).is_err() {
                left += 1;
            }
        }
        left
    }

    /// Take loose items (postcard Vec<ItemInstance>: a Mat haul, regrown gear) into the Stash. Returns how many did
    /// not fit.
    #[func]
    fn import_items(&mut self, bytes: PackedByteArray) -> i64 {
        self.ensure();
        let Ok(items) = postcard::from_bytes::<Vec<ItemInstance>>(bytes.to_vec().as_slice()) else { return -1 };
        let t = self.table.as_ref().unwrap();
        let stash = self.stash.as_mut().unwrap();
        let mut left = 0i64;
        for it in items {
            if stash.grid.auto_place(t.def(it.def), it).is_err() {
                left += 1;
            }
        }
        left
    }

    /// Pack a Runner's rig from the Stash: the hand weapon by handle (-1 for none), up to four quick slots and any
    /// number of backpack items by handle. Packed items leave the Stash; a handle that is missing or does not fit is
    /// skipped. Returns the rig serialised (postcard) for `RoseRun.begin_run_rig`, or empty when nothing packed.
    #[func]
    fn pack_rig(&mut self, hand: i64, quick: PackedInt32Array, pack: PackedInt32Array) -> PackedByteArray {
        self.ensure();
        let t = self.table.as_ref().unwrap().clone();
        let stash = self.stash.as_mut().unwrap();
        let mut rig = inventory::Rig::new(8, 5, 4, 18.0);
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

// ---------------------------------------------------------------- the run

const CHUNKS_TOML: &str = include_str!("../../../data/chunks.toml");

/// Data files beside the project win in debug; the compiled copies ship.
fn data_text(name: &str, shipped: &str) -> String {
    let path = ProjectSettings::singleton().globalize_path(&format!("res://../data/{name}")).to_string();
    std::fs::read_to_string(&path).unwrap_or_else(|_| shipped.to_string())
}

fn chunk_lib() -> run::ChunkLib {
    run::ChunkLib::from_toml(&data_text("chunks.toml", CHUNKS_TOML)).unwrap_or_else(|_| run::ChunkLib::from_toml(CHUNKS_TOML).expect("built-in chunk library"))
}

fn loot_tables() -> run::LootTables {
    run::LootTables::from_toml(&data_text("loot.toml", run::loot::LOOT_TOML)).unwrap_or_else(|_| run::LootTables::shipped())
}

fn creature_defs() -> run::CreatureFile {
    run::CreatureFile::from_toml(&data_text("creatures.toml", run::creature::CREATURES_TOML)).unwrap_or_else(|_| run::CreatureFile::shipped())
}

fn source_state(s: run::SourceState) -> &'static str {
    match s {
        run::SourceState::Closed => "closed",
        run::SourceState::Open => "open",
        run::SourceState::Empty => "empty",
    }
}

/// An item as the Runner sees it in the run: unidentified items keep their name hidden behind the tier and category.
fn item_view(table: &ItemTable, k: usize, it: &ItemInstance) -> VarDictionary {
    let d = table.def(it.def);
    let mut out = VarDictionary::new();
    let tier = format!("{:?}", d.tier).to_lowercase();
    let cat = format!("{:?}", d.category).to_lowercase();
    out.set("k", k as i64);
    out.set("index", it.def.0 as i64);
    out.set("id", d.id.as_str());
    let label = if it.ident == items::Ident::Full { d.name.clone() } else { format!("unidentified {} {}", tier, cat) };
    out.set("label", label.as_str());
    out.set("tier", tier.as_str());
    out.set("category", cat.as_str());
    out.set("w", d.size[0] as i64);
    out.set("h", d.size[1] as i64);
    out.set("weight", d.weight as f64);
    out.set("condition", it.condition as i64);
    out.set("ident", it.ident == items::Ident::Full);
    out.set("found", it.found_in_run);
    out.set("value_blind", items::price(d, it, PriceContext::default()) as i64);
    out
}

fn tiles_array(w: usize, v: &[(u16, u16)]) -> PackedInt32Array {
    PackedInt32Array::from(v.iter().map(|&(x, y)| (y as usize * w + x as usize) as i32).collect::<Vec<i32>>().as_slice())
}

/// The run. `assemble` builds the route; `begin_run` starts the simulation on it; `step` advances it one tick
/// (20 Hz) from the input word; everything else reads state. Godot decides nothing.
#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseRun {
    base: Base<RefCounted>,
    lib: Option<run::ChunkLib>,
    map: Option<run::RouteMap>,
    sim: Option<run::RunSim>,
    mat_lume_gene: f32,
    memory: Option<Vec<u8>>,
}

#[godot_api]
impl RoseRun {
    fn ensure_lib(&mut self) -> &run::ChunkLib {
        if self.lib.is_none() {
            self.lib = Some(chunk_lib());
        }
        self.lib.as_ref().unwrap()
    }

    /// Assemble a route. `chunks` 0 draws the count from the seed (8-14). Returns the chunk count, 0 on failure.
    #[func]
    fn assemble(&mut self, seed: i64, stratum: i64, chunks: i64) -> i64 {
        let lib = self.ensure_lib().clone();
        let lay = if chunks <= 0 { run::route(&lib, seed as u64, stratum as u8) } else { run::assemble(&lib, seed as u64, stratum as u8, chunks as usize) };
        match lay {
            Ok(lay) => {
                let m = run::RouteMap::compile(&lib, &lay);
                let n = m.chunks.len() as i64;
                self.map = Some(m);
                self.sim = None;
                n
            }
            Err(e) => {
                godot_error!("RoseRun.assemble: {e}");
                0
            }
        }
    }

    /// Lume gene of the Mat the route starts from (0..1); set before `begin_run`.
    #[func]
    fn set_mat_lume(&mut self, gene: f64) {
        self.mat_lume_gene = gene.clamp(0.0, 1.0) as f32;
    }

    /// Start this run's fog from a previous run's memory of the route (bits from `memory_bits`); before `begin_run`.
    #[func]
    fn restore_memory(&mut self, bits: PackedByteArray) {
        self.memory = Some(bits.to_vec());
    }

    /// Start the run: creatures spawn, loot sources arm, the Runner stands on the Mat edge with the kit in the quick
    /// slots and the melee weapon in hand. `nonce` makes this run's contents, `route_runs` is how many runs this route
    /// has had before (0: first run), `day` goes into provenance; `kit` is up to four item ids for the quick slots.
    /// `followers`: [{id, name, body, damage}] up to two. `corpses`: [{x, y, chunk, day, rig: PackedByteArray,
    /// raided}] of our dead on this route (from the save); the raider roll runs here.
    #[func]
    fn begin_run(&mut self, nonce: i64, route_runs: i64, day: i64, kit: PackedStringArray, weapon: GString, body: i64, followers: VarArray, corpses: VarArray) {
        let Some(m) = self.map.clone() else { return };
        let table = table();
        let mut rig = inventory::Rig::new(8, 5, 4, 18.0);
        for (i, id) in kit.as_slice().iter().take(4).enumerate() {
            if let Some((_, inst)) = inventory::instance(&table, &id.to_string()) {
                rig.quick[i] = Some(inst);
            }
        }
        if let Some((_, inst)) = inventory::instance(&table, &weapon.to_string()) {
            rig.gear.hands[0] = Some(inst);
        }
        let fs = Self::followers_from(&followers);
        let cs = Self::corpses_from(&corpses);
        self.start(m, table, nonce, route_runs, day, rig, body, fs, cs);
    }

    /// Start the run with a rig packed on the Route Board (`RoseStash.pack_rig` bytes); otherwise as `begin_run`.
    #[func]
    fn begin_run_rig(&mut self, nonce: i64, route_runs: i64, day: i64, rig: PackedByteArray, body: i64, followers: VarArray, corpses: VarArray) {
        let Some(m) = self.map.clone() else { return };
        let table = table();
        let rig = postcard::from_bytes::<inventory::Rig>(rig.to_vec().as_slice()).unwrap_or_else(|_| inventory::Rig::new(8, 5, 4, 18.0));
        let fs = Self::followers_from(&followers);
        let cs = Self::corpses_from(&corpses);
        self.start(m, table, nonce, route_runs, day, rig, body, fs, cs);
    }

    fn followers_from(followers: &VarArray) -> Vec<run::FollowerSetup> {
        let mut fs = Vec::new();
        for v in followers.iter_shared() {
            let Ok(d) = v.try_to::<VarDictionary>() else { continue };
            fs.push(run::FollowerSetup {
                id: d.get("id").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u32,
                name: d.get("name").map(|v| v.to_string()).unwrap_or_default(),
                body: d.get("body").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(5) as u8,
                damage: d.get("damage").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(4) as i32,
            });
        }
        fs
    }

    fn corpses_from(corpses: &VarArray) -> Vec<run::Corpse> {
        let mut cs = Vec::new();
        for v in corpses.iter_shared() {
            let Ok(d) = v.try_to::<VarDictionary>() else { continue };
            let bytes = d.get("rig").and_then(|v| v.try_to::<PackedByteArray>().ok()).map(|b| b.to_vec()).unwrap_or_default();
            let Ok(rig) = postcard::from_bytes::<inventory::Rig>(&bytes) else { continue };
            cs.push(run::Corpse {
                x: d.get("x").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u16,
                y: d.get("y").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u16,
                chunk: d.get("chunk").map(|v| v.to_string()).unwrap_or_default(),
                day: d.get("day").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u32,
                rig,
                raided: d.get("raided").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false),
            });
        }
        cs
    }

    #[allow(clippy::too_many_arguments)]
    fn start(&mut self, m: run::RouteMap, table: ItemTable, nonce: i64, route_runs: i64, day: i64, rig: inventory::Rig, body: i64, fs: Vec<run::FollowerSetup>, cs: Vec<run::Corpse>) {
        let setup = run::RunSetup {
            mat_lume_gene: self.mat_lume_gene,
            memory: self.memory.as_deref(),
            run_nonce: nonce as u64,
            route_runs: route_runs.clamp(0, u16::MAX as i64) as u16,
            day: day.max(0) as u32,
            rig,
            body: body.clamp(1, 10) as u8,
            followers: fs,
            corpses: cs,
        };
        self.sim = Some(run::RunSim::new(m, table, loot_tables(), creature_defs(), setup));
    }

    /// The route card for the assembled route (before or during a run): {chunks, shift_s, danger, danger_band,
    /// burrows, patrol_chunks, known (0-1), known_lockers, known_crates, water (0-1), grates}. `memory` is the
    /// route's memory bits from the save (empty before any run).
    #[func]
    fn card(&self, memory: PackedByteArray) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(m) = self.map_ref() else { return d };
        let bits = memory.to_vec();
        let c = run::card(m, &creature_defs(), if bits.is_empty() { None } else { Some(bits.as_slice()) });
        d.set("chunks", c.chunks as i64);
        d.set("shift_s", c.shift_seconds() as i64);
        d.set("danger", c.danger as f64);
        d.set("density", c.density as f64);
        d.set("danger_band", c.danger_band());
        d.set("burrows", c.burrows as i64);
        d.set("patrol_chunks", c.patrol_chunks as i64);
        d.set("known", c.known as f64);
        d.set("known_lockers", c.known_lockers as i64);
        d.set("known_crates", c.known_crates as i64);
        d.set("water", c.water as f64);
        d.set("grates", c.grates as i64);
        d
    }

    /// The shift clock: {shift_ticks, tick, left_ticks, overtime_ticks}.
    #[func]
    fn shift(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(s) = self.sim.as_ref() else { return d };
        d.set("shift_ticks", s.shift_ticks as i64);
        d.set("tick", s.tick as i64);
        d.set("left_ticks", s.shift_ticks.saturating_sub(s.tick) as i64);
        d.set("overtime_ticks", s.overtime_ticks() as i64);
        d
    }

    /// One byte per chunk in placement order: 1 where any tile has been seen or remembered.
    #[func]
    fn chunk_seen(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.chunk_seen().iter().map(|&b| b as u8).collect::<Vec<u8>>().as_slice()))
    }

    /// Container tiles of the route: [{x, y, kind: "locker"|"crate", tier, chunk}] (the Board draws the known ones).
    #[func]
    fn containers(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for c in &m.containers {
                let mut d = VarDictionary::new();
                d.set("x", c.x as i64);
                d.set("y", c.y as i64);
                d.set("kind", if c.kind == run::ContainerKind::Locker { "locker" } else { "crate" });
                d.set("tier", c.tier as i64);
                d.set("chunk", c.chunk as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// Followers: [{id, name, x, y, tx, ty, facing, hp, max_hp, order, life, carrying, windup}]
    #[func]
    fn followers(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        for f in &s.followers {
            let mut d = VarDictionary::new();
            d.set("id", f.id as i64);
            d.set("name", f.name.as_str());
            d.set("x", f.x as i64);
            d.set("y", f.y as i64);
            let (tx, ty) = f.tile();
            d.set("tx", tx as i64);
            d.set("ty", ty as i64);
            d.set("facing", f.facing as i64);
            d.set("hp", f.hp as i64);
            d.set("max_hp", f.max_hp as i64);
            d.set("order", f.order.name());
            d.set("life", match f.life {
                run::FollowerLife::Up => "up",
                run::FollowerLife::Down { .. } => "down",
                run::FollowerLife::Dead => "dead",
            });
            d.set("carrying", f.carrying.is_some());
            d.set("windup", f.windup_left > 0);
            out.push(&d.to_variant());
        }
        out
    }

    /// The run's outcome once it is over: {result, ticks, rig: bytes, corpse: {x, y, chunk, day, rig: bytes} or null,
    /// followers_extracted, followers_stranded, followers_dead, mat_haul: bytes (Vec<ItemInstance>), regrown: bytes,
    /// kills, chunks_seen, blind_value, route_memory: bytes}. Empty while running.
    #[func]
    fn outcome(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(o) = self.sim.as_ref().and_then(|s| s.outcome.as_ref()) else { return d };
        d.set("result", match o.result {
            run::RunResult::Extracted { .. } => "extracted",
            run::RunResult::Died { .. } => "died",
        });
        d.set("ticks", o.ticks as i64);
        d.set("rig", &PackedByteArray::from(o.rig.as_ref().map(|r| postcard::to_allocvec(r).unwrap_or_default()).unwrap_or_default().as_slice()));
        if let Some(c) = &o.corpse {
            let mut cd = VarDictionary::new();
            cd.set("x", c.x as i64);
            cd.set("y", c.y as i64);
            cd.set("chunk", c.chunk.as_str());
            cd.set("day", c.day as i64);
            cd.set("rig", &PackedByteArray::from(postcard::to_allocvec(&c.rig).unwrap_or_default().as_slice()));
            cd.set("raided", c.raided);
            d.set("corpse", &cd);
        }
        let ids = |v: &Vec<u32>| PackedInt32Array::from(v.iter().map(|&i| i as i32).collect::<Vec<i32>>().as_slice());
        d.set("followers_extracted", &ids(&o.followers_extracted));
        d.set("followers_stranded", &ids(&o.followers_stranded));
        d.set("followers_dead", &ids(&o.followers_dead));
        d.set("mat_haul", &PackedByteArray::from(postcard::to_allocvec(&o.mat_haul).unwrap_or_default().as_slice()));
        d.set("mat_haul_count", o.mat_haul.len() as i64);
        d.set("regrown", &PackedByteArray::from(postcard::to_allocvec(&o.regrown).unwrap_or_default().as_slice()));
        d.set("regrown_count", o.regrown.len() as i64);
        d.set("kills", o.kills as i64);
        d.set("chunks_seen", o.chunks_seen as i64);
        d.set("blind_value", o.blind_value as i64);
        d.set("route_memory", &PackedByteArray::from(o.route_memory.as_slice()));
        d
    }

    /// One tick from the input word (see run::sim::RunInput::pack). Returns false when the run is over.
    #[func]
    fn step(&mut self, buttons: i64, dx: i64, dy: i64) -> bool {
        let Some(sim) = self.sim.as_mut() else { return false };
        sim.step(run::RunInput::unpack(buttons as u16, dx as i16, dy as i16));
        sim.result.is_none()
    }

    /// Development only: put the Runner on a tile (tests and the screenshot script). Not an input; never replayed.
    #[func]
    fn debug_teleport(&mut self, x: i64, y: i64) -> bool {
        let Some(s) = self.sim.as_mut() else { return false };
        if x < 0 || y < 0 || x as usize >= s.map.w || y as usize >= s.map.h || !s.map.passable(x as usize, y as usize) {
            return false;
        }
        s.runner.x = x as i32 * run::creature::FP + run::creature::FP / 2;
        s.runner.y = y as i32 * run::creature::FP + run::creature::FP / 2;
        true
    }

    /// "running", "extracted" or "died".
    #[func]
    fn result(&self) -> GString {
        GString::from(match self.sim.as_ref().and_then(|s| s.result) {
            None => "running",
            Some(run::RunResult::Extracted { .. }) => "extracted",
            Some(run::RunResult::Died { .. }) => "died",
        })
    }

    #[func]
    fn tick(&self) -> i64 {
        self.sim.as_ref().map_or(0, |s| s.tick as i64)
    }

    #[func]
    fn library_size(&mut self) -> i64 {
        self.ensure_lib().chunks.len() as i64
    }

    fn map_ref(&self) -> Option<&run::RouteMap> {
        self.sim.as_ref().map(|s| &s.map).or(self.map.as_ref())
    }

    #[func]
    fn width(&self) -> i64 {
        self.map_ref().map_or(0, |m| m.w as i64)
    }

    #[func]
    fn height(&self) -> i64 {
        self.map_ref().map_or(0, |m| m.h as i64)
    }

    #[func]
    fn chunk_count(&self) -> i64 {
        self.map_ref().map_or(0, |m| m.chunks.len() as i64)
    }

    /// Substrate codes (mat::sim::sub) per tile, live: digging changes it.
    #[func]
    fn substrate(&self) -> PackedByteArray {
        self.map_ref().map_or_else(PackedByteArray::new, |m| PackedByteArray::from(m.substrate.as_slice()))
    }

    #[func]
    fn passable(&self) -> PackedByteArray {
        self.map_ref().map_or_else(PackedByteArray::new, |m| PackedByteArray::from(m.passable_mask().iter().map(|&p| p as u8).collect::<Vec<u8>>().as_slice()))
    }

    #[func]
    fn entry(&self) -> PackedInt32Array {
        self.map_ref().map_or_else(PackedInt32Array::new, |m| tiles_array(m.w, &m.entry))
    }

    #[func]
    fn grates(&self) -> PackedInt32Array {
        self.map_ref().map_or_else(PackedInt32Array::new, |m| tiles_array(m.w, &m.grates))
    }

    /// [{x, y, kind: "sewerat"|"patrol"|"corpse", chunk}]
    #[func]
    fn spawns(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for s in &m.spawns {
                let mut d = VarDictionary::new();
                d.set("x", s.x as i64);
                d.set("y", s.y as i64);
                d.set("kind", match s.kind {
                    run::SpawnKind::Sewerat => "sewerat",
                    run::SpawnKind::Patrol => "patrol",
                    run::SpawnKind::Corpse => "corpse",
                });
                d.set("chunk", s.chunk as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// [{x, y, radius}]
    #[func]
    fn lights(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for l in &m.lights {
                let mut d = VarDictionary::new();
                d.set("x", l.x as i64);
                d.set("y", l.y as i64);
                d.set("radius", l.radius as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// [{x, y, kind: "gas"|"collapse"}]
    #[func]
    fn hazards(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for h in &m.hazards {
                let mut d = VarDictionary::new();
                d.set("x", h.x as i64);
                d.set("y", h.y as i64);
                d.set("kind", if h.kind == run::HazardKind::Gas { "gas" } else { "collapse" });
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// [{id, kind, stratum, x0, y0}] in placement order; 0 is the entry chunk.
    #[func]
    fn chunks(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for c in &m.chunks {
                let mut d = VarDictionary::new();
                d.set("id", c.id.as_str());
                d.set("kind", c.kind.as_str());
                d.set("stratum", c.stratum as i64);
                d.set("x0", c.x0 as i64);
                d.set("y0", c.y0 as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    #[func]
    fn chunk_of(&self) -> PackedInt32Array {
        self.map_ref().map_or_else(PackedInt32Array::new, |m| {
            PackedInt32Array::from(m.chunk_of.iter().map(|&c| if c == run::map::NO_CHUNK { -1 } else { c as i32 }).collect::<Vec<i32>>().as_slice())
        })
    }

    #[func]
    fn connectivity(&self) -> f64 {
        self.map_ref().map_or(0.0, |m| m.connectivity() as f64)
    }

    #[func]
    fn digest(&self) -> GString {
        GString::from(self.map_ref().map_or_else(String::new, |m| m.digest()).as_str())
    }

    // ------------------------------------------------------------ the Runner

    /// {x, y (units of 1/256 tile), tx, ty, facing, sections [4], max_sections [4], stamina (0-100), life, down_left,
    ///  action, action_progress (0-1), statuses [names], lume, see_radius, exposure, exposure_band, notice_range,
    ///  noise_here, lantern, dimming, sneaking, sprinting, moved, ammo, overweight, weight, carry, rig_count,
    ///  rig_free, blind_value, on_extraction, source_here, open_source}
    #[func]
    fn runner(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(s) = self.sim.as_ref() else { return d };
        let r = &s.runner;
        d.set("x", r.x as i64);
        d.set("y", r.y as i64);
        let (tx, ty) = r.tile();
        d.set("tx", tx as i64);
        d.set("ty", ty as i64);
        d.set("facing", r.facing as i64);
        d.set("sections", &PackedInt32Array::from(r.sections.map(|v| v as i32).as_slice()));
        d.set("max_sections", &PackedInt32Array::from(r.max_sections.map(|v| v as i32).as_slice()));
        d.set("stamina", (r.stamina as f64) / 100.0);
        d.set("max_stamina", (r.max_stamina as f64) / 100.0);
        let (life, down_left) = match r.life {
            run::Life::Up => ("up", 0),
            run::Life::Down { left } => ("down", left),
            run::Life::Dead => ("dead", 0),
        };
        d.set("life", life);
        d.set("down_left", down_left as i64);
        let rules = &s.defs.runner;
        let (action, progress) = match r.action {
            run::Action::Idle => ("idle", 0.0),
            run::Action::Windup { left } => ("windup", 1.0 - left as f64 / run::creature::ticks(rules.melee_windup).max(1) as f64),
            run::Action::Swing { .. } => ("swing", 1.0),
            run::Action::Dodge { left, .. } => ("dodge", 1.0 - left as f64 / run::creature::ticks(rules.dodge_s).max(1) as f64),
            run::Action::Reload { left } => ("reload", 1.0 - left as f64 / run::creature::ticks(2.0) as f64),
            run::Action::Dig { progress, .. } => ("dig", progress as f64 / run::creature::ticks(rules.dig_s) as f64),
            run::Action::Extract { progress } => ("extract", progress as f64 / run::creature::ticks(rules.extract_s) as f64),
        };
        d.set("action", action);
        d.set("action_progress", progress.clamp(0.0, 1.0));
        let st = PackedStringArray::from(r.statuses.iter().map(|st| GString::from(st.kind.name())).collect::<Vec<_>>().as_slice());
        d.set("statuses", &st);
        d.set("lume", s.senses.lume.emit() as i64);
        d.set("see_radius", s.senses.lume.see_radius(s.senses.eye) as i64);
        d.set("exposure", s.senses.exposure as i64);
        d.set("exposure_band", run::LightField::band(s.senses.exposure));
        d.set("notice_range", s.senses.notice_range() as i64);
        d.set("noise_here", s.senses.noise.at(tx as usize, ty as usize) as i64);
        d.set("lantern", r.lantern_on);
        d.set("dimming", r.dimming);
        d.set("sneaking", r.sneaking);
        d.set("sprinting", r.sprinting);
        d.set("moved", r.moved);
        d.set("pose", r.pose.name());
        d.set("ammo", r.ammo as i64);
        d.set("overweight", r.rig.overweight(&s.table));
        d.set("weight", r.rig.weight(&s.table) as f64);
        d.set("carry", r.rig.carry_cap as f64);
        d.set("rig_count", r.rig.backpack.count() as i64);
        d.set("rig_free", r.rig.backpack.free_cells() as i64);
        let blind: i64 = r.rig.backpack.items().map(|it| items::price(s.table.def(it.def), it, PriceContext::default()) as i64).sum();
        d.set("blind_value", blind);
        let t16 = (tx as u16, ty as u16);
        d.set("on_extraction", s.map.entry.contains(&t16) || s.map.grates.contains(&t16));
        d.set("source_here", s.source_here().map_or(-1, |i| i as i64));
        d.set("open_source", s.open_source.map_or(-1, |i| i as i64));
        d.set("melee", {
            let (dmg, _, reach) = r.melee(&s.table);
            format!("{dmg} dmg reach {reach}").as_str()
        });
        d.set("hand", r.rig.gear.hands[0].as_ref().map_or("", |h| s.table.def(h.def).id.as_str()));
        d.set("chunk", s.runner_chunk().map_or(-1, |c| c as i64));
        d
    }

    /// Live creatures: [{id, kind, sprite, x, y, tx, ty, facing, hp, max_hp, phase, awareness, windup, visible}]
    #[func]
    fn creatures(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        for c in &s.creatures {
            let mut d = VarDictionary::new();
            d.set("id", c.id as i64);
            d.set("kind", c.kind.key());
            d.set("sprite", s.defs.creature[c.kind.key()].sprite.as_str());
            d.set("x", c.x as i64);
            d.set("y", c.y as i64);
            let (tx, ty) = c.tile();
            d.set("tx", tx as i64);
            d.set("ty", ty as i64);
            d.set("facing", c.facing as i64);
            d.set("hp", c.hp as i64);
            d.set("max_hp", c.max_hp as i64);
            d.set("phase", c.phase.name());
            d.set("awareness", c.awareness as f64);
            d.set("windup", c.windup_left > 0 || c.fuse_left > 0);
            d.set("visible", s.senses.is_visible(tx as usize, ty as usize));
            d.set("heard", !s.senses.is_visible(tx as usize, ty as usize) && c.phase != run::Phase::Calm);
            out.push(&d.to_variant());
        }
        out
    }

    /// Events since the last call, drained: [{kind, ...fields}]
    #[func]
    fn events(&mut self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_mut() else { return out };
        for e in s.drain_events() {
            let mut d = VarDictionary::new();
            match e {
                run::Event::RunnerHit { section, damage, by } => {
                    d.set("kind", "runner_hit");
                    d.set("section", run::runner::SECTION_NAMES[section]);
                    d.set("damage", damage as i64);
                    d.set("by", by as i64);
                }
                run::Event::Bleed { damage } => {
                    d.set("kind", "bleed");
                    d.set("damage", damage as i64);
                }
                run::Event::CreatureHit { id, damage, behind } => {
                    d.set("kind", "creature_hit");
                    d.set("id", id as i64);
                    d.set("damage", damage as i64);
                    d.set("behind", behind);
                }
                run::Event::CreatureDied { id, kind, x, y, source } => {
                    d.set("kind", "creature_died");
                    d.set("id", id as i64);
                    d.set("creature", kind.key());
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("source", source.map_or(-1, |v| v as i64));
                }
                run::Event::CreatureLeft { id } => {
                    d.set("kind", "creature_left");
                    d.set("id", id as i64);
                }
                run::Event::PhaseChange { id, phase } => {
                    d.set("kind", "phase");
                    d.set("id", id as i64);
                    d.set("phase", phase.name());
                }
                run::Event::Swing { x, y, hit } => {
                    d.set("kind", "swing");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("hit", hit);
                }
                run::Event::Shot { x0, y0, x1, y1, by_runner, hit } => {
                    d.set("kind", "shot");
                    d.set("x0", x0 as i64);
                    d.set("y0", y0 as i64);
                    d.set("x1", x1 as i64);
                    d.set("y1", y1 as i64);
                    d.set("by_runner", by_runner);
                    d.set("hit", hit);
                }
                run::Event::Burst { x, y, radius } => {
                    d.set("kind", "burst");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("radius", radius as i64);
                }
                run::Event::Opened { source, revealed, notable, generous, forced, noise } => {
                    d.set("kind", "opened");
                    d.set("source", source as i64);
                    d.set("revealed", revealed as i64);
                    d.set("notable", notable as i64);
                    d.set("generous", generous);
                    d.set("forced", forced);
                    d.set("noise", noise as i64);
                }
                run::Event::Took { source, taken, left } => {
                    d.set("kind", "took");
                    d.set("source", source as i64);
                    d.set("taken", taken as i64);
                    d.set("left", left as i64);
                }
                run::Event::Dug { x, y, pile } => {
                    d.set("kind", "dug");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("pile", pile.map_or(-1, |v| v as i64));
                }
                run::Event::Salted { x, y } => {
                    d.set("kind", "salted");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                }
                run::Event::Used { slot, id } => {
                    d.set("kind", "used");
                    d.set("slot", slot as i64);
                    d.set("id", id.as_str());
                }
                run::Event::Down => d.set("kind", "down"),
                run::Event::Dead => d.set("kind", "dead"),
                run::Event::Extracted => d.set("kind", "extracted"),
                run::Event::Grabbed => d.set("kind", "grabbed"),
                run::Event::FollowerHit { id, damage } => {
                    d.set("kind", "follower_hit");
                    d.set("id", id as i64);
                    d.set("damage", damage as i64);
                }
                run::Event::FollowerDown { id } => {
                    d.set("kind", "follower_down");
                    d.set("id", id as i64);
                }
                run::Event::FollowerDead { id } => {
                    d.set("kind", "follower_dead");
                    d.set("id", id as i64);
                }
                run::Event::FollowerStrike { id, creature, damage } => {
                    d.set("kind", "follower_strike");
                    d.set("id", id as i64);
                    d.set("creature", creature as i64);
                    d.set("damage", damage as i64);
                }
                run::Event::Ordered { order } => {
                    d.set("kind", "ordered");
                    d.set("order", order.name());
                }
                run::Event::PickedUp { id } => {
                    d.set("kind", "picked_up");
                    d.set("id", id as i64);
                }
                run::Event::SetDown { id } => {
                    d.set("kind", "set_down");
                    d.set("id", id as i64);
                }
                run::Event::Dropped { source, id } => {
                    d.set("kind", "dropped");
                    d.set("source", source as i64);
                    d.set("id", id.as_str());
                }
                run::Event::Overtime { minute, packs, patrols } => {
                    d.set("kind", "overtime");
                    d.set("minute", minute as i64);
                    d.set("packs", packs as i64);
                    d.set("patrols", patrols as i64);
                }
            }
            out.push(&d.to_variant());
        }
        out
    }

    // ------------------------------------------------------------ senses

    #[func]
    fn memory_bits(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.fog.memory_bits().as_slice()))
    }

    #[func]
    fn light(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.light.level.as_slice()))
    }

    #[func]
    fn fog(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.fog.state.as_slice()))
    }

    #[func]
    fn noise(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.noise.level.as_slice()))
    }

    /// RGB8 per tile for the fog shader: R light*17, G fog (0/128/255), B noise*17.
    #[func]
    fn overlay_rgb(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.overlay_rgb().as_slice()))
    }

    /// Salted tiles, 1 where salt lies.
    #[func]
    fn salt(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.salt.iter().map(|&b| b as u8).collect::<Vec<u8>>().as_slice()))
    }

    #[func]
    fn is_visible(&self, x: i64, y: i64) -> bool {
        self.sim.as_ref().is_some_and(|s| x >= 0 && y >= 0 && (x as usize) < s.map.w && (y as usize) < s.map.h && s.senses.is_visible(x as usize, y as usize))
    }

    /// [visible tiles, remembered+visible tiles, live noise tiles, tick]
    #[func]
    fn sense_stats(&self) -> PackedInt32Array {
        self.sim.as_ref().map_or_else(PackedInt32Array::new, |s| {
            PackedInt32Array::from(&[s.senses.fog.visible_count() as i32, s.senses.fog.seen_count() as i32, s.senses.noise.live_tiles() as i32, s.tick as i32][..])
        })
    }

    // ------------------------------------------------------------ loot and the rig

    /// [{index, x, y, kind, tier, locked, state, count}]
    #[func]
    fn sources(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(s) = self.sim.as_ref() {
            for (i, src) in s.loot.sources.iter().enumerate() {
                let mut d = VarDictionary::new();
                d.set("index", i as i64);
                d.set("x", src.x as i64);
                d.set("y", src.y as i64);
                d.set("kind", src.kind.name());
                d.set("tier", src.tier as i64);
                d.set("locked", src.locked);
                d.set("state", source_state(src.state));
                d.set("count", src.items.len() as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    #[func]
    fn source_at(&self, x: i64, y: i64) -> i64 {
        self.sim.as_ref().and_then(|s| s.loot.source_at(x.max(0) as usize, y.max(0) as usize)).map_or(-1, |i| i as i64)
    }

    /// The contents of a source as the Runner sees them (unidentified: tier, category, size, weight, no name).
    #[func]
    fn source_items(&self, index: i64) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        let Some(src) = s.loot.sources.get(index.max(0) as usize) else { return out };
        for (k, it) in src.items.iter().enumerate() {
            out.push(&item_view(&s.table, k, it).to_variant());
        }
        out
    }

    /// The backpack as placed items: [{k, index, id, label, tier, category, w, h, weight, condition, ident, x, y}]
    #[func]
    fn rig_items(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        for (k, (_, p)) in s.runner.rig.backpack.iter().enumerate() {
            let mut d = item_view(&s.table, k, &p.item);
            d.set("x", p.x as i64);
            d.set("y", p.y as i64);
            out.push(&d.to_variant());
        }
        out
    }

    /// Quick slots as item ids ("" when empty).
    #[func]
    fn quick_slots(&self) -> PackedStringArray {
        let Some(s) = self.sim.as_ref() else { return PackedStringArray::new() };
        PackedStringArray::from(s.runner.rig.quick.iter().map(|q| GString::from(q.as_ref().map_or("", |it| s.table.def(it.def).id.as_str()))).collect::<Vec<_>>().as_slice())
    }

    /// The rig serialised (postcard) for the Stash to import on extraction.
    #[func]
    fn rig_export(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(postcard::to_allocvec(&s.runner.rig).unwrap_or_default().as_slice()))
    }

    /// [sources, open, empty, opened containers, pity counter, longest dry, creatures alive]
    #[func]
    fn run_stats(&self) -> PackedInt32Array {
        self.sim.as_ref().map_or_else(PackedInt32Array::new, |s| {
            let open = s.loot.sources.iter().filter(|x| x.state == run::SourceState::Open).count() as i32;
            let empty = s.loot.sources.iter().filter(|x| x.state == run::SourceState::Empty).count() as i32;
            PackedInt32Array::from(&[s.loot.sources.len() as i32, open, empty, s.loot.opened as i32, s.loot.pity_n as i32, s.loot.longest_dry as i32, s.creatures.len() as i32][..])
        })
    }
}
