//! The Tweaker Bench (Full Design v3.5, Stash, inventory and the Bench: Bench). Recipes by craft tag from
//! `data/recipes.toml`: a slot names a tag and the stat it drives, any item carrying the tag fills it. The outcome's
//! numbers come from the parts — ±`step` per part tier above or below the recipe tier for dmg, armour, light and
//! mag (clamped `min_mult`..`max_mult`), −1 noise per tier above and +1 below, condition the mean of the parts
//! with the housing slot counted twice — and the product carries its provenance. Three jobs act on gear already
//! in the Stash (@repair, @identify, @spored). The queue finishes jobs at shift end, a job taking its tier in shifts.

use items::{Crafted, GearMods, GearStats, Ident, ItemDef, ItemInstance, ItemTable, Tier};
use serde::{Deserialize, Serialize};

pub const RECIPES_TOML: &str = include_str!("../../../../data/recipes.toml");

#[derive(Clone, Debug, Deserialize)]
pub struct Rules {
    pub tier_shifts: [u8; 4],
    pub step: f32,
    pub min_mult: f32,
    pub max_mult: f32,
    pub repair_gain: u8,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Slot {
    pub tag: String,
    #[serde(default)]
    pub drives: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub tier: u8,
    /// an item id, or @repair / @identify / @spored
    pub product: String,
    #[serde(default = "one")]
    pub count: u32,
    pub slots: Vec<Slot>,
}

fn one() -> u32 {
    1
}

#[derive(Clone, Debug, Deserialize)]
pub struct RecipeFile {
    pub rules: Rules,
    pub recipe: Vec<Recipe>,
}

impl RecipeFile {
    pub fn from_toml(s: &str) -> Result<Self, String> {
        let f: RecipeFile = toml::from_str(s).map_err(|e| e.to_string())?;
        if f.recipe.len() != 60 {
            return Err(format!("{} recipes, the design has 60", f.recipe.len()));
        }
        Ok(f)
    }

    pub fn shipped() -> Self {
        Self::from_toml(RECIPES_TOML).expect("shipped recipes parse")
    }

    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipe.iter().find(|r| r.id == id)
    }

    /// Every product id must exist in the item table and every slot tag must be carried by some item (or be a
    /// special tag); returns the problems.
    pub fn validate(&self, table: &ItemTable) -> Vec<String> {
        let mut out = Vec::new();
        for r in &self.recipe {
            if !r.product.starts_with('@') && table.id_of(&r.product).is_none() {
                out.push(format!("{}: product {} is not an item", r.id, r.product));
            }
            if r.tier == 0 || r.tier > 4 {
                out.push(format!("{}: tier {}", r.id, r.tier));
            }
            if r.slots.len() < 2 || r.slots.len() > 4 {
                out.push(format!("{}: {} slots", r.id, r.slots.len()));
            }
            for s in &r.slots {
                let special = matches!(s.tag.as_str(), "gear" | "unidentified" | "spore");
                if !special && !table.defs().any(|d| d.tags.iter().any(|t| t == &s.tag)) {
                    out.push(format!("{}: no item carries tag {}", r.id, s.tag));
                }
            }
        }
        out
    }

    pub fn shifts(&self, recipe: &Recipe) -> u8 {
        self.rules.tier_shifts[(recipe.tier.clamp(1, 4) - 1) as usize]
    }
}

fn tier_index(t: Tier) -> i32 {
    match t {
        Tier::Common => 0,
        Tier::Uncommon => 1,
        Tier::Rare => 2,
        Tier::Blackline => 3,
        Tier::Relic => 4,
    }
}

/// Does this item fill this slot?
pub fn accepts(slot: &Slot, def: &ItemDef, inst: &ItemInstance) -> bool {
    match slot.tag.as_str() {
        "gear" => def.gear.is_some() && inst.ident == Ident::Full,
        "unidentified" => inst.ident != Ident::Full,
        "spore" => def.id == "prt_spore_tissue" || def.id == "org_spore_cap",
        tag => def.tags.iter().any(|t| t == tag),
    }
}

/// What a recipe makes of these parts (one per slot, in slot order).
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    /// the product instance (a new item, or the gear slot's item changed by a job)
    pub product: ItemInstance,
    pub count: u32,
    /// the def's numbers and the product's, for gear
    pub base: Option<GearStats>,
    pub stats: Option<GearStats>,
    pub shifts: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CraftError {
    WrongPartCount { want: usize, got: usize },
    SlotRefuses { slot: usize, tag: String, item: String },
    UnknownProduct(String),
    IdentifyNeedsUnidentified,
}

/// Compute the outcome without consuming anything; the Bench preview and the queue both call this.
pub fn outcome(table: &ItemTable, file: &RecipeFile, recipe: &Recipe, parts: &[ItemInstance], by: &str, day: u32) -> Result<Outcome, CraftError> {
    if parts.len() != recipe.slots.len() {
        return Err(CraftError::WrongPartCount { want: recipe.slots.len(), got: parts.len() });
    }
    for (i, (s, p)) in recipe.slots.iter().zip(parts).enumerate() {
        let d = table.def(p.def);
        if !accepts(s, d, p) {
            return Err(CraftError::SlotRefuses { slot: i, tag: s.tag.clone(), item: d.id.clone() });
        }
    }
    let shifts = file.shifts(recipe);
    let names: Vec<String> = parts.iter().map(|p| format!("{} ({:?})", table.def(p.def).name, table.def(p.def).tier).to_lowercase()).collect();
    let crafted = Crafted { by: by.to_string(), day, recipe: recipe.id.clone(), parts: names };
    // jobs on the gear slot
    if let Some(job) = recipe.product.strip_prefix('@') {
        let target_i = recipe.slots.iter().position(|s| s.tag == "gear" || s.tag == "unidentified").unwrap_or(0);
        let mut target = parts[target_i].clone();
        match job {
            "repair" => target.condition = target.condition.saturating_add(file.rules.repair_gain).min(100),
            "identify" => {
                if target.ident == Ident::Full {
                    return Err(CraftError::IdentifyNeedsUnidentified);
                }
                target.ident = Ident::Full;
            }
            "spored" => target.spored = true,
            other => return Err(CraftError::UnknownProduct(other.to_string())),
        }
        let d = table.def(target.def);
        let st = target.gear_stats(d);
        return Ok(Outcome { product: target, count: 1, base: st, stats: st, shifts });
    }
    let Some(pid) = table.id_of(&recipe.product) else { return Err(CraftError::UnknownProduct(recipe.product.clone())) };
    let pdef = table.def(pid);
    let mut product = ItemInstance::new(pid);
    product.crafted = Some(crafted);
    // condition: mean of the parts, housing counted twice
    let mut csum = 0u32;
    let mut cw = 0u32;
    for (s, p) in recipe.slots.iter().zip(parts) {
        let w = if s.tag == "housing" { 2 } else { 1 };
        csum += p.condition as u32 * w;
        cw += w;
    }
    product.condition = (csum / cw.max(1)).clamp(1, 100) as u8;
    // stats: each driving slot moves its stat by the part's tier against the recipe's
    let r_tier = recipe.tier as i32 - 1;
    let base = product.gear_stats(pdef);
    if let Some(g) = &pdef.gear {
        let mut mods = GearMods::default();
        let mut mult = [1.0f32; 4]; // dmg, armour, light, mag
        let mut noise = 0i32;
        for (s, p) in recipe.slots.iter().zip(parts) {
            let Some(drive) = &s.drives else { continue };
            let dt = tier_index(table.def(p.def).tier) - r_tier;
            match drive.as_str() {
                "dmg" => mult[0] += file.rules.step * dt as f32,
                "armour" => mult[1] += file.rules.step * dt as f32,
                "light" => mult[2] += file.rules.step * dt as f32,
                "mag" => mult[3] += file.rules.step * dt as f32,
                "noise" => noise -= dt,
                _ => {}
            }
        }
        let clamp = |m: f32| m.clamp(file.rules.min_mult, file.rules.max_mult);
        let scaled = |base: i32, m: f32| if base == 0 { 0 } else { ((base as f32 * clamp(m)).round() as i32).max(if base > 0 { 1 } else { base }) };
        mods.dmg = scaled(g.dmg, mult[0]) - g.dmg;
        mods.armour = scaled(g.armour, mult[1]) - g.armour;
        mods.light = if g.light > 0 { scaled(g.light, mult[2]) - g.light } else { 0 };
        mods.mag = scaled(g.mag, mult[3]) - g.mag;
        mods.noise = (g.noise + noise).max(0) - g.noise;
        product.mods = mods;
    }
    let stats = product.gear_stats(pdef);
    Ok(Outcome { product, count: recipe.count.max(1), base, stats, shifts })
}

/// A queued craft: its parts are out of the Stash until it finishes or is cancelled.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub recipe: String,
    pub parts: Vec<ItemInstance>,
    pub shifts_left: u8,
    pub by: String,
    pub day: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BenchQueue {
    pub jobs: Vec<Job>,
    /// finished products the Stash had no room for
    pub waiting: Vec<ItemInstance>,
    /// highest recipe tier open (1 at launch; Phase E's blueprints raise it)
    pub tier_open: u8,
}

impl BenchQueue {
    pub fn new() -> Self {
        BenchQueue { jobs: Vec::new(), waiting: Vec::new(), tier_open: 1 }
    }

    pub fn is_open(&self, recipe: &Recipe) -> bool {
        recipe.tier <= self.tier_open.max(1)
    }

    /// Queue a job whose outcome has been checked; the caller has already taken the parts out of the Stash.
    pub fn queue(&mut self, file: &RecipeFile, recipe: &Recipe, parts: Vec<ItemInstance>, by: &str, day: u32) {
        self.jobs.push(Job { recipe: recipe.id.clone(), parts, shifts_left: file.shifts(recipe), by: by.to_string(), day });
    }

    /// Cancel a job: its parts come back.
    pub fn cancel(&mut self, index: usize) -> Vec<ItemInstance> {
        if index < self.jobs.len() {
            self.jobs.remove(index).parts
        } else {
            Vec::new()
        }
    }

    /// Shift end: every job loses a shift; finished ones become products (plus anything still waiting for room).
    pub fn end_shift(&mut self, table: &ItemTable, file: &RecipeFile) -> Vec<ItemInstance> {
        let mut out: Vec<ItemInstance> = std::mem::take(&mut self.waiting);
        let mut keep = Vec::new();
        for mut job in std::mem::take(&mut self.jobs) {
            job.shifts_left = job.shifts_left.saturating_sub(1);
            if job.shifts_left > 0 {
                keep.push(job);
                continue;
            }
            let Some(recipe) = file.get(&job.recipe) else { out.extend(job.parts); continue };
            match outcome(table, file, recipe, &job.parts, &job.by, job.day) {
                Ok(o) => {
                    for _ in 0..o.count {
                        out.push(o.product.clone());
                    }
                }
                Err(_) => out.extend(job.parts),
            }
        }
        self.jobs = keep;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEMS: &str = include_str!("../../../../data/items.toml");

    fn table() -> ItemTable {
        ItemTable::from_toml(ITEMS).unwrap()
    }

    fn inst(t: &ItemTable, id: &str, condition: u8) -> ItemInstance {
        let mut i = ItemInstance::new(t.id_of(id).unwrap());
        i.condition = condition;
        i
    }

    #[test]
    fn the_sixty_recipes_validate_against_the_item_table() {
        let t = table();
        let f = RecipeFile::shipped();
        let problems = f.validate(&t);
        assert!(problems.is_empty(), "{problems:?}");
        let per_tier: Vec<usize> = (1..=4).map(|k| f.recipe.iter().filter(|r| r.tier == k).count()).collect();
        assert_eq!(per_tier, vec![15, 17, 16, 12]);
    }

    #[test]
    fn a_better_blade_makes_a_better_sword_and_the_sword_remembers() {
        let t = table();
        let f = RecipeFile::shipped();
        let sword = f.get("scrap_sword").unwrap();
        // common blade stock, common pipe housing, common wire: a tier-2 recipe from common parts → one tier under,
        // 15 % off the damage and one more noise
        let plain = outcome(&t, &f, sword, &[inst(&t, "prt_blade_stock", 100), inst(&t, "prt_pipe", 100), inst(&t, "prt_wire", 100)], "the Bench", 3).unwrap();
        let base = plain.base.unwrap();
        let st = plain.stats.unwrap();
        assert_eq!(st.dmg, (base.dmg as f32 * 0.85).round() as i32, "plain {st:?} base {base:?}");
        assert_eq!(st.noise, base.noise + 1);
        // a Rare blade (one tier over the recipe) adds 15 %: the design's 13 → 15; a Rare gold trace quiets it by one
        let good = outcome(&t, &f, sword, &[inst(&t, "chr_mono_tendons", 100), inst(&t, "prt_pipe", 60), inst(&t, "prt_gold_trace", 100)], "Sable", 3).unwrap();
        let gs = good.stats.unwrap();
        assert_eq!(gs.dmg, (base.dmg as f32 * 1.15).round() as i32, "good {gs:?}");
        assert_eq!((base.dmg, gs.dmg), (13, 15));
        assert_eq!(gs.noise, (base.noise - 1).max(0));
        // condition: housing counted twice → (100 + 60×2 + 100) / 4 = 80
        assert_eq!(good.product.condition, 80);
        let c = good.product.crafted.as_ref().unwrap();
        assert_eq!((c.by.as_str(), c.day, c.recipe.as_str()), ("Sable", 3, "scrap_sword"));
        assert_eq!(c.parts.len(), 3);
        assert!(c.parts[2].contains("rare"), "{:?}", c.parts);
        // the instance's numbers are what the def reader sees
        let d = t.def(good.product.def);
        assert_eq!(good.product.gear_stats(d).unwrap(), gs);
        assert_eq!(good.shifts, 1);
    }

    #[test]
    fn slots_refuse_the_wrong_tag_and_jobs_change_the_gear() {
        let t = table();
        let f = RecipeFile::shipped();
        let sword = f.get("scrap_sword").unwrap();
        let bad = outcome(&t, &f, sword, &[inst(&t, "prt_pipe", 100), inst(&t, "prt_pipe", 100), inst(&t, "prt_wire", 100)], "x", 0);
        assert!(matches!(bad, Err(CraftError::SlotRefuses { slot: 0, .. })), "{bad:?}");
        let repair = f.get("repair").unwrap();
        let worn = inst(&t, "ger_pipe", 30);
        let fixed = outcome(&t, &f, repair, &[worn, inst(&t, "prt_pipe", 100)], "x", 0).unwrap();
        assert_eq!(fixed.product.condition, 70);
        let identify = f.get("identify").unwrap();
        let mut unknown = inst(&t, "chr_street_eye", 50);
        unknown.ident = Ident::Unknown;
        let known = outcome(&t, &f, identify, &[unknown, inst(&t, "prt_lens", 100)], "x", 0).unwrap();
        assert_eq!(known.product.ident, Ident::Full);
        let spored = f.get("spored").unwrap();
        let tagged = outcome(&t, &f, spored, &[inst(&t, "ger_sword", 90), inst(&t, "prt_spore_tissue", 100)], "x", 0).unwrap();
        assert!(tagged.product.spored && tagged.shifts == 1);
    }

    #[test]
    fn the_queue_finishes_at_shift_end_by_tier_and_cancel_returns_parts() {
        let t = table();
        let f = RecipeFile::shipped();
        let mut q = BenchQueue::new();
        let salt = f.get("salt_bags").unwrap();
        assert!(q.is_open(salt) && !q.is_open(f.get("monowire").unwrap()));
        q.queue(&f, salt, vec![inst(&t, "org_brine", 100), inst(&t, "prt_cloth", 100)], "x", 1);
        q.queue(&f, f.get("downer").unwrap(), vec![inst(&t, "org_mould", 100), inst(&t, "org_mould", 100), inst(&t, "prt_cloth", 100)], "x", 1);
        let first = q.end_shift(&t, &f);
        assert_eq!(first.len(), 3, "salt bags come in threes: {first:?}");
        assert!(first.iter().all(|i| t.def(i.def).id == "prt_salt"));
        assert_eq!(q.jobs.len(), 1, "the tier-3 downer takes two shifts");
        let back = q.cancel(0);
        assert_eq!(back.len(), 3);
        assert!(q.end_shift(&t, &f).is_empty());
    }
}
