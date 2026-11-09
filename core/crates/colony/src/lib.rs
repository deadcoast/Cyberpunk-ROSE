//! The city on the Mat (Full Design v3.9, Colony mode): the 24 structures from `data/structures.toml`, placement
//! and building, zones and their filters, homes and the shift's meals, light, water and sleep, districts that form
//! around a hub and elect a foreman, and the foreman's petitions. Every number here is the design's; the Godot
//! screens draw what this crate holds and the roster (`people`) reads the `ShiftContext` it writes.

use std::collections::HashMap;

use mat::sim::{hy, sub};
use mat::MatSim;
use people::{Job, Need, Record, Roster, ShiftContext, ShiftWork, Stat, Status, TraitFile};
use serde::{Deserialize, Serialize};

pub const STRUCTURES_TOML: &str = include_str!("../../../../data/structures.toml");

/// Signal a structure needs under it unless its def says otherwise, and the floor for any zone cell.
pub const SIGNAL_FLOOR: f32 = 0.3;
/// Tiles from a hub within which structures and beds belong to its district.
pub const DISTRICT_RADIUS: i32 = 8;
/// Built structures a hub needs to form a district.
pub const DISTRICT_MIN_STRUCTURES: usize = 3;
/// Shifts a foreman serves before the next election.
pub const TERM_SHIFTS: u8 = 10;
/// Shifts a district gets to answer a petition.
pub const PETITION_SHIFTS: u8 = 5;
/// Shifts of a need under 30 before the foreman petitions.
pub const PETITION_AFTER: u8 = 2;
/// Nutrient a cell needs for a meal, and what the meal takes.
pub const MEAL_NUTRIENT: f32 = 0.2;
pub const MEAL_TAKES: f32 = 0.05;
pub const PASTE_EDDIES: u32 = 2;
/// City tiers by population: Hamlet, Quarter, City, Empire.
pub const CITY_TIERS: [usize; 4] = [20, 100, 500, 2000];
pub const CITY_TIER_NAMES: [&str; 4] = ["Hamlet", "Quarter", "City", "Empire"];
pub const DISTRICT_CAP: [usize; 4] = [1, 3, 8, 20];

// ------------------------------------------------------------------ data

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cost {
    #[serde(default)]
    pub parts: u32,
    #[serde(default)]
    pub chrome: u32,
    #[serde(default)]
    pub eddies: u32,
}

impl Cost {
    pub fn times(self, n: u32) -> Cost {
        Cost { parts: self.parts * n, chrome: self.chrome * n, eddies: self.eddies * n }
    }
    pub fn is_zero(self) -> bool {
        self.parts == 0 && self.chrome == 0 && self.eddies == 0
    }
    pub fn text(self) -> String {
        let mut v = Vec::new();
        if self.parts > 0 {
            v.push(format!("{} Parts", self.parts));
        }
        if self.chrome > 0 {
            v.push(format!("{} Chrome", self.chrome));
        }
        if self.eddies > 0 {
            v.push(format!("{} e", self.eddies));
        }
        if v.is_empty() { "nothing".into() } else { v.join(", ") }
    }
}

fn default_tiers() -> u8 {
    3
}
fn default_hp() -> i32 {
    100
}

#[derive(Clone, Debug, Deserialize)]
pub struct StructureDef {
    pub id: String,
    pub name: String,
    pub footprint: [u8; 2],
    #[serde(default)]
    pub sigma: f32,
    #[serde(default)]
    pub node: bool,
    #[serde(default)]
    pub grate: bool,
    #[serde(default)]
    pub relay: bool,
    #[serde(default)]
    pub prebuilt: bool,
    #[serde(default)]
    pub heart_tier: u8,
    #[serde(default = "default_tiers")]
    pub tiers: u8,
    pub cost: Cost,
    pub build: u32,
    #[serde(default)]
    pub hosts: String,
    /// level of the hosted job's skill a slimo needs to take a slot (the clinic Medic 5, the Table Appraise 3)
    #[serde(default)]
    pub gate: u8,
    #[serde(default)]
    pub slots: Vec<u8>,
    #[serde(default = "default_hp")]
    pub hp: i32,
    #[serde(default)]
    pub light: Vec<i32>,
    #[serde(default)]
    pub stock: Vec<i32>,
    #[serde(default)]
    pub beds: Vec<i32>,
    #[serde(default)]
    pub rest: Vec<i32>,
    #[serde(default)]
    pub cells: Vec<i32>,
    #[serde(default)]
    pub capacity: Vec<i32>,
    #[serde(default)]
    pub brine: Vec<i32>,
    #[serde(default)]
    pub dry_radius: Vec<i32>,
    #[serde(default)]
    pub attention: Vec<i32>,
    #[serde(default)]
    pub mood: Vec<i32>,
    #[serde(default)]
    pub identify: Vec<i32>,
    #[serde(default)]
    pub range: Vec<i32>,
    #[serde(default)]
    pub damage: Vec<i32>,
    #[serde(default)]
    pub salt: Vec<i32>,
    #[serde(default)]
    pub upkeep: Vec<i32>,
    #[serde(default)]
    pub glut_bonus: Vec<i32>,
    #[serde(default)]
    pub recipe_tier: Vec<i32>,
    #[serde(default)]
    pub crew: Vec<i32>,
    #[serde(default)]
    pub stash_grid: Vec<[u8; 2]>,
    pub text: String,
    pub art: String,
}

fn at(v: &[i32], tier: u8) -> i32 {
    if v.is_empty() {
        0
    } else {
        v[(tier.max(1) as usize - 1).min(v.len() - 1)]
    }
}

impl StructureDef {
    pub fn job(&self) -> Option<Job> {
        if self.hosts.is_empty() { None } else { Job::from_key(&self.hosts) }
    }
    pub fn slots_at(&self, tier: u8) -> u8 {
        if self.slots.is_empty() { 0 } else { self.slots[(tier.max(1) as usize - 1).min(self.slots.len() - 1)] }
    }
    pub fn light_at(&self, tier: u8) -> i32 {
        at(&self.light, tier)
    }
    pub fn beds_at(&self, tier: u8) -> i32 {
        at(&self.beds, tier)
    }
    pub fn rest_at(&self, tier: u8) -> i32 {
        at(&self.rest, tier)
    }
    pub fn cells_at(&self, tier: u8) -> i32 {
        at(&self.cells, tier)
    }
    pub fn stock_at(&self, tier: u8) -> i32 {
        at(&self.stock, tier)
    }
    pub fn value_at(&self, field: &str, tier: u8) -> i32 {
        match field {
            "light" => at(&self.light, tier),
            "stock" => at(&self.stock, tier),
            "beds" => at(&self.beds, tier),
            "rest" => at(&self.rest, tier),
            "cells" => at(&self.cells, tier),
            "capacity" => at(&self.capacity, tier),
            "brine" => at(&self.brine, tier),
            "dry_radius" => at(&self.dry_radius, tier),
            "attention" => at(&self.attention, tier),
            "mood" => at(&self.mood, tier),
            "identify" => at(&self.identify, tier),
            "range" => at(&self.range, tier),
            "damage" => at(&self.damage, tier),
            "salt" => at(&self.salt, tier),
            "upkeep" => at(&self.upkeep, tier),
            "glut_bonus" => at(&self.glut_bonus, tier),
            "recipe_tier" => at(&self.recipe_tier, tier),
            "crew" => at(&self.crew, tier),
            _ => 0,
        }
    }
    /// The per-tier numbers the def carries, in words: "light 4 / 6 / 8".
    pub fn effects(&self) -> Vec<(String, Vec<i32>)> {
        let mut out = Vec::new();
        for (k, v) in [
            ("light", &self.light),
            ("stock", &self.stock),
            ("beds", &self.beds),
            ("rest", &self.rest),
            ("cells", &self.cells),
            ("capacity", &self.capacity),
            ("brine", &self.brine),
            ("dry radius", &self.dry_radius),
            ("attention", &self.attention),
            ("mood", &self.mood),
            ("identify", &self.identify),
            ("range", &self.range),
            ("damage", &self.damage),
            ("salt", &self.salt),
            ("upkeep", &self.upkeep),
            ("glut", &self.glut_bonus),
            ("recipe tier", &self.recipe_tier),
            ("crew", &self.crew),
        ] {
            if !v.is_empty() {
                out.push((k.to_string(), v.clone()));
            }
        }
        out
    }
    /// Cost of a tier (tier 2 is twice tier 1, tier 3 three times).
    pub fn cost_at(&self, tier: u8) -> Cost {
        self.cost.times(tier.max(1) as u32)
    }
    pub fn build_at(&self, tier: u8) -> u32 {
        self.build * tier.max(1) as u32
    }
    pub fn hp_at(&self, tier: u8) -> i32 {
        self.hp * tier.max(1) as i32
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct StructureFile {
    pub structure: Vec<StructureDef>,
}

impl StructureFile {
    pub fn from_toml(s: &str) -> Result<Self, String> {
        let f: StructureFile = toml::from_str(s).map_err(|e| e.to_string())?;
        if f.structure.len() != 24 {
            return Err(format!("{} structures, the design has 24", f.structure.len()));
        }
        let mut ids = std::collections::HashSet::new();
        for d in &f.structure {
            if !ids.insert(d.id.clone()) {
                return Err(format!("structure {} twice", d.id));
            }
            if d.footprint[0] == 0 || d.footprint[1] == 0 {
                return Err(format!("{}: empty footprint", d.id));
            }
            if !d.hosts.is_empty() && Job::from_key(&d.hosts).is_none() {
                return Err(format!("{}: hosts unknown job {}", d.id, d.hosts));
            }
            if !d.hosts.is_empty() && d.slots.is_empty() {
                return Err(format!("{}: hosts {} with no slots", d.id, d.hosts));
            }
            if !d.prebuilt && d.cost.is_zero() {
                return Err(format!("{}: free to build", d.id));
            }
            if !d.prebuilt && d.build == 0 {
                return Err(format!("{}: no build points", d.id));
            }
            if d.tiers < 1 || d.tiers > 4 {
                return Err(format!("{}: {} tiers", d.id, d.tiers));
            }
        }
        for want in ["heart", "pit", "pod", "bunkhouse", "bench", "route_board", "lume_farm", "lung", "shrine", "lantern", "forge"] {
            if !ids.contains(want) {
                return Err(format!("structure {want} missing"));
            }
        }
        Ok(f)
    }
    pub fn shipped() -> Self {
        Self::from_toml(STRUCTURES_TOML).expect("shipped structures")
    }
    pub fn index(&self, id: &str) -> Option<usize> {
        self.structure.iter().position(|d| d.id == id)
    }
    pub fn get(&self, id: &str) -> Option<&StructureDef> {
        self.structure.iter().find(|d| d.id == id)
    }
}

// ------------------------------------------------------------------ the city

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Structure {
    pub id: u32,
    /// index into the StructureFile
    pub def: usize,
    pub x: u16,
    pub y: u16,
    pub tier: u8,
    pub hp: i32,
    pub built: bool,
    /// build points so far toward the current tier (or upgrade)
    pub progress: u32,
    /// the current tier's cost has been taken from the Stash
    pub paid: bool,
    /// an upgrade in progress: the tier being built toward
    pub upgrading_to: Option<u8>,
    pub district: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneKind {
    Stockpile,
    Hab,
    Quarantine,
    Kill,
}

impl ZoneKind {
    pub fn name(self) -> &'static str {
        match self {
            ZoneKind::Stockpile => "stockpile",
            ZoneKind::Hab => "hab",
            ZoneKind::Quarantine => "quarantine",
            ZoneKind::Kill => "kill",
        }
    }
    pub fn from_key(k: &str) -> Option<ZoneKind> {
        Some(match k {
            "stockpile" => ZoneKind::Stockpile,
            "hab" => ZoneKind::Hab,
            "quarantine" => ZoneKind::Quarantine,
            "kill" => ZoneKind::Kill,
            _ => return None,
        })
    }
}

/// What a stockpile takes (Full Design, Zones): categories, a tier floor, a condition floor.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub categories: Vec<String>,
    pub min_tier: u8,
    pub min_condition: u8,
}

impl Filter {
    pub fn accepts(&self, category: &str, tier: u8, condition: u8) -> bool {
        (self.categories.is_empty() || self.categories.iter().any(|c| c == category)) && tier >= self.min_tier && condition >= self.min_condition
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: u32,
    pub kind: ZoneKind,
    /// cell indices
    pub cells: Vec<u32>,
    pub filter: Filter,
    /// the Rack or Cold Store a stockpile hangs on
    pub anchor: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Petition {
    pub need: Need,
    /// structure ids that answer it, the first the one asked for
    pub asks: Vec<String>,
    pub shifts_left: u8,
    pub missed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct District {
    pub id: u32,
    pub name: String,
    pub hub: (u16, u16),
    pub structures: Vec<u32>,
    pub members: Vec<u32>,
    pub foreman: Option<u32>,
    pub term_left: u8,
    /// shifts the foreman has served, for ties
    pub served: u32,
    pub priorities: [u8; 9],
    pub petition: Option<Petition>,
    /// shifts of the +10 for a met petition left
    pub boon_shifts: u8,
    /// a wake: no jobs this shift
    pub wake: bool,
    /// consecutive shifts each need averaged under 30
    pub low: [u8; 5],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaceError {
    OffMap,
    NotMat(u16, u16),
    Signal(u16, u16),
    NeedsNode,
    NeedsGrate,
    NeedsRelay,
    Overlaps(u32),
    HeartTier(u8),
    Unknown,
}

impl std::fmt::Display for PlaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceError::OffMap => write!(f, "off the map"),
            PlaceError::NotMat(x, y) => write!(f, "nothing to stand on at {x},{y}"),
            PlaceError::Signal(x, y) => write!(f, "signal too low at {x},{y}"),
            PlaceError::NeedsNode => write!(f, "needs a Node under it"),
            PlaceError::NeedsGrate => write!(f, "needs a grate within two tiles"),
            PlaceError::NeedsRelay => write!(f, "needs a Relay under it"),
            PlaceError::Overlaps(id) => write!(f, "overlaps structure #{id}"),
            PlaceError::HeartTier(t) => write!(f, "needs the Heart at tier {t}"),
            PlaceError::Unknown => write!(f, "no such structure"),
        }
    }
}

/// One shift's work and its consequences, for the bridge to apply to the Stash and the roster to read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShiftOutcome {
    /// (slimo, job, structure) this shift
    pub assignments: Vec<(u32, Job, u32)>,
    pub built: Vec<u32>,
    pub upgraded: Vec<u32>,
    /// ghosts the Stash could not pay for: (structure, what it lacks)
    pub waiting: Vec<(u32, Cost)>,
    /// Bench jobs the Tweak slots advance
    pub bench_slots: u32,
    /// items the Appraise slots identify
    pub identifies: u32,
    /// salt made, doses made, brine made
    pub salt: u32,
    pub doses: u32,
    pub brine: u32,
    /// eddies the lanterns burn
    pub upkeep: u32,
    /// slimos a Medic treated
    pub treated: Vec<u32>,
    /// slimos who ate paste (2 e each)
    pub paste_meals: u32,
    /// who worked the market, for the sales' Trade XP
    pub sellers: Vec<u32>,
    pub contexts: HashMap<u32, ShiftContext>,
    pub work: ShiftWork,
    pub districts_formed: Vec<u32>,
    pub elections: Vec<(u32, u32)>,
    pub petitions_raised: Vec<(u32, Need)>,
    pub petitions_met: Vec<u32>,
    pub petitions_missed: Vec<u32>,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Colony {
    pub structures: Vec<Structure>,
    pub zones: Vec<Zone>,
    pub districts: Vec<District>,
    /// slimo -> structure with their bed
    pub homes: HashMap<u32, u32>,
    pub next_id: u32,
    pub shift: u32,
    /// names not yet given to a district
    pub names_left: Vec<String>,
}

impl Default for Colony {
    fn default() -> Self {
        Self::new()
    }
}

pub const DISTRICT_NAMES: [&str; 20] = [
    "Sump Row", "Grate Nine", "the Culvert", "Brine Steps", "Lamp Cut", "the Outfall", "Rebar Lane", "Dimmers' Reach", "the Pipeworks", "Saltline",
    "Low Drain", "the Wick", "Ninth Chamber", "Cord End", "the Shallows", "Rust Gate", "Flicker Row", "the Long Pipe", "Crawlspace", "Heart Ward",
];

fn cheb(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

impl Colony {
    pub fn new() -> Self {
        Colony { structures: Vec::new(), zones: Vec::new(), districts: Vec::new(), homes: HashMap::new(), next_id: 1, shift: 0, names_left: DISTRICT_NAMES.iter().map(|s| s.to_string()).collect() }
    }

    /// The city tier from the population: 1 Hamlet (under 100), 2 Quarter, 3 City, 4 Empire.
    pub fn city_tier(population: usize) -> u8 {
        1 + CITY_TIERS[1..].iter().filter(|&&t| population >= t).count() as u8
    }

    pub fn get(&self, id: u32) -> Option<&Structure> {
        self.structures.iter().find(|s| s.id == id)
    }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Structure> {
        self.structures.iter_mut().find(|s| s.id == id)
    }
    pub fn heart(&self, defs: &StructureFile) -> Option<&Structure> {
        let hi = defs.index("heart")?;
        self.structures.iter().find(|s| s.def == hi)
    }
    pub fn heart_tier(&self, defs: &StructureFile) -> u8 {
        self.heart(defs).map(|h| h.tier).unwrap_or(1)
    }

    /// Footprint cells of a def anchored at (x, y) (the anchor is the top-left tile).
    pub fn footprint(def: &StructureDef, x: u16, y: u16) -> Vec<(u16, u16)> {
        let mut v = Vec::new();
        for dy in 0..def.footprint[1] {
            for dx in 0..def.footprint[0] {
                v.push((x + dx as u16, y + dy as u16));
            }
        }
        v
    }

    fn occupied(&self, defs: &StructureFile, x: u16, y: u16) -> Option<u32> {
        for s in &self.structures {
            let d = &defs.structure[s.def];
            if x >= s.x && x < s.x + d.footprint[0] as u16 && y >= s.y && y < s.y + d.footprint[1] as u16 {
                return Some(s.id);
            }
        }
        None
    }

    /// May this def stand here (Full Design, Building)?
    pub fn can_place(&self, defs: &StructureFile, mat: &MatSim, def_id: &str, x: u16, y: u16) -> Result<(), PlaceError> {
        let di = defs.index(def_id).ok_or(PlaceError::Unknown)?;
        let def = &defs.structure[di];
        if def.heart_tier > 0 && self.heart_tier(defs) < def.heart_tier {
            return Err(PlaceError::HeartTier(def.heart_tier));
        }
        let (w, h) = (mat.w as u16, mat.h as u16);
        if x + def.footprint[0] as u16 > w || y + def.footprint[1] as u16 > h {
            return Err(PlaceError::OffMap);
        }
        // the anchor stands on the Mat with signal; the rest of the footprint on walkable floor or more Mat
        let ai = y as usize * mat.w + x as usize;
        if mat.hy[ai] != hy::CORD && mat.hy[ai] != hy::NODE {
            return Err(PlaceError::NotMat(x, y));
        }
        if mat.sig[ai] < def.sigma {
            return Err(PlaceError::Signal(x, y));
        }
        for (cx, cy) in Self::footprint(def, x, y) {
            let i = cy as usize * mat.w + cx as usize;
            let on_mat = mat.hy[i] == hy::CORD || mat.hy[i] == hy::NODE;
            let floor = matches!(mat.sub[i], sub::OPEN | sub::PIPE | sub::BRICK | sub::GRATE | sub::DEPOSIT);
            if !on_mat && !floor {
                return Err(PlaceError::NotMat(cx, cy));
            }
            if let Some(id) = self.occupied(defs, cx, cy) {
                return Err(PlaceError::Overlaps(id));
            }
        }
        if def.node && mat.hy[ai] != hy::NODE {
            return Err(PlaceError::NeedsNode);
        }
        if def.relay && !mat.relays.contains(&(x as usize, y as usize)) {
            return Err(PlaceError::NeedsRelay);
        }
        if def.grate {
            let mut near = false;
            for dy in -2i32..=2 {
                for dx in -2i32..=2 {
                    let (gx, gy) = (x as i32 + dx, y as i32 + dy);
                    if gx >= 0 && gy >= 0 && (gx as usize) < mat.w && (gy as usize) < mat.h && mat.sub[gy as usize * mat.w + gx as usize] == sub::GRATE {
                        near = true;
                    }
                }
            }
            if !near {
                return Err(PlaceError::NeedsGrate);
            }
        }
        Ok(())
    }

    /// Place a ghost; `prebuilt` stands it at once at the tier given.
    pub fn place(&mut self, defs: &StructureFile, mat: &MatSim, def_id: &str, x: u16, y: u16) -> Result<u32, PlaceError> {
        self.can_place(defs, mat, def_id, x, y)?;
        let di = defs.index(def_id).ok_or(PlaceError::Unknown)?;
        let id = self.next_id;
        self.next_id += 1;
        self.structures.push(Structure { id, def: di, x, y, tier: 1, hp: defs.structure[di].hp_at(1), built: false, progress: 0, paid: false, upgrading_to: None, district: None });
        Ok(id)
    }

    /// Stand a structure at once, built, at a tier (the founding, tests).
    pub fn place_built(&mut self, defs: &StructureFile, mat: &MatSim, def_id: &str, x: u16, y: u16, tier: u8) -> Result<u32, PlaceError> {
        let id = self.place(defs, mat, def_id, x, y)?;
        let s = self.get_mut(id).unwrap();
        s.built = true;
        s.paid = true;
        s.tier = tier.max(1);
        s.hp = defs.structure[s.def].hp_at(s.tier);
        Ok(id)
    }

    /// Cancel a ghost (built structures are not removed here; floods and raiders do that, Phase D). Returns the
    /// cost to refund when it was paid.
    pub fn cancel(&mut self, defs: &StructureFile, id: u32) -> Option<Cost> {
        let i = self.structures.iter().position(|s| s.id == id && !s.built)?;
        let s = self.structures.remove(i);
        if s.paid { Some(defs.structure[s.def].cost_at(1)) } else { None }
    }

    /// Start an upgrade in place: tier 2 needs the Quarter, tier 3 the City (Full Design, Building).
    pub fn upgrade(&mut self, defs: &StructureFile, id: u32, city_tier: u8) -> Result<u8, String> {
        let s = self.get_mut(id).ok_or("no such structure")?;
        let d = &defs.structure[s.def];
        if !s.built {
            return Err("not built yet".into());
        }
        if s.upgrading_to.is_some() {
            return Err("already upgrading".into());
        }
        let to = s.tier + 1;
        if to > d.tiers {
            return Err("at its top tier".into());
        }
        if d.prebuilt && d.id == "heart" {
            return Err("the Heart's tier is the city's".into());
        }
        if to > city_tier {
            return Err(format!("needs the {}", CITY_TIER_NAMES[(to as usize - 1).min(3)]));
        }
        s.upgrading_to = Some(to);
        s.progress = 0;
        s.paid = false;
        Ok(to)
    }

    /// The founding city: the Heart (3 × 3 on the heart node), a Route Board and a tier-1 Tweaker Bench beside it,
    /// a Pit on every Relay. Fails only when the Mat has no Heart.
    pub fn found(&mut self, defs: &StructureFile, mat: &MatSim) -> Result<(), String> {
        let (hx, hy_) = mat.heart.ok_or("the Mat has no Heart")?;
        let ax = hx.saturating_sub(1) as u16;
        let ay = hy_.saturating_sub(1) as u16;
        // the Heart stands whatever the tissue under its ring: it grew the ring
        let di = defs.index("heart").ok_or("no heart def")?;
        let id = self.next_id;
        self.next_id += 1;
        self.structures.push(Structure { id, def: di, x: ax, y: ay, tier: 1, hp: defs.structure[di].hp_at(1), built: true, progress: 0, paid: true, upgrading_to: None, district: None });
        // the Route Board and the Bench on the nearest tissue that takes them
        for (what, tier) in [("route_board", 1u8), ("bench", 1u8)] {
            let mut placed = false;
            'search: for r in 2..12i32 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() != r && dy.abs() != r {
                            continue;
                        }
                        let (x, y) = (hx as i32 + dx, hy_ as i32 + dy);
                        if x < 0 || y < 0 {
                            continue;
                        }
                        if self.place_built(defs, mat, what, x as u16, y as u16, tier).is_ok() {
                            placed = true;
                            break 'search;
                        }
                    }
                }
            }
            if !placed {
                return Err(format!("no tissue near the Heart for the {what}"));
            }
        }
        for &(rx, ry) in &mat.relays {
            let _ = self.place_built(defs, mat, "pit", rx as u16, ry as u16, 1);
        }
        Ok(())
    }

    // ------------------------------------------------------------ zones

    /// A cell a zone may hold: signalled Mat, or walkable floor touching signalled Mat.
    pub fn zone_cell_ok(mat: &MatSim, c: usize) -> bool {
        if c >= mat.hy.len() {
            return false;
        }
        let signalled = |i: usize| (mat.hy[i] == hy::CORD || mat.hy[i] == hy::NODE) && mat.sig[i] >= SIGNAL_FLOOR;
        if signalled(c) {
            return true;
        }
        if !matches!(mat.sub[c], sub::OPEN | sub::PIPE | sub::BRICK | sub::GRATE | sub::DEPOSIT) {
            return false;
        }
        let (x, y) = ((c % mat.w) as i32, (c / mat.w) as i32);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && (nx as usize) < mat.w && (ny as usize) < mat.h && signalled(ny as usize * mat.w + nx as usize) {
                    return true;
                }
            }
        }
        false
    }

    /// Paint a zone over cells on or beside signalled Mat; cells already in a zone or off it are dropped.
    pub fn paint_zone(&mut self, mat: &MatSim, kind: ZoneKind, cells: &[u32], filter: Filter, anchor: Option<u32>) -> Option<u32> {
        let taken: std::collections::HashSet<u32> = self.zones.iter().flat_map(|z| z.cells.iter().copied()).collect();
        let good: Vec<u32> = cells.iter().copied().filter(|&c| Self::zone_cell_ok(mat, c as usize) && !taken.contains(&c)).collect();
        if good.is_empty() {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.zones.push(Zone { id, kind, cells: good, filter, anchor });
        Some(id)
    }

    pub fn erase_zone(&mut self, id: u32) -> bool {
        let n = self.zones.len();
        self.zones.retain(|z| z.id != id);
        self.zones.len() < n
    }

    pub fn zone_at(&self, cell: u32) -> Option<&Zone> {
        self.zones.iter().find(|z| z.cells.contains(&cell))
    }

    // ------------------------------------------------------------ homes

    /// Beds: (structure id, beds, rest, is bunkhouse) for every built home.
    fn beds(&self, defs: &StructureFile) -> Vec<(u32, i32, i32, bool)> {
        self.structures.iter().filter(|s| s.built).filter_map(|s| {
            let d = &defs.structure[s.def];
            let b = d.beds_at(s.tier);
            if b > 0 { Some((s.id, b, d.rest_at(s.tier), d.id == "bunkhouse")) } else { None }
        }).collect()
    }

    /// Give every slimo without a bed the nearest free one to the Heart, hab zones first; a Hermit takes a Pod over
    /// a Bunkhouse and a Pack Animal the reverse. Beds of the dead and the departed are freed first.
    pub fn assign_homes(&mut self, defs: &StructureFile, roster: &Roster, traits: &TraitFile, mat: &MatSim) {
        let alive: std::collections::HashSet<u32> = roster.records.iter().map(|r| r.id).collect();
        self.homes.retain(|slimo, home| alive.contains(slimo) && self.structures.iter().any(|s| s.id == *home && s.built));
        let beds = self.beds(defs);
        let mut used: HashMap<u32, i32> = HashMap::new();
        for home in self.homes.values() {
            *used.entry(*home).or_insert(0) += 1;
        }
        let heart = mat.heart.map(|(x, y)| (x as i32, y as i32)).unwrap_or((0, 0));
        let hab_cells: std::collections::HashSet<u32> = self.zones.iter().filter(|z| z.kind == ZoneKind::Hab).flat_map(|z| z.cells.iter().copied()).collect();
        for r in &roster.records {
            if self.homes.contains_key(&r.id) || !r.depth.can_act() {
                continue;
            }
            let hermit = r.has_flag(traits, "pod_alone_plus_10");
            let pack = r.has_flag(traits, "bunkhouse_plus_10");
            let mut best: Option<(i32, i32, u32)> = None; // (rank, distance, id)
            for &(id, cap, _, bunk) in &beds {
                if used.get(&id).copied().unwrap_or(0) >= cap {
                    continue;
                }
                let s = self.get(id).unwrap();
                let in_hab = hab_cells.contains(&((s.y as usize * mat.w + s.x as usize) as u32));
                let pref = if (hermit && !bunk) || (pack && bunk) { 0 } else if (hermit && bunk) || (pack && !bunk) { 2 } else { 1 };
                let rank = if in_hab { pref } else { pref + 3 };
                let d = cheb((s.x as i32, s.y as i32), heart);
                if best.is_none_or(|(br, bd, _)| (rank, d) < (br, bd)) {
                    best = Some((rank, d, id));
                }
            }
            if let Some((_, _, id)) = best {
                self.homes.insert(r.id, id);
                *used.entry(id).or_insert(0) += 1;
            }
        }
    }

    pub fn home_of(&self, slimo: u32) -> Option<&Structure> {
        self.homes.get(&slimo).and_then(|id| self.get(*id))
    }

    /// Beds in the city and beds taken.
    pub fn bed_count(&self, defs: &StructureFile) -> (i32, i32) {
        (self.beds(defs).iter().map(|b| b.1).sum(), self.homes.len() as i32)
    }

    // ------------------------------------------------------------ light, water, signal

    /// Light radius sources: (x, y, radius) of every built Lume Farm, Lantern Post and the Heart.
    fn lights(&self, defs: &StructureFile, lanterns_lit: bool) -> Vec<(i32, i32, i32)> {
        self.structures
            .iter()
            .filter(|s| s.built)
            .filter_map(|s| {
                let d = &defs.structure[s.def];
                let r = d.light_at(s.tier);
                if r == 0 || (d.id == "lantern" && !lanterns_lit) {
                    None
                } else {
                    Some((s.x as i32 + d.footprint[0] as i32 / 2, s.y as i32 + d.footprint[1] as i32 / 2, r))
                }
            })
            .collect()
    }

    fn lit_at(lights: &[(i32, i32, i32)], x: i32, y: i32) -> bool {
        lights.iter().any(|&(lx, ly, r)| cheb((lx, ly), (x, y)) <= r)
    }

    fn dry_at(&self, defs: &StructureFile, mat: &MatSim, x: i32, y: i32) -> bool {
        let i = y as usize * mat.w + x as usize;
        if mat.sub[i] != sub::WATER {
            return true;
        }
        self.structures.iter().filter(|s| s.built).any(|s| {
            let d = &defs.structure[s.def];
            let r = d.value_at("dry_radius", s.tier);
            r > 0 && cheb((s.x as i32, s.y as i32), (x, y)) <= r
        })
    }

    /// Where a slimo stands this shift: its bed, or the Heart's edge when it has none.
    fn stand(&self, mat: &MatSim, slimo: u32) -> (i32, i32) {
        if let Some(h) = self.home_of(slimo) {
            (h.x as i32, h.y as i32)
        } else {
            let (hx, hy_) = mat.heart.unwrap_or((0, 0));
            (hx as i32, hy_ as i32)
        }
    }

    // ------------------------------------------------------------ districts

    /// Form districts where a Node (or a Relay with a Shrine) has three built structures within reach, greedily by
    /// count, up to the city tier's cap; refresh membership and claim structures. Returns the ids formed this call.
    pub fn form_districts(&mut self, mat: &MatSim, city_tier: u8) -> Vec<u32> {
        let cap = DISTRICT_CAP[(city_tier.max(1) as usize - 1).min(3)];
        let mut formed = Vec::new();
        let claimed = |me: &Colony, sid: u32| me.districts.iter().any(|d| d.structures.contains(&sid));
        while self.districts.len() < cap {
            // candidate hubs: node tiles with the most unclaimed built structures within reach
            let mut best: Option<(usize, (u16, u16))> = None;
            let built: Vec<(u32, i32, i32)> = self.structures.iter().filter(|s| s.built && !claimed(self, s.id)).map(|s| (s.id, s.x as i32, s.y as i32)).collect();
            for y in 0..mat.h {
                for x in 0..mat.w {
                    let i = y * mat.w + x;
                    if mat.hy[i] != hy::NODE {
                        continue;
                    }
                    if self.districts.iter().any(|d| cheb((d.hub.0 as i32, d.hub.1 as i32), (x as i32, y as i32)) <= DISTRICT_RADIUS) {
                        continue;
                    }
                    let n = built.iter().filter(|&&(_, sx, sy)| cheb((sx, sy), (x as i32, y as i32)) <= DISTRICT_RADIUS).count();
                    if n >= DISTRICT_MIN_STRUCTURES && best.is_none_or(|(bn, _)| n > bn) {
                        best = Some((n, (x as u16, y as u16)));
                    }
                }
            }
            let Some((_, hub)) = best else { break };
            let id = self.next_id;
            self.next_id += 1;
            let name = if self.names_left.is_empty() { format!("District {id}") } else { self.names_left.remove(0) };
            self.districts.push(District { id, name, hub, structures: Vec::new(), members: Vec::new(), foreman: None, term_left: 0, served: 0, priorities: [2; 9], petition: None, boon_shifts: 0, wake: false, low: [0; 5] });
            formed.push(id);
        }
        // structures to the nearest hub within reach; beds decide members
        for s in self.structures.iter_mut() {
            let mut best: Option<(i32, u32)> = None;
            for d in &self.districts {
                let dist = cheb((d.hub.0 as i32, d.hub.1 as i32), (s.x as i32, s.y as i32));
                if dist <= DISTRICT_RADIUS && best.is_none_or(|(bd, _)| dist < bd) {
                    best = Some((dist, d.id));
                }
            }
            s.district = best.map(|b| b.1);
        }
        for d in self.districts.iter_mut() {
            d.structures = self.structures.iter().filter(|s| s.district == Some(d.id)).map(|s| s.id).collect();
        }
        formed
    }

    /// Membership: a slimo belongs to the district of its bed, else of its job's structure.
    fn refresh_members(&mut self, roster: &Roster, jobs: &[(u32, Job, u32)]) {
        let struct_district: HashMap<u32, Option<u32>> = self.structures.iter().map(|s| (s.id, s.district)).collect();
        for d in self.districts.iter_mut() {
            d.members.clear();
        }
        for r in &roster.records {
            let by_home = self.homes.get(&r.id).and_then(|h| struct_district.get(h).copied().flatten());
            let by_job = jobs.iter().find(|j| j.0 == r.id).and_then(|j| struct_district.get(&j.2).copied().flatten());
            if let Some(did) = by_home.or(by_job) {
                if let Some(d) = self.districts.iter_mut().find(|d| d.id == did) {
                    d.members.push(r.id);
                }
            }
        }
    }

    pub fn district_of(&self, slimo: u32) -> Option<&District> {
        self.districts.iter().find(|d| d.members.contains(&slimo))
    }

    /// Elections (research 14): weight Voice + Mood / 20, Orator × 2; on a run or stranded cannot stand; ties to
    /// the longer-serving. Runs where the term is out or the foreman is gone. Returns (district, foreman) pairs.
    pub fn elect(&mut self, roster: &Roster, traits: &TraitFile) -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        for d in self.districts.iter_mut() {
            let foreman_ok = d.foreman.is_some_and(|f| d.members.contains(&f) && roster.get(f).is_some_and(|r| r.is_up()));
            if foreman_ok && d.term_left > 0 {
                d.term_left -= 1;
                d.served += 1;
                continue;
            }
            if !foreman_ok && d.foreman.is_some() {
                // the foreman died or left: a wake, no jobs this shift
                d.wake = roster.get(d.foreman.unwrap()).is_none();
                d.foreman = None;
                d.served = 0;
            }
            let mut best: Option<(f32, u32, u32)> = None; // (weight, served, id)
            for &m in &d.members {
                let Some(r) = roster.get(m) else { continue };
                if !r.is_up() || r.status == Status::OnRun || r.status == Status::Stranded {
                    continue;
                }
                let mut w = r.stat(Stat::Voice) as f32 + r.mood() as f32 / 20.0;
                if r.has_flag(traits, "election_x2") {
                    w *= 2.0;
                }
                let served = if d.foreman == Some(m) { d.served } else { 0 };
                if best.is_none_or(|(bw, bs, bid)| (w, served, std::cmp::Reverse(m)) > (bw, bs, std::cmp::Reverse(bid))) {
                    best = Some((w, served, m));
                }
            }
            if let Some((_, _, id)) = best {
                let changed = d.foreman != Some(id);
                d.foreman = Some(id);
                d.term_left = TERM_SHIFTS;
                if changed {
                    d.served = 0;
                    if let Some(r) = roster.get(id) {
                        d.priorities = r.priorities;
                    }
                    out.push((d.id, id));
                }
            }
        }
        out
    }

    /// The priority a member's job runs at: the foreman's where higher.
    pub fn priority_for(&self, record: &Record, job: Job) -> u8 {
        let own = record.priorities[job.index()];
        match self.district_of(record.id) {
            Some(d) if d.foreman.is_some() => own.max(d.priorities[job.index()]),
            _ => own,
        }
    }

    /// Petitions (research 14): a need averaging under 30 for two shifts raises one; the deadline counts down; the
    /// structure standing in the district answers it. Returns (raised, met, missed).
    fn petitions(&mut self, defs: &StructureFile, roster: &Roster) -> (Vec<(u32, Need)>, Vec<u32>, Vec<u32>) {
        let (mut raised, mut met, mut missed) = (Vec::new(), Vec::new(), Vec::new());
        for d in self.districts.iter_mut() {
            if d.boon_shifts > 0 {
                d.boon_shifts -= 1;
            }
            let members: Vec<&Record> = d.members.iter().filter_map(|m| roster.get(*m)).collect();
            if members.is_empty() {
                continue;
            }
            // open petition: answered?
            if let Some(p) = d.petition.clone() {
                let answered = d.structures.iter().any(|sid| self.structures.iter().any(|s| s.id == *sid && s.built && p.asks.contains(&defs.structure[s.def].id)));
                if answered {
                    d.petition = None;
                    d.boon_shifts = 3;
                    met.push(d.id);
                    continue;
                }
                let mut p = p;
                if !p.missed {
                    if p.shifts_left > 0 {
                        p.shifts_left -= 1;
                    } else {
                        p.missed = true;
                        missed.push(d.id);
                    }
                }
                d.petition = Some(p);
                continue;
            }
            // averages of the five needs
            for (k, need) in [Need::Fed, Need::Lit, Need::Dry, Need::Signal, Need::Rest].into_iter().enumerate() {
                let avg = members.iter().map(|r| r.need(need)).sum::<i32>() / members.len() as i32;
                if avg < 30 {
                    d.low[k] = d.low[k].saturating_add(1);
                } else {
                    d.low[k] = 0;
                }
            }
            if d.foreman.is_none() {
                continue;
            }
            let worst = [Need::Fed, Need::Lit, Need::Dry, Need::Signal, Need::Rest].into_iter().enumerate().filter(|(k, _)| d.low[*k] >= PETITION_AFTER).min_by_key(|(k, _)| {
                let need = [Need::Fed, Need::Lit, Need::Dry, Need::Signal, Need::Rest][*k];
                members.iter().map(|r| r.need(need)).sum::<i32>()
            });
            if let Some((_, need)) = worst {
                let asks: Vec<String> = match need {
                    Need::Fed => vec!["pit".into()],
                    Need::Lit => vec!["lume_farm".into(), "lantern".into()],
                    Need::Dry => vec!["lung".into()],
                    Need::Rest => vec!["bunkhouse".into(), "pod".into()],
                    _ => vec!["shrine".into()],
                };
                d.petition = Some(Petition { need, asks, shifts_left: PETITION_SHIFTS, missed: false });
                raised.push((d.id, need));
            }
        }
        (raised, met, missed)
    }

    // ------------------------------------------------------------ the shift

    /// Open job slots: (job, structure id) for every built structure that hosts one, and two Build slots per ghost.
    pub fn slots(&self, defs: &StructureFile) -> Vec<(Job, u32)> {
        let mut v = Vec::new();
        for s in &self.structures {
            let d = &defs.structure[s.def];
            if !s.built || s.upgrading_to.is_some() {
                v.push((Job::Build, s.id));
                v.push((Job::Build, s.id));
                continue;
            }
            if let Some(job) = d.job() {
                for _ in 0..d.slots_at(s.tier) {
                    v.push((job, s.id));
                }
            }
        }
        v
    }

    /// Hand the open slots to the highest utilities (Full Design, Jobs by utility): one slot per slimo on duty this
    /// shift of the day, filled best pair first; a district in a wake sits out.
    pub fn assign_jobs(&self, defs: &StructureFile, roster: &Roster, traits: &TraitFile, shift_of_day: u8) -> Vec<(u32, Job, u32)> {
        let slots = self.slots(defs);
        let waking: std::collections::HashSet<u32> = self.districts.iter().filter(|d| d.wake).flat_map(|d| d.members.iter().copied()).collect();
        let mut pairs: Vec<(f32, u32, usize)> = Vec::new();
        for r in &roster.records {
            // a district in a wake sits out; a slimo whose shift was a run (shift_job set by apply_run) is done
            if waking.contains(&r.id) || r.shift_job.is_some() || !r.on_duty(shift_of_day) {
                continue;
            }
            let mut by_job: HashMap<Job, Option<f32>> = HashMap::new();
            for (k, &(job, sid)) in slots.iter().enumerate() {
                let gate = self.get(sid).map(|s| defs.structure[s.def].gate).unwrap_or(0);
                if r.skill(job.skill()) < gate {
                    continue;
                }
                let u = *by_job.entry(job).or_insert_with(|| r.utility_at(traits, job, self.priority_for(r, job)));
                if let Some(u) = u {
                    pairs.push((u, r.id, k));
                }
            }
        }
        pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut taken_slot = vec![false; slots.len()];
        let mut taken_slimo = std::collections::HashSet::new();
        let mut out = Vec::new();
        for (_, slimo, k) in pairs {
            if taken_slot[k] || taken_slimo.contains(&slimo) {
                continue;
            }
            taken_slot[k] = true;
            taken_slimo.insert(slimo);
            out.push((slimo, slots[k].0, slots[k].1));
        }
        out
    }

    /// The shift (Full Design, Homes, meals, light, water, sleep; Building; Districts; Petitions). `pay` takes a
    /// cost out of the Stash and says whether it could; `ledger_eddies` is what the ledger holds for paste and oil;
    /// `shift_of_day` 0 Dim, 1 Dark, 2 Flood. The roster is read, not written: the bridge applies `work` and
    /// `contexts` through `Roster::end_shift` and the treatments through `Record::treat`.
    #[allow(clippy::too_many_arguments)]
    pub fn end_shift(&mut self, defs: &StructureFile, mat: &mut MatSim, roster: &Roster, traits: &TraitFile, shift_of_day: u8, ledger_eddies: u32, pay: &mut dyn FnMut(Cost) -> bool) -> ShiftOutcome {
        self.shift += 1;
        let mut out = ShiftOutcome::default();
        let city_tier = Self::city_tier(roster.records.len());
        // homes and districts first: jobs read the foreman's priorities
        self.assign_homes(defs, roster, traits, mat);
        out.districts_formed = self.form_districts(mat, city_tier);
        let jobs = self.assign_jobs(defs, roster, traits, shift_of_day);
        self.refresh_members(roster, &jobs);
        out.elections = self.elect(roster, traits);
        for (did, who) in &out.elections {
            let d = self.districts.iter().find(|d| d.id == *did).unwrap();
            out.lines.push(format!("{} elected {} foreman", d.name, roster.get(*who).map(|r| r.full_name()).unwrap_or_default()));
        }
        for did in &out.districts_formed {
            let d = self.districts.iter().find(|d| d.id == *did).unwrap();
            out.lines.push(format!("{} is a district: {} structures at {},{}", d.name, d.structures.len(), d.hub.0, d.hub.1));
        }
        // the work
        let mut eddies_left = ledger_eddies;
        let mut builders: HashMap<u32, Vec<(i32, bool)>> = HashMap::new(); // ghost -> (hands, forge in district)
        let forge_districts: std::collections::HashSet<u32> = self.structures.iter().filter(|s| s.built && defs.structure[s.def].id == "forge").filter_map(|s| s.district).collect();
        for &(slimo, job, sid) in &jobs {
            let Some(r) = roster.get(slimo) else { continue };
            let Some(s) = self.get(sid) else { continue };
            let d = &defs.structure[s.def];
            match job {
                Job::Build => builders.entry(sid).or_default().push((r.stat(Stat::Hands), s.district.is_some_and(|dd| forge_districts.contains(&dd)))),
                Job::Tweak => {
                    out.bench_slots += 1;
                    out.work.crafts_by.push(slimo);
                }
                Job::Appraise => {
                    out.identifies += d.value_at("identify", s.tier).max(0) as u32;
                    out.work.identified_by.push(slimo);
                }
                Job::Haul => out.sellers.push(slimo),
                Job::Tend => out.work.tend_shifts_by.push(slimo),
                Job::Cook => match d.id.as_str() {
                    "still" => out.doses += 1,
                    "salt_works" => out.salt += d.value_at("salt", s.tier).max(0) as u32,
                    // the clinic: a Cook at a Lung is the medic
                    "lung" => out.treated.push(slimo),
                    _ => {}
                },
                _ => {}
            }
            if d.id == "lung" {
                out.brine += d.value_at("brine", s.tier).max(0) as u32;
            }
        }
        // the Medic slots treat: each treats one patient, the most injured member first
        let mut patients: Vec<u32> = roster.records.iter().filter(|r| !r.injuries.is_empty()).map(|r| r.id).collect();
        patients.sort_by_key(|id| std::cmp::Reverse(roster.get(*id).map(|r| r.injuries.iter().map(|(_, s)| *s as u32).sum::<u32>()).unwrap_or(0)));
        let medics = out.treated.len();
        out.work.treatments_by = out.treated.clone();
        out.treated = patients.into_iter().take(medics).collect();
        // building: pay, then points
        for (sid, crew) in builders {
            let Some(idx) = self.structures.iter().position(|s| s.id == sid) else { continue };
            let def = defs.structure[self.structures[idx].def].clone();
            let to = self.structures[idx].upgrading_to.unwrap_or(1);
            if !self.structures[idx].paid {
                let cost = def.cost_at(to);
                if pay(cost) {
                    self.structures[idx].paid = true;
                } else {
                    out.waiting.push((sid, cost));
                    continue;
                }
            }
            let points: i32 = crew.iter().map(|(h, forge)| if *forge { h * 2 } else { *h }).sum();
            let s = &mut self.structures[idx];
            s.progress += points.max(0) as u32;
            if s.progress >= def.build_at(to) {
                if s.built {
                    s.tier = to;
                    s.upgrading_to = None;
                    s.hp = def.hp_at(to);
                    out.upgraded.push(sid);
                    out.lines.push(format!("{} upgraded to tier {}", def.name, to));
                } else {
                    s.built = true;
                    s.hp = def.hp_at(1);
                    out.built.push(sid);
                    out.lines.push(format!("{} built at {},{}", def.name, s.x, s.y));
                }
                s.progress = 0;
            }
        }
        // lanterns burn oil
        let lanterns = self.structures.iter().filter(|s| s.built && defs.structure[s.def].id == "lantern").map(|s| defs.structure[s.def].value_at("upkeep", s.tier).max(0) as u32).sum::<u32>();
        let lanterns_lit = lanterns <= eddies_left;
        if lanterns_lit {
            eddies_left -= lanterns;
            out.upkeep = lanterns;
        }
        // the contexts: where each slimo stands, what the Mat gives it
        let lights = self.lights(defs, lanterns_lit);
        let dark = shift_of_day == 1;
        let working: std::collections::HashSet<u32> = jobs.iter().map(|j| j.0).collect();
        let heart_tier = self.heart_tier(defs);
        for r in &roster.records {
            if r.status == Status::OnRun || r.status == Status::Stranded || r.status == Status::Dead {
                continue;
            }
            let (x, y) = self.stand(mat, r.id);
            let i = y as usize * mat.w + x as usize;
            let on_mat = mat.hy[i] == hy::CORD || mat.hy[i] == hy::NODE;
            let signalled = on_mat && mat.sig[i] >= SIGNAL_FLOOR;
            let home = self.home_of(r.id).map(|h| (defs.structure[h.def].id.clone(), h.tier, h.district));
            let in_bunkhouse = home.as_ref().is_some_and(|h| h.0 == "bunkhouse");
            let in_pod = home.as_ref().is_some_and(|h| h.0 == "pod");
            let alone_in_pod = in_pod && self.homes.values().filter(|hid| Some(**hid) == self.homes.get(&r.id).copied()).count() == 1;
            // a meal: the Mat's nutrient at the home, else paste
            let mut fed = false;
            if signalled && mat.n[i] >= MEAL_NUTRIENT {
                mat.n[i] -= MEAL_TAKES;
                fed = true;
            } else if eddies_left >= PASTE_EDDIES {
                eddies_left -= PASTE_EDDIES;
                out.paste_meals += 1;
                fed = true;
            }
            // a slimo who ran this shift worked it; everyone without a slot slept
            let asleep = !working.contains(&r.id) && r.shift_job.is_none();
            let mut extra = 0;
            if let Some(d) = self.district_of(r.id) {
                if d.boon_shifts > 0 {
                    extra += 10;
                }
                if d.petition.as_ref().is_some_and(|p| p.missed) {
                    extra -= 10;
                }
                if d.structures.iter().any(|sid| self.get(*sid).is_some_and(|s| s.built && defs.structure[s.def].id == "shrine")) {
                    extra += 5;
                }
            }
            // a home a tier under the city's (Full Design: expectations scale with the city tier)
            if let Some((_, tier, _)) = &home {
                let gap = heart_tier as i32 - *tier as i32;
                if gap > 0 {
                    extra -= 5 * gap;
                }
            }
            out.contexts.insert(
                r.id,
                ShiftContext {
                    on_mat,
                    signalled,
                    lit: Self::lit_at(&lights, x, y),
                    wet: !self.dry_at(defs, mat, x, y),
                    pod_dark: asleep && in_pod && dark,
                    asleep,
                    fed,
                    in_bunkhouse,
                    alone_in_pod,
                    white_light_near: false,
                    narrow: false,
                    death_voice: 0,
                    extra_target: extra,
                },
            );
        }
        // petitions read last shift's needs; the wake is over after one shift
        let (raised, met, missed) = self.petitions(defs, roster);
        for (did, need) in &raised {
            let d = self.districts.iter().find(|d| d.id == *did).unwrap();
            let fm = d.foreman.and_then(|f| roster.get(f)).map(|r| r.full_name()).unwrap_or_default();
            out.lines.push(format!("{} petitions for {}: {} within {} shifts", fm, d.name, need.name().to_lowercase(), PETITION_SHIFTS));
        }
        for did in &met {
            let d = self.districts.iter().find(|d| d.id == *did).unwrap();
            out.lines.push(format!("{} has its answer", d.name));
        }
        for did in &missed {
            let d = self.districts.iter().find(|d| d.id == *did).unwrap();
            out.lines.push(format!("{}'s petition went unanswered", d.name));
        }
        out.petitions_raised = raised;
        out.petitions_met = met;
        out.petitions_missed = missed;
        for d in self.districts.iter_mut() {
            d.wake = false;
        }
        out.assignments = jobs;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mat::genome::{CONDUCTANCE, VIGOR};
    use mat::Genome;
    use people::{NameFile, Roster, TraitFile};

    fn grown() -> MatSim {
        let g = Genome::default().with(VIGOR, 0.8).with(CONDUCTANCE, 0.8);
        let mut m = MatSim::new(96, 48, 3, g);
        m.seed_heart(48, 24);
        m.place_relay(20, 24);
        m.place_relay(76, 24);
        for _ in 0..300 {
            m.feed(48, 24, 0.6);
            m.feed(20, 24, 0.4);
            m.feed(76, 24, 0.4);
            m.step();
        }
        m
    }

    fn city() -> (Colony, StructureFile, MatSim, Roster, TraitFile) {
        let defs = StructureFile::shipped();
        let mat = grown();
        let mut c = Colony::new();
        c.found(&defs, &mat).unwrap();
        let t = TraitFile::shipped();
        let mut r = Roster::new();
        r.found(&t, &NameFile::shipped(), 3, 1);
        (c, defs, mat, r, t)
    }

    /// A tile near (x, y) where a def can stand.
    fn spot(c: &Colony, defs: &StructureFile, mat: &MatSim, id: &str, near: (i32, i32)) -> Option<(u16, u16)> {
        for r in 0..20i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    let (x, y) = (near.0 + dx, near.1 + dy);
                    if x >= 0 && y >= 0 && c.can_place(defs, mat, id, x as u16, y as u16).is_ok() {
                        return Some((x as u16, y as u16));
                    }
                }
            }
        }
        None
    }

    #[test]
    fn the_twenty_four_load_and_the_founding_stands_a_heart_a_board_a_bench_and_pits() {
        let (c, defs, mat, _, _) = city();
        assert_eq!(defs.structure.len(), 24);
        let names: Vec<&str> = c.structures.iter().map(|s| defs.structure[s.def].id.as_str()).collect();
        assert_eq!(names[..3], ["heart", "route_board", "bench"]);
        assert_eq!(names.iter().filter(|n| **n == "pit").count(), mat.relays.len());
        assert!(c.structures.iter().all(|s| s.built));
        assert_eq!(c.heart_tier(&defs), 1);
        assert_eq!(Colony::city_tier(6), 1);
        assert_eq!(Colony::city_tier(100), 2);
        assert_eq!(Colony::city_tier(2000), 4);
        // a Pit costs 4 Parts and 8 points; tier 2 doubles both; a lantern is the cheapest thing in the city
        let pit = defs.get("pit").unwrap();
        assert_eq!(pit.cost_at(2), Cost { parts: 8, chrome: 0, eddies: 0 });
        assert_eq!(pit.build_at(2), 16);
        assert_eq!(defs.get("lantern").unwrap().cost.parts, 2);
    }

    #[test]
    fn placement_follows_the_mat_the_signal_and_the_footprint() {
        let (mut c, defs, mat, _, _) = city();
        // off the Mat is refused; on cord with signal is taken; the same tiles twice overlap
        let rock = (0..mat.hy.len()).position(|i| mat.hy[i] == hy::EMPTY).unwrap();
        assert!(matches!(c.can_place(&defs, &mat, "pod", (rock % mat.w) as u16, (rock / mat.w) as u16), Err(PlaceError::NotMat(..)) | Err(PlaceError::OffMap)));
        let (x, y) = spot(&c, &defs, &mat, "pod", (48, 24)).expect("a pod spot");
        let id = c.place(&defs, &mat, "pod", x, y).unwrap();
        assert!(matches!(c.can_place(&defs, &mat, "pod", x, y), Err(PlaceError::Overlaps(i)) if i == id));
        assert!(!c.get(id).unwrap().built);
        // a Vault needs the Heart at tier 2; a Shrine a Relay; a Market a grate
        assert_eq!(c.can_place(&defs, &mat, "vault", x, y), Err(PlaceError::HeartTier(2)));
        assert!(matches!(c.can_place(&defs, &mat, "shrine", x, y), Err(PlaceError::NeedsRelay) | Err(PlaceError::Overlaps(_))));
        let (rx, ry) = mat.relays[0];
        // the relay carries a founding Pit; a shrine there overlaps it, so the rule that fires is the overlap
        assert!(c.can_place(&defs, &mat, "shrine", rx as u16, ry as u16).is_err());
        assert_eq!(c.can_place(&defs, &mat, "nothing", 1, 1), Err(PlaceError::Unknown));
        // cancel a ghost
        assert!(c.cancel(&defs, id).is_none());
        assert!(c.get(id).is_none());
    }

    #[test]
    fn building_pays_once_then_adds_hands_per_shift_and_a_forge_doubles() {
        let (mut c, defs, mut mat, mut r, t) = city();
        // everyone prioritises Build and nothing else, so the ghost gets the crew
        for rec in r.records.iter_mut() {
            rec.priorities = [0; 9];
            rec.priorities[Job::Build.index()] = 4;
        }
        let (x, y) = spot(&c, &defs, &mat, "bunkhouse", (48, 24)).unwrap();
        let id = c.place(&defs, &mat, "bunkhouse", x, y).unwrap();
        // a Stash that cannot pay: the ghost waits and names the cost
        let mut paid: Vec<Cost> = Vec::new();
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 0, &mut |_| false);
        assert_eq!(o.waiting, vec![(id, Cost { parts: 10, chrome: 0, eddies: 0 })]);
        assert_eq!(c.get(id).unwrap().progress, 0);
        // paid: two builders (two Build slots per ghost) add their Hands
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 0, &mut |cost| {
            paid.push(cost);
            true
        });
        assert_eq!(paid, vec![Cost { parts: 10, chrome: 0, eddies: 0 }]);
        let builders: Vec<i32> = o.assignments.iter().filter(|a| a.1 == Job::Build && a.2 == id).map(|a| r.get(a.0).unwrap().stat(Stat::Hands)).collect();
        assert_eq!(builders.len(), 2);
        assert_eq!(c.get(id).unwrap().progress as i32, builders.iter().sum::<i32>());
        // shifts until it stands: 16 points
        let mut shifts = 1;
        while !c.get(id).unwrap().built && shifts < 10 {
            c.end_shift(&defs, &mut mat, &r, &t, 0, 0, &mut |_| true);
            shifts += 1;
        }
        assert!(c.get(id).unwrap().built, "bunkhouse never stood");
        assert_eq!(c.get(id).unwrap().hp, 100);
        // an upgrade needs the Quarter
        assert_eq!(c.upgrade(&defs, id, 1), Err("needs the Quarter".to_string()));
        assert_eq!(c.upgrade(&defs, id, 2), Ok(2));
        assert_eq!(defs.get("bunkhouse").unwrap().cost_at(2).parts, 20);
    }

    #[test]
    fn homes_meals_light_and_sleep_reach_the_contexts() {
        let (mut c, defs, mut mat, mut r, t) = city();
        for rec in r.records.iter_mut() {
            rec.priorities = [0; 9];
        }
        // no beds: everyone sleeps on the Mat, nobody in a pod
        let o = c.end_shift(&defs, &mut mat, &r, &t, 1, 100, &mut |_| true);
        assert!(o.contexts.values().all(|cx| cx.asleep && !cx.pod_dark && !cx.in_bunkhouse));
        assert_eq!(c.bed_count(&defs), (0, 0));
        // a pod stands: one slimo takes it, lit by the Heart when within 4 tiles; a Dark shift sleeps +30 there
        let (x, y) = spot(&c, &defs, &mat, "pod", (48, 24)).unwrap();
        let id = c.place_built(&defs, &mat, "pod", x, y, 1).unwrap();
        let o = c.end_shift(&defs, &mut mat, &r, &t, 1, 100, &mut |_| true);
        assert_eq!(c.bed_count(&defs), (1, 1));
        let (who, _) = c.homes.iter().next().map(|(a, b)| (*a, *b)).unwrap();
        let cx = o.contexts[&who];
        assert!(cx.pod_dark && cx.alone_in_pod && cx.asleep);
        let heart = mat.heart.unwrap();
        assert_eq!(cx.lit, cheb((x as i32, y as i32), (heart.0 as i32, heart.1 as i32)) <= 4);
        // a Dim shift: the pod's +30 is Dark's
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(!o.contexts[&who].pod_dark && o.contexts[&who].asleep);
        // meals: the Mat under the home feeds when it carries nutrient, else paste from the ledger
        let i = y as usize * mat.w + x as usize;
        mat.n[i] = 0.5;
        let before = mat.n[i];
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 0, &mut |_| true);
        assert!(o.contexts[&who].fed);
        assert!((before - mat.n[i] - MEAL_TAKES).abs() < 1e-5);
        mat.n[i] = 0.0;
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 0, &mut |_| true);
        assert!(!o.contexts[&who].fed, "no nutrient and no eddies: hungry");
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 2, &mut |_| true);
        assert!(o.contexts[&who].fed && o.paste_meals >= 1, "two eddies buy paste");
        let _ = id;
    }

    #[test]
    fn a_district_forms_at_three_structures_elects_by_voice_and_petitions_for_a_failing_need() {
        let (mut c, defs, mut mat, mut r, t) = city();
        for rec in r.records.iter_mut() {
            rec.priorities = [0; 9];
        }
        // three pods around the Heart (with the Heart, the Board and the Bench already there) make the first district
        let heart = mat.heart.unwrap();
        let mut placed = 0;
        for _ in 0..3 {
            if let Some((x, y)) = spot(&c, &defs, &mat, "pod", (heart.0 as i32, heart.1 as i32)) {
                c.place_built(&defs, &mat, "pod", x, y, 1).unwrap();
                placed += 1;
            }
        }
        assert_eq!(placed, 3);
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(c.districts.len(), 1, "the Hamlet caps districts at one");
        let d = c.districts[0].clone();
        assert_eq!(o.districts_formed, vec![d.id]);
        assert!(d.structures.len() >= 3);
        assert!(!d.members.is_empty(), "the pods' sleepers are members");
        // the election: the member with the highest Voice + Mood / 20 (everyone at Mood 60)
        let want = d.members.iter().filter_map(|m| r.get(*m)).max_by(|a, b| {
            let wa = a.stat(Stat::Voice) as f32 * if a.has_flag(&t, "election_x2") { 2.0 } else { 1.0 };
            let wb = b.stat(Stat::Voice) as f32 * if b.has_flag(&t, "election_x2") { 2.0 } else { 1.0 };
            wa.partial_cmp(&wb).unwrap().then(b.id.cmp(&a.id))
        }).unwrap().id;
        assert_eq!(d.foreman, Some(want));
        assert_eq!(d.term_left, TERM_SHIFTS);
        assert_eq!(o.elections, vec![(d.id, want)]);
        // the foreman's priorities stand over a member's where higher
        let fm = r.get_mut(want).unwrap();
        fm.priorities[Job::Haul.index()] = 4;
        c.districts[0].priorities = fm.priorities;
        let member = d.members.iter().find(|m| **m != want).copied().unwrap();
        assert_eq!(c.priority_for(r.get(member).unwrap(), Job::Haul), 4);
        // a failing need for two shifts petitions for the structure that answers it, with five shifts to build
        for m in &d.members {
            r.get_mut(*m).unwrap().needs[Need::Lit.index()] = 10;
        }
        let o1 = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(o1.petitions_raised.is_empty(), "one shift under 30 is not yet a petition");
        let o2 = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(o2.petitions_raised, vec![(d.id, Need::Lit)]);
        let p = c.districts[0].petition.clone().unwrap();
        assert_eq!(p.asks[0], "lume_farm");
        assert_eq!(p.shifts_left, PETITION_SHIFTS);
        // a Lantern Post in the district answers it: the members' target gains 10 for three shifts
        let (x, y) = spot(&c, &defs, &mat, "lantern", (d.hub.0 as i32, d.hub.1 as i32)).unwrap();
        c.place_built(&defs, &mat, "lantern", x, y, 1).unwrap();
        let o3 = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(o3.petitions_met, vec![d.id]);
        assert!(c.districts[0].petition.is_none());
        let o4 = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(o4.contexts[&member].extra_target >= 10, "the boon reaches the context: {}", o4.contexts[&member].extra_target);
        // a lantern burns an eddy a shift and lights its tiles
        assert_eq!(o4.upkeep, 1);
    }

    #[test]
    fn zones_take_cells_on_or_beside_signalled_mat_once_and_filter_by_category_tier_and_condition() {
        let (mut c, _defs, mat, _, _) = city();
        let heart = mat.heart.unwrap();
        let hi = (heart.1 * mat.w + heart.0) as u32;
        let rock = (0..mat.hy.len()).position(|i| mat.sub[i] == sub::ROCK && mat.hy[i] == hy::EMPTY).unwrap() as u32;
        let f = Filter { categories: vec!["chrome".into(), "parts".into()], min_tier: 2, min_condition: 40 };
        let id = c.paint_zone(&mat, ZoneKind::Stockpile, &[hi, hi + 1, rock], f.clone(), None).expect("a zone");
        let z = c.zone_at(hi).unwrap();
        assert_eq!(z.id, id);
        assert!(!z.cells.contains(&rock), "rock is no zone");
        assert!(c.paint_zone(&mat, ZoneKind::Hab, &[hi], Filter::default(), None).is_none(), "a cell belongs to one zone");
        assert!(f.accepts("chrome", 2, 40));
        assert!(!f.accepts("curio", 3, 90));
        assert!(!f.accepts("parts", 1, 90));
        assert!(!f.accepts("parts", 2, 39));
        assert!(Filter::default().accepts("anything", 1, 0));
        assert!(c.erase_zone(id) && c.zone_at(hi).is_none());
    }

    #[test]
    fn a_runner_home_from_a_run_takes_no_slot_and_does_not_sleep() {
        let (mut c, defs, mut mat, mut r, t) = city();
        let id = r.eligible()[0].id;
        r.depart(id, &[]);
        r.apply_run(&t, &people::RunReport { runner: id, extracted: true, ..Default::default() });
        assert_eq!(r.get(id).unwrap().shift_job, Some(Job::Scav));
        let o = c.end_shift(&defs, &mut mat, &r, &t, 1, 100, &mut |_| true);
        assert!(!o.assignments.iter().any(|a| a.0 == id), "the Runner was given a second job");
        assert!(!o.contexts[&id].asleep, "the Runner slept through a run");
    }

    #[test]
    fn the_schedule_sends_the_city_to_bed_in_dark_and_any_rests_the_tired() {
        let (c, defs, _mat, mut r, t) = city();
        // Dim: the default row works; Dark: nobody takes a slot; Flood: Any works while Rest is 40 or more
        assert!(!c.assign_jobs(&defs, &r, &t, 0).is_empty());
        assert!(c.assign_jobs(&defs, &r, &t, 1).is_empty(), "the default row rests in Dark");
        let flood = c.assign_jobs(&defs, &r, &t, 2).len();
        assert!(flood > 0);
        for rec in r.records.iter_mut() {
            rec.needs[Need::Rest.index()] = 39;
        }
        assert!(c.assign_jobs(&defs, &r, &t, 2).is_empty(), "Any sleeps under Rest 40");
        // a row set to Work in Dark takes a slot there
        let id = r.records[0].id;
        r.get_mut(id).unwrap().schedule[1] = people::Duty::Work;
        r.get_mut(id).unwrap().needs[Need::Rest.index()] = 80;
        assert_eq!(c.assign_jobs(&defs, &r, &t, 1).iter().map(|j| j.0).collect::<Vec<_>>(), vec![id]);
    }

    #[test]
    fn jobs_go_to_the_highest_utility_one_slot_each() {
        let (mut c, defs, mut mat, r, t) = city();
        // the founding Bench has one Tweak slot; the Pits one Tend slot each; everyone at priority 2
        let jobs = c.assign_jobs(&defs, &r, &t, 0);
        let slots = c.slots(&defs);
        assert!(!slots.is_empty());
        assert!(jobs.len() <= slots.len().min(r.records.len()));
        let mut seen = std::collections::HashSet::new();
        for (slimo, _, _) in &jobs {
            assert!(seen.insert(*slimo), "a slimo took two slots");
        }
        // the Tweak slot goes to the best Tweak utility among those not better used elsewhere
        let tweak = jobs.iter().find(|j| j.1 == Job::Tweak).expect("the Bench is worked");
        let u = r.get(tweak.0).unwrap().utility(&t, Job::Tweak).unwrap();
        for other in &r.records {
            if other.id != tweak.0 && !jobs.iter().any(|j| j.0 == other.id) {
                assert!(other.utility(&t, Job::Tweak).unwrap_or(-1.0) <= u + 1e-6, "an idle slimo wanted the Bench more");
            }
        }
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(o.bench_slots, 1);
        assert_eq!(o.work.crafts_by.len(), 1);
    }
}

