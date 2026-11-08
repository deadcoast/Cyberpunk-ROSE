//! inventory — grid inventories (rig backpacks, containers, the Stash), gear slots, weight.
//! A grid is w x h cells; an item occupies size.w x size.h cells (rotation swaps them).

use items::{ItemDef, ItemId, ItemInstance, ItemTable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fn items(&self) -> impl Iterator<Item = &ItemInstance> {
        self.items.values().map(|p| &p.item)
    }
    pub fn count(&self) -> usize {
        self.items.len()
    }
    pub fn free_cells(&self) -> usize {
        self.cells.iter().filter(|c| c.is_none()).count()
    }
    pub fn weight(&self, table: &ItemTable) -> f32 {
        self.items.values().map(|p| table.def(p.item.def).weight).sum()
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
}

impl Stash {
    pub fn new() -> Self {
        let (w, h) = STASH_DIMS[0];
        Stash { tier: 1, grid: Grid::new(w, h), containers: Vec::new() }
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
        self.grid.items().chain(self.containers.iter().flat_map(|c| c.grid.items()))
    }
    /// Blackline and Relic items outside a Vault: what the Attention system counts.
    pub fn exposed_contraband(&self, table: &ItemTable) -> usize {
        use items::Tier;
        let hot = |it: &ItemInstance| matches!(table.def(it.def).tier, Tier::Blackline | Tier::Relic);
        self.grid.items().filter(|i| hot(i)).count()
            + self.containers.iter().filter(|c| !c.vault).flat_map(|c| c.grid.items()).filter(|i| hot(i)).count()
    }
    pub fn count(&self) -> usize {
        self.all_items().count()
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
