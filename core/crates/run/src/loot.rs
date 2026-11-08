//! Loot: tables from data/loot.toml resolved D2-style (NoDrop, category, tier row, item), notable picks on Dota's
//! pseudo-random distribution so streaks stay short, first-run generosity, tools that buy certainty, and the
//! sources a route offers (lockers, crates, dug piles, newcomer corpses, caches). Research 08 has the reasoning.
//!
//! Contents roll per run from (route seed, run nonce, source index): where a locker is can be learned from the route
//! card, what is in it cannot.

use crate::map::{ContainerKind, RouteMap, SpawnKind};
use inventory::Rig;
use items::{Category, ItemDef, ItemInstance, ItemTable, Provenance, Tier};
use mat::rng::Pcg32;
use serde::Deserialize;
use std::collections::HashMap;

pub const LOOT_TOML: &str = include_str!("../../../../data/loot.toml");

const TIERS: [Tier; 5] = [Tier::Common, Tier::Uncommon, Tier::Rare, Tier::Blackline, Tier::Relic];

#[derive(Clone, Debug, Deserialize)]
pub struct Extra {
    pub item: String,
    pub chance: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceDef {
    pub count: [u8; 2],
    pub nodrop: f32,
    pub condition: [u8; 2],
    pub categories: HashMap<String, u32>,
    pub rows: Vec<[u32; 5]>,
    pub notable: usize,
    #[serde(default)]
    pub extra: Option<Extra>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Rules {
    pub generous_opens: u32,
    pub locker_locked: f32,
    pub locker_locked_per_tier: f32,
    pub force_damage: u8,
    pub noise_locker: u8,
    pub noise_locker_pry: u8,
    pub noise_locker_force: u8,
    pub noise_crate_pry: u8,
    pub noise_crate_force: u8,
    pub noise_corpse: u8,
    pub noise_pile: u8,
}

#[derive(Debug, Deserialize)]
struct LootFile {
    source: HashMap<String, SourceDef>,
    rules: Rules,
}

#[derive(Clone, Debug)]
pub struct LootTables {
    pub sources: HashMap<String, SourceDef>,
    pub rules: Rules,
    /// PRD constant per (source name, tier index 0..5), 0 when the row has no notables, 1 when only notables
    prd: HashMap<(String, usize), f32>,
}

impl LootTables {
    pub fn from_toml(text: &str) -> Result<LootTables, String> {
        let f: LootFile = toml::from_str(text).map_err(|e| format!("loot.toml: {e}"))?;
        let mut prd = HashMap::new();
        for (name, s) in &f.source {
            if s.rows.len() != 6 {
                return Err(format!("loot.toml: source {name} needs 6 tier rows, has {}", s.rows.len()));
            }
            if s.count[0] == 0 || s.count[1] < s.count[0] {
                return Err(format!("loot.toml: source {name} has a bad count"));
            }
            if s.categories.is_empty() {
                return Err(format!("loot.toml: source {name} has no categories"));
            }
            for (t, row) in s.rows.iter().enumerate() {
                let total: u32 = row.iter().sum();
                if total == 0 {
                    return Err(format!("loot.toml: source {name} row {} sums to 0", t + 1));
                }
                let notable: u32 = row[s.notable.min(4)..].iter().sum();
                let p = notable as f32 / total as f32;
                prd.insert((name.clone(), t), prd_constant(p));
            }
        }
        Ok(LootTables { sources: f.source, rules: f.rules, prd })
    }

    pub fn shipped() -> LootTables {
        LootTables::from_toml(LOOT_TOML).expect("shipped loot tables")
    }

    pub fn source(&self, name: &str) -> Option<&SourceDef> {
        self.sources.get(name)
    }

    pub fn prd_c(&self, name: &str, tier: u8) -> f32 {
        *self.prd.get(&(name.to_string(), tier_row(tier))).unwrap_or(&0.0)
    }
}

fn tier_row(tier: u8) -> usize {
    (tier.clamp(1, 6) - 1) as usize
}

/// Expected proc probability of a PRD with constant `c` (P(N) = c * N, reset on proc).
pub fn prd_expected(c: f32) -> f32 {
    if c <= 0.0 {
        return 0.0;
    }
    if c >= 1.0 {
        return 1.0;
    }
    let mut expected_n = 0.0f32;
    let mut cum_fail = 1.0f32;
    let mut n = 1u32;
    loop {
        let p = (c * n as f32).min(1.0);
        expected_n += n as f32 * cum_fail * p;
        cum_fail *= 1.0 - p;
        if p >= 1.0 || n > 10_000 {
            break;
        }
        n += 1;
    }
    1.0 / expected_n
}

/// The constant C whose PRD averages the nominal probability `p` (Dota's table, solved by bisection).
pub fn prd_constant(p: f32) -> f32 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0f32, p);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if prd_expected(mid) > p {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Locker,
    Crate,
    Pile,
    Corpse,
    Cache,
    /// a dead Oathkeeper scout
    ScoutBody,
    /// a dead sewerat
    RatCorpse,
}

impl SourceKind {
    pub fn table(self) -> &'static str {
        match self {
            SourceKind::Locker => "locker",
            SourceKind::Crate => "crate",
            SourceKind::Pile => "deposit",
            SourceKind::Corpse => "corpse_newcomer",
            SourceKind::Cache => "cache",
            SourceKind::ScoutBody => "scout_drop",
            SourceKind::RatCorpse => "rat_salvage",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            SourceKind::Locker => "locker",
            SourceKind::Crate => "crate",
            SourceKind::Pile => "pile",
            SourceKind::Corpse => "corpse",
            SourceKind::Cache => "cache",
            SourceKind::ScoutBody => "scout body",
            SourceKind::RatCorpse => "rat corpse",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceState {
    Closed,
    Open,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenMethod {
    Hands,
    PryBar,
    Key,
}

#[derive(Clone, Debug)]
pub struct LootSource {
    pub x: u16,
    pub y: u16,
    pub kind: SourceKind,
    pub tier: u8,
    pub chunk: u16,
    pub locked: bool,
    pub state: SourceState,
    pub items: Vec<ItemInstance>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenResult {
    pub revealed: usize,
    pub notable: usize,
    pub damaged: usize,
    pub noise: u8,
    pub generous: bool,
    pub forced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenError {
    NoSuchSource,
    AlreadyOpen,
}

#[derive(Clone, Debug)]
pub struct LootState {
    pub sources: Vec<LootSource>,
    pub seed: u64,
    pub run_nonce: u64,
    /// runs of this route before this one
    pub route_runs: u16,
    pub day: u32,
    pub stratum: u8,
    pub opened: u32,
    /// picks since the last notable (PRD counter)
    pub pity_n: u32,
    pub longest_dry: u32,
}

fn mix(a: u64, b: u64, c: u64, tag: u64) -> u64 {
    let mut z = a ^ b.rotate_left(21) ^ c.rotate_left(42) ^ tag.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl LootState {
    /// Sources from the compiled route: lockers and crates as authored, newcomer corpses from the corpse spawns.
    /// Locked state rolls here from (seed, nonce, index).
    pub fn new(map: &RouteMap, tables: &LootTables, run_nonce: u64, route_runs: u16, day: u32) -> LootState {
        let mut sources = Vec::new();
        for c in &map.containers {
            let kind = if c.kind == ContainerKind::Crate { SourceKind::Crate } else { SourceKind::Locker };
            sources.push(LootSource { x: c.x, y: c.y, kind, tier: c.tier, chunk: c.chunk, locked: false, state: SourceState::Closed, items: Vec::new() });
        }
        for s in map.spawns.iter().filter(|s| s.kind == SpawnKind::Corpse) {
            sources.push(LootSource { x: s.x, y: s.y, kind: SourceKind::Corpse, tier: map.stratum, chunk: s.chunk, locked: false, state: SourceState::Closed, items: Vec::new() });
        }
        for (i, s) in sources.iter_mut().enumerate() {
            if s.kind == SourceKind::Locker {
                let mut r = Pcg32::new(mix(map.seed, run_nonce, i as u64, 0x10C));
                let p = tables.rules.locker_locked + tables.rules.locker_locked_per_tier * (s.tier.max(1) - 1) as f32;
                s.locked = r.f32() < p;
            }
        }
        LootState { sources, seed: map.seed, run_nonce, route_runs, day, stratum: map.stratum, opened: 0, pity_n: 0, longest_dry: 0 }
    }

    pub fn source_at(&self, x: usize, y: usize) -> Option<usize> {
        self.sources.iter().position(|s| s.x as usize == x && s.y as usize == y)
    }

    /// A dug deposit becomes a pile on its tile, already open with its contents.
    pub fn add_pile(&mut self, tables: &LootTables, table: &ItemTable, x: usize, y: usize, chunk: u16) -> usize {
        self.add_open_source(tables, table, SourceKind::Pile, x, y, chunk)
    }

    /// A source that appears during the run, already open: a dug pile, a dead scout, a dead sewerat.
    pub fn add_open_source(&mut self, tables: &LootTables, table: &ItemTable, kind: SourceKind, x: usize, y: usize, chunk: u16) -> usize {
        let idx = self.sources.len();
        let mut src = LootSource { x: x as u16, y: y as u16, kind, tier: self.stratum, chunk, locked: false, state: SourceState::Open, items: Vec::new() };
        let mut rng = Pcg32::new(mix(self.seed, self.run_nonce, idx as u64 + 10_000, 0x917E));
        let generous = kind == SourceKind::Pile && self.generous();
        let (items, _) = self.roll(tables, table, kind.table(), self.stratum, generous, 0, &mut rng, chunk);
        src.items = items;
        if src.items.is_empty() {
            src.state = SourceState::Empty;
        }
        self.sources.push(src);
        idx
    }

    fn generous(&self) -> bool {
        self.route_runs == 0 && self.opened < 3
    }

    /// Open a source. Contents are generated on the first open; the method decides noise and bonuses.
    pub fn open(&mut self, tables: &LootTables, table: &ItemTable, idx: usize, method: OpenMethod) -> Result<OpenResult, OpenError> {
        let n = self.sources.len();
        if idx >= n {
            return Err(OpenError::NoSuchSource);
        }
        if self.sources[idx].state != SourceState::Closed {
            return Err(OpenError::AlreadyOpen);
        }
        let (kind, tier, chunk, locked) = {
            let s = &self.sources[idx];
            (s.kind, s.tier, s.chunk, s.locked)
        };
        let r = &tables.rules;
        let (noise, extra_picks, forced) = match (kind, method) {
            (SourceKind::Locker, _) if !locked => (r.noise_locker, 0, false),
            (SourceKind::Locker, OpenMethod::Hands) => (r.noise_locker_force, 0, true),
            (SourceKind::Locker, _) => (r.noise_locker_pry, 0, false),
            (SourceKind::Crate, OpenMethod::PryBar) | (SourceKind::Cache, OpenMethod::PryBar) => (r.noise_crate_pry, 1, false),
            (SourceKind::Crate, _) | (SourceKind::Cache, _) => (r.noise_crate_force, 0, false),
            (SourceKind::Corpse, _) | (SourceKind::ScoutBody, _) | (SourceKind::RatCorpse, _) => (r.noise_corpse, 0, false),
            (SourceKind::Pile, _) => (r.noise_pile, 0, false),
        };
        let generous = self.generous() && matches!(kind, SourceKind::Locker | SourceKind::Crate | SourceKind::Cache);
        let mut rng = Pcg32::new(mix(self.seed, self.run_nonce, idx as u64, 0x0B5E));
        let (mut items, notable) = self.roll(tables, table, kind.table(), tier, generous, extra_picks, &mut rng, chunk);
        let mut damaged = 0;
        if forced {
            for (k, it) in items.iter_mut().enumerate() {
                if k % 2 == 0 {
                    it.condition = it.condition.saturating_sub(r.force_damage);
                    damaged += 1;
                }
            }
        }
        let revealed = items.len();
        let s = &mut self.sources[idx];
        s.items = items;
        s.state = if revealed == 0 { SourceState::Empty } else { SourceState::Open };
        if matches!(kind, SourceKind::Locker | SourceKind::Crate | SourceKind::Cache) {
            self.opened += 1;
        }
        Ok(OpenResult { revealed, notable, damaged, noise, generous, forced })
    }

    /// The D2-style resolution with PRD on notables. Returns (items, notable count).
    #[allow(clippy::too_many_arguments)]
    pub fn roll(&mut self, tables: &LootTables, table: &ItemTable, source: &str, tier: u8, generous: bool, extra_picks: u8, rng: &mut Pcg32, chunk: u16) -> (Vec<ItemInstance>, usize) {
        let Some(def) = tables.source(source) else { return (Vec::new(), 0) };
        // S2 tops out at Uncommon (design): in S1-S2 a crate's +1 tier buys picks and condition, not a higher row;
        // generosity is the one scripted exception
        let capped = if self.stratum <= 2 { tier.clamp(1, 6).min(self.stratum.max(1)) } else { tier.clamp(1, 6) };
        let eff_tier = if generous { (capped + 1).min(6) } else { capped };
        let row = def.rows[tier_row(eff_tier)];
        let c = tables.prd_c(source, eff_tier);
        let notable_from = def.notable.min(4);
        let picks = rng.range(def.count[0] as i32, def.count[1] as i32) as u8 + extra_picks;
        let mut out = Vec::new();
        let mut notables = 0usize;
        let cats: Vec<(Category, u32)> = def
            .categories
            .iter()
            .filter_map(|(k, w)| category_of(k).map(|c| (c, *w)))
            .collect();
        let cat_total: u32 = cats.iter().map(|(_, w)| w).sum();
        for _ in 0..picks {
            if def.nodrop > 0.0 && rng.f32() < def.nodrop {
                continue;
            }
            // notable by PRD
            let notable_w: u32 = row[notable_from..].iter().sum();
            let plain_w: u32 = row[..notable_from].iter().sum();
            let pick_notable = if notable_w == 0 {
                false
            } else if plain_w == 0 {
                true
            } else {
                let hit = rng.f32() < c * (self.pity_n + 1) as f32;
                if hit {
                    self.pity_n = 0;
                } else {
                    self.pity_n += 1;
                    self.longest_dry = self.longest_dry.max(self.pity_n);
                }
                hit
            };
            let tier_idx = if pick_notable { weighted(rng, &row[notable_from..]) + notable_from } else { weighted(rng, &row[..notable_from]) };
            // category
            let mut roll = rng.below(cat_total.max(1));
            let mut cat = cats[0].0;
            for (c, w) in &cats {
                if roll < *w {
                    cat = *c;
                    break;
                }
                roll -= w;
            }
            let Some((id, idef)) = pick_item(table, cat, tier_idx, rng) else { continue };
            let cond = rng.range(def.condition[0] as i32, def.condition[1] as i32) as u8;
            let inst = ItemInstance::found(id, cond, Provenance { stratum: self.stratum, chunk, day: self.day });
            if idef.tier >= TIERS[notable_from] {
                notables += 1;
            }
            out.push(inst);
        }
        if let Some(extra) = &def.extra {
            if rng.f32() < extra.chance {
                if let Some(id) = table.id_of(&extra.item) {
                    let cond = rng.range(def.condition[0] as i32, def.condition[1] as i32) as u8;
                    out.push(ItemInstance::found(id, cond, Provenance { stratum: self.stratum, chunk, day: self.day }));
                }
            }
        }
        // rare last: the manifest unrolls common first and the notable with a longer beat (Pass 2, Pillar 1)
        out.sort_by_key(|it| table.def(it.def).tier);
        (out, notables)
    }

    /// Move item `k` of source `idx` into the rig's backpack. Ok(true) taken, Ok(false) no room.
    pub fn take(&mut self, table: &ItemTable, idx: usize, k: usize, rig: &mut Rig) -> Result<bool, OpenError> {
        let s = self.sources.get_mut(idx).ok_or(OpenError::NoSuchSource)?;
        if s.state != SourceState::Open || k >= s.items.len() {
            return Err(OpenError::NoSuchSource);
        }
        let def = table.def(s.items[k].def);
        let item = s.items[k].clone();
        match rig.backpack.auto_place(def, item) {
            Ok(_) => {
                s.items.remove(k);
                if s.items.is_empty() {
                    s.state = SourceState::Empty;
                }
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Take everything that fits, heaviest value first. Returns (taken, left).
    pub fn take_all(&mut self, table: &ItemTable, idx: usize, rig: &mut Rig) -> (usize, usize) {
        let Some(s) = self.sources.get(idx) else { return (0, 0) };
        if s.state != SourceState::Open {
            return (0, 0);
        }
        // value order so the best things go in first when space is short
        let mut order: Vec<usize> = (0..s.items.len()).collect();
        order.sort_by_key(|&k| std::cmp::Reverse(value_of(table.def(s.items[k].def), &s.items[k])));
        let mut taken = 0;
        // take by descending index so removals do not shift pending indices
        let mut picks: Vec<usize> = Vec::new();
        for k in order {
            picks.push(k);
        }
        picks.sort_unstable_by(|a, b| b.cmp(a));
        for k in picks {
            if let Ok(true) = self.take(table, idx, k, rig) {
                taken += 1;
            }
        }
        let left = self.sources[idx].items.len();
        (taken, left)
    }

    pub fn open_count(&self) -> usize {
        self.sources.iter().filter(|s| s.state == SourceState::Open).count()
    }
}

/// Sell value if identified: base x condition / 100.
pub fn value_of(def: &ItemDef, inst: &ItemInstance) -> u32 {
    (def.value as f32 * inst.condition as f32 / 100.0).round() as u32
}

fn category_of(s: &str) -> Option<Category> {
    Some(match s {
        "chrome" => Category::Chrome,
        "part" => Category::Part,
        "organic" => Category::Organic,
        "tech" => Category::Tech,
        "gear" => Category::Gear,
        "contraband" => Category::Contraband,
        "key" => Category::Key,
        "curio" => Category::Curio,
        "relic" => Category::Relic,
        _ => return None,
    })
}

fn weighted(rng: &mut Pcg32, w: &[u32]) -> usize {
    let total: u32 = w.iter().sum();
    if total == 0 {
        return 0;
    }
    let mut roll = rng.below(total);
    for (i, &x) in w.iter().enumerate() {
        if roll < x {
            return i;
        }
        roll -= x;
    }
    w.len() - 1
}

/// An item of this category at this tier, uniformly; steps down then up a tier when the table has none.
fn pick_item<'a>(table: &'a ItemTable, cat: Category, tier_idx: usize, rng: &mut Pcg32) -> Option<(items::ItemId, &'a ItemDef)> {
    let order: Vec<usize> = {
        let mut v = vec![tier_idx];
        for d in 1..5 {
            if tier_idx >= d {
                v.push(tier_idx - d);
            }
        }
        for u in 1..5 {
            if tier_idx + u < 5 {
                v.push(tier_idx + u);
            }
        }
        v
    };
    for t in order {
        let tier = TIERS[t];
        let cands: Vec<usize> = table.defs.iter().enumerate().filter(|(_, d)| d.category == cat && d.tier == tier && d.id != "org_corpse_tag").map(|(i, _)| i).collect();
        if !cands.is_empty() {
            let i = cands[rng.below(cands.len() as u32) as usize];
            return Some((items::ItemId(i as u16), &table.defs[i]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;

    const ITEMS: &str = include_str!("../../../../data/items.toml");

    fn setup(seed: u64, stratum: u8) -> (RouteMap, LootTables, ItemTable) {
        let l = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        let m = RouteMap::compile(&l, &route(&l, seed, stratum).unwrap());
        (m, LootTables::shipped(), ItemTable::from_toml(ITEMS).unwrap())
    }

    #[test]
    fn tables_parse_and_prd_constants_average_to_nominal() {
        let t = LootTables::shipped();
        assert!(t.sources.len() >= 7);
        for (p, c_expect) in [(0.05, 0.0038), (0.10, 0.0147), (0.25, 0.0847), (0.30, 0.1196)] {
            let c = prd_constant(p);
            assert!((c - c_expect).abs() < 0.002, "p {p}: C {c} vs Dota's {c_expect}");
            assert!((prd_expected(c) - p).abs() < 0.002);
        }
        assert_eq!(t.prd_c("locker", 1), 0.0, "no notables at tier 1");
        assert!(t.prd_c("locker", 3) > 0.0 && t.prd_c("locker", 3) < 0.12);
        assert_eq!(t.prd_c("scout_drop", 2), 1.0);
    }

    #[test]
    fn sources_come_from_the_route_and_lockers_lock_by_tier() {
        let (m, t, _) = setup(11, 1);
        let s = LootState::new(&m, &t, 1, 0, 1);
        let corpses = m.spawns.iter().filter(|s| s.kind == SpawnKind::Corpse).count();
        assert_eq!(s.sources.len(), m.containers.len() + corpses);
        assert!(s.sources.iter().any(|s| s.kind == SourceKind::Locker));
        let mut locked = 0;
        let mut lockers = 0;
        for seed in 0..60u64 {
            let (m, t, _) = setup(seed, 1);
            let s = LootState::new(&m, &t, seed, 0, 1);
            for src in s.sources.iter().filter(|s| s.kind == SourceKind::Locker) {
                lockers += 1;
                locked += src.locked as usize;
            }
        }
        let frac = locked as f32 / lockers as f32;
        assert!((0.2..0.4).contains(&frac), "locked fraction {frac}");
    }

    #[test]
    fn s1_and_s2_lockers_top_out_at_uncommon_and_tiers_climb() {
        let (_, t, table) = setup(1, 1);
        let mut by_stratum: Vec<(f32, [u32; 5])> = Vec::new();
        for stratum in 1..=3u8 {
            let mut total_value = 0u64;
            let mut opens = 0u64;
            let mut tiers = [0u32; 5];
            for seed in 0..40u64 {
                let (m, _, _) = setup(seed, stratum);
                let mut st = LootState::new(&m, &t, 77, 1, 1); // not the first run: no generosity
                for i in 0..st.sources.len() {
                    let locker = st.sources[i].kind == SourceKind::Locker;
                    let r = st.open(&t, &table, i, OpenMethod::PryBar).unwrap();
                    assert!(r.revealed <= 5);
                    opens += 1;
                    for it in &st.sources[i].items {
                        let d = table.def(it.def);
                        if locker {
                            tiers[d.tier.index()] += 1;
                        }
                        total_value += value_of(d, it) as u64;
                        assert!(it.found_in_run && it.ident == items::Ident::Unknown);
                        assert_eq!(it.provenance.unwrap().stratum, stratum);
                    }
                }
            }
            by_stratum.push((total_value as f32 / opens as f32, tiers));
        }
        // S1 and S2 lockers: common and uncommon only
        assert_eq!(by_stratum[0].1[2] + by_stratum[0].1[3] + by_stratum[0].1[4], 0, "S1 {:?}", by_stratum[0].1);
        assert_eq!(by_stratum[1].1[2] + by_stratum[1].1[3] + by_stratum[1].1[4], 0, "S2 {:?}", by_stratum[1].1);
        assert!(by_stratum[2].1[2] > 0, "S3 lockers never rolled a Rare");
        // value per container, all sources, roughly doubles each stratum (design): between x1.4 and x3.2 per step
        let means: Vec<f32> = by_stratum.iter().map(|b| b.0).collect();
        for w in means.windows(2) {
            let ratio = w[1] / w[0];
            assert!((1.4..=3.2).contains(&ratio), "stratum value ratio {ratio:.2}: {means:?}");
        }
    }

    #[test]
    fn generosity_applies_to_the_first_three_opens_of_a_first_run_only() {
        let (m, t, table) = setup(5, 1);
        let mut first = LootState::new(&m, &t, 3, 0, 1);
        let idx: Vec<usize> = (0..first.sources.len()).filter(|&i| matches!(first.sources[i].kind, SourceKind::Crate | SourceKind::Locker)).take(4).collect();
        assert!(idx.len() >= 4, "route needs four containers for this test");
        let mut flags = Vec::new();
        for &i in &idx {
            flags.push(first.open(&t, &table, i, OpenMethod::PryBar).unwrap().generous);
        }
        assert_eq!(flags, vec![true, true, true, false]);
        let mut later = LootState::new(&m, &t, 3, 1, 1);
        assert!(!later.open(&t, &table, idx[0], OpenMethod::PryBar).unwrap().generous);
        // same seed, nonce and index: same contents; a different nonce: different contents somewhere
        let mut again = LootState::new(&m, &t, 3, 1, 1);
        again.open(&t, &table, idx[0], OpenMethod::PryBar).unwrap();
        assert_eq!(later.sources[idx[0]].items, again.sources[idx[0]].items);
        let mut other = LootState::new(&m, &t, 4, 1, 1);
        let mut differs = false;
        for &i in &idx {
            other.open(&t, &table, i, OpenMethod::PryBar).unwrap();
            if other.sources[i].items != first.sources[i].items {
                differs = true;
            }
        }
        assert!(differs);
    }

    #[test]
    fn tools_change_noise_and_yield_and_forcing_damages() {
        let (m, t, table) = setup(9, 2);
        let mut st = LootState::new(&m, &t, 1, 1, 1);
        let crate_i = (0..st.sources.len()).find(|&i| st.sources[i].kind == SourceKind::Crate).unwrap();
        let pry = st.open(&t, &table, crate_i, OpenMethod::PryBar).unwrap();
        assert_eq!(pry.noise, t.rules.noise_crate_pry);
        assert_eq!(st.open(&t, &table, crate_i, OpenMethod::PryBar), Err(OpenError::AlreadyOpen));
        // the same crate forced by hand in a fresh state: louder and never more items
        let mut st2 = LootState::new(&m, &t, 1, 1, 1);
        let hands = st2.open(&t, &table, crate_i, OpenMethod::Hands).unwrap();
        assert_eq!(hands.noise, t.rules.noise_crate_force);
        assert!(hands.revealed <= pry.revealed);
        // a locked locker forced by hand damages half its contents
        let mut found = None;
        for seed in 0..80u64 {
            let (m, _, _) = setup(seed, 1);
            let st = LootState::new(&m, &t, 2, 1, 1);
            if let Some(i) = (0..st.sources.len()).find(|&i| st.sources[i].kind == SourceKind::Locker && st.sources[i].locked) {
                found = Some((m, i));
                break;
            }
        }
        let (m, i) = found.expect("a locked locker in 80 routes");
        let mut st = LootState::new(&m, &t, 2, 1, 1);
        let r = st.open(&t, &table, i, OpenMethod::Hands).unwrap();
        assert!(r.forced && r.noise == t.rules.noise_locker_force);
        assert_eq!(r.damaged, st.sources[i].items.len().div_ceil(2));
        let mut st3 = LootState::new(&m, &t, 2, 1, 1);
        let r3 = st3.open(&t, &table, i, OpenMethod::PryBar).unwrap();
        assert!(!r3.forced && r3.damaged == 0 && r3.noise == t.rules.noise_locker_pry);
    }

    #[test]
    fn piles_corpses_and_oathkeeper_drops_roll_their_tables() {
        let (m, t, table) = setup(2, 1);
        let mut st = LootState::new(&m, &t, 1, 1, 1);
        let pile = st.add_pile(&t, &table, 5, 12, 0);
        assert!(st.sources[pile].kind == SourceKind::Pile);
        assert!(st.sources[pile].items.len() <= 2);
        for it in &st.sources[pile].items {
            let d = table.def(it.def);
            assert!(matches!(d.category, Category::Part | Category::Chrome));
            assert!((40..=80).contains(&it.condition));
        }
        if let Some(ci) = (0..st.sources.len()).find(|&i| st.sources[i].kind == SourceKind::Corpse) {
            let r = st.open(&t, &table, ci, OpenMethod::Hands).unwrap();
            assert_eq!(r.noise, t.rules.noise_corpse);
            for it in &st.sources[ci].items {
                assert!(it.condition <= 60);
            }
        }
        let mut rng = Pcg32::new(99);
        let mut blackline = 0;
        let mut cloth = 0;
        for _ in 0..50 {
            let (items, _) = st.roll(&t, &table, "scout_drop", 3, false, 0, &mut rng, 0);
            assert!(!items.is_empty());
            for it in &items {
                let d = table.def(it.def);
                if d.id == "con_vestment_cloth" {
                    cloth += 1;
                } else {
                    assert_eq!(d.tier, Tier::Blackline, "{}", d.id);
                    blackline += 1;
                }
            }
        }
        assert!(blackline >= 50 && (10..=40).contains(&cloth), "blackline {blackline} cloth {cloth}");
        let (items, _) = st.roll(&t, &table, "carrier_drop", 4, false, 0, &mut rng, 0);
        assert!(items.iter().any(|it| table.def(it.def).id == "ger_lamp_pole"));
    }

    #[test]
    fn prd_keeps_dry_streaks_short_and_take_moves_items_into_the_rig() {
        let (m, t, table) = setup(4, 3);
        let mut st = LootState::new(&m, &t, 8, 1, 1);
        let mut rng = Pcg32::new(5);
        let c = t.prd_c("locker", 3);
        let bound = (1.0 / c).ceil() as u32;
        let mut notables = 0usize;
        let mut picks = 0usize;
        for _ in 0..400 {
            let (items, n) = st.roll(&t, &table, "locker", 3, false, 0, &mut rng, 0);
            notables += n;
            picks += items.len();
        }
        assert!(st.longest_dry < bound, "longest dry streak {} vs PRD bound {bound}", st.longest_dry);
        let rate = notables as f32 / picks.max(1) as f32;
        assert!((0.06..0.20).contains(&rate), "notable rate {rate}");
        // taking
        let i = (0..st.sources.len()).find(|&i| st.sources[i].kind == SourceKind::Crate).unwrap();
        st.open(&t, &table, i, OpenMethod::PryBar).unwrap();
        let before = st.sources[i].items.len();
        let mut rig = Rig::new(8, 5, 4, 18.0);
        let (taken, left) = st.take_all(&table, i, &mut rig);
        assert_eq!(taken + left, before);
        assert_eq!(rig.backpack.count(), taken);
        if left == 0 {
            assert_eq!(st.sources[i].state, SourceState::Empty);
        }
        let mut tiny = Rig::new(1, 1, 0, 5.0);
        let mut st2 = LootState::new(&m, &t, 8, 1, 1);
        st2.open(&t, &table, i, OpenMethod::PryBar).unwrap();
        let (taken2, left2) = st2.take_all(&table, i, &mut tiny);
        assert!(taken2 <= 1 && left2 >= before - 1);
    }
}
