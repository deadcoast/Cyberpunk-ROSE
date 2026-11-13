//! items — the item table (data/items.toml), item instances, identification, condition, pricing, feed value, sets.
//! Eleven shared fields on every item so every item can be sold, fed, crafted or kept.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const GENES: [&str; 10] = [
    "vigor", "branching", "thrift", "appetite", "lume", "toxicity", "chrome", "feral", "conductance", "plasticity",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Chrome,
    Part,
    Organic,
    Tech,
    Gear,
    Contraband,
    Key,
    Curio,
    Relic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Common,
    Uncommon,
    Rare,
    Blackline,
    Relic,
}

impl Tier {
    /// genome drift magnitude when fed (GDD, Mat spec)
    pub fn drift(self) -> f32 {
        match self {
            Tier::Common => 0.01,
            Tier::Uncommon => 0.03,
            Tier::Rare => 0.07,
            Tier::Blackline => 0.15,
            Tier::Relic => 0.40,
        }
    }
    /// Appraise skill needed to identify on sight
    pub fn appraise_skill(self) -> u8 {
        match self {
            Tier::Common => 0,
            Tier::Uncommon => 2,
            Tier::Rare => 4,
            Tier::Blackline => 7,
            Tier::Relic => 10,
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GearDef {
    pub slot: String,
    pub kind: String,
    #[serde(default)]
    pub dmg: i32,
    #[serde(default)]
    pub armour: i32,
    #[serde(default)]
    pub noise: i32,
    #[serde(default)]
    pub light: i32,
    #[serde(default)]
    pub mag: i32,
    #[serde(default)]
    pub stat: String,
    #[serde(default)]
    pub bonus: i32,
    #[serde(default)]
    pub weakness: String,
    #[serde(default)]
    pub grid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub category: Category,
    pub tier: Tier,
    /// grid cells (w, h)
    pub size: [u8; 2],
    pub weight: f32,
    /// base price in eddies; 0 = unsellable
    pub value: u32,
    /// nutrient per unit when fed to a Pit
    pub feed: f32,
    #[serde(default)]
    pub bias: HashMap<String, f32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub set: Option<String>,
    #[serde(default)]
    pub gear: Option<GearDef>,
    #[serde(default)]
    pub codex: String,
}

impl ItemDef {
    /// The ten-gene bias vector the Pit applies when this is fed.
    pub fn bias_vector(&self) -> [f32; 10] {
        let mut v = [0.0; 10];
        for (k, w) in &self.bias {
            if let Some(i) = GENES.iter().position(|g| g == k) {
                v[i] = *w;
            }
        }
        v
    }
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
    pub fn cells(&self) -> u32 {
        self.size[0] as u32 * self.size[1] as u32
    }
}

#[derive(Debug, Deserialize)]
struct Table {
    item: Vec<ItemDef>,
}

#[derive(Clone, Debug, Default)]
pub struct ItemTable {
    pub defs: Vec<ItemDef>,
    index: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum TableError {
    Parse(String),
    DuplicateId(String),
    BadSize(String),
    BadGene(String, String),
}

impl std::fmt::Display for TableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableError::Parse(e) => write!(f, "items.toml: {e}"),
            TableError::DuplicateId(id) => write!(f, "duplicate item id {id}"),
            TableError::BadSize(id) => write!(f, "item {id}: size must be 1x1 .. 4x4"),
            TableError::BadGene(id, g) => write!(f, "item {id}: unknown gene {g}"),
        }
    }
}

impl ItemTable {
    pub fn from_toml(src: &str) -> Result<Self, TableError> {
        let t: Table = toml::from_str(src).map_err(|e| TableError::Parse(e.to_string()))?;
        let mut tab = ItemTable::default();
        for d in t.item {
            if d.size[0] == 0 || d.size[1] == 0 || d.size[0] > 4 || d.size[1] > 4 {
                return Err(TableError::BadSize(d.id));
            }
            for g in d.bias.keys() {
                if !GENES.contains(&g.as_str()) {
                    return Err(TableError::BadGene(d.id, g.clone()));
                }
            }
            if tab.index.contains_key(&d.id) {
                return Err(TableError::DuplicateId(d.id));
            }
            tab.index.insert(d.id.clone(), tab.defs.len());
            tab.defs.push(d);
        }
        Ok(tab)
    }

    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.index.get(id).map(|&i| &self.defs[i])
    }
    pub fn def(&self, idx: ItemId) -> &ItemDef {
        &self.defs[idx.0 as usize]
    }
    pub fn id_of(&self, id: &str) -> Option<ItemId> {
        self.index.get(id).map(|&i| ItemId(i as u16))
    }
    pub fn len(&self) -> usize {
        self.defs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }
    pub fn by_category(&self, c: Category) -> impl Iterator<Item = &ItemDef> {
        self.defs.iter().filter(move |d| d.category == c)
    }
    pub fn defs(&self) -> impl Iterator<Item = &ItemDef> {
        self.defs.iter()
    }
}

/// Index into the table; stable for a given data file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u16);

/// The item table shipped with the game (data/items.toml).
pub const ITEMS_TOML: &str = include_str!("../../../../data/items.toml");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ident {
    Unknown,
    Partial,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub stratum: u8,
    pub chunk: u16,
    pub day: u32,
}

/// Where a crafted item came from: who made it, when, and every part by name and tier (Full Design, Bench).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Crafted {
    pub by: String,
    pub day: u32,
    pub recipe: String,
    /// "Mono-Edge Blade (rare)", in slot order
    pub parts: Vec<String>,
}

/// Per-instance deltas over the def's gear numbers; a crafted sword keeps its own damage. Zero means "as the def".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GearMods {
    pub dmg: i32,
    pub armour: i32,
    pub noise: i32,
    pub light: i32,
    pub mag: i32,
}

/// The gear numbers of one instance: the def's, moved by the instance's mods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GearStats {
    pub dmg: i32,
    pub armour: i32,
    pub noise: i32,
    pub light: i32,
    pub mag: i32,
}

/// One physical item.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ItemInstance {
    pub def: ItemId,
    pub condition: u8,
    pub ident: Ident,
    pub found_in_run: bool,
    pub provenance: Option<Provenance>,
    pub pinned: bool,
    /// Bench-crafted insurance: regrows at the Heart on the Runner's death, once (Full Design, Death and recovery)
    pub spored: bool,
    /// set when the Bench made it
    pub crafted: Option<Crafted>,
    /// the Bench's changes to the def's gear numbers
    pub mods: GearMods,
    /// the contract a named item or corpse tag belongs to (Economy: recover)
    #[serde(default)]
    pub bound: Option<u32>,
}

impl ItemInstance {
    pub fn new(def: ItemId) -> Self {
        ItemInstance { def, condition: 100, ident: Ident::Full, found_in_run: false, provenance: None, pinned: false, spored: false, crafted: None, mods: GearMods::default(), bound: None }
    }
    pub fn found(def: ItemId, condition: u8, prov: Provenance) -> Self {
        ItemInstance { def, condition, ident: Ident::Unknown, found_in_run: true, provenance: Some(prov), pinned: false, spored: false, crafted: None, mods: GearMods::default(), bound: None }
    }

    /// The gear numbers of this instance (None for an item that is not gear).
    pub fn gear_stats(&self, def: &ItemDef) -> Option<GearStats> {
        let g = def.gear.as_ref()?;
        Some(GearStats {
            dmg: (g.dmg + self.mods.dmg).max(0),
            armour: (g.armour + self.mods.armour).max(0),
            noise: (g.noise + self.mods.noise).max(0),
            light: g.light + self.mods.light,
            mag: (g.mag + self.mods.mag).max(0),
        })
    }
}

/// Market context for a price.
#[derive(Clone, Copy, Debug)]
pub struct PriceContext {
    /// faction demand multiplier for this category, 0.6 .. 1.6
    pub demand: f32,
    /// units of this item already sold at this grate this shift
    pub glut_units: u32,
    /// Corpo pays 1.25 for found-in-run
    pub corpo: bool,
}

impl Default for PriceContext {
    fn default() -> Self {
        PriceContext { demand: 1.0, glut_units: 0, corpo: false }
    }
}

/// price = base x condition/100 x demand x glut x found-in-run premium. Unidentified sells blind at 40%.
pub fn price(def: &ItemDef, inst: &ItemInstance, ctx: PriceContext) -> u32 {
    if def.value == 0 {
        return 0;
    }
    let cond = inst.condition as f32 / 100.0;
    let glut = (1.0 - 0.05 * ctx.glut_units as f32).max(0.25);
    let fir = if inst.found_in_run && ctx.corpo { 1.25 } else { 1.0 };
    let blind = if inst.ident == Ident::Unknown { 0.40 } else { 1.0 };
    (def.value as f32 * cond * ctx.demand * glut * fir * blind).round() as u32
}

/// What one stall pays (Full Design v3.15, Economy: Price), made by the economy crate from the faction whose street
/// the grate is: the day's demand by category and tier (the walk and the surface's events), the faction's own item
/// multipliers, its reputation (1 + 0.1 x rep) and the stall's haggling (1 + 0.02 x Trade), whether it fences
/// Blackline, its found-in-run premium.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Book {
    /// the faction, by index in the factions file (u8::MAX: none, the 1.0 book)
    pub faction: u8,
    /// [Category as usize][Tier as usize]
    pub demand: [[f32; 5]; 9],
    pub items: Vec<(ItemId, f32)>,
    pub rep: f32,
    pub haggle: f32,
    pub fence: bool,
    pub fir: f32,
    /// false: the faction's stalls are closed to the city (reputation −3)
    pub open: bool,
}

impl Default for Book {
    fn default() -> Self {
        Book { faction: u8::MAX, demand: [[1.0; 5]; 9], items: Vec::new(), rep: 1.0, haggle: 1.0, fence: false, fir: 1.0, open: true }
    }
}

impl Book {
    /// The stall's multiplier for an item before condition, glut and blind (None: it will not buy it).
    pub fn wants(&self, def: &ItemDef, id: ItemId) -> Option<f32> {
        if !self.open || def.value == 0 || (def.tier == Tier::Blackline && !self.fence) {
            return None;
        }
        let own = self.items.iter().find(|(i, _)| *i == id).map_or(1.0, |(_, m)| *m);
        Some(self.demand[def.category as usize][def.tier as usize] * own * self.rep * self.haggle)
    }
}

/// What a stall pays for an item with `glut_units` of its category already sold there this shift: base x
/// condition/100 x the book's multiplier x glut (-5 % a unit, floor 25 %) x found-in-run premium x blind (40 %), at
/// least 1 e. None when the stall will not buy it.
pub fn sell_at(def: &ItemDef, inst: &ItemInstance, book: &Book, glut_units: u32) -> Option<u32> {
    let m = book.wants(def, inst.def)?;
    let cond = inst.condition as f32 / 100.0;
    let glut = (1.0 - 0.05 * glut_units as f32).max(0.25);
    let fir = if inst.found_in_run { book.fir } else { 1.0 };
    let blind = if inst.ident == Ident::Unknown { 0.40 } else { 1.0 };
    Some(((def.value as f32 * cond * m * glut * fir * blind).round() as u32).max(1))
}

/// What a Pit gets: nutrient (halved below condition 20) and the bias vector scaled by the tier's drift.
pub fn feed_value(def: &ItemDef, inst: &ItemInstance) -> (f32, [f32; 10], f32) {
    let n = if inst.condition < 20 { def.feed * 0.5 } else { def.feed };
    (n, def.bias_vector(), def.tier.drift())
}

/// Identification by a slimo: full if skill meets the tier's bar, partial if within 2.
pub fn appraise(def: &ItemDef, inst: &mut ItemInstance, skill: u8) -> Ident {
    let need = def.tier.appraise_skill();
    inst.ident = if skill >= need {
        Ident::Full
    } else if skill + 2 >= need && inst.ident == Ident::Unknown {
        Ident::Partial
    } else {
        inst.ident
    };
    inst.ident
}

/// Use and water lower condition; a Bench raises it. Clamped 0..100.
pub fn wear(inst: &mut ItemInstance, amount: i32) {
    inst.condition = (inst.condition as i32 - amount).clamp(0, 100) as u8;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetBonus {
    pub set: String,
    pub owned: usize,
    pub complete: bool,
}

/// Curio sets: three of a set in the Stash completes it.
pub fn set_bonuses<'a>(table: &ItemTable, items: impl Iterator<Item = &'a ItemInstance>) -> Vec<SetBonus> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for it in items {
        if let Some(s) = &table.def(it.def).set {
            *counts.entry(s.clone()).or_insert(0) += 1;
        }
    }
    let mut v: Vec<SetBonus> = counts.into_iter().map(|(set, owned)| SetBonus { set, owned, complete: owned >= 3 }).collect();
    v.sort_by(|a, b| a.set.cmp(&b.set));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = include_str!("../../../../data/items.toml");

    #[test]
    fn table_loads_with_164_items_and_all_categories() {
        let t = ItemTable::from_toml(DATA).unwrap();
        assert_eq!(t.len(), 164);
        for c in [Category::Chrome, Category::Part, Category::Organic, Category::Tech, Category::Gear, Category::Contraband, Category::Key, Category::Curio, Category::Relic] {
            assert!(t.by_category(c).count() > 0, "{c:?}");
        }
        assert_eq!(t.by_category(Category::Chrome).count(), 38);
        assert_eq!(t.by_category(Category::Gear).count(), 31);
    }

    #[test]
    fn every_gear_item_has_a_gear_block_and_every_relic_is_unsellable() {
        let t = ItemTable::from_toml(DATA).unwrap();
        for d in &t.defs {
            if d.category == Category::Gear {
                assert!(d.gear.is_some(), "{}", d.id);
            }
            if d.category == Category::Relic {
                assert_eq!(d.value, 0, "{}", d.id);
                assert!(d.feed >= 1.0);
            }
            if d.category == Category::Chrome {
                assert!(d.feed > 0.0 && !d.bias.is_empty(), "{}", d.id);
            }
        }
    }

    #[test]
    fn pricing_formula() {
        let t = ItemTable::from_toml(DATA).unwrap();
        let def = t.get("chr_street_eye").unwrap();
        let mut inst = ItemInstance::new(t.id_of("chr_street_eye").unwrap());
        assert_eq!(price(def, &inst, PriceContext::default()), 15);
        inst.condition = 50;
        assert_eq!(price(def, &inst, PriceContext::default()), 8);
        inst.condition = 100;
        inst.found_in_run = true;
        assert_eq!(price(def, &inst, PriceContext { corpo: true, ..Default::default() }), 19);
        assert_eq!(price(def, &inst, PriceContext { glut_units: 10, ..Default::default() }), 8);
        inst.ident = Ident::Unknown;
        assert_eq!(price(def, &inst, PriceContext::default()), 6);
    }

    #[test]
    fn appraisal_bars_follow_tier() {
        let t = ItemTable::from_toml(DATA).unwrap();
        let def = t.get("chr_lamp_core").unwrap(); // blackline: needs 7
        let mut inst = ItemInstance::found(t.id_of("chr_lamp_core").unwrap(), 80, Provenance { stratum: 4, chunk: 3, day: 2 });
        assert_eq!(appraise(def, &mut inst, 3), Ident::Unknown);
        assert_eq!(appraise(def, &mut inst, 5), Ident::Partial);
        assert_eq!(appraise(def, &mut inst, 7), Ident::Full);
    }

    #[test]
    fn feed_vector_and_condition_penalty() {
        let t = ItemTable::from_toml(DATA).unwrap();
        let def = t.get("chr_cortex_shunt").unwrap();
        let mut inst = ItemInstance::new(t.id_of("chr_cortex_shunt").unwrap());
        let (n, v, drift) = feed_value(def, &inst);
        assert!((n - 2.7).abs() < 1e-6);
        assert!((v[8] - 0.7).abs() < 1e-6); // conductance
        assert!((drift - 0.07).abs() < 1e-6);
        wear(&mut inst, 85);
        assert_eq!(inst.condition, 15);
        assert!((feed_value(def, &inst).0 - 1.35).abs() < 1e-6);
    }

    #[test]
    fn sets_complete_at_three() {
        let t = ItemTable::from_toml(DATA).unwrap();
        let ids = ["cur_street_sign", "cur_ooze_toy", "cur_arcade_token", "cur_rosary"];
        let items: Vec<ItemInstance> = ids.iter().map(|i| ItemInstance::new(t.id_of(i).unwrap())).collect();
        let b = set_bonuses(&t, items.iter());
        let street = b.iter().find(|s| s.set == "street").unwrap();
        assert!(street.complete && street.owned == 3);
        assert!(!b.iter().find(|s| s.set == "reliquary").unwrap().complete);
    }
}
