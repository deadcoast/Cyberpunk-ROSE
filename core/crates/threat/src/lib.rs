//! threat — the surface's answer (Full Design v3.14, Colony mode: Pressure; Pass 1's Threats; research 18): Attention
//! per grate zone, the three stages that come down a grate when a zone is loud enough (Patrol, Sweep, Cleansing), their
//! bodies on the city's map and what they do to the Mat and the city, the city's defence, Blooms from cut-off strains
//! and their crawlers, and the Flood shift's water. The stages and the bodies are data (`data/threats.toml`).
//!
//! Effects on the Mat (white light, bleach, Toxicity) run on the Mat's step; bodies run on the city's clock (2 ticks a
//! second). Every random draw comes from a stream seeded by (seed, shift), so a city replays.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use colony::{Colony, StructureFile};
use logistics::City;
use mat::rng::Pcg32;
use mat::sim::{hy, sub};
use mat::MatSim;
use people::{Depth, Job, Roster, Skill};
use serde::{Deserialize, Serialize};

pub const THREATS_TOML: &str = include_str!("../../../../data/threats.toml");

/// City ticks a second.
pub const HZ: f32 = 2.0;

// ------------------------------------------------------------------ rules

#[derive(Clone, Debug, Deserialize)]
pub struct AttentionRules {
    pub max: f32,
    pub decay: f32,
    pub near: i32,
    pub near_grate: Vec<String>,
    pub in_zone: Vec<String>,
    pub per_eddies: u32,
    pub exposed: f32,
    pub blackline_sold: f32,
    pub kill: f32,
    pub open_grates: f32,
    pub defector: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StageDef {
    pub id: String,
    pub name: String,
    pub at: f32,
    /// the first stratum it comes down in
    pub from: u8,
    pub quiet: u32,
    pub squad: Vec<(String, u32)>,
    pub map_s: f32,
    pub salvage: [u32; 2],
    pub salvage_per: String,
    pub relics: [u32; 2],
    pub arrive_in: u32,
    #[serde(default)]
    pub break_at: f32,
    #[serde(default)]
    pub flood_steps: u32,
    #[serde(default)]
    pub lung_cut: f32,
    #[serde(default)]
    pub flamers_at: f32,
    #[serde(default)]
    pub lasts: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BodyDef {
    pub name: String,
    pub hp: i32,
    pub walk: u8,
    #[serde(default)]
    pub lamp: i32,
    /// what its lamp does to a tile of Mat a step
    #[serde(default = "one")]
    pub lamp_burn: f32,
    #[serde(default)]
    pub shot: i32,
    #[serde(default)]
    pub shot_s: f32,
    #[serde(default)]
    pub range: i32,
    #[serde(default)]
    pub strike: i32,
    #[serde(default)]
    pub strike_s: f32,
    #[serde(default)]
    pub bleach: i32,
    #[serde(default)]
    pub flame: f32,
    #[serde(default)]
    pub burn: f32,
    #[serde(default)]
    pub bite: i32,
    #[serde(default)]
    pub bite_s: f32,
    #[serde(default)]
    pub chew: f32,
    pub oathkeeper: bool,
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize)]
pub struct DefenceRules {
    pub turret_s: f32,
    pub guard_strike: i32,
    pub guard_s: f32,
    pub toxicity: f32,
    pub panic_mood: i32,
    pub bleed_at: f32,
    pub tame_within: i32,
    pub tamed_within: i32,
    pub tamed_bite: i32,
    pub tamed_bite_s: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BloomRules {
    pub feral_drive: f32,
    pub per_node: u32,
    pub most_per_node: u32,
    pub quarantine_after: u32,
    pub borders: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FloodRules {
    pub roll: [u8; 2],
    pub damage: f32,
    pub curfew_damage: f32,
    pub drowns: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ThreatFile {
    pub attention: AttentionRules,
    pub stage: Vec<StageDef>,
    pub body: BTreeMap<String, BodyDef>,
    pub defence: DefenceRules,
    pub bloom: BloomRules,
    pub flood: FloodRules,
}

impl ThreatFile {
    pub fn from_toml(text: &str) -> Result<ThreatFile, String> {
        let f: ThreatFile = toml::from_str(text).map_err(|e| e.to_string())?;
        for st in &f.stage {
            for (role, _) in &st.squad {
                if !f.body.contains_key(role) {
                    return Err(format!("stage {} sends an unknown body {role}", st.id));
                }
            }
        }
        for want in ["scout", "sprayer", "carrier", "flamer", "crawler"] {
            if !f.body.contains_key(want) {
                return Err(format!("no body {want}"));
            }
        }
        Ok(f)
    }
    pub fn shipped() -> ThreatFile {
        ThreatFile::from_toml(THREATS_TOML).expect("threats.toml")
    }
    pub fn stage_index(&self, id: &str) -> Option<usize> {
        self.stage.iter().position(|s| s.id == id)
    }
}

// ------------------------------------------------------------------ state

/// One grate's zone (Full Design, Pressure: Attention).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    /// the grate's column on the top row
    pub grate: usize,
    pub attention: f32,
    /// the shift each stage last came down here
    pub last: Vec<Option<u32>>,
    /// this shift: eddies sold at its grate, Oathkeepers killed in it, Blackline pieces sold at it (v13)
    pub sold: u32,
    pub kills: u32,
    #[serde(default)]
    pub blackline: u32,
    /// what the last shift end added and took, for the card
    pub gained: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Scout,
    Sprayer,
    Carrier,
    Flamer,
    Crawler,
    /// a crawler a Pen tamed: the city's
    Tamed,
}

impl Role {
    pub fn key(self) -> &'static str {
        match self {
            Role::Scout => "scout",
            Role::Sprayer => "sprayer",
            Role::Carrier => "carrier",
            Role::Flamer => "flamer",
            Role::Crawler | Role::Tamed => "crawler",
        }
    }
    fn from_key(k: &str) -> Role {
        match k {
            "scout" => Role::Scout,
            "sprayer" => Role::Sprayer,
            "carrier" => Role::Carrier,
            "flamer" => Role::Flamer,
            _ => Role::Crawler,
        }
    }
    pub fn oathkeeper(self) -> bool {
        matches!(self, Role::Scout | Role::Sprayer | Role::Carrier | Role::Flamer)
    }
    pub fn hostile(self) -> bool {
        self != Role::Tamed
    }
}

/// What hit a body last: it shoots back at that.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Foe {
    Structure(u32),
    Slimo(u32),
    Body(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Goal {
    /// map the zone's brightest source: walk there, stay
    Map(u16, u16),
    /// toward the Heart
    Heart,
    /// a structure to flame (the Pits, then the Heart)
    Burn(u32),
    /// back up the grate
    Leave,
    /// a crawler hunting the city
    Hunt,
    /// a tamed crawler at its Pen
    Pen(u32),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub id: u32,
    pub role: Role,
    pub x: u16,
    pub y: u16,
    pub hp: f32,
    pub max_hp: f32,
    /// the event it came down with (0: a Bloom's)
    pub event: u32,
    pub zone: u16,
    pub goal: Goal,
    pub foe: Option<Foe>,
    /// seconds to its next shot, bite or strike
    pub cd: f32,
    /// ticks to its next step
    pub step: u8,
    /// seconds left mapping its source
    pub mapping: f32,
    /// a Bloom crawler's Node; a tamed one's Pen
    pub home: usize,
    pub pen: Option<u32>,
    pub dead: bool,
    pub gone: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// a Cleansing's bleach flood
    Flood,
    Bodies,
    Leaving,
    Done,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: u32,
    pub stage: usize,
    pub zone: u16,
    pub started: u64,
    pub phase: Phase,
    /// Mat steps of bleach flood left
    pub flood_left: u32,
    pub sent: u32,
    pub fallen: u32,
    pub flamers_out: bool,
    /// the lamp carriers' way from the grate to the Heart, tiles, when they set out
    pub way: u32,
    /// Mat tiles it burned, structures it ruined, for the log
    pub burned: u32,
    pub ruined: u32,
}

/// An event due down a grate this shift.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Due {
    pub stage: usize,
    pub zone: u16,
    /// city ticks from now
    pub in_ticks: u32,
}

/// The Flood shift's water (Full Design, Pressure: Floods).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Flood {
    /// the day's roll less the Lungs, and a Cleansing's sluice on top
    pub level: u8,
    pub sluice: u8,
    /// tiles turned to Water, with what they were
    pub changed: Vec<(u32, u8)>,
    /// a Flood shift's roll before the Lungs (what a run on the shift rises by); None on another shift
    #[serde(default)]
    pub roll: Option<u8>,
}

/// A line for the city's log.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub loss: bool,
    pub decision: bool,
    pub what: String,
}

impl Note {
    fn landing(what: String) -> Note {
        Note { loss: false, decision: false, what }
    }
    fn loss(what: String) -> Note {
        Note { loss: true, decision: false, what }
    }
}

/// What the shift's end hands the game.
#[derive(Clone, Debug, Default)]
pub struct Ended {
    pub notes: Vec<Note>,
    /// Blackline and Relics to the Stash
    pub blackline: u32,
    pub relics: u32,
    /// structures the shift's flood drowned (they lost HP and worked nothing)
    pub drowned: Vec<u32>,
    /// bodies that fell (role keys), stages repelled (stage ids), hazards lived through (flood, bloom, sweep,
    /// cleansing) this shift
    pub killed: Vec<String>,
    pub repelled: Vec<String>,
    pub lived: Vec<String>,
}

/// The shift's own numbers the gauge reads (Full Design, Pressure: Attention).
#[derive(Clone, Debug, Default)]
pub struct Shown {
    /// Blackline and Relic items outside a Vault
    pub exposed: u32,
    /// Oathkeepers a run killed this shift
    pub run_kills: u32,
}

/// The city's people as the threats read them this shift: who stands Guard where, who tends which Pen, each slimo's
/// Fight, and the D2s Conscription makes militia.
#[derive(Clone, Debug, Default)]
pub struct Posts {
    /// (slimo, structure) on Guard
    pub guards: Vec<(u32, u32)>,
    /// (slimo, Pen) on Tend
    pub tenders: Vec<(u32, u32)>,
    pub fight: BTreeMap<u32, i32>,
    pub d2: BTreeSet<u32>,
}

impl Posts {
    pub fn from_plan(plan: &[(u32, Job, u32)], roster: &Roster) -> Posts {
        Posts {
            guards: plan.iter().filter(|a| a.1 == Job::Guard).map(|a| (a.0, a.2)).collect(),
            tenders: plan.iter().filter(|a| a.1 == Job::Tend).map(|a| (a.0, a.2)).collect(),
            fight: roster.records.iter().map(|r| (r.id, r.skill(Skill::Fight) as i32)).collect(),
            d2: roster.records.iter().filter(|r| r.depth == Depth::D2 && r.is_up()).map(|r| r.id).collect(),
        }
    }
}

/// What a save carries of the threats (v12): the gauges, the bodies and events down, what is due, the water.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThreatSave {
    pub stratum: u8,
    pub zones: Vec<Zone>,
    pub bodies: Vec<Body>,
    pub events: Vec<Event>,
    pub due: Vec<Due>,
    pub flood: Flood,
    pub next_id: u32,
    pub shift: u32,
    pub owed: (u32, u32),
}

#[derive(Clone, Debug)]
pub struct Threats {
    pub rules: ThreatFile,
    pub seed: u64,
    /// the stratum the city stands in (the Sump, 1, until a Relay breaches up — Phase E)
    pub stratum: u8,
    pub zones: Vec<Zone>,
    /// the zone of every tile (u16::MAX where no ground leads)
    zone_of: Vec<u16>,
    pub bodies: Vec<Body>,
    pub events: Vec<Event>,
    pub due: Vec<Due>,
    pub flood: Flood,
    pub next_id: u32,
    /// the shift counter (the game's day)
    pub shift: u32,
    /// damage slimos took this shift, and the ones the white light panicked
    pub hurt: BTreeMap<u32, f32>,
    pub panicked: BTreeSet<u32>,
    /// Mat with no signal at the last shift end (Quarantine Protocol)
    dark: Vec<bool>,
    dark_shifts: Vec<u8>,
    /// bleach on a tile, Mat steps it still shows (the view)
    pub stain: Vec<u8>,
    last_step: u64,
    cds: HashMap<(u32, u32), f32>,
    fields: HashMap<(usize, bool), Vec<u16>>,
    fields_at: u64,
    /// salvage owed when events end
    pub owed: (u32, u32),
    /// what fell, what was repelled and what the city lived through since the last shift end (Full Design v3.18:
    /// the trophies and Authority's deeds)
    pub killed: Vec<String>,
    pub repelled: Vec<String>,
    pub lived: Vec<String>,
}

const NONE16: u16 = u16::MAX;

fn walkable_sub(s: u8) -> bool {
    matches!(s, sub::OPEN | sub::PIPE | sub::DEPOSIT | sub::GRATE | sub::WATER)
}

/// A tile a body can walk: open ground, pipe, a Deposit, a grate, water. The Mat grows into brick and concrete; its
/// threads there are no tunnel.
pub fn walkable(sim: &MatSim, i: usize) -> bool {
    walkable_sub(sim.sub[i])
}

/// The Filter Lungs the strain holds (Full Design v3.18, Organs): each holds the Flood a tile lower.
pub fn lung_organs(sim: &MatSim) -> usize {
    progress::StrainFile::shipped().organ_index("lung").map_or(0, |k| sim.organs_of(k).count())
}

/// Attention a Lume Bulb out of a housing adds at a grate within reach a shift (Full Design v3.18, Organs).
pub const BULB_ATTENTION: f32 = 1.0;

/// A Garden strain's Lung quiets the light within 3 tiles of it (Pass 2).
fn garden_quiet(sim: &MatSim, garden: bool, t: (i32, i32)) -> bool {
    garden && progress::StrainFile::shipped().organ_index("lung").is_some_and(|k| sim.organs_of(k).any(|o| ((o.i as usize % sim.w) as i32 - t.0).abs().max((o.i as usize / sim.w) as i32 - t.1).abs() <= 3))
}

/// A Lume Bulb's Attention line by its rule (Pass 2: 50; Organ rules: 25, 75, never).
pub fn bulb_line(rule: u8) -> f32 {
    match rule {
        1 => 25.0,
        2 => 75.0,
        3 => f32::INFINITY,
        _ => 50.0,
    }
}

fn dist2(a: (u16, u16), b: (u16, u16)) -> i32 {
    let dx = a.0 as i32 - b.0 as i32;
    let dy = a.1 as i32 - b.1 as i32;
    dx * dx + dy * dy
}

fn cheb(a: (u16, u16), b: (u16, u16)) -> i32 {
    (a.0 as i32 - b.0 as i32).abs().max((a.1 as i32 - b.1 as i32).abs())
}

impl Threats {
    /// The zones of a stratum: a grate's zone is the tiles nearer it than any other grate along the ground the Mat can
    /// grow on, ties to the lower column.
    pub fn new(sim: &MatSim, seed: u64) -> Threats {
        let rules = ThreatFile::shipped();
        let grates: Vec<usize> = (0..sim.w).filter(|&x| sim.sub[x] == sub::GRATE).collect();
        let zones = grates.iter().map(|&g| Zone { grate: g, attention: 0.0, last: vec![None; rules.stage.len()], sold: 0, kills: 0, blackline: 0, gained: 0.0 }).collect();
        let n = sim.w * sim.h;
        let mut t = Threats {
            rules,
            seed,
            stratum: 1,
            zones,
            zone_of: Vec::new(),
            bodies: Vec::new(),
            events: Vec::new(),
            due: Vec::new(),
            flood: Flood::default(),
            next_id: 1,
            shift: 1,
            hurt: BTreeMap::new(),
            panicked: BTreeSet::new(),
            dark: vec![false; n],
            dark_shifts: vec![0; n],
            stain: vec![0; n],
            last_step: sim.tick,
            cds: HashMap::new(),
            fields: HashMap::new(),
            fields_at: 0,
            owed: (0, 0),
            killed: Vec::new(),
            repelled: Vec::new(),
            lived: Vec::new(),
        };
        t.rezone(sim);
        t
    }

    /// What a save keeps.
    pub fn save(&self) -> ThreatSave {
        ThreatSave { stratum: self.stratum, zones: self.zones.clone(), bodies: self.bodies.clone(), events: self.events.clone(), due: self.due.clone(), flood: self.flood.clone(), next_id: self.next_id, shift: self.shift, owed: self.owed }
    }

    /// The threats a save kept, on its stratum (an empty save is a fresh start).
    pub fn restore(sim: &MatSim, seed: u64, s: ThreatSave) -> Threats {
        let mut t = Threats::new(sim, seed);
        if s.zones.len() == t.zones.len() {
            t.stratum = s.stratum.max(1);
            t.zones = s.zones;
            t.bodies = s.bodies;
            t.events = s.events;
            t.due = s.due;
            t.flood = s.flood;
            t.next_id = s.next_id.max(1);
            t.shift = s.shift.max(1);
            t.owed = s.owed;
        }
        t
    }

    /// Rebuild what a save does not carry.
    pub fn rezone(&mut self, sim: &MatSim) {
        let n = sim.w * sim.h;
        let mut zone_of = vec![NONE16; n];
        let mut q = VecDeque::new();
        for (k, z) in self.zones.iter().enumerate() {
            zone_of[z.grate] = k as u16;
            q.push_back(z.grate);
        }
        while let Some(i) = q.pop_front() {
            let (x, y) = (i % sim.w, i / sim.w);
            for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
                if nx < sim.w && ny < sim.h {
                    let j = ny * sim.w + nx;
                    if zone_of[j] == NONE16 && sub::GROWABLE[sim.sub[j] as usize] {
                        zone_of[j] = zone_of[i];
                        q.push_back(j);
                    }
                }
            }
        }
        self.zone_of = zone_of;
        if self.dark.len() != n {
            self.dark = vec![false; n];
            self.dark_shifts = vec![0; n];
            self.stain = vec![0; n];
        }
        self.last_step = sim.tick;
    }

    /// The zone a tile is in.
    pub fn zone_at(&self, sim: &MatSim, x: u16, y: u16) -> Option<usize> {
        let z = *self.zone_of.get(y as usize * sim.w + x as usize)?;
        (z != NONE16).then_some(z as usize)
    }

    /// The zone the Heart stands in: where the Stash's goods and a run's kills count.
    pub fn heart_zone(&self, sim: &MatSim) -> Option<usize> {
        let (x, y) = sim.heart?;
        self.zone_at(sim, x as u16, y as u16)
    }

    /// The gauge's step for a zone: 0 quiet, then one a line passed (Patrol, Sweep, Cleansing).
    pub fn step_of(&self, zone: usize) -> usize {
        let a = self.zones[zone].attention;
        self.rules.stage.iter().filter(|s| a >= s.at).count()
    }

    /// An event of this stage or worse is down in a zone.
    pub fn down_in(&self, zone: usize, stage: usize) -> bool {
        self.events.iter().any(|e| e.zone as usize == zone && e.stage >= stage && e.phase != Phase::Done)
    }

    /// Any Oathkeeper or crawler standing on the map.
    pub fn hostiles(&self) -> usize {
        self.bodies.iter().filter(|b| !b.dead && !b.gone && b.role.hostile()).count()
    }

    fn rng(&self, salt: u64) -> Pcg32 {
        Pcg32::new(self.seed ^ 0x7E2E_A700_0000 ^ ((self.shift as u64) << 20) ^ salt)
    }

    fn body_def(&self, role: Role) -> &BodyDef {
        &self.rules.body[role.key()]
    }

    // -------------------------------------------------------------- the shift

    /// The shift starts: a Flood shift's water rises, the drowned structures stop.
    pub fn start_shift(&mut self, city: &mut City, colony: &mut Colony, defs: &StructureFile, shift: u32, flood_shift: bool) -> Vec<Note> {
        self.shift = shift;
        let mut notes = Vec::new();
        self.hurt.clear();
        self.panicked.clear();
        // every Filter Lung holds the water a tile lower; a Lung Housing stands on one (Full Design v3.18, Organs)
        let lungs = lung_organs(&city.sim) as i32;
        self.flood.roll = flood_shift.then(|| self.roll_for(shift));
        let level = self.flood.roll.map_or(0, |roll| (roll as i32 - lungs).max(0) as u8);
        self.flood.level = level;
        self.reflood(city, colony, defs);
        if flood_shift {
            notes.push(if level > 0 { Note::loss(format!("Flood shift: the water rises {level} tile{}", if level == 1 { "" } else { "s" })) } else { Note::landing("Flood shift: the water holds".into()) });
            // Armoured Pods: the homes do not drown (Full Design v3.18, Research)
            let dry_homes = colony.progress.tech.flag("homes_dont_drown");
            let drowns: Vec<usize> = self.rules.flood.drowns.iter().filter(|d| !(dry_homes && matches!(d.as_str(), "pod" | "bunkhouse"))).filter_map(|d| defs.index(d)).collect();
            for s in colony.structures.iter_mut().filter(|s| s.built && drowns.contains(&s.def)) {
                let d = &defs.structure[s.def];
                let wet = s.tiles(d).iter().any(|&(x, y)| city.sim.sub[y as usize * city.sim.w + x as usize] == sub::WATER);
                if wet {
                    s.drowned = true;
                    notes.push(Note::loss(format!("{} at {},{} drowned", d.name, s.x, s.y)));
                }
            }
        }
        notes
    }

    /// The water as the flood and the sluice have it: restore what the last rise changed, then raise the water joined
    /// to the stratum's lowest water by the level, held back by Gates.
    fn reflood(&mut self, city: &mut City, colony: &Colony, defs: &StructureFile) {
        let sim = &mut city.sim;
        for &(i, was) in &self.flood.changed {
            sim.sub[i as usize] = was;
        }
        self.flood.changed.clear();
        let rise = (self.flood.level + self.flood.sluice) as usize;
        if rise == 0 {
            return;
        }
        let (w, h) = (sim.w, sim.h);
        // the water body: the water joined to the stratum's lowest water
        let mut body = vec![false; w * h];
        let Some(low) = (0..w * h).rev().find(|&i| sim.sub[i] == sub::WATER).map(|i| i / w) else { return };
        let mut q: VecDeque<usize> = (0..w).map(|x| low * w + x).filter(|&i| sim.sub[i] == sub::WATER).collect();
        for &i in &q {
            body[i] = true;
        }
        while let Some(i) = q.pop_front() {
            let (x, y) = (i % w, i / w);
            for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
                if nx < w && ny < h {
                    let j = ny * w + nx;
                    if !body[j] && sim.sub[j] == sub::WATER {
                        body[j] = true;
                        q.push_back(j);
                    }
                }
            }
        }
        let Some(surface) = (0..w * h).filter(|&i| body[i]).map(|i| i / w).min() else { return };
        let line = surface.saturating_sub(rise);
        // Gates and Sphincters hold the water (a Sphincter always does: Full Design v3.18, Organs)
        let gated = |x: usize, y: usize| colony.structures.iter().any(|s| s.built && defs.structure[s.def].shuts && s.covers(&defs.structure[s.def], x as u16, y as u16));
        let mut q: VecDeque<usize> = (0..w * h).filter(|&i| body[i]).collect();
        let mut seen = body;
        while let Some(i) = q.pop_front() {
            let (x, y) = (i % w, i / w);
            for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
                if nx < w && ny < h && ny >= line {
                    let j = ny * w + nx;
                    if !seen[j] && matches!(sim.sub[j], sub::OPEN | sub::PIPE | sub::DEPOSIT | sub::GRATE) && !gated(nx, ny) {
                        seen[j] = true;
                        self.flood.changed.push((j as u32, sim.sub[j]));
                        sim.sub[j] = sub::WATER;
                        q.push_back(j);
                    }
                }
            }
        }
    }

    /// The tiles under water now that were not before the flood.
    /// A Flood shift's roll before the Lungs, for a run on it (the Lungs are the city's); None on another shift.
    pub fn flood_roll(&self) -> Option<u8> {
        self.flood.roll
    }

    /// The roll a shift's Flood has, rolled evenly from the save's seed and the shift (Pass 1: 0-3, by the seed): a
    /// run's overtime that runs into the next shift reads the next one's.
    pub fn roll_for(&self, shift: u32) -> u8 {
        let mut r = Pcg32::new(self.seed ^ 0x7E2E_A700_0000 ^ ((shift as u64) << 20) ^ 1);
        let (lo, hi) = (self.rules.flood.roll[0] as u32, self.rules.flood.roll[1] as u32);
        (lo + r.below(hi - lo + 1)) as u8
    }

    pub fn flooded(&self) -> impl Iterator<Item = usize> + '_ {
        self.flood.changed.iter().map(|c| c.0 as usize)
    }

    /// The shift ends: events on a shift's clock leave, the flood falls back and the drowned pay, the Blooms spawn and
    /// starve, Quarantine severs, the gauge moves, and the next shift's events fall due.
    #[allow(clippy::too_many_arguments)]
    pub fn end_shift(&mut self, city: &mut City, colony: &mut Colony, defs: &StructureFile, shown: &Shown, open_grates: bool, curfew: bool, quarantine: bool) -> Ended {
        let mut out = Ended::default();
        // a Sweep leaves at its shift's end
        for e in self.events.iter_mut().filter(|e| e.phase == Phase::Bodies && self.rules.stage[e.stage].id == "sweep") {
            e.phase = Phase::Leaving;
        }
        for b in self.bodies.iter_mut().filter(|b| !b.dead && !b.gone && b.event != 0) {
            if self.events.iter().any(|e| e.id == b.event && e.phase == Phase::Leaving) {
                b.goal = Goal::Leave;
            }
        }
        // the flood falls back; what drowned loses HP
        let share = if curfew { self.rules.flood.curfew_damage } else { self.rules.flood.damage };
        let mut ruin = Vec::new();
        for s in colony.structures.iter_mut().filter(|s| s.drowned) {
            let max = defs.structure[s.def].hp_at(s.tier) as f32;
            s.hp -= (max * share).round() as i32;
            s.drowned = false;
            out.drowned.push(s.id);
            if s.hp <= 0 {
                ruin.push(s.id);
            }
        }
        for id in ruin {
            if let Some(s) = colony.ruin(defs, id) {
                out.notes.push(Note::loss(format!("{} at {},{} ruined by the water", defs.structure[s.def].name, s.x, s.y)));
            }
        }
        if self.flood.level > 0 && !city.sim.heart_lost {
            self.lived.push("flood".into());
        }
        self.flood.level = 0;
        self.reflood(city, colony, defs);
        // the Blooms: crawlers starve without their Bloom; its Nodes spawn
        let sim = &city.sim;
        let feral: Vec<usize> = (0..sim.hy.len()).filter(|&i| sim.is_bloom(i)).collect();
        if feral.is_empty() {
            let mut starved = 0;
            for b in self.bodies.iter_mut().filter(|b| b.role == Role::Crawler && !b.dead) {
                b.dead = true;
                starved += 1;
            }
            if starved > 0 {
                out.notes.push(Note::landing(format!("{starved} Bloom crawler{} starved with their Bloom", if starved == 1 { "" } else { "s" })));
                if !city.sim.heart_lost {
                    self.lived.push("bloom".into());
                }
            }
        } else if sim.genome.0[mat::genome::FERAL] > self.rules.bloom.feral_drive {
            let mut spawned = 0;
            for &i in feral.iter().filter(|&&i| sim.hy[i] == hy::NODE) {
                let standing = self.bodies.iter().filter(|b| b.role == Role::Crawler && !b.dead && b.home == i).count() as u32;
                for _ in 0..self.rules.bloom.per_node.min(self.rules.bloom.most_per_node.saturating_sub(standing)) {
                    let (x, y) = ((i % sim.w) as u16, (i / sim.w) as u16);
                    let zone = self.zone_at(sim, x, y).unwrap_or(0) as u16;
                    let id = self.spawn(Role::Crawler, x, y, 0, zone, Goal::Hunt);
                    if let Some(b) = self.bodies.iter_mut().find(|b| b.id == id) {
                        b.home = i;
                    }
                    spawned += 1;
                }
            }
            if spawned > 0 {
                out.notes.push(Note::loss(format!("a Bloom spawned {spawned} crawler{}", if spawned == 1 { "" } else { "s" })));
            }
        }
        // the Spore Sacs (Full Design v3.18, Organs): every 4 shifts a crawler from each in a Bloom; in a Bramble strain
        // a tamed sewerat to a Pen within 8 with room, unless its rule holds them
        if self.shift.is_multiple_of(4) {
            if let Some(sac) = progress::StrainFile::shipped().organ_index("sac") {
                let bramble = colony.progress.strain.holds("bramble");
                let sacs: Vec<(usize, u8)> = city.sim.organs_of(sac).map(|o| (o.i as usize, o.rule)).collect();
                let pen_def = defs.index("pen");
                for (i, rule) in sacs {
                    let (x, y) = ((i % city.sim.w) as u16, (i / city.sim.w) as u16);
                    let zone = self.zone_at(&city.sim, x, y).unwrap_or(0) as u16;
                    if city.sim.is_bloom(i) {
                        let id = self.spawn(Role::Crawler, x, y, 0, zone, Goal::Hunt);
                        if let Some(b) = self.bodies.iter_mut().find(|b| b.id == id) {
                            b.home = i;
                        }
                        out.notes.push(Note::loss(format!("a Spore Sac at {x},{y} let a crawler out")));
                    } else if bramble && rule == 0 {
                        let pen = colony.structures.iter().filter(|st| st.built && Some(st.def) == pen_def && cheb((st.x, st.y), (x, y)) <= 8).find(|st| {
                            let cap = defs.structure[st.def].value_at("capacity", st.tier) as usize;
                            self.bodies.iter().filter(|b| b.pen == Some(st.id) && !b.dead).count() < cap
                        }).map(|st| st.id);
                        if let Some(pen) = pen {
                            let id = self.spawn(Role::Tamed, x, y, 0, zone, Goal::Pen(pen));
                            if let Some(b) = self.bodies.iter_mut().find(|b| b.id == id) {
                                b.pen = Some(pen);
                            }
                            out.notes.push(Note::landing(format!("a Spore Sac at {x},{y} tamed a sewerat into the Pen")));
                        }
                    }
                }
            }
        }
        // Quarantine Protocol: Mat with no signal at enough shift ends running is severed
        let sim = &mut city.sim;
        let mut severed = 0;
        for i in 0..sim.hy.len() {
            let dark = matches!(sim.hy[i], hy::TIP | hy::CORD | hy::NODE) && sim.sig[i] <= 0.0;
            self.dark_shifts[i] = if dark { self.dark_shifts[i].saturating_add(1) } else { 0 };
            self.dark[i] = dark;
            if quarantine && dark && self.dark_shifts[i] as u32 >= self.rules.bloom.quarantine_after {
                sim.sever(i % sim.w, i / sim.w);
                severed += 1;
            }
        }
        if severed > 0 {
            out.notes.push(Note::landing(format!("Quarantine Protocol severed {severed} tiles of dark Mat")));
        }
        // the gauge
        let sim = &city.sim;
        let heart_zone = self.heart_zone(sim);
        let garden = colony.progress.strain.holds("garden");
        let bulb_kind = progress::StrainFile::shipped().organ_index("bulb");
        let bulbs: Vec<(usize, u8)> = bulb_kind.map(|k| sim.organs_of(k).map(|o| (o.i as usize, o.rule)).collect()).unwrap_or_default();
        let reliquary = colony.progress.collections.set_flag("attention_decay_plus_1");
        let lume = sim.genome.0[mat::genome::LUME];
        let near = self.rules.attention.near;
        let near_defs: Vec<usize> = self.rules.attention.near_grate.iter().filter_map(|d| defs.index(d)).collect();
        let zone_defs: Vec<usize> = self.rules.attention.in_zone.iter().filter_map(|d| defs.index(d)).collect();
        for k in 0..self.zones.len() {
            let g = (self.zones[k].grate as u16, 0u16);
            let mut gain = 0.0f32;
            for i in 0..sim.hy.len() {
                if sim.hy[i] == hy::NODE && !sim.is_bloom(i) {
                    let t = ((i % sim.w) as u16, (i / sim.w) as u16);
                    if (t.0 as i32 - g.0 as i32).abs() + t.1 as i32 <= near {
                        gain += lume;
                    }
                }
            }
            for s in colony.structures.iter().filter(|s| s.built && !colony.dimmed.contains(&s.id)) {
                let a = defs.structure[s.def].value_at("attention", s.tier) as f32;
                if a <= 0.0 {
                    continue;
                }
                if defs.structure[s.def].id == "lume_farm" && garden_quiet(sim, garden, (s.x as i32, s.y as i32)) {
                    continue;
                }
                let near_it = near_defs.contains(&s.def) && (s.x as i32 - g.0 as i32).abs() + s.y as i32 <= near;
                let in_it = zone_defs.contains(&s.def) && self.zone_at(sim, s.x, s.y) == Some(k);
                if near_it || in_it {
                    gain += a;
                }
            }
            // a Lume Bulb out of a housing: +1 at a grate within reach, unless dark or a Garden's Lung quiets it
            for &(bi, _) in &bulbs {
                let t = ((bi % sim.w) as i32, (bi / sim.w) as i32);
                if colony.dark_bulbs.contains(&(bi as u32)) || colony.occupied(defs, t.0 as u16, t.1 as u16).is_some() || garden_quiet(sim, garden, t) {
                    continue;
                }
                if (t.0 - g.0 as i32).abs() + t.1 <= near {
                    gain += BULB_ATTENTION;
                }
            }
            gain += (self.zones[k].sold / self.rules.attention.per_eddies.max(1)) as f32;
            gain += self.zones[k].kills as f32 * self.rules.attention.kill;
            gain += self.zones[k].blackline as f32 * self.rules.attention.blackline_sold;
            if Some(k) == heart_zone {
                gain += shown.exposed as f32 * self.rules.attention.exposed + shown.run_kills as f32 * self.rules.attention.kill;
            }
            let reached = city.markets.contains(&self.zones[k].grate);
            if open_grates && reached {
                gain *= self.rules.attention.open_grates;
            }
            let z = &mut self.zones[k];
            z.gained = gain;
            let decay = self.rules.attention.decay + if reliquary { 1.0 } else { 0.0 };
            z.attention = (z.attention + gain - decay).clamp(0.0, self.rules.attention.max);
            z.sold = 0;
            z.kills = 0;
            z.blackline = 0;
        }
        // a Lume Bulb's rule (Pass 2): dark while its zone's Attention is over its line; a Lume Farm on it with it
        colony.dark_bulbs.clear();
        colony.dimmed.clear();
        for &(bi, rule) in &bulbs {
            let (x, y) = ((bi % sim.w) as u16, (bi / sim.w) as u16);
            let over = self.zone_at(sim, x, y).is_some_and(|k| self.zones[k].attention > bulb_line(rule));
            if over {
                colony.dark_bulbs.insert(bi as u32);
                if let Some(sid) = colony.occupied(defs, x, y) {
                    colony.dimmed.insert(sid);
                }
            }
        }
        // the next shift's events: the highest stage due in each zone, outside its quiet spells
        self.due.clear();
        let next = self.shift + 1;
        for k in 0..self.zones.len() {
            let a = self.zones[k].attention;
            let mut pick = None;
            for st in (0..self.rules.stage.len()).rev() {
                if a < self.rules.stage[st].at || self.rules.stage[st].from > self.stratum {
                    continue;
                }
                let quiet = (st..self.rules.stage.len()).any(|s| self.zones[k].last[s].is_some_and(|d| next.saturating_sub(d) < self.rules.stage[s].quiet));
                if !quiet && !self.down_in(k, 0) {
                    pick = Some(st);
                }
                break;
            }
            if let Some(st) = pick {
                let mut r = self.rng(100 + k as u64);
                let in_ticks = r.below(self.rules.stage[st].arrive_in.max(1));
                self.due.push(Due { stage: st, zone: k as u16, in_ticks });
                out.notes.push(Note { loss: false, decision: true, what: format!("grate {} is loud enough for a {}", k + 1, self.rules.stage[st].name) });
            }
        }
        // salvage owed; what fell, was repelled and lived through
        out.blackline = std::mem::take(&mut self.owed.0);
        out.relics = std::mem::take(&mut self.owed.1);
        out.killed = std::mem::take(&mut self.killed);
        out.repelled = std::mem::take(&mut self.repelled);
        out.lived = std::mem::take(&mut self.lived);
        self.bodies.retain(|b| !(b.dead || b.gone));
        self.events.retain(|e| e.phase != Phase::Done);
        out
    }

    fn spawn(&mut self, role: Role, x: u16, y: u16, event: u32, zone: u16, goal: Goal) -> u32 {
        let d = self.body_def(role);
        let hp = d.hp as f32;
        let id = self.next_id;
        self.next_id += 1;
        self.bodies.push(Body { id, role, x, y, hp, max_hp: hp, event, zone, goal, foe: None, cd: 0.0, step: 0, mapping: 0.0, home: 0, pen: None, dead: false, gone: false });
        id
    }

    /// Send a stage down a zone's grate now (the scheduler, and the proof's forced Cleansing).
    pub fn send(&mut self, city: &City, colony: &Colony, defs: &StructureFile, stage: usize, zone: usize) -> Vec<Note> {
        let st = self.rules.stage[stage].clone();
        let id = self.next_id;
        self.next_id += 1;
        self.zones[zone].last[stage] = Some(self.shift);
        let lungs = colony.structures.iter().filter(|s| s.built && defs.structure[s.def].id == "lung").count() as f32;
        let flood_left = (st.flood_steps as f32 * (1.0 - st.lung_cut * lungs).max(0.0)).round() as u32;
        let mut e = Event { id, stage, zone: zone as u16, started: city.tick, phase: Phase::Bodies, flood_left, sent: 0, fallen: 0, flamers_out: false, way: 0, burned: 0, ruined: 0 };
        let mut notes = vec![Note::loss(format!("a {} came down grate {}", st.name, zone + 1))];
        if flood_left > 0 {
            e.phase = Phase::Flood;
            notes.push(Note::loss(format!("the Oathkeepers opened the sluice at grate {}: bleach in the water for {flood_left} Mat steps", zone + 1)));
        } else {
            e.sent = self.send_squad(city, colony, defs, &st, id, zone, false);
        }
        self.events.push(e);
        notes
    }

    #[allow(clippy::too_many_arguments)]
    fn send_squad(&mut self, city: &City, colony: &Colony, defs: &StructureFile, st: &StageDef, event: u32, zone: usize, flamers: bool) -> u32 {
        let g = (self.zones[zone].grate as u16, 0u16);
        let source = self.brightest(&city.sim, colony, defs, zone);
        let mut n = 0;
        for (role, count) in &st.squad {
            let role = Role::from_key(role);
            if (role == Role::Flamer) != flamers {
                continue;
            }
            for _ in 0..*count {
                let goal = match (st.id.as_str(), role) {
                    ("cleansing", Role::Flamer) => self.burn_target(colony, defs).map(Goal::Burn).unwrap_or(Goal::Heart),
                    ("cleansing", _) => Goal::Heart,
                    _ => Goal::Map(source.0, source.1),
                };
                let id = self.spawn(role, g.0, g.1, event, zone as u16, goal);
                if let Some(b) = self.bodies.iter_mut().find(|b| b.id == id) {
                    b.mapping = st.map_s;
                }
                n += 1;
            }
        }
        n
    }

    /// The zone's brightest source (Pass 1: squads path toward the brightest Attention source): the tile with the
    /// most Attention weight within 3 tiles — Lume Nodes, Lume Farms, Stills — else the Heart.
    pub fn brightest(&self, sim: &MatSim, colony: &Colony, defs: &StructureFile, zone: usize) -> (u16, u16) {
        let lume = sim.genome.0[mat::genome::LUME];
        let mut src: Vec<((u16, u16), f32)> = Vec::new();
        for i in 0..sim.hy.len() {
            if sim.hy[i] == hy::NODE && !sim.is_bloom(i) {
                let t = ((i % sim.w) as u16, (i / sim.w) as u16);
                if self.zone_at(sim, t.0, t.1) == Some(zone) {
                    src.push((t, lume));
                }
            }
        }
        for st in colony.structures.iter().filter(|st| st.built) {
            let a = defs.structure[st.def].value_at("attention", st.tier) as f32;
            if a > 0.0 && self.zone_at(sim, st.x, st.y) == Some(zone) {
                src.push(((st.x, st.y), a));
            }
        }
        let heart = sim.heart.map(|(x, y)| (x as u16, y as u16)).unwrap_or((0, 0));
        let mut best: Option<(f32, (u16, u16))> = None;
        for &(t, _) in &src {
            let w: f32 = src.iter().filter(|(u, _)| cheb(t, *u) <= 3).map(|(_, w)| w).sum();
            if best.is_none_or(|b| w > b.0 + 1e-4 || ((w - b.0).abs() <= 1e-4 && (t.1, t.0) < (b.1 .1, b.1 .0))) {
                best = Some((w, t));
            }
        }
        best.map(|b| b.1).unwrap_or(heart)
    }

    fn burn_target(&self, colony: &Colony, defs: &StructureFile) -> Option<u32> {
        let pits = colony.pits(defs);
        // the Stills first (Pass 1: the Downer chain is "the Oathkeepers' first target"; Full Design v3.15, Economy),
        // then the Pits (pits[0] is the Heart), then the Heart
        colony.stills(defs).first().copied().or_else(|| pits.iter().skip(1).copied().next()).or_else(|| pits.first().copied())
    }
}

// ------------------------------------------------------------------ the city's clock

/// The tiles a structure stands on.
fn tiles_of(colony: &Colony, defs: &StructureFile, id: u32) -> Vec<(u16, u16)> {
    colony.get(id).map(|st| st.tiles(&defs.structure[st.def])).unwrap_or_default()
}

impl Threats {
    /// Distance in steps to `goal` over the ground bodies walk; through Gates when `through`.
    fn field(&mut self, sim: &MatSim, gates: &[usize], goal: usize, through: bool) -> &Vec<u16> {
        if sim.tick != self.fields_at {
            self.fields.clear();
            self.fields_at = sim.tick;
        }
        self.fields.entry((goal, through)).or_insert_with(|| {
            let (w, h) = (sim.w, sim.h);
            let mut d = vec![NONE16; w * h];
            d[goal] = 0;
            let mut q = VecDeque::from([goal]);
            while let Some(i) = q.pop_front() {
                let (x, y) = (i % w, i / w);
                for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
                    if nx < w && ny < h {
                        let j = ny * w + nx;
                        if d[j] == NONE16 && walkable(sim, j) && (through || !gates.contains(&j)) {
                            d[j] = d[i] + 1;
                            q.push_back(j);
                        }
                    }
                }
            }
            d
        })
    }

    /// One city tick: the due events come down, the bodies walk and fight, the city's defence answers, and on a Mat
    /// step the white light, the bleach, the flames and the Mat's Toxicity land.
    pub fn tick(&mut self, city: &mut City, colony: &mut Colony, defs: &StructureFile, posts: &Posts, conscription: bool) -> Vec<Note> {
        let mut notes = Vec::new();
        // the due events
        let mut send = Vec::new();
        for d in self.due.iter_mut() {
            if d.in_ticks == 0 {
                send.push((d.stage, d.zone as usize));
            } else {
                d.in_ticks -= 1;
            }
        }
        self.due.retain(|d| d.in_ticks > 0 || !send.iter().any(|s| s.1 == d.zone as usize));
        for (st, z) in send {
            notes.extend(self.send(city, colony, defs, st, z));
        }
        if self.bodies.is_empty() && self.events.is_empty() && self.stain.iter().all(|s| *s == 0) {
            self.last_step = city.sim.tick;
            return notes;
        }
        let gate_def = defs.index("gate");
        // what shuts: the Sphincter Gates always, a Sphincter while white light is within its rule's reach
        let lamps: Vec<(u16, u16)> = self.bodies.iter().filter(|b| !b.dead && !b.gone && self.body_def(b.role).lamp > 0).map(|b| (b.x, b.y)).collect();
        let shut = |st: &colony::Structure| -> bool {
            let d = &defs.structure[st.def];
            if !d.shuts {
                return false;
            }
            if d.category != "organ" {
                return true;
            }
            let i = st.y as usize * city.sim.w + st.x as usize;
            let rule = city.sim.organ_at(i).map_or(0, |o| o.rule);
            match rule {
                3 => true,
                4 => false,
                r => {
                    let reach = [8i32, 4, 12][r as usize];
                    lamps.iter().any(|&l| cheb(l, (st.x, st.y)) <= reach)
                }
            }
        };
        let gates: Vec<usize> = colony.structures.iter().filter(|st| st.built && shut(st)).flat_map(|st| st.tiles(&defs.structure[st.def])).map(|(x, y)| y as usize * city.sim.w + x as usize).collect();
        let w = city.sim.w;
        let heart = city.sim.heart.map(|(x, y)| (x as u16, y as u16)).unwrap_or((0, 0));
        let slimos: Vec<(u32, (u16, u16))> = city.ids.iter().zip(city.pos.iter()).map(|(&id, &p)| (id, ((p as usize % w) as u16, (p as usize / w) as u16))).collect();
        let dt = 1.0 / HZ;
        // ---- the bodies: goals, steps
        for k in 0..self.bodies.len() {
            if self.bodies[k].dead || self.bodies[k].gone {
                continue;
            }
            let b = self.bodies[k].clone();
            let def = self.body_def(b.role).clone();
            let pos = (b.x, b.y);
            // where it is going
            let goal_tile: Option<(u16, u16)> = match b.goal {
                Goal::Map(x, y) => {
                    if cheb(pos, (x, y)) <= 1 {
                        let left = b.mapping - dt;
                        self.bodies[k].mapping = left;
                        if left <= 0.0 {
                            let sweep = self.events.iter().any(|e| e.id == b.event && self.rules.stage[e.stage].id == "sweep");
                            self.bodies[k].goal = if sweep { Goal::Heart } else { Goal::Leave };
                        }
                        None
                    } else {
                        Some((x, y))
                    }
                }
                Goal::Heart => {
                    if cheb(pos, heart) > 2 {
                        Some(heart)
                    } else {
                        // a Sweep's path ends at the Heart: it turns for its grate
                        if self.events.iter().any(|e| e.id == b.event && self.rules.stage[e.stage].id == "sweep") {
                            self.bodies[k].goal = Goal::Leave;
                        }
                        None
                    }
                }
                Goal::Burn(sid) => {
                    let tiles = tiles_of(colony, defs, sid);
                    if tiles.is_empty() {
                        self.bodies[k].goal = self.burn_target(colony, defs).map(Goal::Burn).unwrap_or(Goal::Leave);
                        None
                    } else if tiles.iter().any(|&t| cheb(pos, t) <= 1) {
                        None
                    } else {
                        Some(tiles[0])
                    }
                }
                Goal::Leave => {
                    let g = (self.zones[b.zone as usize].grate as u16, 0);
                    if pos == g {
                        self.bodies[k].gone = true;
                        None
                    } else {
                        Some(g)
                    }
                }
                Goal::Hunt => {
                    let prey = slimos.iter().filter(|(_, p)| cheb(pos, *p) <= 12).min_by_key(|(id, p)| (dist2(pos, *p), *id)).map(|(_, p)| *p);
                    let prey = prey.or_else(|| colony.structures.iter().filter(|st| st.built).map(|st| (st.x, st.y)).min_by_key(|&t| (dist2(pos, t), t.1, t.0)));
                    prey.filter(|&p| cheb(pos, p) > 1)
                }
                Goal::Pen(pid) => {
                    let pen = colony.get(pid).map(|st| (st.x, st.y));
                    match pen {
                        None => {
                            self.bodies[k].dead = true;
                            None
                        }
                        Some(pt) => {
                            let near = self.bodies.iter().filter(|o| !o.dead && !o.gone && o.role.oathkeeper() && cheb((o.x, o.y), pt) <= self.rules.defence.tamed_within).min_by_key(|o| (dist2(pos, (o.x, o.y)), o.id)).map(|o| (o.x, o.y));
                            match near {
                                Some(o) if cheb(pos, o) > 1 => Some(o),
                                Some(_) => None,
                                None => (cheb(pos, pt) > 1).then_some(pt),
                            }
                        }
                    }
                }
            };
            if self.bodies[k].dead || self.bodies[k].gone {
                continue;
            }
            // a step every `walk` ticks; a Cleansing's lamp carriers, and its flamers behind them, a tile a Mat step
            let crawl = matches!(b.role, Role::Carrier | Role::Flamer) && self.events.iter().any(|e| e.id == b.event && self.rules.stage[e.stage].id == "cleansing");
            let walk = if crawl { city.mat_every as u8 } else { def.walk.max(1) };
            if self.bodies[k].step > 0 {
                self.bodies[k].step -= 1;
                continue;
            }
            let Some(gt) = goal_tile else { continue };
            let goal = gt.1 as usize * w + gt.0 as usize;
            let here = pos.1 as usize * w + pos.0 as usize;
            let mut through = false;
            if self.field(&city.sim, &gates, goal, false)[here] == NONE16 {
                through = true;
            }
            let f = self.field(&city.sim, &gates, goal, through);
            let cur = f[here];
            let mut next = None;
            for (nx, ny) in [(pos.0.wrapping_sub(1), pos.1), (pos.0 + 1, pos.1), (pos.0, pos.1.wrapping_sub(1)), (pos.0, pos.1 + 1)] {
                if (nx as usize) < w && (ny as usize) < city.sim.h {
                    let j = ny as usize * w + nx as usize;
                    if f[j] != NONE16 && f[j] < cur && next.is_none_or(|(v, _)| f[j] < v) {
                        next = Some((f[j], (nx, ny)));
                    }
                }
            }
            let Some((_, nt)) = next else { continue };
            let ni = nt.1 as usize * w + nt.0 as usize;
            if gates.contains(&ni) {
                // the only way is through a Gate, and a Gate is muscle: a flamer burns it, the rest wait at it
                if def.flame > 0.0 {
                    if let Some(gid) = colony.occupied(defs, nt.0, nt.1) {
                        if let Some(g) = colony.get_mut(gid) {
                            g.hp -= (def.flame * dt).ceil() as i32;
                        }
                    }
                }
                continue;
            }
            self.bodies[k].x = nt.0;
            self.bodies[k].y = nt.1;
            self.bodies[k].step = walk - 1;
        }
        // ---- the bodies' hands: shots, flames, bites
        let mut struct_dmg: BTreeMap<u32, f32> = BTreeMap::new();
        let mut body_dmg: BTreeMap<u32, (f32, Option<Foe>)> = BTreeMap::new();
        for k in 0..self.bodies.len() {
            let b = self.bodies[k].clone();
            if b.dead || b.gone {
                continue;
            }
            let def = self.body_def(b.role).clone();
            let pos = (b.x, b.y);
            self.bodies[k].cd = (b.cd - dt).max(0.0);
            match b.role {
                Role::Flamer => {
                    if let Goal::Burn(sid) = b.goal {
                        if tiles_of(colony, defs, sid).iter().any(|&t| cheb(pos, t) <= 1) {
                            *struct_dmg.entry(sid).or_default() += def.flame * dt;
                        }
                    }
                    for (id, p) in slimos.iter().filter(|(_, p)| cheb(pos, *p) <= 1) {
                        let _ = p;
                        *self.hurt.entry(*id).or_default() += def.flame * dt;
                    }
                }
                Role::Crawler => {
                    if self.bodies[k].cd <= 0.0 {
                        if let Some((id, _)) = slimos.iter().find(|(_, p)| cheb(pos, *p) <= 1) {
                            *self.hurt.entry(*id).or_default() += def.bite as f32;
                            self.bodies[k].cd = def.bite_s;
                        }
                    }
                    if let Some(sid) = colony.structures.iter().filter(|st| st.built).find(|st| st.tiles(&defs.structure[st.def]).iter().any(|&t| cheb(pos, t) <= 1)).map(|st| st.id) {
                        *struct_dmg.entry(sid).or_default() += def.chew * dt;
                    }
                }
                Role::Tamed => {
                    if self.bodies[k].cd <= 0.0 {
                        if let Some(o) = self.bodies.iter().find(|o| !o.dead && !o.gone && o.role.oathkeeper() && cheb(pos, (o.x, o.y)) <= 1) {
                            let e = body_dmg.entry(o.id).or_insert((0.0, None));
                            e.0 += self.rules.defence.tamed_bite as f32;
                            e.1 = Some(Foe::Body(b.id));
                            self.bodies[k].cd = self.rules.defence.tamed_bite_s;
                        }
                    }
                }
                Role::Carrier | Role::Sprayer => {
                    // a pole for whoever stands beside it: the city's slimos, a tamed crawler
                    if def.strike > 0 && self.bodies[k].cd <= 0.0 {
                        if let Some((id, _)) = slimos.iter().find(|(_, p)| cheb(pos, *p) <= 1) {
                            *self.hurt.entry(*id).or_default() += def.strike as f32;
                            self.bodies[k].cd = def.strike_s;
                        } else if let Some(t) = self.bodies.iter().find(|o| !o.dead && !o.gone && o.role == Role::Tamed && cheb(pos, (o.x, o.y)) <= 1) {
                            body_dmg.entry(t.id).or_insert((0.0, None)).0 += def.strike as f32;
                            self.bodies[k].cd = def.strike_s;
                        }
                    }
                }
                Role::Scout => {
                    // fired on, a scout shoots back at what hit it
                    let Some(foe) = b.foe else { continue };
                    let at: Option<(u16, u16)> = match foe {
                        Foe::Structure(sid) => tiles_of(colony, defs, sid).into_iter().min_by_key(|&t| dist2(pos, t)),
                        Foe::Slimo(id) => slimos.iter().find(|(s, _)| *s == id).map(|(_, p)| *p),
                        Foe::Body(id) => self.bodies.iter().find(|o| o.id == id && !o.dead && !o.gone).map(|o| (o.x, o.y)),
                    };
                    let Some(at) = at else {
                        self.bodies[k].foe = None;
                        continue;
                    };
                    if def.shot > 0 && self.bodies[k].cd <= 0.0 && dist2(pos, at) <= def.range * def.range {
                        self.bodies[k].cd = def.shot_s;
                        match foe {
                            Foe::Structure(sid) => *struct_dmg.entry(sid).or_default() += def.shot as f32,
                            Foe::Slimo(id) => *self.hurt.entry(id).or_default() += def.shot as f32,
                            Foe::Body(id) => body_dmg.entry(id).or_insert((0.0, None)).0 += def.shot as f32,
                        }
                    }
                }
            }
        }
        // ---- the city's defence: Turrets, Guards on Gates, the militia
        // Scrap Turrets with a Guard in the seat; the Maws and Venom Glands by themselves (Full Design v3.18, Organs);
        // Blackline arms strike 2 harder
        let blackline = colony.progress.tech.flag("strike_plus_2");
        let tox = city.sim.genome.0[mat::genome::TOXICITY];
        for st in colony.structures.iter().filter(|st| st.built && defs.structure[st.def].value_at("damage", st.tier) > 0) {
            let d = &defs.structure[st.def];
            let seated = d.hosts == "guard";
            if seated && !posts.guards.iter().any(|g| g.1 == st.id) {
                continue;
            }
            let cd = self.cds.entry((st.id, 0)).or_insert(0.0);
            *cd = (*cd - dt).max(0.0);
            if *cd > 0.0 {
                continue;
            }
            let r = d.value_at("range", st.tier);
            let mut dmg = d.value_at("damage", st.tier) as f32;
            if d.id == "gland" {
                dmg = 2.0 + 4.0 * tox;
            }
            if seated && blackline {
                dmg += 2.0;
            }
            let oath_only = d.oath_only;
            if let Some(t) = self.bodies.iter().filter(|o| !o.dead && !o.gone && o.role.hostile() && (!oath_only || o.role.oathkeeper()) && dist2((st.x, st.y), (o.x, o.y)) <= r * r).min_by_key(|o| (dist2((st.x, st.y), (o.x, o.y)), o.id)) {
                let e = body_dmg.entry(t.id).or_insert((0.0, None));
                e.0 += dmg;
                e.1 = Some(Foe::Structure(st.id));
                *cd = self.rules.defence.turret_s;
            }
        }
        let fight = |id: u32| posts.fight.get(&id).copied().unwrap_or(0);
        let mut strikers: Vec<(u32, (u16, u16))> = Vec::new();
        for st in colony.structures.iter().filter(|st| st.built && Some(st.def) == gate_def) {
            if let Some(g) = posts.guards.iter().find(|g| g.1 == st.id) {
                strikers.push((g.0, (st.x, st.y)));
            }
        }
        if conscription && self.events.iter().any(|e| e.phase != Phase::Done) {
            for (id, p) in &slimos {
                if posts.d2.contains(id) && !strikers.iter().any(|s| s.0 == *id) {
                    strikers.push((*id, *p));
                }
            }
        }
        for (id, at) in strikers {
            let cd = self.cds.entry((id, 1)).or_insert(0.0);
            *cd = (*cd - dt).max(0.0);
            if *cd > 0.0 {
                continue;
            }
            if let Some(t) = self.bodies.iter().filter(|o| !o.dead && !o.gone && o.role.hostile() && cheb(at, (o.x, o.y)) <= 1).min_by_key(|o| o.id) {
                let e = body_dmg.entry(t.id).or_insert((0.0, None));
                e.0 += (self.rules.defence.guard_strike + fight(id) + if blackline { 2 } else { 0 }) as f32;
                e.1 = Some(Foe::Slimo(id));
                *cd = self.rules.defence.guard_s;
            }
        }
        // ---- a World-Eater strain digests the Oathkeepers standing on it, half a point a tick (Pass 2; v3.18)
        if colony.progress.strain.holds("world_eater") {
            for o in self.bodies.iter().filter(|o| !o.dead && !o.gone && o.role.oathkeeper()) {
                let i = o.y as usize * w + o.x as usize;
                if city.sim.is_hyphal_at(i) && !city.sim.is_feral(i) {
                    body_dmg.entry(o.id).or_insert((0.0, None)).0 += 0.5;
                }
            }
        }
        // ---- a Mat step: white light, bleach, flame underfoot, Toxicity; the Cleansing's sluice
        let stepped = city.sim.tick != self.last_step;
        self.last_step = city.sim.tick;
        if stepped {
            for v in self.stain.iter_mut() {
                *v = v.saturating_sub(1);
            }
            let tox = self.rules.defence.toxicity * city.sim.genome.0[mat::genome::TOXICITY];
            let mut burned_by: BTreeMap<u32, u32> = BTreeMap::new();
            // the white light: a tile in any pool takes the worst lamp on it, once
            let mut lit: BTreeMap<usize, (f32, u32)> = BTreeMap::new();
            for k in 0..self.bodies.len() {
                let b = self.bodies[k].clone();
                if b.dead || b.gone || b.role == Role::Tamed {
                    continue;
                }
                let def = self.body_def(b.role).clone();
                let here = b.y as usize * w + b.x as usize;
                if def.lamp > 0 {
                    let r = def.lamp;
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy > r * r {
                                continue;
                            }
                            let (x, y) = (b.x as i32 + dx, b.y as i32 + dy);
                            if x < 0 || y < 0 || x >= w as i32 || y >= city.sim.h as i32 {
                                continue;
                            }
                            let i = y as usize * w + x as usize;
                            let e = lit.entry(i).or_insert((0.0, b.event));
                            if def.lamp_burn > e.0 {
                                *e = (def.lamp_burn, b.event);
                            }
                        }
                    }
                    for (id, p) in &slimos {
                        if dist2((b.x, b.y), *p) <= r * r && self.panicked.insert(*id) {
                            if let Some(m) = city.ids.iter().position(|x| x == id) {
                                city.goal[m] = city.home[m];
                                city.state[m] = logistics::city::state::TO_HOME;
                            }
                        }
                    }
                }
                if def.bleach > 0 {
                    // along tissue and water, 3 tiles, twice as far over water, never across salt
                    let mut cost: HashMap<usize, f32> = HashMap::new();
                    let mut q = VecDeque::from([(here, 0.0f32)]);
                    cost.insert(here, 0.0);
                    while let Some((i, c)) = q.pop_front() {
                        city.sim.bleach(i);
                        self.stain[i] = 8;
                        let (x, y) = (i % w, i / w);
                        for (nx, ny) in [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)] {
                            if nx < w && ny < city.sim.h {
                                let j = ny * w + nx;
                                let water = city.sim.sub[j] == sub::WATER;
                                if city.sim.salt[j] || !(water || city.sim.is_hyphal_at(j)) {
                                    continue;
                                }
                                let nc = c + if water { 0.5 } else { 1.0 };
                                if nc <= def.bleach as f32 && cost.get(&j).is_none_or(|&o| nc < o) {
                                    cost.insert(j, nc);
                                    q.push_back((j, nc));
                                }
                            }
                        }
                    }
                    if city.sim.burn(here, 1.0) {
                        *burned_by.entry(b.event).or_default() += 1;
                    }
                }
                if def.burn > 0.0 && city.sim.burn(here, def.burn) {
                    *burned_by.entry(b.event).or_default() += 1;
                }
                if b.role.oathkeeper() && city.sim.is_hyphal_at(here) && !city.sim.is_bloom(here) && tox > 0.0 {
                    body_dmg.entry(b.id).or_insert((0.0, None)).0 += tox;
                }
            }
            for (i, (dmg, ev)) in lit {
                if city.sim.burn(i, dmg) {
                    *burned_by.entry(ev).or_default() += 1;
                }
            }
            for (ev, n) in burned_by {
                if let Some(e) = self.events.iter_mut().find(|e| e.id == ev) {
                    e.burned += n;
                }
            }
            // the sluice
            let mut sluice_end = Vec::new();
            for e in self.events.iter_mut().filter(|e| e.phase == Phase::Flood) {
                if e.flood_left > 0 {
                    e.flood_left -= 1;
                }
                if e.flood_left == 0 {
                    sluice_end.push(e.id);
                }
            }
            let sluicing = self.events.iter().any(|e| e.phase == Phase::Flood);
            let want = u8::from(sluicing && sluice_end.len() < self.events.iter().filter(|e| e.phase == Phase::Flood).count());
            if want != self.flood.sluice {
                self.flood.sluice = want;
                self.reflood(city, colony, defs);
            }
            if self.flood.sluice > 0 {
                for i in 0..city.sim.sub.len() {
                    if city.sim.sub[i] == sub::WATER {
                        city.sim.bleach(i);
                        self.stain[i] = 8;
                    }
                }
            }
            for id in sluice_end {
                let (st, zone) = {
                    let e = self.events.iter().find(|e| e.id == id).unwrap();
                    (self.rules.stage[e.stage].clone(), e.zone as usize)
                };
                let n = self.send_squad(city, colony, defs, &st, id, zone, false);
                let g = self.zones[zone].grate;
                let way = self.field(&city.sim, &gates, city.sim.heart.map(|(x, y)| y * w + x).unwrap_or(0), true)[g];
                if let Some(e) = self.events.iter_mut().find(|e| e.id == id) {
                    e.phase = Phase::Bodies;
                    e.sent += n;
                    e.way = if way == NONE16 { 1 } else { way as u32 };
                }
                notes.push(Note::loss(format!("the white light comes down grate {}", zone + 1)));
            }
        }
        // ---- what landed: bodies, structures
        for (id, (dmg, foe)) in body_dmg {
            if let Some(b) = self.bodies.iter_mut().find(|b| b.id == id && !b.dead) {
                b.hp -= dmg;
                if foe.is_some() && b.role.oathkeeper() {
                    b.foe = foe;
                }
            }
        }
        for (sid, dmg) in struct_dmg {
            if let Some(st) = colony.get_mut(sid) {
                st.hp -= dmg.ceil() as i32;
            }
        }
        // the dead
        for k in 0..self.bodies.len() {
            if self.bodies[k].dead || self.bodies[k].gone || self.bodies[k].hp > 0.0 {
                continue;
            }
            self.bodies[k].dead = true;
            let b = self.bodies[k].clone();
            if b.role.hostile() {
                self.killed.push(b.role.key().to_string());
            }
            if b.role.oathkeeper() {
                let z = self.zone_at(&city.sim, b.x, b.y).unwrap_or(b.zone as usize);
                self.zones[z].kills += 1;
                if let Some(e) = self.events.iter_mut().find(|e| e.id == b.event) {
                    e.fallen += 1;
                    let st = &self.rules.stage[e.stage];
                    if st.salvage_per == "body" {
                        let mut r = Pcg32::new(self.seed ^ ((b.id as u64) << 8) ^ 0xB1AC);
                        self.owed.0 += st.salvage[0] + r.below(st.salvage[1] - st.salvage[0] + 1);
                    }
                }
                notes.push(Note::landing(format!("the {} fell at {},{}", self.body_def(b.role).name, b.x, b.y)));
            } else {
                notes.push(Note::landing(format!("a {} fell at {},{}", self.body_def(b.role).name, b.x, b.y)));
            }
        }
        // ruined structures
        let ruined: Vec<u32> = colony.structures.iter().filter(|st| st.built && st.hp <= 0).map(|st| st.id).collect();
        for sid in ruined {
            if let Some(st) = colony.ruin(defs, sid) {
                let name = defs.structure[st.def].name.clone();
                if defs.structure[st.def].id == "heart" {
                    city.sim.heart_lost = true;
                    notes.push(Note::loss("the Heart is burned: the city has lost its signal".into()));
                } else {
                    notes.push(Note::loss(format!("{name} at {},{} ruined", st.x, st.y)));
                }
                for e in self.events.iter_mut().filter(|e| e.phase != Phase::Done) {
                    e.ruined += 1;
                }
            }
        }
        // taming
        let pen_def = defs.index("pen");
        for st in colony.structures.iter().filter(|st| st.built && Some(st.def) == pen_def) {
            if !posts.tenders.iter().any(|t| t.1 == st.id) {
                continue;
            }
            let cap = defs.structure[st.def].value_at("capacity", st.tier) as usize;
            let have = self.bodies.iter().filter(|b| b.pen == Some(st.id) && !b.dead).count();
            let near = self.rules.defence.tame_within;
            for b in self.bodies.iter_mut().filter(|b| b.role == Role::Crawler && !b.dead && cheb((b.x, b.y), (st.x, st.y)) <= near).take(cap.saturating_sub(have)) {
                b.role = Role::Tamed;
                b.pen = Some(st.id);
                b.goal = Goal::Pen(st.id);
                b.hp = b.max_hp;
                notes.push(Note::landing(format!("the Pen at {},{} tamed a crawler", st.x, st.y)));
            }
        }
        // the events' turns
        let now = city.tick;
        let heart_cell = city.sim.heart.map(|(x, y)| y * w + x).unwrap_or(0);
        let ids: Vec<u32> = self.events.iter().map(|e| e.id).collect();
        for id in ids {
            let (stage, zone, sent, fallen, phase, flamers_out, way, started) = {
                let e = self.events.iter().find(|e| e.id == id).unwrap();
                (e.stage, e.zone as usize, e.sent, e.fallen, e.phase, e.flamers_out, e.way, e.started)
            };
            let st = self.rules.stage[stage].clone();
            if phase == Phase::Flood || phase == Phase::Done {
                continue;
            }
            let mine: Vec<usize> = (0..self.bodies.len()).filter(|&k| self.bodies[k].event == id).collect();
            let standing = mine.iter().filter(|&&k| !self.bodies[k].dead && !self.bodies[k].gone).count();
            // a Cleansing's flamers set out when its lamp carriers are far enough along
            if st.id == "cleansing" && !flamers_out && phase == Phase::Bodies {
                let carriers: Vec<usize> = mine.iter().copied().filter(|&k| self.bodies[k].role == Role::Carrier && !self.bodies[k].dead).collect();
                let f = self.field(&city.sim, &gates, heart_cell, true).clone();
                let along = carriers.iter().map(|&k| way.saturating_sub(f[self.bodies[k].y as usize * w + self.bodies[k].x as usize].min(way as u16) as u32)).max().unwrap_or(way);
                // halfway, or out as long as halfway would take them (a Gate holds them)
                let out_steps = (now.saturating_sub(started) / city.mat_every.max(1)) as f32;
                if along as f32 >= st.flamers_at * way as f32 || out_steps >= st.flamers_at * way as f32 + st.flood_steps as f32 || carriers.is_empty() {
                    let n = self.send_squad(city, colony, defs, &st, id, zone, true);
                    if let Some(e) = self.events.iter_mut().find(|e| e.id == id) {
                        e.flamers_out = true;
                        e.sent += n;
                    }
                    notes.push(Note::loss(format!("flamers come down grate {} for the Pits and the Heart", zone + 1)));
                    continue;
                }
            }
            let leave = (st.break_at > 0.0 && fallen as f32 >= st.break_at * sent as f32) || (st.lasts > 0 && now.saturating_sub(started) >= st.lasts);
            if leave && phase == Phase::Bodies {
                for &k in &mine {
                    self.bodies[k].goal = Goal::Leave;
                }
                if let Some(e) = self.events.iter_mut().find(|e| e.id == id) {
                    e.phase = Phase::Leaving;
                }
            }
            if standing == 0 && (st.id != "cleansing" || flamers_out) {
                // repelled: it left past its break line with the Heart standing (Full Design v3.18, Authority)
                if st.break_at > 0.0 && fallen as f32 >= st.break_at * sent as f32 && !city.sim.heart_lost {
                    self.repelled.push(st.id.clone());
                }
                if matches!(st.id.as_str(), "sweep" | "cleansing") && !city.sim.heart_lost {
                    self.lived.push(st.id.clone());
                }
                let e = self.events.iter_mut().find(|e| e.id == id).unwrap();
                e.phase = Phase::Done;
                if st.salvage_per == "event" && sent > 0 && fallen > 0 {
                    let mut r = Pcg32::new(self.seed ^ ((id as u64) << 12) ^ 0x5A17);
                    let roll = st.salvage[0] + r.below(st.salvage[1] - st.salvage[0] + 1);
                    let relic = st.relics[0] + r.below(st.relics[1] - st.relics[0] + 1);
                    self.owed.0 += (roll as f32 * fallen as f32 / sent as f32).round() as u32;
                    self.owed.1 += (relic as f32 * fallen as f32 / sent as f32).round() as u32;
                }
                notes.push(Note { loss: false, decision: false, what: format!("the {} at grate {} is over: {fallen} of {sent} fell, {} tiles of Mat burned, {} structures ruined", st.name, zone + 1, e.burned, e.ruined) });
            }
        }
        notes
    }

    /// Feral tissue within `r` tiles of a tile: a Bloom borders it.
    pub fn bloom_near(&self, sim: &MatSim, x: u16, y: u16) -> bool {
        let r = self.rules.bloom.borders;
        for dy in -r..=r {
            for dx in -r..=r {
                let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                if xx >= 0 && yy >= 0 && (xx as usize) < sim.w && (yy as usize) < sim.h && sim.is_bloom(yy as usize * sim.w + xx as usize) {
                    return true;
                }
            }
        }
        false
    }

    /// A slimo's hurt this shift against its HP: dead, bleeding, or neither.
    pub fn verdict(&self, hp: f32, id: u32) -> (bool, bool) {
        let h = self.hurt.get(&id).copied().unwrap_or(0.0);
        (h >= hp, h >= hp * self.rules.defence.bleed_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn city() -> (City, Colony, StructureFile, Threats) {
        let city = City::game(logistics::founding(11), 11);
        let defs = StructureFile::shipped();
        let mut colony = Colony::new();
        colony.found(&defs, &city.sim).unwrap();
        let t = Threats::new(&city.sim, 11);
        (city, colony, defs, t)
    }

    fn run(c: &mut City, col: &mut Colony, defs: &StructureFile, t: &mut Threats, posts: &Posts, ticks: u32) -> Vec<Note> {
        let mut notes = Vec::new();
        for _ in 0..ticks {
            c.step();
            notes.extend(t.tick(c, col, defs, posts, false));
        }
        notes
    }

    #[test]
    fn the_gauge_counts_lume_and_trade_decays_and_knows_its_stratum() {
        let (mut c, mut col, defs, mut t) = city();
        let g = t.zones.iter().position(|z| c.markets.contains(&z.grate)).expect("the founding reaches a grate");
        t.zones[g].sold = 250;
        t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, false);
        let a = t.zones[g].attention;
        // 25 from the trade, Lume from the Nodes near the grate, less 2
        assert!(a >= 23.0, "attention {a}");
        // a Sump city above 60 still draws a Patrol, never a Sweep
        t.zones[g].attention = 70.0;
        t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, false);
        let due = t.due.iter().find(|d| d.zone as usize == g).expect("due");
        assert_eq!(t.rules.stage[due.stage].id, "patrol");
        // the quiet spell: a Patrol just down, no Patrol next shift
        t.zones[g].last[due.stage] = Some(t.shift + 1);
        t.shift += 1;
        t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, false);
        assert!(!t.due.iter().any(|d| d.zone as usize == g), "quiet");
    }

    #[test]
    fn a_patrol_maps_its_source_and_leaves_and_its_lamps_only_smoke_the_cords() {
        let (mut c, mut col, defs, mut t) = city();
        let g = t.zones.iter().position(|z| c.markets.contains(&z.grate)).unwrap();
        let cells = c.sim.hyphal_count();
        t.send(&c, &col, &defs, 0, g);
        let notes = run(&mut c, &mut col, &defs, &mut t, &Posts::default(), 900);
        assert!(notes.iter().any(|n| n.what.contains("is over")), "{:?}", notes.iter().map(|n| &n.what).collect::<Vec<_>>());
        assert!(t.bodies.iter().all(|b| b.gone || b.dead));
        let e = &t.events[0];
        assert_eq!(e.burned, 0, "a scout's lamp smokes, it does not kill, at Chrome Affinity 0.5");
        assert!(c.sim.hyphal_count() + 5 >= cells);
    }

    #[test]
    fn white_light_kills_where_the_armour_is_thin() {
        let (mut c, mut col, defs, mut t) = city();
        c.sim.genome = c.sim.genome.with(mat::genome::CHROME, 0.0);
        let g = t.zones.iter().position(|z| c.markets.contains(&z.grate)).unwrap();
        t.send(&c, &col, &defs, 0, g);
        run(&mut c, &mut col, &defs, &mut t, &Posts::default(), 900);
        assert!(t.events[0].burned > 0, "armour 1: 0.3 a step kills");
    }

    #[test]
    fn a_turret_with_a_guard_kills_a_scout_for_salvage_and_attention() {
        let (mut c, mut col, defs, mut t) = city();
        let g = t.zones.iter().position(|z| c.markets.contains(&z.grate)).unwrap();
        let gx = t.zones[g].grate as u16;
        // a Turret on the first anchor it can stand within its range of the grate
        let mut placed = None;
        'p: for y in 0..12u16 {
            for x in gx.saturating_sub(6)..gx + 6 {
                if let Ok(id) = col.place_built(&defs, &c.sim, "turret", x, y, 1) {
                    placed = Some(id);
                    break 'p;
                }
            }
        }
        let Some(turret) = placed else { return };
        let posts = Posts { guards: vec![(1, turret)], ..Default::default() };
        t.send(&c, &col, &defs, 0, g);
        run(&mut c, &mut col, &defs, &mut t, &posts, 900);
        let e = &t.events[0];
        assert!(e.fallen >= 1, "the Turret killed nothing");
        assert!(t.owed.0 >= 1, "a scout's 1-2 Blackline");
        let kills: u32 = t.zones.iter().map(|z| z.kills).sum();
        assert_eq!(kills, e.fallen);
    }

    #[test]
    fn a_gate_holds_the_lamps_and_only_flame_opens_it() {
        let (mut c, mut col, defs, mut t) = city();
        let g = t.zones.iter().position(|z| c.markets.contains(&z.grate)).unwrap();
        let gx = t.zones[g].grate;
        // a Gate across the throat under the grate, wherever its anchor takes it
        let mut gate = None;
        for (x, rot) in [(gx, false), (gx.saturating_sub(1), false)] {
            if let Ok(id) = col.place_r(&defs, &c.sim, "gate", x as u16, 1, rot) {
                let s = col.get_mut(id).unwrap();
                s.built = true;
                s.hp = 400;
                gate = Some(id);
                break;
            }
        }
        let Some(gate) = gate else { return };
        let st = t.rules.stage_index("cleansing").unwrap();
        t.send(&c, &col, &defs, st, g);
        run(&mut c, &mut col, &defs, &mut t, &Posts::default(), 600);
        // the lamp carriers wait at the Gate: none passes it while it stands
        let hp_mid = col.get(gate).map(|s| s.hp);
        assert_eq!(hp_mid, Some(400), "no carrier breaks a Gate");
        assert!(t.bodies.iter().filter(|b| b.role == Role::Carrier && !b.dead).all(|b| b.y <= 1));
    }

    #[test]
    fn a_cut_off_piece_goes_feral_and_blooms_and_its_crawlers_starve_with_it() {
        let (mut c, mut col, defs, mut t) = city();
        c.sim.genome = c.sim.genome.with(mat::genome::FERAL, 0.8);
        c.sim.p = c.sim.genome.params();
        // cut the founding's cord to the grate below its top: the top is cut off
        let joined = logistics::joined_to_heart(&c.sim);
        let w = c.sim.w;
        let top: Vec<usize> = (0..joined.len()).filter(|&i| joined[i] && i / w <= 6).collect();
        assert!(!top.is_empty());
        let cut: Vec<usize> = (0..joined.len()).filter(|&i| i / w == 7 && c.sim.is_hyphal_at(i)).collect();
        for i in cut {
            c.sim.sever(i % w, i / w);
        }
        for _ in 0..40 {
            c.sim.step();
        }
        let bloom: Vec<usize> = (0..c.sim.hy.len()).filter(|&i| c.sim.is_bloom(i)).collect();
        assert!(!bloom.is_empty(), "the cut-off top went feral");
        // its Nodes spawn crawlers at the shift's end
        if bloom.iter().any(|&i| c.sim.hy[i] == hy::NODE) {
            t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, false);
            assert!(t.bodies.iter().any(|b| b.role == Role::Crawler));
            // the Bloom gone, they starve
            for &i in &bloom {
                c.sim.sever(i % w, i / w);
            }
            t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, false);
            assert!(t.bodies.iter().all(|b| b.role != Role::Crawler));
        }
    }

    #[test]
    fn a_flood_rises_drowns_a_pod_on_it_and_falls_back() {
        let (mut c, mut col, defs, mut t) = city();
        let before: Vec<u8> = c.sim.sub.clone();
        // roll until the flood rises at least a tile
        let mut level = 0;
        for shift in 1..40 {
            t.start_shift(&mut c, &mut col, &defs, shift, true);
            if t.flood.level > 0 {
                level = t.flood.level;
                break;
            }
        }
        assert!(level > 0);
        let flooded: Vec<usize> = t.flooded().collect();
        assert!(!flooded.is_empty());
        t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, false);
        assert_eq!(c.sim.sub, before, "the water falls back at the shift's end");
    }

    #[test]
    fn quarantine_severs_mat_dark_two_shift_ends_running() {
        let (mut c, mut col, defs, mut t) = city();
        let w = c.sim.w;
        let cut: Vec<usize> = (0..c.sim.hy.len()).filter(|&i| i / w == 7 && c.sim.is_hyphal_at(i)).collect();
        for i in cut {
            c.sim.sever(i % w, i / w);
        }
        c.sim.compute_signal();
        let dark = (0..c.sim.hy.len()).filter(|&i| c.sim.is_hyphal_at(i) && c.sim.sig[i] <= 0.0).count();
        assert!(dark > 0);
        t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, true);
        let after_one = (0..c.sim.hy.len()).filter(|&i| c.sim.is_hyphal_at(i) && c.sim.sig[i] <= 0.0).count();
        assert_eq!(after_one, dark, "one shift dark: not yet");
        t.end_shift(&mut c, &mut col, &defs, &Shown::default(), false, false, true);
        c.sim.compute_signal();
        let after_two = (0..c.sim.hy.len()).filter(|&i| c.sim.is_hyphal_at(i) && c.sim.sig[i] <= 0.0).count();
        assert_eq!(after_two, 0, "two shift ends dark: severed");
    }

    #[test]
    fn the_threats_round_trip_through_their_save() {
        let (c, col, defs, mut t) = city();
        let g = t.zones.iter().position(|z| c.markets.contains(&z.grate)).unwrap();
        t.zones[g].attention = 42.0;
        t.send(&c, &col, &defs, 0, g);
        let s = t.save();
        let back = Threats::restore(&c.sim, 11, s.clone());
        assert_eq!(back.save(), s);
    }
}
