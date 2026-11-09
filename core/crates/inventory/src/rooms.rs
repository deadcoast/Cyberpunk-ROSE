//! The Stash's other rooms: the stockpiles (Full Design v3.12, Colony mode: Stockpiles). A room is the Stash's side
//! of a stockpile zone — its filter and priority mirror the colony's zone, its cells are the zone's — and holds one
//! item a cell: on the zone's Mat cells as cargo lands, on the store's own footprint while it waits for a hauler,
//! on the zone's cells off the Mat once a hauler has carried it there.

use items::{ItemDef, ItemInstance, Tier};
use serde::{Deserialize, Serialize};

/// Handles at or above this name an item in a room, not in the Heart's grid.
pub const SHELF: u32 = 1 << 30;
/// Handles from this to SHELF name an item on the Heart's floor: what the grid has no room for.
pub const FLOOR: u32 = 1 << 29;

/// The Heart's grid stands between priorities 2 and 3: a room at 3–4 draws from it, one at 1–2 takes its overflow.
pub const GRID_PRIORITY: f32 = 2.5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shelved {
    /// stable within the Stash: the handle is SHELF + id
    pub id: u32,
    pub item: ItemInstance,
    /// the zone cell it sits on; None while it waits on the store's footprint for a hauler
    pub cell: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Room {
    /// the colony's zone id
    pub zone: u32,
    /// the Rack's or Cold Store's structure id
    pub anchor: u32,
    pub name: String,
    /// cells it may hold (the store's 8 / 12 / 16, at most the painted zone)
    pub cap: u16,
    /// 1–4 on a job's scale: 4 is wanted first
    pub priority: u8,
    /// a Cold Store: Organics here do not spoil
    pub cold: bool,
    /// the filter: categories by name (empty = any), a tier floor (0 Common … 4 Relic), a condition floor
    pub categories: Vec<String>,
    pub min_tier: u8,
    pub min_condition: u8,
    /// the zone's cells on the Mat (cargo lands there) and off it (a hauler carries there)
    pub mat_cells: Vec<u32>,
    pub off_cells: Vec<u32>,
    /// items that may wait on the store's footprint (2 a Rack, 4 a Cold Store)
    pub footprint: u16,
    pub items: Vec<Shelved>,
}

pub fn tier_rank(t: Tier) -> u8 {
    match t {
        Tier::Common => 0,
        Tier::Uncommon => 1,
        Tier::Rare => 2,
        Tier::Blackline => 3,
        Tier::Relic => 4,
    }
}

impl Room {
    pub fn accepts(&self, def: &ItemDef, item: &ItemInstance) -> bool {
        let cat = format!("{:?}", def.category).to_lowercase();
        (self.categories.is_empty() || self.categories.iter().any(|c| c.to_lowercase() == cat)) && tier_rank(def.tier) >= self.min_tier && item.condition >= self.min_condition
    }

    fn used(&self, cells: &[u32]) -> usize {
        cells.iter().filter(|c| self.items.iter().any(|s| s.cell == Some(**c))).count()
    }

    fn waiting(&self) -> usize {
        self.items.iter().filter(|s| s.cell.is_none()).count()
    }

    /// How many more items cargo can bring now: free Mat cells and free footprint, within the cap.
    pub fn room_for(&self) -> usize {
        let landing = (self.mat_cells.len() - self.used(&self.mat_cells)) + (self.footprint as usize).saturating_sub(self.waiting());
        landing.min((self.cap as usize).saturating_sub(self.items.len()))
    }

    /// Put an arriving item down: a free Mat cell, else the footprint. Returns false when there is no room.
    pub fn land(&mut self, id: u32, item: ItemInstance) -> bool {
        if self.room_for() == 0 {
            return false;
        }
        let cell = self.mat_cells.iter().copied().find(|c| !self.items.iter().any(|s| s.cell == Some(*c)));
        self.items.push(Shelved { id, item, cell });
        true
    }

    /// A hauler's shift: up to `n` waiting items carried onto free cells off the Mat. Returns how many.
    pub fn haul(&mut self, n: usize) -> usize {
        let mut moved = 0;
        while moved < n {
            let Some(free) = self.off_cells.iter().copied().find(|c| !self.items.iter().any(|s| s.cell == Some(*c))) else { break };
            let Some(k) = self.items.iter().position(|s| s.cell.is_none()) else { break };
            self.items[k].cell = Some(free);
            moved += 1;
        }
        moved
    }

    /// The zone changed shape: items on cells no longer in it wait on the footprint (they are not lost).
    pub fn reshape(&mut self, mat_cells: Vec<u32>, off_cells: Vec<u32>) {
        self.mat_cells = mat_cells;
        self.off_cells = off_cells;
        for s in self.items.iter_mut() {
            if let Some(c) = s.cell {
                if !self.mat_cells.contains(&c) && !self.off_cells.contains(&c) {
                    s.cell = None;
                }
            }
        }
    }
}
