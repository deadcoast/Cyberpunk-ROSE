//! The city on the Mat (Full Design v3.9, Colony mode): the 24 structures from `data/structures.toml`, placement
//! and building, zones and their filters, homes and the shift's meals, light, water and sleep, districts that form
//! around a hub and elect a foreman, and the foreman's petitions. Every number here is the design's; the Godot
//! screens draw what this crate holds and the roster (`people`) reads the `ShiftContext` it writes.

pub mod charter;

use std::collections::{HashMap, HashSet};

pub use charter::{Charter, LawDef, LawFile, SignedLaw};
use mat::sim::{hy, sub};
use mat::MatSim;
use people::{Job, Need, Record, Roster, ShiftContext, ShiftWork, Stat, Status, TraitFile};
use serde::{Deserialize, Serialize};

pub const STRUCTURES_TOML: &str = include_str!("../../../../data/structures.toml");

/// Signal a structure needs under it unless its def says otherwise, and the floor for any zone cell.
pub const SIGNAL_FLOOR: f32 = 0.3;
/// Cords from a hub within which structures and beds belong to its district (Pass 2: 12 cords).
pub const DISTRICT_CORDS: u32 = 12;
/// Shifts a dissolved district's Node cannot hub again.
pub const HUB_BAR_SHIFTS: u32 = 10;
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
    /// the failing need (Mood for the unrest petition at 10)
    pub need: Need,
    /// structure ids that answer it, the first the one asked for (empty when it asks a repeal)
    pub asks: Vec<String>,
    /// the law it asks repealed (the unrest petition, Pass 2)
    pub repeal: Option<String>,
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
    /// the members' average Mood, read at the start of the shift (Pass 2's district Mood)
    pub mood: i32,
    /// consecutive shifts that average has been under 30
    pub low_mood: u8,
    /// on strike: members take no jobs until the average is back at 30
    pub strike: bool,
    /// the average is under 20: members leave when a destination is there (Phase D)
    pub ready_to_defect: bool,
    /// petitions refused; the second dissolves the district
    pub refusals: u8,
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
    /// slimos who ate paste, and the eddies it cost (2 e each, 1 under Rationing)
    pub paste_meals: u32,
    pub paste_eddies: u32,
    /// who worked the market, for the sales' Trade XP
    pub sellers: Vec<u32>,
    pub contexts: HashMap<u32, ShiftContext>,
    pub work: ShiftWork,
    pub districts_formed: Vec<u32>,
    pub elections: Vec<(u32, u32)>,
    /// districts striking this shift, districts ready to defect
    pub strikes: Vec<u32>,
    pub ready_to_defect: Vec<u32>,
    /// laws that reached a district this shift: (law, district)
    pub laws_arrived: Vec<(String, u32)>,
    pub authority_gained: u32,
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
    /// the laws, Authority and the board's history (Full Design, The Charter)
    pub charter: Charter,
    /// Nodes a dissolved district stood on, and the shift they may hub again
    pub barred_hubs: Vec<(u16, u16, u32)>,
    /// structures that lost signal priority with their district (read by the routing, PAC-043)
    pub low_priority: Vec<u32>,
    /// cord distance from the Heart per tile, refreshed each shift
    #[serde(skip)]
    pub heart_dist: Vec<u32>,
    /// each slimo's wave distance this shift (its hub, its bed or the Heart)
    #[serde(skip)]
    pub wave: HashMap<u32, u32>,
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
        Colony {
            structures: Vec::new(),
            zones: Vec::new(),
            districts: Vec::new(),
            homes: HashMap::new(),
            next_id: 1,
            shift: 0,
            names_left: DISTRICT_NAMES.iter().map(|s| s.to_string()).collect(),
            charter: Charter::default(),
            barred_hubs: Vec::new(),
            low_priority: Vec::new(),
            heart_dist: Vec::new(),
            wave: HashMap::new(),
        }
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

    // ------------------------------------------------------------ distances on the Mat

    /// The tiles of a structure's footprint that are on the Mat (its anchor at least).
    fn mat_tiles(defs: &StructureFile, mat: &MatSim, s: &Structure) -> Vec<usize> {
        Self::footprint(&defs.structure[s.def], s.x, s.y)
            .into_iter()
            .map(|(x, y)| y as usize * mat.w + x as usize)
            .filter(|&i| i < mat.hy.len() && (mat.hy[i] == hy::CORD || mat.hy[i] == hy::NODE))
            .collect()
    }

    /// Refresh the cord distance from the Heart (the wave's yardstick).
    pub fn refresh_heart_dist(&mut self, mat: &MatSim) {
        let starts: Vec<usize> = mat.heart.map(|(x, y)| vec![y * mat.w + x]).unwrap_or_default();
        self.heart_dist = charter::cord_distances(mat, &starts, u32::MAX);
    }

    fn dist_at(&self, i: usize) -> u32 {
        if self.heart_dist.is_empty() { 0 } else { self.heart_dist.get(i).copied().unwrap_or(u32::MAX) }
    }

    /// How far the wave has to walk to reach a slimo: its district's hub, else its bed, else the Heart (0).
    pub fn wave_dist(&self, defs: &StructureFile, mat: &MatSim, slimo: u32) -> u32 {
        if self.heart_dist.is_empty() {
            return 0;
        }
        if let Some(d) = self.district_of(slimo) {
            return self.dist_at(d.hub.1 as usize * mat.w + d.hub.0 as usize);
        }
        if let Some(h) = self.home_of(slimo) {
            return Self::mat_tiles(defs, mat, h).into_iter().map(|i| self.dist_at(i)).min().unwrap_or(u32::MAX);
        }
        0
    }

    /// The wave's distance to a district's hub.
    pub fn district_dist(&self, mat: &MatSim, d: &District) -> u32 {
        self.dist_at(d.hub.1 as usize * mat.w + d.hub.0 as usize)
    }

    // ------------------------------------------------------------ districts

    /// Form districts where a Node has three built structures within 12 cords, greedily by count, up to the city
    /// tier's cap; then give every structure to the nearest hub within 12 cords. Returns the ids formed.
    pub fn form_districts(&mut self, defs: &StructureFile, mat: &MatSim, city_tier: u8) -> Vec<u32> {
        let cap = DISTRICT_CAP[(city_tier.max(1) as usize - 1).min(3)];
        let mut formed = Vec::new();
        self.barred_hubs.retain(|b| b.2 > self.shift);
        while self.districts.len() < cap {
            let claimed: HashSet<u32> = self.districts.iter().flat_map(|d| d.structures.iter().copied()).collect();
            let built: Vec<Vec<usize>> = self.structures.iter().filter(|s| s.built && !claimed.contains(&s.id)).map(|s| Self::mat_tiles(defs, mat, s)).collect();
            if built.len() < DISTRICT_MIN_STRUCTURES {
                break;
            }
            let hubs: Vec<usize> = self.districts.iter().map(|d| d.hub.1 as usize * mat.w + d.hub.0 as usize).collect();
            let near_hub = charter::cord_reach(mat, &hubs, DISTRICT_CORDS);
            let mut best: Option<(usize, usize)> = None; // (count, tile)
            for i in 0..mat.hy.len() {
                if mat.hy[i] != hy::NODE || near_hub.contains_key(&i) {
                    continue;
                }
                let (x, y) = ((i % mat.w) as u16, (i / mat.w) as u16);
                if self.barred_hubs.iter().any(|b| b.0 == x && b.1 == y) {
                    continue;
                }
                let reach = charter::cord_reach(mat, &[i], DISTRICT_CORDS);
                let n = built.iter().filter(|tiles| tiles.iter().any(|t| reach.contains_key(t))).count();
                if n >= DISTRICT_MIN_STRUCTURES && best.is_none_or(|(bn, bi)| n > bn || (n == bn && i < bi)) {
                    best = Some((n, i));
                }
            }
            let Some((_, i)) = best else { break };
            let id = self.next_id;
            self.next_id += 1;
            let name = if self.names_left.is_empty() { format!("District {id}") } else { self.names_left.remove(0) };
            let hub = ((i % mat.w) as u16, (i / mat.w) as u16);
            self.districts.push(District {
                id,
                name,
                hub,
                structures: Vec::new(),
                members: Vec::new(),
                foreman: None,
                term_left: 0,
                served: 0,
                priorities: [2; 9],
                petition: None,
                boon_shifts: 0,
                wake: false,
                low: [0; 5],
                mood: 60,
                low_mood: 0,
                strike: false,
                ready_to_defect: false,
                refusals: 0,
            });
            // the district's own structures before the next hub is sought
            let reach = charter::cord_reach(mat, &[i], DISTRICT_CORDS);
            let mine: Vec<u32> = self.structures.iter().filter(|s| s.built && !claimed.contains(&s.id) && Self::mat_tiles(defs, mat, s).iter().any(|t| reach.contains_key(t))).map(|s| s.id).collect();
            self.districts.last_mut().unwrap().structures = mine;
            formed.push(id);
        }
        // every structure to the nearest hub within 12 cords
        let reaches: Vec<(u32, HashMap<usize, u32>)> = self.districts.iter().map(|d| (d.id, charter::cord_reach(mat, &[d.hub.1 as usize * mat.w + d.hub.0 as usize], DISTRICT_CORDS))).collect();
        let tiles: Vec<Vec<usize>> = self.structures.iter().map(|s| Self::mat_tiles(defs, mat, s)).collect();
        for (s, t) in self.structures.iter_mut().zip(tiles.iter()) {
            let mut best: Option<(u32, u32)> = None;
            for (did, reach) in &reaches {
                if let Some(dist) = t.iter().filter_map(|i| reach.get(i)).min() {
                    if best.is_none_or(|(bd, _)| *dist < bd) {
                        best = Some((*dist, *did));
                    }
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

    /// A district's voice on the Charter: a built Charter Board among its structures, or the Heart (its own board).
    pub fn has_board(&self, defs: &StructureFile, d: &District) -> bool {
        d.structures.iter().any(|sid| self.get(*sid).is_some_and(|s| s.built && matches!(defs.structure[s.def].id.as_str(), "charter_board" | "heart")))
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

    /// The structures that answer a failing need.
    fn asks_for(need: Need) -> Vec<String> {
        match need {
            Need::Fed => vec!["pit".into()],
            Need::Lit => vec!["lume_farm".into(), "lantern".into()],
            Need::Dry => vec!["lung".into()],
            Need::Rest => vec!["bunkhouse".into(), "pod".into()],
            _ => vec!["shrine".into()],
        }
    }

    /// Unrest (Pass 2), read off each district's average Mood at the start of the shift: strike after two shifts
    /// under 30; ready to defect under 20; under 10 the foreman petitions (through a Board) to repeal the law that
    /// costs the district the most Mood, or for the structure its worst need lacks.
    fn unrest(&mut self, defs: &StructureFile, mat: &MatSim, roster: &Roster, out: &mut ShiftOutcome) {
        let boards: HashSet<u32> = self.districts.iter().filter(|d| self.has_board(defs, d)).map(|d| d.id).collect();
        let dists: HashMap<u32, u32> = self.districts.iter().map(|d| (d.id, self.district_dist(mat, d))).collect();
        let charter = &self.charter;
        for d in self.districts.iter_mut() {
            let members: Vec<&Record> = d.members.iter().filter_map(|m| roster.get(*m)).collect();
            if members.is_empty() {
                d.strike = false;
                d.ready_to_defect = false;
                continue;
            }
            d.mood = members.iter().map(|r| r.mood()).sum::<i32>() / members.len() as i32;
            if d.mood < 30 {
                d.low_mood = d.low_mood.saturating_add(1);
            } else {
                d.low_mood = 0;
            }
            let was = d.strike;
            d.strike = d.mood < 30 && (d.low_mood >= 2 || was);
            if d.strike {
                out.strikes.push(d.id);
                if !was {
                    out.lines.push(format!("{} is on strike: Mood {}", d.name, d.mood));
                }
            }
            d.ready_to_defect = d.mood < 20;
            if d.ready_to_defect {
                out.ready_to_defect.push(d.id);
            }
            if d.mood < 10 && d.foreman.is_some() && boards.contains(&d.id) {
                let dist = dists[&d.id];
                let costliest = charter.in_force(dist).filter(|l| l.mood < 0).min_by_key(|l| (l.mood, l.shift)).map(|l| (l.id.clone(), l.name.clone()));
                let standing_repeal = d.petition.as_ref().and_then(|p| p.repeal.clone());
                if let Some((law, name)) = costliest {
                    if standing_repeal.as_deref() != Some(law.as_str()) {
                        d.petition = Some(Petition { need: Need::Mood, asks: Vec::new(), repeal: Some(law), shifts_left: PETITION_SHIFTS, missed: false });
                        out.petitions_raised.push((d.id, Need::Mood));
                        out.lines.push(format!("{} petitions the Charter: repeal {}", d.name, name));
                    }
                } else if d.petition.is_none() {
                    let worst = [Need::Fed, Need::Lit, Need::Dry, Need::Signal, Need::Rest].into_iter().min_by_key(|n| members.iter().map(|r| r.need(*n)).sum::<i32>()).unwrap();
                    d.petition = Some(Petition { need: worst, asks: Self::asks_for(worst), repeal: None, shifts_left: PETITION_SHIFTS, missed: false });
                    out.petitions_raised.push((d.id, worst));
                    out.lines.push(format!("{} petitions the Charter: {}", d.name, worst.name().to_lowercase()));
                }
            }
        }
    }

    /// Petitions (research 14): a need averaging under 30 for two shifts raises one through a Board; the deadline
    /// counts down; the structure standing in the district (or the repeal having reached it) answers it.
    /// Returns (raised, met, missed).
    fn petitions(&mut self, defs: &StructureFile, mat: &MatSim, roster: &Roster) -> (Vec<(u32, Need)>, Vec<u32>, Vec<u32>) {
        let (mut raised, mut met, mut missed) = (Vec::new(), Vec::new(), Vec::new());
        let boards: HashSet<u32> = self.districts.iter().filter(|d| self.has_board(defs, d)).map(|d| d.id).collect();
        let dists: HashMap<u32, u32> = self.districts.iter().map(|d| (d.id, self.district_dist(mat, d))).collect();
        let charter = &self.charter;
        let structures = &self.structures;
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
                let answered = match &p.repeal {
                    Some(law) => !charter.law(law).is_some_and(|l| l.at(dists[&d.id])),
                    None => d.structures.iter().any(|sid| structures.iter().any(|s| s.id == *sid && s.built && p.asks.contains(&defs.structure[s.def].id))),
                };
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
            if d.foreman.is_none() || !boards.contains(&d.id) {
                continue;
            }
            let worst = [Need::Fed, Need::Lit, Need::Dry, Need::Signal, Need::Rest].into_iter().enumerate().filter(|(k, _)| d.low[*k] >= PETITION_AFTER).min_by_key(|(k, _)| {
                let need = [Need::Fed, Need::Lit, Need::Dry, Need::Signal, Need::Rest][*k];
                members.iter().map(|r| r.need(need)).sum::<i32>()
            });
            if let Some((_, need)) = worst {
                d.petition = Some(Petition { need, asks: Self::asks_for(need), repeal: None, shifts_left: PETITION_SHIFTS, missed: false });
                raised.push((d.id, need));
            }
        }
        (raised, met, missed)
    }

    /// Refuse a district's standing petition: it closes; the second refusal dissolves the district (Pass 2) — its
    /// hub is removed, its foreman stands down, its structures lose signal priority, and the Node cannot hub again
    /// for ten shifts. Returns whether the district was dissolved.
    pub fn refuse_petition(&mut self, district: u32) -> Result<bool, String> {
        let idx = self.districts.iter().position(|d| d.id == district).ok_or("no such district")?;
        if self.districts[idx].petition.is_none() {
            return Err("no petition stands".into());
        }
        let d = &mut self.districts[idx];
        d.petition = None;
        d.refusals += 1;
        if d.refusals < 2 {
            self.charter.history.push(format!("shift {}: refused {}'s petition", self.shift, d.name));
            return Ok(false);
        }
        let d = self.districts.remove(idx);
        self.barred_hubs.push((d.hub.0, d.hub.1, self.shift + HUB_BAR_SHIFTS));
        for s in self.structures.iter_mut() {
            if s.district == Some(d.id) {
                s.district = None;
                if !self.low_priority.contains(&s.id) {
                    self.low_priority.push(s.id);
                }
            }
        }
        self.charter.history.push(format!("shift {}: refused {} twice; the district is dissolved", self.shift, d.name));
        Ok(true)
    }

    // ------------------------------------------------------------ the Charter

    /// Sign a law: the districts with a Board vote through their foremen (against when it costs Mood), the
    /// Authority is paid and the wave leaves the Heart.
    pub fn sign(&mut self, defs: &StructureFile, mat: &MatSim, law: &LawDef) -> Result<(), String> {
        let voters = self.districts.iter().filter(|d| d.foreman.is_some() && self.has_board(defs, d)).count() as u32;
        let votes = if law.mood < 0 { (0, voters) } else { (voters, 0) };
        self.charter.sign(law, self.shift, charter::wave_speed(mat), votes)
    }

    pub fn repeal(&mut self, mat: &MatSim, law: &str) -> Result<(), String> {
        self.charter.repeal(law, self.shift, charter::wave_speed(mat))
    }

    /// Dissolve a D5 into the Heart's Pit (Full Design, Dissolution): 0.30 nutrient, the name on the Memorial, and
    /// the per-event Mood hit next shift unless the Rite is in force at the Heart.
    pub fn dissolve(&mut self, roster: &mut Roster, mat: &mut MatSim, slimo: u32) -> Result<String, String> {
        let r = roster.get(slimo).ok_or("no such slimo")?;
        if r.depth != people::Depth::D5 {
            return Err("only a D5 can be dissolved".into());
        }
        let name = r.full_name();
        roster.dissolve(slimo).ok_or("no such slimo")?;
        self.homes.remove(&slimo);
        if let Some((x, y)) = mat.heart {
            mat.feed(x, y, charter::DISSOLUTION_NUTRIENT);
        }
        let rite = self.charter.flag_at("dissolution_rite", 0);
        if !rite {
            self.charter.dissolution_hit = true;
        }
        Ok(format!("{name} was dissolved into the Heart{}", if rite { " by the Rite" } else { "; the city felt it" }))
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
        // a district in a wake or on strike sits out
        let out_: HashSet<u32> = self.districts.iter().filter(|d| d.wake || d.strike).flat_map(|d| d.members.iter().copied()).collect();
        let mut pairs: Vec<(f32, u32, usize)> = Vec::new();
        for r in &roster.records {
            // a slimo whose shift was a run (shift_job set by apply_run) is done; Curfew sends everyone home in Dark
            if out_.contains(&r.id) || r.shift_job.is_some() || !r.on_duty(shift_of_day) {
                continue;
            }
            if shift_of_day == 1 && self.charter.flag_at("curfew", self.wave.get(&r.id).copied().unwrap_or(0)) {
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
        // the wave's yardstick, homes and districts first: unrest and jobs read them
        self.refresh_heart_dist(mat);
        self.assign_homes(defs, roster, traits, mat);
        out.districts_formed = self.form_districts(defs, mat, city_tier);
        // Authority by deeds: a district formed (Pass 2)
        out.authority_gained = out.districts_formed.len() as u32;
        self.charter.earn(out.authority_gained);
        self.refresh_members(roster, &[]);
        self.wave = roster.records.iter().map(|r| (r.id, self.wave_dist(defs, mat, r.id))).collect();
        self.unrest(defs, mat, roster, &mut out);
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
            // the laws the wave has brought to this slimo
            let wd = self.wave_dist(defs, mat, r.id);
            let rationed = self.charter.flag_at("meal_draw_half", wd);
            // a meal: the Mat's nutrient at the home, else paste; Rationing halves the draw
            let (takes, paste) = if rationed { (MEAL_TAKES * 0.5, PASTE_EDDIES / 2) } else { (MEAL_TAKES, PASTE_EDDIES) };
            let mut fed = false;
            if signalled && mat.n[i] >= MEAL_NUTRIENT {
                mat.n[i] -= takes;
                fed = true;
            } else if eddies_left >= paste {
                eddies_left -= paste;
                out.paste_meals += 1;
                out.paste_eddies += paste;
                fed = true;
            }
            // a slimo who ran this shift worked it; everyone without a slot slept
            let asleep = !working.contains(&r.id) && r.shift_job.is_none();
            let mut extra = self.charter.mood_at(wd);
            if self.charter.dissolution_hit {
                extra += charter::DISSOLUTION_HIT;
            }
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
        self.charter.dissolution_hit = false;
        // petitions read last shift's needs; the wake is over after one shift
        let (raised, met, missed) = self.petitions(defs, mat, roster);
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
        out.petitions_raised.extend(raised);
        out.petitions_met = met;
        out.petitions_missed = missed;
        for d in self.districts.iter_mut() {
            d.wake = false;
        }
        // what the waves brought to a district this shift (each law announces itself once per district)
        let dists: Vec<(u32, String, u32)> = self.districts.iter().map(|d| (d.id, d.name.clone(), self.district_dist(mat, d))).collect();
        for l in self.charter.laws.iter_mut() {
            for (did, name, dist) in &dists {
                let here = l.at(*dist);
                if here && !l.reached.contains(did) {
                    l.reached.push(*did);
                    out.lines.push(format!("{} reached {}", l.name, name));
                    out.laws_arrived.push((l.id.clone(), *did));
                } else if !here && l.reached.contains(did) {
                    l.reached.retain(|x| x != did);
                    out.lines.push(format!("{} lifted in {}", l.name, name));
                }
            }
        }
        let farthest = self.heart_dist.iter().copied().filter(|d| *d != u32::MAX).max().unwrap_or(0);
        self.charter.advance(charter::wave_speed(mat), farthest);
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

    /// A tile within the district's 12 cords where a def can stand, nearest the hub first.
    fn spot_in(c: &Colony, defs: &StructureFile, mat: &MatSim, id: &str, hub: (u16, u16)) -> Option<(u16, u16)> {
        let reach = charter::cord_reach(mat, &[hub.1 as usize * mat.w + hub.0 as usize], DISTRICT_CORDS);
        let mut tiles: Vec<(u32, usize)> = reach.iter().map(|(i, d)| (*d, *i)).collect();
        tiles.sort();
        tiles.into_iter().map(|(_, i)| ((i % mat.w) as u16, (i / mat.w) as u16)).find(|&(x, y)| c.can_place(defs, mat, id, x, y).is_ok())
    }

    /// The founded city with three pods by the Heart and one shift run, so the Heart's district stands.
    fn with_district() -> (Colony, StructureFile, MatSim, Roster, TraitFile) {
        let (mut c, defs, mut mat, mut r, t) = city();
        for rec in r.records.iter_mut() {
            rec.priorities = [0; 9];
        }
        let heart = mat.heart.unwrap();
        for _ in 0..3 {
            let (x, y) = spot_in(&c, &defs, &mat, "pod", (heart.0 as u16, heart.1 as u16)).unwrap();
            c.place_built(&defs, &mat, "pod", x, y, 1).unwrap();
        }
        c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(c.districts.len(), 1);
        (c, defs, mat, r, t)
    }

    #[test]
    fn a_district_holds_what_stands_within_twelve_cords_and_its_forming_pays_the_first_law() {
        let (c, defs, mat, _, _) = with_district();
        let d = &c.districts[0];
        let reach = charter::cord_reach(&mat, &[d.hub.1 as usize * mat.w + d.hub.0 as usize], DISTRICT_CORDS);
        for sid in &d.structures {
            let s = c.get(*sid).unwrap();
            assert!(Colony::mat_tiles(&defs, &mat, s).iter().any(|t| reach.contains_key(t)), "{} is not within 12 cords", defs.structure[s.def].id);
        }
        // the Heart's district speaks through the Heart's own board
        assert!(c.has_board(&defs, d));
        // Authority by deeds: the district formed is one, and one signs Rationing
        assert_eq!(c.charter.authority, 1);
        assert_eq!((c.charter.earned, c.charter.ivan_state()), (1, 1));
        let laws = LawFile::shipped();
        assert!(c.charter.cannot_sign(laws.get("rationing").unwrap()).is_none());
        assert_eq!(c.charter.cannot_sign(laws.get("conscription").unwrap()), Some("needs 2 Authority".into()));
    }

    #[test]
    fn rationing_halves_the_draw_and_costs_five_and_the_wave_reaches_the_heart_at_once() {
        let (mut c, defs, mut mat, r, t) = with_district();
        let laws = LawFile::shipped();
        c.sign(&defs, &mat, laws.get("rationing").unwrap()).unwrap();
        assert_eq!(c.charter.authority, 0);
        let sl = c.charter.law("rationing").unwrap();
        assert_eq!(sl.reach, charter::wave_speed(&mat));
        assert_eq!((sl.votes_for, sl.votes_against), (0, 1), "the Heart's foreman votes against a law that costs Mood");
        // a slimo with a bed on nutrient eats half a meal's draw
        let (who, home) = c.homes.iter().map(|(a, b)| (*a, *b)).next().unwrap();
        let h = c.get(home).unwrap();
        let i = h.y as usize * mat.w + h.x as usize;
        mat.n[i] = 0.5;
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(o.contexts[&who].fed);
        assert!((0.5 - mat.n[i] - MEAL_TAKES * 0.5).abs() < 1e-5, "draw {}", 0.5 - mat.n[i]);
        assert!(o.contexts[&who].extra_target <= -5);
        assert!(o.laws_arrived.iter().any(|(l, _)| l == "rationing"));
        // paste costs 1 e
        mat.n[i] = 0.0;
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 1, &mut |_| true);
        assert!(o.contexts[&who].fed && o.paste_eddies >= 1);
        // a repeal is free and the Heart is clear of it the shift it is repealed
        c.repeal(&mat, "rationing").unwrap();
        assert_eq!(c.charter.authority, 0);
        mat.n[i] = 0.5;
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!((0.5 - mat.n[i] - MEAL_TAKES).abs() < 1e-5);
        assert!(o.contexts[&who].extra_target > -5);
    }

    #[test]
    fn unrest_strikes_after_two_shifts_under_30_petitions_at_10_and_two_refusals_dissolve() {
        let (mut c, defs, mut mat, mut r, t) = with_district();
        let laws = LawFile::shipped();
        c.sign(&defs, &mat, laws.get("rationing").unwrap()).unwrap();
        let did = c.districts[0].id;
        let members = c.districts[0].members.clone();
        for rec in r.records.iter_mut() {
            rec.priorities = [2; 9];
        }
        let set_mood = |r: &mut Roster, m: i32| {
            for id in &members {
                r.get_mut(*id).unwrap().needs[Need::Mood.index()] = m;
            }
        };
        set_mood(&mut r, 25);
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(o.strikes.is_empty(), "one shift under 30 is not a strike");
        set_mood(&mut r, 25);
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(o.strikes, vec![did]);
        assert!(!o.assignments.iter().any(|a| members.contains(&a.0)), "strikers took jobs");
        // under 20: ready to defect (the destinations are Phase D's)
        set_mood(&mut r, 15);
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(o.ready_to_defect, vec![did]);
        // under 10: the foreman petitions the Charter to repeal Rationing, the law that costs the district Mood
        set_mood(&mut r, 5);
        c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        let p = c.districts[0].petition.clone().unwrap();
        assert_eq!(p.repeal.as_deref(), Some("rationing"));
        // refused once: it closes; raised again, refused again: the district is dissolved and its Node barred
        assert_eq!(c.refuse_petition(did), Ok(false));
        set_mood(&mut r, 5);
        c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(c.districts[0].petition.as_ref().is_some_and(|p| p.repeal.is_some()));
        let hub = c.districts[0].hub;
        assert_eq!(c.refuse_petition(did), Ok(true));
        assert!(c.districts.is_empty());
        assert!(c.barred_hubs.iter().any(|b| (b.0, b.1) == hub));
        assert!(!c.low_priority.is_empty());
        assert!(c.structures.iter().all(|s| s.district.is_none()));
        assert!(c.charter.history.last().unwrap().contains("dissolved"));
    }

    #[test]
    fn answering_a_repeal_petition_is_the_repeal_reaching_the_district() {
        let (mut c, defs, mut mat, mut r, t) = with_district();
        c.sign(&defs, &mat, LawFile::shipped().get("rationing").unwrap()).unwrap();
        let did = c.districts[0].id;
        for id in c.districts[0].members.clone() {
            r.get_mut(id).unwrap().needs[Need::Mood.index()] = 5;
        }
        c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(c.districts[0].petition.as_ref().is_some_and(|p| p.repeal.is_some()));
        c.repeal(&mat, "rationing").unwrap();
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert_eq!(o.petitions_met, vec![did]);
        assert_eq!(c.districts[0].boon_shifts, 3);
    }

    #[test]
    fn a_d5_dissolves_into_the_heart_and_the_city_feels_it_without_the_rite() {
        let (mut c, defs, mut mat, mut r, t) = with_district();
        // make the first record a D5 (2 % of creation)
        let id = r.records[0].id;
        r.get_mut(id).unwrap().depth = people::Depth::D5;
        let (hx, hy_) = mat.heart.unwrap();
        let before = mat.pits.get(&(hx, hy_)).copied().unwrap_or(0.0);
        let not_d5 = r.records.iter().find(|x| x.depth != people::Depth::D5).unwrap().id;
        assert_eq!(c.dissolve(&mut r, &mut mat, not_d5).unwrap_err(), "only a D5 can be dissolved");
        let line = c.dissolve(&mut r, &mut mat, id).unwrap();
        assert!(line.contains("the city felt it"));
        assert!(r.get(id).is_none() && r.memorial.iter().any(|m| m.id == id));
        assert!((mat.pits[&(hx, hy_)] - before - charter::DISSOLUTION_NUTRIENT).abs() < 1e-6, "the Heart's Pit took 0.30");
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        let other = r.records[0].id;
        assert!(o.contexts[&other].extra_target <= charter::DISSOLUTION_HIT, "the per-event hit");
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(o.contexts[&other].extra_target > charter::DISSOLUTION_HIT, "the hit is one shift");
        // with the Rite: no hit, a standing -3
        c.charter.authority = 2;
        c.sign(&defs, &mat, LawFile::shipped().get("dissolution_rite").unwrap()).unwrap();
        let id2 = r.records[0].id;
        r.get_mut(id2).unwrap().depth = people::Depth::D5;
        let line = c.dissolve(&mut r, &mut mat, id2).unwrap();
        assert!(line.contains("by the Rite"));
        let o = c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        let other = r.records[0].id;
        let x = o.contexts[&other].extra_target;
        assert!(x > charter::DISSOLUTION_HIT && x <= -3, "standing -3 only: {x}");
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
        let (x, y) = spot_in(&c, &defs, &mat, "lantern", d.hub).expect("a lantern spot inside the district");
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
    fn any_works_while_rested_and_curfew_sends_the_city_home_in_dark() {
        let (mut c, defs, mut mat, mut r, t) = with_district();
        for rec in r.records.iter_mut() {
            rec.priorities = [2; 9];
        }
        // the default row is Any in every shift: rested slimos work Dark too
        assert!(!c.assign_jobs(&defs, &r, &t, 1).is_empty());
        for rec in r.records.iter_mut() {
            rec.needs[Need::Rest.index()] = 39;
        }
        assert!(c.assign_jobs(&defs, &r, &t, 1).is_empty(), "Any sleeps under Rest 40");
        for rec in r.records.iter_mut() {
            rec.needs[Need::Rest.index()] = 80;
        }
        // Curfew: the Dark is Rest whatever the row; Dim still works
        c.charter.authority = 1;
        c.sign(&defs, &mat, LawFile::shipped().get("curfew").unwrap()).unwrap();
        c.end_shift(&defs, &mut mat, &r, &t, 0, 100, &mut |_| true);
        assert!(c.assign_jobs(&defs, &r, &t, 1).is_empty(), "Curfew: nobody works Dark");
        assert!(!c.assign_jobs(&defs, &r, &t, 0).is_empty());
        // a row set to Work in Dark obeys the Curfew too
        let id = r.records[0].id;
        r.get_mut(id).unwrap().schedule[1] = people::Duty::Work;
        assert!(c.assign_jobs(&defs, &r, &t, 1).is_empty());
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

