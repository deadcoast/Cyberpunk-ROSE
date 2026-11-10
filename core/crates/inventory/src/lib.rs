//! inventory — grid inventories (rig backpacks, containers, the Stash), gear slots, weight.
//! A grid is w x h cells; an item occupies size.w x size.h cells (rotation swaps them).

use items::{ItemDef, ItemId, ItemInstance, ItemTable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod ledger;
pub mod rooms;
pub use ledger::{Delivery, Dest, Landing, Ledger, Order, Queued, Quota, Shipment, ShiftReport, Verb, Want};
pub use rooms::{Room, Shelved, FLOOR, GRID_PRIORITY, SHELF};

pub type Handle = u32;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placed {
    pub item: ItemInstance,
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
    pub rotated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub w: u8,
    pub h: u8,
    cells: Vec<Option<Handle>>,
    items: HashMap<Handle, Placed>,
    next: Handle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlaceError {
    OutOfBounds,
    Occupied,
    NoRoom,
}

impl Grid {
    pub fn new(w: u8, h: u8) -> Self {
        Grid { w, h, cells: vec![None; w as usize * h as usize], items: HashMap::new(), next: 1 }
    }

    #[inline]
    fn idx(&self, x: u8, y: u8) -> usize {
        y as usize * self.w as usize + x as usize
    }

    pub fn fits(&self, x: u8, y: u8, w: u8, h: u8) -> Result<(), PlaceError> {
        if x as u16 + w as u16 > self.w as u16 || y as u16 + h as u16 > self.h as u16 {
            return Err(PlaceError::OutOfBounds);
        }
        for yy in y..y + h {
            for xx in x..x + w {
                if self.cells[self.idx(xx, yy)].is_some() {
                    return Err(PlaceError::Occupied);
                }
            }
        }
        Ok(())
    }

    /// Place at (x, y); rotation swaps the footprint.
    pub fn place(&mut self, def: &ItemDef, item: ItemInstance, x: u8, y: u8, rotated: bool) -> Result<Handle, PlaceError> {
        let (w, h) = if rotated { (def.size[1], def.size[0]) } else { (def.size[0], def.size[1]) };
        self.fits(x, y, w, h)?;
        let handle = self.next;
        self.next += 1;
        for yy in y..y + h {
            for xx in x..x + w {
                let i = self.idx(xx, yy);
                self.cells[i] = Some(handle);
            }
        }
        self.items.insert(handle, Placed { item, x, y, w, h, rotated });
        Ok(handle)
    }

    /// First free position, scanning rows; tries the rotated footprint when the upright one fails.
    pub fn auto_place(&mut self, def: &ItemDef, item: ItemInstance) -> Result<Handle, PlaceError> {
        for rotated in [false, true] {
            let (w, h) = if rotated { (def.size[1], def.size[0]) } else { (def.size[0], def.size[1]) };
            if w == h && rotated {
                break;
            }
            for y in 0..self.h {
                for x in 0..self.w {
                    if self.fits(x, y, w, h).is_ok() {
                        return self.place(def, item, x, y, rotated);
                    }
                }
            }
        }
        Err(PlaceError::NoRoom)
    }

    pub fn remove(&mut self, handle: Handle) -> Option<ItemInstance> {
        let p = self.items.remove(&handle)?;
        for yy in p.y..p.y + p.h {
            for xx in p.x..p.x + p.w {
                let i = self.idx(xx, yy);
                self.cells[i] = None;
            }
        }
        Some(p.item)
    }

    pub fn at(&self, x: u8, y: u8) -> Option<Handle> {
        if x >= self.w || y >= self.h {
            return None;
        }
        self.cells[self.idx(x, y)]
    }

    pub fn get(&self, handle: Handle) -> Option<&Placed> {
        self.items.get(&handle)
    }
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut Placed> {
        self.items.get_mut(&handle)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&Handle, &Placed)> {
        self.items.iter()
    }
    /// Placements in reading order (row, then column), so a grid lists the same way every time.
    pub fn placed(&self) -> Vec<(Handle, &Placed)> {
        let mut v: Vec<(Handle, &Placed)> = self.items.iter().map(|(h, p)| (*h, p)).collect();
        v.sort_by_key(|(_, p)| (p.y, p.x));
        v
    }
    /// The items in handle order (the map's own order differs between processes).
    pub fn items(&self) -> impl Iterator<Item = &ItemInstance> {
        let mut v: Vec<(&Handle, &Placed)> = self.items.iter().collect();
        v.sort_unstable_by_key(|(h, _)| **h);
        v.into_iter().map(|(_, p)| &p.item)
    }
    pub fn count(&self) -> usize {
        self.items.len()
    }
    pub fn free_cells(&self) -> usize {
        self.cells.iter().filter(|c| c.is_none()).count()
    }
    pub fn weight(&self, table: &ItemTable) -> f32 {
        // summed in handle order, so a float sum is the same in every process
        self.items().map(|i| table.def(i.def).weight).sum()
    }
    /// Re-pack every item first-fit in the given order, keeping each item's handle (anything not named keeps its
    /// relative order after them). A grid that held the items before holds them after.
    pub fn repack(&mut self, order: &[Handle]) -> bool {
        let snapshot = (self.cells.clone(), self.items.clone());
        let mut placed: Vec<(Handle, Placed)> = order.iter().filter_map(|h| self.items.remove(h).map(|p| (*h, p))).collect();
        let mut rest: Vec<Handle> = self.items.keys().copied().collect();
        rest.sort_unstable();
        for h in rest {
            if let Some(p) = self.items.remove(&h) {
                placed.push((h, p));
            }
        }
        self.cells.fill(None);
        let mut leftover = Vec::new();
        for (h, mut p) in placed {
            let mut done = false;
            'outer: for flip in [false, true] {
                let (w, hh) = if flip { (p.h, p.w) } else { (p.w, p.h) };
                if flip && w == hh {
                    break;
                }
                for y in 0..self.h {
                    for x in 0..self.w {
                        if self.fits(x, y, w, hh).is_ok() {
                            p.x = x;
                            p.y = y;
                            if flip {
                                p.rotated = !p.rotated;
                                p.w = w;
                                p.h = hh;
                            }
                            for yy in y..y + hh {
                                for xx in x..x + w {
                                    let i = self.idx(xx, yy);
                                    self.cells[i] = Some(h);
                                }
                            }
                            self.items.insert(h, p.clone());
                            done = true;
                            break 'outer;
                        }
                    }
                }
            }
            if !done {
                leftover.push((h, p));
            }
        }
        // first-fit can fail where the old packing succeeded; then the grid goes back exactly as it was
        if !leftover.is_empty() {
            self.cells = snapshot.0;
            self.items = snapshot.1;
            return false;
        }
        true
    }

    /// Grow the grid, keeping every placement (new cells to the right and below).
    pub fn resize(&mut self, w: u8, h: u8) {
        assert!(w >= self.w && h >= self.h, "grids only grow");
        let mut cells = vec![None; w as usize * h as usize];
        for y in 0..self.h {
            for x in 0..self.w {
                cells[y as usize * w as usize + x as usize] = self.cells[self.idx(x, y)];
            }
        }
        self.cells = cells;
        self.w = w;
        self.h = h;
    }
}

/// Gear slots on a Runner.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GearSlots {
    pub head: Option<ItemInstance>,
    pub body: Option<ItemInstance>,
    pub feet: Option<ItemInstance>,
    pub hands: [Option<ItemInstance>; 2],
    pub back: Option<ItemInstance>,
    pub implants: [Option<ItemInstance>; 4],
}

/// A Runner's carried inventory: backpack grid, chest-rig quick slots, gear.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rig {
    pub backpack: Grid,
    pub quick: Vec<Option<ItemInstance>>,
    pub gear: GearSlots,
    /// kg the Runner carries before sprint halves and noise doubles
    pub carry_cap: f32,
}

impl Rig {
    pub fn new(backpack_w: u8, backpack_h: u8, quick_slots: usize, carry_cap: f32) -> Self {
        Rig { backpack: Grid::new(backpack_w, backpack_h), quick: vec![None; quick_slots], gear: GearSlots::default(), carry_cap }
    }
    pub fn weight(&self, table: &ItemTable) -> f32 {
        let mut w = self.backpack.weight(table);
        for q in self.quick.iter().flatten() {
            w += table.def(q.def).weight;
        }
        let g = &self.gear;
        for it in [&g.head, &g.body, &g.feet, &g.back].into_iter().flatten() {
            w += table.def(it.def).weight;
        }
        for it in g.hands.iter().flatten().chain(g.implants.iter().flatten()) {
            w += table.def(it.def).weight;
        }
        w
    }
    pub fn overweight(&self, table: &ItemTable) -> bool {
        self.weight(table) > self.carry_cap
    }
}

/// Stash tiers: grid size by Heart tier.
pub const STASH_DIMS: [(u8, u8); 4] = [(12, 10), (16, 14), (20, 18), (24, 22)];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub grid: Grid,
    /// category filter as strings ("chrome", "gear"); empty = any
    pub accepts: Vec<String>,
    /// a Vault hides Blackline and Relics from Attention
    pub vault: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stash {
    pub tier: u8,
    pub grid: Grid,
    pub containers: Vec<Container>,
    /// the stockpiles: the Stash's other rooms (v10)
    #[serde(default)]
    pub rooms: Vec<Room>,
    #[serde(default)]
    pub next_shelf: u32,
    /// the Heart's floor: what the grid has no room for, still the Stash's — the quotas pull it, the verbs work on
    /// it, a stockpile takes it, and it tries the grid again at every shift end (v3.12)
    #[serde(default)]
    pub floor: Vec<Shelved>,
}

/// Organics spoil this much a shift outside a Cold Store (Full Design v3.12, Condition).
pub const SPOIL_PER_SHIFT: u8 = 2;

impl Stash {
    pub fn new() -> Self {
        let (w, h) = STASH_DIMS[0];
        Stash { tier: 1, grid: Grid::new(w, h), containers: Vec::new(), rooms: Vec::new(), next_shelf: 0, floor: Vec::new() }
    }
    pub fn upgrade(&mut self) -> bool {
        if (self.tier as usize) < STASH_DIMS.len() {
            self.tier += 1;
            let (w, h) = STASH_DIMS[self.tier as usize - 1];
            self.grid.resize(w, h);
            true
        } else {
            false
        }
    }
    pub fn all_items(&self) -> impl Iterator<Item = &ItemInstance> {
        self.grid.items().chain(self.containers.iter().flat_map(|c| c.grid.items())).chain(self.rooms.iter().flat_map(|r| r.items.iter().map(|s| &s.item))).chain(self.floor.iter().map(|s| &s.item))
    }

    /// Grow the Heart's grid to a tier (the city's): it never shrinks.
    pub fn set_tier(&mut self, tier: u8) {
        while self.tier < tier.min(STASH_DIMS.len() as u8) {
            if !self.upgrade() {
                break;
            }
        }
    }

    /// An item by handle in any room: the Heart's grid below SHELF, a stockpile at or above it.
    pub fn get_any(&self, h: Handle) -> Option<&ItemInstance> {
        if (FLOOR..SHELF).contains(&h) {
            return self.floor.iter().find(|s| FLOOR + s.id == h).map(|s| &s.item);
        }
        if h < SHELF {
            return self.grid.get(h).map(|p| &p.item);
        }
        self.rooms.iter().flat_map(|r| r.items.iter()).find(|s| SHELF + s.id == h).map(|s| &s.item)
    }

    pub fn get_any_mut(&mut self, h: Handle) -> Option<&mut ItemInstance> {
        if (FLOOR..SHELF).contains(&h) {
            return self.floor.iter_mut().find(|s| FLOOR + s.id == h).map(|s| &mut s.item);
        }
        if h < SHELF {
            return self.grid.get_mut(h).map(|p| &mut p.item);
        }
        self.rooms.iter_mut().flat_map(|r| r.items.iter_mut()).find(|s| SHELF + s.id == h).map(|s| &mut s.item)
    }

    /// The stockpile zone an item lies in (None: the Heart's grid).
    pub fn room_of(&self, h: Handle) -> Option<u32> {
        if h < SHELF {
            return None;
        }
        self.rooms.iter().find(|r| r.items.iter().any(|s| SHELF + s.id == h)).map(|r| r.zone)
    }

    /// Take an item out of whichever room holds it.
    pub fn take(&mut self, h: Handle) -> Option<ItemInstance> {
        if (FLOOR..SHELF).contains(&h) {
            let k = self.floor.iter().position(|s| FLOOR + s.id == h)?;
            return Some(self.floor.remove(k).item);
        }
        if h < SHELF {
            return self.grid.remove(h);
        }
        for r in self.rooms.iter_mut() {
            if let Some(k) = r.items.iter().position(|s| SHELF + s.id == h) {
                return Some(r.items.remove(k).item);
            }
        }
        None
    }

    /// Every item with its handle, the Heart's grid first, then its floor, then each room in order (the floor is
    /// the Heart's: no room).
    pub fn handles(&self) -> Vec<(Handle, Option<u32>, &ItemInstance)> {
        let mut v: Vec<(Handle, Option<u32>, &ItemInstance)> = self.grid.placed().into_iter().map(|(h, p)| (h, None, &p.item)).collect();
        v.extend(self.floor.iter().map(|s| (FLOOR + s.id, None, &s.item)));
        for r in &self.rooms {
            for s in &r.items {
                v.push((SHELF + s.id, Some(r.zone), &s.item));
            }
        }
        v
    }

    /// The grid's overflow onto the Heart's floor.
    pub fn to_floor(&mut self, item: ItemInstance) -> Handle {
        let id = self.next_shelf;
        self.next_shelf += 1;
        self.floor.push(Shelved { id, item, cell: None });
        FLOOR + id
    }

    /// Into the grid if it has room, else onto the Heart's floor. True when the grid took it.
    pub fn place_or_floor(&mut self, table: &ItemTable, item: ItemInstance) -> bool {
        match self.grid.auto_place(table.def(item.def), item.clone()) {
            Ok(_) => true,
            Err(_) => {
                self.to_floor(item);
                false
            }
        }
    }

    /// The floor tries the grid again (shift end), oldest first.
    pub fn sweep_floor(&mut self, table: &ItemTable) {
        let mut keep = Vec::new();
        for s in std::mem::take(&mut self.floor) {
            if self.grid.auto_place(table.def(s.item.def), s.item.clone()).is_err() {
                keep.push(s);
            }
        }
        self.floor = keep;
    }

    /// Put an item into a room as cargo lands there; false when it has no room.
    pub fn shelve(&mut self, zone: u32, item: ItemInstance) -> Result<Handle, ItemInstance> {
        let id = self.next_shelf;
        let Some(r) = self.rooms.iter_mut().find(|r| r.zone == zone) else { return Err(item) };
        if r.land(id, item.clone()) {
            self.next_shelf += 1;
            Ok(SHELF + id)
        } else {
            Err(item)
        }
    }

    /// Put an item back where it can go: its room if it has one and room is left, else the Heart's grid.
    pub fn put_back(&mut self, table: &ItemTable, item: ItemInstance, room: Option<u32>) -> Result<Handle, ItemInstance> {
        let item = match room {
            Some(z) => match self.shelve(z, item) {
                Ok(h) => return Ok(h),
                Err(it) => it,
            },
            None => item,
        };
        self.grid.auto_place(table.def(item.def), item.clone()).map_err(|_| item)
    }

    /// The shift's spoiling: Organics lose SPOIL_PER_SHIFT condition wherever they lie but a Cold Store. Returns how
    /// many items spoiled a step.
    pub fn spoil(&mut self, table: &ItemTable) -> usize {
        let organic = |it: &ItemInstance| table.def(it.def).category == items::Category::Organic;
        let mut n = 0;
        for h in self.grid.placed().into_iter().map(|(h, _)| h).collect::<Vec<_>>() {
            if let Some(p) = self.grid.get_mut(h) {
                if organic(&p.item) && p.item.condition > 0 {
                    p.item.condition = p.item.condition.saturating_sub(SPOIL_PER_SHIFT);
                    n += 1;
                }
            }
        }
        for r in self.rooms.iter_mut().filter(|r| !r.cold) {
            for s in r.items.iter_mut() {
                if organic(&s.item) && s.item.condition > 0 {
                    s.item.condition = s.item.condition.saturating_sub(SPOIL_PER_SHIFT);
                    n += 1;
                }
            }
        }
        for s in self.floor.iter_mut() {
            if organic(&s.item) && s.item.condition > 0 {
                s.item.condition = s.item.condition.saturating_sub(SPOIL_PER_SHIFT);
                n += 1;
            }
        }
        n
    }
    /// Blackline and Relic items outside a Vault: what the Attention system counts.
    pub fn exposed_contraband(&self, table: &ItemTable) -> usize {
        use items::Tier;
        let hot = |it: &ItemInstance| matches!(table.def(it.def).tier, Tier::Blackline | Tier::Relic);
        self.grid.items().filter(|i| hot(i)).count()
            + self.containers.iter().filter(|c| !c.vault).flat_map(|c| c.grid.items()).filter(|i| hot(i)).count()
            + self.rooms.iter().flat_map(|r| r.items.iter()).filter(|s| hot(&s.item)).count()
            + self.floor.iter().filter(|s| hot(&s.item)).count()
    }
    pub fn count(&self) -> usize {
        self.all_items().count()
    }

    /// Re-pack the main grid in an order: "tier" (best first), "value", "weight", "condition", "category"; ties by
    /// name so the result is the same every time. Every item keeps its instance; only placement changes.
    pub fn sort_by(&mut self, table: &ItemTable, key: &str) {
        let mut order: Vec<(Handle, &Placed)> = self.grid.placed();
        order.sort_by(|(_, a), (_, b)| {
            let (da, db) = (table.def(a.item.def), table.def(b.item.def));
            let primary = match key {
                "value" => db.value.cmp(&da.value),
                "weight" => db.weight.partial_cmp(&da.weight).unwrap_or(std::cmp::Ordering::Equal),
                "condition" => b.item.condition.cmp(&a.item.condition),
                "category" => da.category.cmp(&db.category),
                _ => db.tier.cmp(&da.tier),
            };
            primary.then_with(|| da.name.cmp(&db.name))
        });
        let handles: Vec<Handle> = order.into_iter().map(|(h, _)| h).collect();
        self.grid.repack(&handles);
    }

}

impl Default for Stash {
    fn default() -> Self {
        Self::new()
    }
}

pub fn instance(table: &ItemTable, id: &str) -> Option<(ItemId, ItemInstance)> {
    let iid = table.id_of(id)?;
    Some((iid, ItemInstance::new(iid)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = include_str!("../../../../data/items.toml");

    #[test]
    fn sorting_the_stash_keeps_every_item_and_its_handle() {
        let t = table();
        let mut st = Stash::new();
        let mut hs = Vec::new();
        for id in ["prt_pipe", "chr_street_eye", "ger_nail_rifle", "cur_ooze_toy", "org_paste", "prt_plate"] {
            let (_, it) = instance(&t, id).unwrap();
            hs.push(st.grid.auto_place(t.def(it.def), it).unwrap());
        }
        let before: std::collections::HashMap<Handle, items::ItemId> = st.grid.iter().map(|(h, p)| (*h, p.item.def)).collect();
        st.sort_by(&t, "value");
        assert_eq!(st.count(), 6);
        for (h, def) in before {
            assert_eq!(st.grid.get(h).map(|p| p.item.def), Some(def), "handle {h} changed identity");
        }
        // the most valuable item sits first in reading order
        let first = st.grid.placed()[0].1.item.def;
        let max = st.grid.items().map(|i| t.def(i.def).value).max().unwrap();
        assert_eq!(t.def(first).value, max);
    }

    fn table() -> ItemTable {
        ItemTable::from_toml(DATA).unwrap()
    }

    #[test]
    fn placement_respects_footprint_and_rotation() {
        let t = table();
        let mut g = Grid::new(4, 4);
        let (_, pipe) = instance(&t, "prt_pipe").unwrap(); // 1x3
        let def = t.get("prt_pipe").unwrap();
        let h1 = g.place(def, pipe.clone(), 0, 0, false).unwrap();
        assert_eq!(g.at(0, 2), Some(h1));
        assert_eq!(g.place(def, pipe.clone(), 0, 1, false), Err(PlaceError::Occupied));
        assert_eq!(g.place(def, pipe.clone(), 3, 2, false), Err(PlaceError::OutOfBounds));
        let h2 = g.place(def, pipe.clone(), 1, 3, true).unwrap(); // rotated: 3x1
        assert_eq!(g.at(3, 3), Some(h2));
        assert_eq!(g.free_cells(), 16 - 6);
        assert!(g.remove(h1).is_some());
        assert_eq!(g.at(0, 0), None);
    }

    #[test]
    fn auto_place_fills_and_reports_no_room() {
        let t = table();
        let mut g = Grid::new(2, 2);
        let def = t.get("chr_optic_slag").unwrap(); // 1x1
        for _ in 0..4 {
            g.auto_place(def, ItemInstance::new(t.id_of("chr_optic_slag").unwrap())).unwrap();
        }
        assert_eq!(g.auto_place(def, ItemInstance::new(t.id_of("chr_optic_slag").unwrap())), Err(PlaceError::NoRoom));
        let big = t.get("prt_tarp").unwrap(); // 2x2 cannot fit a 2x2 grid that is full
        assert_eq!(Grid::new(2, 2).auto_place(big, ItemInstance::new(t.id_of("prt_tarp").unwrap())).is_ok(), true);
    }

    #[test]
    fn stash_upgrades_keep_items_and_vaults_hide_contraband() {
        let t = table();
        let mut s = Stash::new();
        let core = t.get("chr_lamp_core").unwrap();
        s.grid.auto_place(core, ItemInstance::new(t.id_of("chr_lamp_core").unwrap())).unwrap();
        assert_eq!(s.exposed_contraband(&t), 1);
        assert!(s.upgrade());
        assert_eq!(s.grid.w, 16);
        assert_eq!(s.count(), 1);
        let mut vault = Container { name: "Vault".into(), grid: Grid::new(4, 4), accepts: vec![], vault: true };
        vault.grid.auto_place(core, ItemInstance::new(t.id_of("chr_lamp_core").unwrap())).unwrap();
        s.containers.push(vault);
        assert_eq!(s.exposed_contraband(&t), 1);
        assert_eq!(s.count(), 2);
    }

    #[test]
    fn rig_weight_and_overweight() {
        let t = table();
        let mut rig = Rig::new(6, 4, 4, 12.0);
        let plate = t.get("ger_plate_vest").unwrap(); // 6 kg
        rig.gear.body = Some(ItemInstance::new(t.id_of("ger_plate_vest").unwrap()));
        let _ = plate;
        let rifle = t.get("ger_nail_rifle").unwrap(); // 3.5 kg, 1x4 does not fit 6x4 upright... rotated 4x1 does
        rig.backpack.auto_place(rifle, ItemInstance::new(t.id_of("ger_nail_rifle").unwrap())).unwrap();
        assert!((rig.weight(&t) - 9.5).abs() < 1e-5);
        assert!(!rig.overweight(&t));
        rig.backpack.auto_place(t.get("con_filtered_water").unwrap(), ItemInstance::new(t.id_of("con_filtered_water").unwrap())).unwrap();
        assert!(rig.overweight(&t));
    }
}
