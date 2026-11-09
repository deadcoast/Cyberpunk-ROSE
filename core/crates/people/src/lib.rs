//! Slimo records (Full Design v3.7, Runners and slimos). A record is what the player loses when a slimo dies:
//! Mod-Depth and Past, six stats, eight skills with XP, two traits from the forty in `data/traits.toml`, six needs
//! and a Mood that moves toward its target at bounded rates, injuries, the run record and bonds. The `Roster` holds
//! every slimo in the city, creates newcomers from the seed, moves needs once per shift, pays XP from what the run
//! and the shift report say happened, and scores a slimo's utility for a job. Nothing here draws or decides for the
//! colony; the colony crate hands jobs to the highest utility and the Route Board reads who may run.

use mat::rng::Pcg32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const TRAITS_TOML: &str = include_str!("../../../../data/traits.toml");
pub const NAMES_TOML: &str = include_str!("../../../../data/names.toml");

// ------------------------------------------------------------------ enums

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Depth {
    D1,
    D2,
    D3,
    D4,
    D5,
}

impl Depth {
    pub fn all() -> [Depth; 5] {
        [Depth::D1, Depth::D2, Depth::D3, Depth::D4, Depth::D5]
    }
    pub fn index(self) -> usize {
        match self {
            Depth::D1 => 0,
            Depth::D2 => 1,
            Depth::D3 => 2,
            Depth::D4 => 3,
            Depth::D5 => 4,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Depth::D1 => "Skin-deep",
            Depth::D2 => "Limbed",
            Depth::D3 => "Core-modded",
            Depth::D4 => "Deep",
            Depth::D5 => "Dissolved",
        }
    }
    pub fn tag(self) -> &'static str {
        match self {
            Depth::D1 => "D1",
            Depth::D2 => "D2",
            Depth::D3 => "D3",
            Depth::D4 => "D4",
            Depth::D5 => "D5",
        }
    }
    /// Veins: the body's own lume.
    pub fn veins(self) -> u8 {
        match self {
            Depth::D1 => 1,
            Depth::D2 => 2,
            Depth::D3 => 3,
            Depth::D4 => 5,
            Depth::D5 => 0,
        }
    }
    /// Mat-sense is the depth's.
    pub fn matsense(self) -> i32 {
        match self {
            Depth::D1 => 1,
            Depth::D2 => 3,
            Depth::D3 => 5,
            Depth::D4 => 8,
            Depth::D5 => 1,
        }
    }
    /// A run leaves the Mat: D4 cannot, D5 cannot act.
    pub fn can_run(self) -> bool {
        matches!(self, Depth::D1 | Depth::D2 | Depth::D3)
    }
    pub fn can_act(self) -> bool {
        self != Depth::D5
    }
    /// Stat leans at creation: [body, nerve, hands, eye, voice] (Mat-sense is set, not leaned).
    fn leans(self) -> [i32; 5] {
        match self {
            Depth::D1 => [-1, 0, 0, 1, 2],
            Depth::D2 => [2, 0, 1, 0, 0],
            Depth::D3 => [0, 1, 2, 0, 0],
            Depth::D4 => [-2, 2, 0, 0, -1],
            Depth::D5 => [0, 0, 0, 0, 0],
        }
    }
    /// Skill levels the body brings.
    fn skills(self) -> Vec<(Skill, u8)> {
        match self {
            Depth::D1 => vec![(Skill::Trade, 2)],
            Depth::D2 => vec![(Skill::Scav, 1)],
            Depth::D3 => vec![(Skill::Tweak, 2)],
            Depth::D4 => vec![(Skill::Shepherd, 3)],
            Depth::D5 => vec![],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Past {
    Gang,
    Nomad,
    Corpo,
    Badge,
    Civ,
}

impl Past {
    pub fn all() -> [Past; 5] {
        [Past::Gang, Past::Nomad, Past::Corpo, Past::Badge, Past::Civ]
    }
    pub fn name(self) -> &'static str {
        match self {
            Past::Gang => "Gang",
            Past::Nomad => "Nomad",
            Past::Corpo => "Corpo",
            Past::Badge => "Badge",
            Past::Civ => "Civ",
        }
    }
    fn leans(self) -> [i32; 5] {
        // [body, nerve, hands, eye, voice]
        match self {
            Past::Gang => [1, 1, 0, 0, 0],
            Past::Nomad => [0, 1, 0, 1, 0],
            Past::Corpo => [0, 0, 0, 1, 1],
            Past::Badge => [1, 0, 0, 0, 1],
            Past::Civ => [0, 0, 1, 0, 0],
        }
    }
    fn skills(self) -> Vec<(Skill, u8)> {
        match self {
            Past::Gang => vec![(Skill::Fight, 2)],
            Past::Nomad => vec![(Skill::Scav, 2), (Skill::Shepherd, 1)],
            Past::Corpo => vec![(Skill::Appraise, 2), (Skill::Tweak, 1)],
            Past::Badge => vec![(Skill::Guard, 2), (Skill::Fight, 1)],
            Past::Civ => vec![(Skill::Trade, 1), (Skill::Medic, 1)],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stat {
    Body,
    Nerve,
    Hands,
    Eye,
    MatSense,
    Voice,
}

impl Stat {
    pub fn all() -> [Stat; 6] {
        [Stat::Body, Stat::Nerve, Stat::Hands, Stat::Eye, Stat::MatSense, Stat::Voice]
    }
    pub fn name(self) -> &'static str {
        match self {
            Stat::Body => "Body",
            Stat::Nerve => "Nerve",
            Stat::Hands => "Hands",
            Stat::Eye => "Eye",
            Stat::MatSense => "Mat-sense",
            Stat::Voice => "Voice",
        }
    }
    fn from_key(k: &str) -> Option<Stat> {
        Some(match k {
            "body" => Stat::Body,
            "nerve" => Stat::Nerve,
            "hands" => Stat::Hands,
            "eye" => Stat::Eye,
            "matsense" => Stat::MatSense,
            "voice" => Stat::Voice,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Scav,
    Fight,
    Tweak,
    Shepherd,
    Trade,
    Appraise,
    Medic,
    Guard,
}

impl Skill {
    pub fn all() -> [Skill; 8] {
        [Skill::Scav, Skill::Fight, Skill::Tweak, Skill::Shepherd, Skill::Trade, Skill::Appraise, Skill::Medic, Skill::Guard]
    }
    pub fn index(self) -> usize {
        Skill::all().iter().position(|s| *s == self).unwrap()
    }
    pub fn name(self) -> &'static str {
        match self {
            Skill::Scav => "Scav",
            Skill::Fight => "Fight",
            Skill::Tweak => "Tweak",
            Skill::Shepherd => "Shepherd",
            Skill::Trade => "Trade",
            Skill::Appraise => "Appraise",
            Skill::Medic => "Medic",
            Skill::Guard => "Guard",
        }
    }
    pub fn from_key(k: &str) -> Option<Skill> {
        Skill::all().into_iter().find(|s| s.name().eq_ignore_ascii_case(k))
    }
    /// The mastery perk at 8, in words.
    pub fn mastery(self) -> &'static str {
        match self {
            Skill::Scav => "containers show their tier through fog",
            Skill::Fight => "wind-up a tenth faster",
            Skill::Tweak => "one free part per craft",
            Skill::Shepherd => "tips within 6 tiles extend at double rate",
            Skill::Trade => "glut recovers in one shift at their market",
            Skill::Appraise => "identifies Relics",
            Skill::Medic => "fractures heal in one shift",
            Skill::Guard => "strikes two creatures a swing",
        }
    }
}

/// XP a level costs: 10 + 5 × level (325 from 0 to 10).
pub fn xp_to_next(level: u8) -> u32 {
    10 + 5 * level as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Job {
    Haul,
    Build,
    Scav,
    Tweak,
    Tend,
    Guard,
    Cook,
    Appraise,
    Research,
}

impl Job {
    pub fn all() -> [Job; 9] {
        [Job::Haul, Job::Build, Job::Scav, Job::Tweak, Job::Tend, Job::Guard, Job::Cook, Job::Appraise, Job::Research]
    }
    pub fn index(self) -> usize {
        Job::all().iter().position(|j| *j == self).unwrap()
    }
    pub fn name(self) -> &'static str {
        match self {
            Job::Haul => "Haul",
            Job::Build => "Build",
            Job::Scav => "Scav",
            Job::Tweak => "Tweak",
            Job::Tend => "Tend",
            Job::Guard => "Guard",
            Job::Cook => "Cook",
            Job::Appraise => "Appraise",
            Job::Research => "Research",
        }
    }
    pub fn from_key(k: &str) -> Option<Job> {
        Job::all().into_iter().find(|j| j.name().eq_ignore_ascii_case(k))
    }
    /// The skill a job runs on.
    pub fn skill(self) -> Skill {
        match self {
            Job::Haul => Skill::Scav,
            Job::Build => Skill::Tweak,
            Job::Scav => Skill::Scav,
            Job::Tweak => Skill::Tweak,
            Job::Tend => Skill::Shepherd,
            Job::Guard => Skill::Guard,
            Job::Cook => Skill::Medic,
            Job::Appraise => Skill::Appraise,
            Job::Research => Skill::Tweak,
        }
    }
    /// Skill gates for the mid-tier jobs (Full Design: Research Tweak 5, the Appraisal Table Appraise 3).
    pub fn gate(self) -> u8 {
        match self {
            Job::Research => 5,
            Job::Appraise => 3,
            _ => 0,
        }
    }
    /// Jobs that leave the Mat (a D4 cannot take them).
    pub fn off_mat(self) -> bool {
        matches!(self, Job::Scav | Job::Guard | Job::Cook)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Need {
    Fed,
    Lit,
    Dry,
    Signal,
    Rest,
    Mood,
}

impl Need {
    pub fn all() -> [Need; 6] {
        [Need::Fed, Need::Lit, Need::Dry, Need::Signal, Need::Rest, Need::Mood]
    }
    pub fn index(self) -> usize {
        Need::all().iter().position(|n| *n == self).unwrap()
    }
    pub fn name(self) -> &'static str {
        match self {
            Need::Fed => "Fed",
            Need::Lit => "Lit",
            Need::Dry => "Dry",
            Need::Signal => "Signal",
            Need::Rest => "Rest",
            Need::Mood => "Mood",
        }
    }
    fn from_key(k: &str) -> Option<Need> {
        Need::all().into_iter().find(|n| n.name().eq_ignore_ascii_case(k))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Injury {
    Bleeding,
    Fracture,
    ChromeShock,
    Soaked,
    Blinded,
    FungalFever,
}

impl Injury {
    pub fn name(self) -> &'static str {
        match self {
            Injury::Bleeding => "Bleeding",
            Injury::Fracture => "Fracture",
            Injury::ChromeShock => "Chrome Shock",
            Injury::Soaked => "Soaked",
            Injury::Blinded => "Blinded",
            Injury::FungalFever => "Fungal Fever",
        }
    }
    /// Shifts to heal untreated (a Medic treats at four times the rate).
    pub fn shifts(self) -> u8 {
        match self {
            Injury::Bleeding => 1,
            Injury::Fracture => 12,
            Injury::ChromeShock => 255,
            Injury::Soaked => 1,
            Injury::Blinded => 1,
            Injury::FungalFever => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Idle,
    Working(Job),
    Resting,
    OnRun,
    Stranded,
    Down,
    Dead,
    Feral,
}

// ------------------------------------------------------------------ traits (data)

#[derive(Clone, Debug, Deserialize)]
pub struct TraitDef {
    pub id: String,
    pub name: String,
    pub text: String,
    #[serde(default)]
    pub stats: HashMap<String, i32>,
    #[serde(default)]
    pub need_rate: HashMap<String, f32>,
    #[serde(default)]
    pub mood: i32,
    #[serde(default)]
    pub loves: String,
    #[serde(default)]
    pub hates: String,
    #[serde(default)]
    pub skill_xp: HashMap<String, f32>,
    #[serde(default)]
    pub rig_rows: i32,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub carry: f32,
    #[serde(default)]
    pub defect_line: i32,
    #[serde(default)]
    pub bond_runs: u8,
    #[serde(default)]
    pub appraise_level: u8,
    #[serde(default)]
    pub tweak_level: u8,
    #[serde(default)]
    pub excludes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TraitFile {
    #[serde(rename = "trait")]
    pub traits: Vec<TraitDef>,
}

impl TraitFile {
    pub fn from_toml(s: &str) -> Result<Self, String> {
        let f: TraitFile = toml::from_str(s).map_err(|e| e.to_string())?;
        if f.traits.len() != 40 {
            return Err(format!("{} traits, the design has forty", f.traits.len()));
        }
        for t in &f.traits {
            for k in t.stats.keys() {
                if Stat::from_key(k).is_none() {
                    return Err(format!("{}: unknown stat {k}", t.id));
                }
            }
            for k in t.need_rate.keys() {
                if Need::from_key(k).is_none() || k.eq_ignore_ascii_case("mood") {
                    return Err(format!("{}: unknown need rate {k}", t.id));
                }
            }
            for j in [&t.loves, &t.hates] {
                if !j.is_empty() && Job::from_key(j).is_none() {
                    return Err(format!("{}: unknown job {j}", t.id));
                }
            }
            for k in t.skill_xp.keys() {
                if Skill::from_key(k).is_none() {
                    return Err(format!("{}: unknown skill {k}", t.id));
                }
            }
            for e in &t.excludes {
                if !f.traits.iter().any(|o| &o.id == e) {
                    return Err(format!("{}: excludes unknown trait {e}", t.id));
                }
            }
        }
        Ok(f)
    }
    pub fn shipped() -> Self {
        Self::from_toml(TRAITS_TOML).expect("shipped traits")
    }
    pub fn get(&self, id: &str) -> Option<&TraitDef> {
        self.traits.iter().find(|t| t.id == id)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NameFile {
    pub first: Vec<String>,
    pub epithet: Vec<String>,
}

impl NameFile {
    pub fn shipped() -> Self {
        toml::from_str(NAMES_TOML).expect("shipped names")
    }
}

// ------------------------------------------------------------------ the record

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bond {
    pub with: u32,
    pub runs_together: u8,
    pub bonded: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub epithet: String,
    pub depth: Depth,
    pub past: Past,
    /// [body, nerve, hands, eye, matsense, voice], 1-10
    pub stats: [i32; 6],
    /// levels 0-10 in Skill::all order
    pub skills: [u8; 8],
    pub xp: [u32; 8],
    pub traits: [String; 2],
    /// 0-100 in Need::all order; Mood last
    pub needs: [i32; 6],
    pub mood_target: i32,
    pub injuries: Vec<(Injury, u8)>,
    /// job priorities 0-4 in Job::all order (0 never)
    pub priorities: [u8; 9],
    pub status: Status,
    pub runs: u32,
    pub survived: u32,
    pub kills: u32,
    pub hauls: u32,
    pub crafts: u32,
    pub bonds: Vec<Bond>,
    pub joined_day: u32,
    /// the job they are working this shift (for the Mood lean and XP); None when idle, resting or out
    pub shift_job: Option<Job>,
}

/// What the run needs from a record (Full Design: Technical architecture).
#[derive(Clone, Debug, PartialEq)]
pub struct RunnerSetup {
    pub id: u32,
    pub name: String,
    pub body: u8,
    pub nerve: u8,
    pub hands: u8,
    pub eye: u8,
    pub veins: u8,
    pub rig_rows: u8,
    pub carry_bonus_kg: f32,
    pub flags: Vec<String>,
}

impl RunnerSetup {
    /// A stand-in with the table's numbers at every stat but Body (the Phase B runs, the bot, tests): no traits.
    pub fn plain(body: u8) -> RunnerSetup {
        RunnerSetup { id: 0, name: "the Runner".into(), body: body.clamp(1, 10), nerve: 5, hands: 5, eye: 3, veins: 2, rig_rows: 5, carry_bonus_kg: 0.0, flags: Vec::new() }
    }
    pub fn has(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }
    /// The carry limit in kg (Full Design, The record in a run): 18 + 1 per Body above 5 + the traits'.
    pub fn carry_kg(&self) -> f32 {
        18.0 + (self.body as f32 - 5.0) + self.carry_bonus_kg
    }
    /// Follower back-off line as a percentage of HP: 30, a Coward 60, a Fearless 0.
    pub fn back_off_pct(&self) -> u8 {
        if self.has("never_flee") {
            0
        } else if self.has("flee_at_60") {
            60
        } else {
            30
        }
    }
}

impl Record {
    pub fn stat(&self, s: Stat) -> i32 {
        self.stats[Stat::all().iter().position(|x| *x == s).unwrap()]
    }
    pub fn skill(&self, s: Skill) -> u8 {
        self.skills[s.index()]
    }
    pub fn need(&self, n: Need) -> i32 {
        self.needs[n.index()]
    }
    pub fn mood(&self) -> i32 {
        self.needs[Need::Mood.index()]
    }
    pub fn full_name(&self) -> String {
        if self.epithet.is_empty() { self.name.clone() } else { format!("{} '{}'", self.name, self.epithet) }
    }
    pub fn has_trait(&self, id: &str) -> bool {
        self.traits.iter().any(|t| t == id)
    }
    pub fn is_up(&self) -> bool {
        matches!(self.status, Status::Idle | Status::Working(_) | Status::Resting)
    }
    /// May this slimo be the Runner or a follower of a run now?
    pub fn can_run(&self) -> bool {
        self.depth.can_run() && self.is_up() && self.mood() >= 20
    }
    pub fn has_flag(&self, traits: &TraitFile, flag: &str) -> bool {
        self.traits.iter().filter_map(|t| traits.get(t)).any(|t| t.flags.iter().any(|f| f == flag))
    }
    /// The unrest line the Mood stands below, as a severity: 1 strike (<30), 2 defection (<20; a Devout's 25),
    /// 3 petition (<10), 0 none.
    pub fn unrest_line(&self, traits: &TraitFile) -> u8 {
        let defect = self.traits.iter().filter_map(|t| traits.get(t)).map(|t| t.defect_line).max().unwrap_or(0).max(20);
        let m = self.mood();
        if m < 10 {
            3
        } else if m < defect {
            2
        } else if m < 30 {
            1
        } else {
            0
        }
    }
    pub fn trait_rate(&self, traits: &TraitFile, need: Need) -> f32 {
        let key = need.name().to_lowercase();
        self.traits.iter().filter_map(|t| traits.get(t)).map(|t| t.need_rate.get(&key).copied().unwrap_or(1.0)).product()
    }
    pub fn xp_mult(&self, traits: &TraitFile, skill: Skill) -> f32 {
        let key = skill.name().to_lowercase();
        self.traits.iter().filter_map(|t| traits.get(t)).map(|t| t.skill_xp.get(&key).copied().unwrap_or(1.0)).product()
    }
    pub fn carry_bonus(&self, traits: &TraitFile) -> f32 {
        self.traits.iter().filter_map(|t| traits.get(t)).map(|t| t.carry).sum()
    }
    pub fn rig_rows(&self, traits: &TraitFile) -> i32 {
        self.traits.iter().filter_map(|t| traits.get(t)).map(|t| t.rig_rows).sum()
    }
    pub fn bond_runs(&self, traits: &TraitFile) -> u8 {
        self.traits.iter().filter_map(|t| traits.get(t)).map(|t| t.bond_runs).filter(|b| *b > 0).min().unwrap_or(3)
    }

    /// Add XP to a skill, levelling as it goes; returns levels gained.
    pub fn gain_xp(&mut self, traits: &TraitFile, skill: Skill, base: u32, district_mult: f32) -> u8 {
        let i = skill.index();
        if self.skills[i] >= 10 {
            return 0;
        }
        let amount = (base as f32 * self.xp_mult(traits, skill) * district_mult).round() as u32;
        self.xp[i] += amount;
        let mut gained = 0;
        while self.skills[i] < 10 && self.xp[i] >= xp_to_next(self.skills[i]) {
            self.xp[i] -= xp_to_next(self.skills[i]);
            self.skills[i] += 1;
            gained += 1;
        }
        if self.skills[i] >= 10 {
            self.xp[i] = 0;
        }
        gained
    }

    pub fn runner_setup(&self, traits: &TraitFile) -> RunnerSetup {
        let mut flags: Vec<String> = self.traits.iter().filter_map(|t| traits.get(t)).flat_map(|t| t.flags.iter().cloned()).collect();
        flags.sort();
        RunnerSetup {
            id: self.id,
            name: self.full_name(),
            body: self.stat(Stat::Body).clamp(1, 10) as u8,
            nerve: self.stat(Stat::Nerve).clamp(1, 10) as u8,
            hands: self.stat(Stat::Hands).clamp(1, 10) as u8,
            eye: self.stat(Stat::Eye).clamp(1, 10) as u8,
            veins: self.depth.veins(),
            rig_rows: (5 + self.rig_rows(traits)).clamp(3, 8) as u8,
            carry_bonus_kg: self.carry_bonus(traits),
            flags,
        }
    }
}

// ------------------------------------------------------------------ the shift

/// Where a slimo was and what it did this shift, for the needs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShiftContext {
    pub on_mat: bool,
    pub signalled: bool,
    pub lit: bool,
    pub wet: bool,
    /// asleep in a Pod during Dark
    pub pod_dark: bool,
    pub asleep: bool,
    /// a meal was had (the Mat fed them, or paste)
    pub fed: bool,
    pub in_bunkhouse: bool,
    pub alone_in_pod: bool,
    pub white_light_near: bool,
    pub narrow: bool,
    /// Voice of a slimo who died this shift (0 none); doubled for a bonded partner by the caller
    pub death_voice: i32,
    /// the city tier's expectation and the Charter's laws (PAC-041/042)
    pub extra_target: i32,
}

pub const MOOD_UP: i32 = 4;
pub const MOOD_DOWN: i32 = 3;

impl Record {
    /// Move the six needs one shift (Full Design: Needs, Mood).
    pub fn end_shift(&mut self, traits: &TraitFile, ctx: ShiftContext) {
        if !self.depth.can_act() || self.status == Status::Dead {
            return;
        }
        // Fed
        let fed_rate = self.trait_rate(traits, Need::Fed);
        let fi = Need::Fed.index();
        if ctx.fed {
            self.needs[fi] = 100;
        } else {
            self.needs[fi] -= (15.0 * fed_rate).round() as i32;
        }
        // Lit: D4 has no Lit need, D1 loses more
        if self.depth != Depth::D4 {
            let base = if self.depth == Depth::D1 { 15.0 } else { 10.0 };
            let d = if ctx.lit { 20.0 } else { -base * self.trait_rate(traits, Need::Lit) };
            self.needs[Need::Lit.index()] += d.round() as i32;
        } else {
            self.needs[Need::Lit.index()] = 100;
        }
        // Dry
        let d = if ctx.wet { -25.0 * self.trait_rate(traits, Need::Dry) } else { 15.0 };
        self.needs[Need::Dry.index()] += d.round() as i32;
        // Signal
        let d = if ctx.on_mat && ctx.signalled { 25.0 } else { -(if self.depth == Depth::D4 { 40.0 } else { 20.0 }) * self.trait_rate(traits, Need::Signal) };
        self.needs[Need::Signal.index()] += d.round() as i32;
        // Rest
        let d = if ctx.pod_dark {
            30.0 * self.trait_rate(traits, Need::Rest)
        } else if ctx.asleep {
            15.0 * self.trait_rate(traits, Need::Rest)
        } else if self.shift_job.is_some() || self.status == Status::OnRun {
            -12.0
        } else {
            -6.0
        };
        self.needs[Need::Rest.index()] += d.round() as i32;
        for i in 0..5 {
            self.needs[i] = self.needs[i].clamp(0, 100);
        }
        // consequences
        if self.needs[Need::Fed.index()] == 0 {
            self.stats[0] = (self.stats[0] - 1).max(1);
        }
        // Mood target
        let mut target = 60 + ctx.extra_target;
        for i in 0..5 {
            let v = self.needs[i];
            if v < 20 {
                target -= 20;
            } else if v < 40 {
                target -= 10;
            } else if v > 80 {
                target += 5;
            }
        }
        for t in self.traits.iter().filter_map(|t| traits.get(t)) {
            target += t.mood;
            if let Some(job) = self.shift_job {
                if Job::from_key(&t.loves) == Some(job) {
                    target += 10;
                }
                if Job::from_key(&t.hates) == Some(job) {
                    target -= 15;
                }
            }
            if ctx.white_light_near && t.flags.iter().any(|f| f == "white_light_mood_plus_10") {
                target += 10;
            }
            if ctx.narrow && t.flags.iter().any(|f| f == "narrow_mood_minus_10") {
                target -= 10;
            }
            if ctx.in_bunkhouse && t.flags.iter().any(|f| f == "bunkhouse_plus_10") {
                target += 10;
            }
            if ctx.in_bunkhouse && t.flags.iter().any(|f| f == "bunkhouse_minus_10") {
                target -= 10;
            }
            if ctx.alone_in_pod && t.flags.iter().any(|f| f == "pod_alone_plus_10") {
                target += 10;
            }
            if self.needs[Need::Fed.index()] > 80 && t.flags.iter().any(|f| f == "fed_80_mood_plus_5") {
                target += 5;
            }
        }
        target -= 5 * self.injuries.len() as i32;
        target -= ctx.death_voice;
        self.mood_target = target.clamp(0, 100);
        // Mood moves toward the target at bounded rates
        let m = &mut self.needs[Need::Mood.index()];
        if *m < self.mood_target {
            *m = (*m + MOOD_UP).min(self.mood_target);
        } else if *m > self.mood_target {
            *m = (*m - MOOD_DOWN).max(self.mood_target);
        }
        // injuries heal (a quarter rate untreated: one shift in four)
        let mut keep = Vec::new();
        for (inj, left) in self.injuries.drain(..) {
            let left = if inj == Injury::ChromeShock { left } else { left.saturating_sub(1) };
            if left > 0 {
                keep.push((inj, left));
            }
        }
        self.injuries = keep;
    }

    /// A Medic treats: every injury loses four shifts (eight with Healer's Hands); Chrome Shock needs a Bench.
    pub fn treat(&mut self, traits: &TraitFile, medic_has_healers_hands: bool) -> usize {
        let mut treated = 0;
        let step = if medic_has_healers_hands { 8 } else { 4 };
        let _ = traits;
        let mut keep = Vec::new();
        for (inj, left) in self.injuries.drain(..) {
            if inj == Injury::ChromeShock {
                keep.push((inj, left));
                continue;
            }
            treated += 1;
            let left = left.saturating_sub(step);
            if left > 0 {
                keep.push((inj, left));
            }
        }
        self.injuries = keep;
        treated
    }
}

// ------------------------------------------------------------------ job utility

impl Record {
    /// Utility for a job now: None when the slimo cannot or will not take it (Full Design: Jobs by utility).
    pub fn utility(&self, traits: &TraitFile, job: Job) -> Option<f32> {
        if !self.depth.can_act() || !self.is_up() {
            return None;
        }
        if self.depth == Depth::D4 && job.off_mat() {
            return None;
        }
        if self.depth == Depth::D1 && job == Job::Tend {
            return None;
        }
        let pri = self.priorities[job.index()];
        if pri == 0 {
            return None;
        }
        if self.skill(job.skill()) < job.gate() {
            return None;
        }
        if self.mood() < 20 {
            return None;
        }
        if self.need(Need::Rest) < 20 {
            return None;
        }
        let mut u = pri as f32 * (0.6 + self.skill(job.skill()) as f32 / 10.0);
        for t in self.traits.iter().filter_map(|t| traits.get(t)) {
            if Job::from_key(&t.loves) == Some(job) {
                u *= 1.4;
            }
            if Job::from_key(&t.hates) == Some(job) {
                u *= 0.5;
            }
        }
        if self.need(Need::Fed) < 30 && job == Job::Cook {
            u *= 2.0;
        }
        if self.need(Need::Rest) < 40 && job != Job::Tend {
            u *= 0.5;
        }
        Some(u)
    }
}

// ------------------------------------------------------------------ the roster

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Roster {
    pub records: Vec<Record>,
    pub next_id: u32,
    /// the memorial: dead slimos' records
    pub memorial: Vec<Record>,
}

/// What a run tells the roster (Full Design: Skills and XP).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunReport {
    pub runner: u32,
    pub followers: Vec<u32>,
    pub extracted: bool,
    pub runner_died: bool,
    pub followers_extracted: Vec<u32>,
    pub followers_stranded: Vec<u32>,
    pub followers_dead: Vec<u32>,
    pub pulls: u32,
    pub items_taken: u32,
    pub kills: u32,
    pub hits_survived: u32,
    pub follower_strikes: u32,
    /// injuries the Runner carries home, in order
    pub injuries: Vec<Injury>,
}

/// What a shift tells the roster.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShiftWork {
    pub crafts_by: Vec<u32>,
    pub sales_by: Vec<(u32, u32)>,
    pub identified_by: Vec<u32>,
    pub treatments_by: Vec<u32>,
    pub tend_shifts_by: Vec<u32>,
}

impl Default for Roster {
    fn default() -> Self {
        Self::new()
    }
}

impl Roster {
    /// Empty; ids start at 100 so a record id never reads as a count or an index.
    pub fn new() -> Self {
        Roster { records: Vec::new(), next_id: 100, memorial: Vec::new() }
    }

    pub fn get(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Record> {
        self.records.iter_mut().find(|r| r.id == id)
    }

    /// Roll a newcomer (Full Design: Creation). `deep_weights` shifts the depth table toward the deep (chapter 5).
    pub fn create(&mut self, traits: &TraitFile, names: &NameFile, rng: &mut Pcg32, day: u32, deep: bool) -> u32 {
        let depth_w: [u32; 5] = if deep { [10, 25, 30, 25, 10] } else { [30, 40, 20, 8, 2] };
        let depth = Depth::all()[weighted(rng, &depth_w)];
        let past = Past::all()[weighted(rng, &[25, 20, 15, 10, 30])];
        // stats
        let mut s5 = [4i32; 5];
        for (i, l) in depth.leans().iter().enumerate() {
            s5[i] += l;
        }
        for (i, l) in past.leans().iter().enumerate() {
            s5[i] += l;
        }
        for _ in 0..3 {
            let i = rng.below(5) as usize;
            s5[i] += 1;
        }
        let mut stats = [s5[0], s5[1], s5[2], s5[3], depth.matsense(), s5[4]];
        if depth == Depth::D5 {
            stats = [1; 6];
        }
        // traits: two from the forty, no excluded pair
        let mut picked: Vec<&TraitDef> = Vec::new();
        let mut guard = 0;
        while picked.len() < 2 && guard < 200 {
            guard += 1;
            let t = &traits.traits[rng.below(traits.traits.len() as u32) as usize];
            if picked.iter().any(|p| p.id == t.id || p.excludes.contains(&t.id) || t.excludes.contains(&p.id)) {
                continue;
            }
            picked.push(t);
        }
        for t in &picked {
            for (k, v) in &t.stats {
                if let Some(st) = Stat::from_key(k) {
                    let i = Stat::all().iter().position(|x| *x == st).unwrap();
                    stats[i] += v;
                }
            }
        }
        for v in &mut stats {
            *v = (*v).clamp(1, 10);
        }
        // skills from the body and the past, and a trait's level
        let mut skills = [0u8; 8];
        for (sk, lv) in depth.skills().into_iter().chain(past.skills()) {
            skills[sk.index()] = (skills[sk.index()] + lv).min(10);
        }
        for t in &picked {
            skills[Skill::Appraise.index()] = (skills[Skill::Appraise.index()] + t.appraise_level).min(10);
            skills[Skill::Tweak.index()] = (skills[Skill::Tweak.index()] + t.tweak_level).min(10);
        }
        if depth == Depth::D5 {
            skills = [0; 8];
        }
        let name = names.first[rng.below(names.first.len() as u32) as usize].clone();
        let epithet = if rng.below(4) == 0 { names.epithet[rng.below(names.epithet.len() as u32) as usize].clone() } else { String::new() };
        let id = self.next_id;
        self.next_id += 1;
        let mut priorities = [2u8; 9];
        // the body's role leads
        match depth {
            Depth::D1 => priorities[Job::Appraise.index()] = 3,
            Depth::D2 => priorities[Job::Scav.index()] = 3,
            Depth::D3 => priorities[Job::Tweak.index()] = 3,
            Depth::D4 => {
                priorities[Job::Tend.index()] = 4;
                for j in Job::all() {
                    if j.off_mat() {
                        priorities[j.index()] = 0;
                    }
                }
            }
            Depth::D5 => priorities = [0; 9],
        }
        if depth == Depth::D1 {
            priorities[Job::Tend.index()] = 0;
        }
        self.records.push(Record {
            id,
            name,
            epithet,
            depth,
            past,
            stats,
            skills,
            xp: [0; 8],
            traits: [picked.first().map(|t| t.id.clone()).unwrap_or_default(), picked.get(1).map(|t| t.id.clone()).unwrap_or_default()],
            needs: [100, 100, 100, 100, 100, 60],
            mood_target: 60,
            injuries: Vec::new(),
            priorities,
            status: if depth == Depth::D5 { Status::Down } else { Status::Idle },
            runs: 0,
            survived: 0,
            kills: 0,
            hauls: 0,
            crafts: 0,
            bonds: Vec::new(),
            joined_day: day,
            shift_job: None,
        });
        id
    }

    /// The founding crew: six slimos who can run, so the first Route Board has a Runner and followers to pick.
    pub fn found(&mut self, traits: &TraitFile, names: &NameFile, seed: u64, day: u32) {
        let mut rng = Pcg32::new(seed ^ 0x5EED_5110);
        let mut runners = 0;
        let mut guard = 0;
        while self.records.len() < 6 && guard < 60 {
            guard += 1;
            let id = self.create(traits, names, &mut rng, day, false);
            let r = self.get(id).unwrap();
            if r.depth.can_run() {
                runners += 1;
            } else if runners < 3 && self.records.len() >= 4 {
                // keep the founders mostly runners: drop a non-runner when there is no room left for three
                self.records.pop();
            }
        }
    }

    /// Who may go on a run now, Runners first by Scav then Fight.
    pub fn eligible(&self) -> Vec<&Record> {
        let mut v: Vec<&Record> = self.records.iter().filter(|r| r.can_run()).collect();
        v.sort_by_key(|r| std::cmp::Reverse((r.skill(Skill::Scav) as u32 * 2 + r.skill(Skill::Fight) as u32, r.id)));
        v
    }

    /// Mark who is out on the run.
    pub fn depart(&mut self, runner: u32, followers: &[u32]) {
        for id in std::iter::once(runner).chain(followers.iter().copied()) {
            if let Some(r) = self.get_mut(id) {
                r.status = Status::OnRun;
                r.shift_job = None;
            }
        }
    }

    /// The run came home: XP, record lines, status, bonds, injuries, the dead to the memorial.
    pub fn apply_run(&mut self, traits: &TraitFile, report: &RunReport) {
        let all: Vec<u32> = std::iter::once(report.runner).chain(report.followers.iter().copied()).collect();
        // bonds first, on everyone who went and came back up
        let survivors: Vec<u32> = all.iter().copied().filter(|id| !(report.runner_died && *id == report.runner) && !report.followers_dead.contains(id)).collect();
        for &a in &survivors {
            let Some(ra) = self.get(a).cloned() else { continue };
            let need = ra.bond_runs(traits);
            for &b in &survivors {
                if a == b {
                    continue;
                }
                if let Some(r) = self.get_mut(a) {
                    let bond = match r.bonds.iter_mut().find(|bd| bd.with == b) {
                        Some(bd) => bd,
                        None => {
                            r.bonds.push(Bond { with: b, runs_together: 0, bonded: false });
                            r.bonds.last_mut().unwrap()
                        }
                    };
                    bond.runs_together = bond.runs_together.saturating_add(1);
                    if bond.runs_together >= need {
                        bond.bonded = true;
                    }
                }
            }
        }
        // the Runner: the run was this shift's work, and a run is the Scav job (off the Mat), so the shift's Rest
        // and the loved / hated lean read it
        if let Some(r) = self.get_mut(report.runner) {
            r.runs += 1;
            r.hauls += report.pulls;
            r.kills += report.kills;
            r.shift_job = Some(Job::Scav);
            r.gain_xp(traits, Skill::Scav, report.pulls * 2 + report.items_taken, 1.0);
            r.gain_xp(traits, Skill::Fight, report.kills * 3 + report.hits_survived, 1.0);
            for inj in &report.injuries {
                if !r.injuries.iter().any(|(i, _)| i == inj) {
                    r.injuries.push((*inj, inj.shifts()));
                }
            }
            if report.runner_died {
                r.status = Status::Dead;
            } else {
                r.survived += 1;
                r.status = Status::Idle;
            }
        }
        // followers
        for &id in &report.followers {
            if let Some(r) = self.get_mut(id) {
                r.runs += 1;
                r.shift_job = Some(Job::Guard);
                r.gain_xp(traits, Skill::Guard, report.follower_strikes * 2 / report.followers.len().max(1) as u32, 1.0);
                r.gain_xp(traits, Skill::Fight, report.kills, 1.0);
                if report.followers_dead.contains(&id) {
                    r.status = Status::Dead;
                } else if report.followers_stranded.contains(&id) {
                    r.status = Status::Stranded;
                    r.survived += 1;
                } else {
                    r.status = Status::Idle;
                    r.survived += 1;
                }
            }
        }
        // the dead to the memorial; the survivors' Mood takes the hit at shift end through death_voice
        let dead: Vec<Record> = self.records.iter().filter(|r| r.status == Status::Dead).cloned().collect();
        self.records.retain(|r| r.status != Status::Dead);
        self.memorial.extend(dead);
    }

    /// A stranded follower recovered on a later run.
    pub fn recover(&mut self, id: u32) {
        if let Some(r) = self.get_mut(id) {
            if r.status == Status::Stranded {
                r.status = Status::Idle;
            }
        }
    }

    /// Shift end for the city: needs, Mood and XP from the shift's work. `ctx_of` gives each slimo's context.
    pub fn end_shift(&mut self, traits: &TraitFile, work: &ShiftWork, ctx_of: impl Fn(&Record) -> ShiftContext) {
        let teacher_present = self.records.iter().any(|r| r.has_flag(traits, "district_xp_x1_25") && r.is_up());
        let mult = if teacher_present { 1.25 } else { 1.0 };
        for id in &work.crafts_by {
            if let Some(r) = self.get_mut(*id) {
                r.crafts += 1;
                r.gain_xp(traits, Skill::Tweak, 4, mult);
            }
        }
        for (id, n) in &work.sales_by {
            if let Some(r) = self.get_mut(*id) {
                r.gain_xp(traits, Skill::Trade, 2 * n, mult);
            }
        }
        for id in &work.identified_by {
            if let Some(r) = self.get_mut(*id) {
                r.gain_xp(traits, Skill::Appraise, 3, mult);
            }
        }
        for id in &work.treatments_by {
            if let Some(r) = self.get_mut(*id) {
                r.gain_xp(traits, Skill::Medic, 3, mult);
            }
        }
        for id in &work.tend_shifts_by {
            if let Some(r) = self.get_mut(*id) {
                r.gain_xp(traits, Skill::Shepherd, 1, mult);
            }
        }
        let n = self.records.len();
        for i in 0..n {
            let ctx = ctx_of(&self.records[i]);
            self.records[i].end_shift(traits, ctx);
        }
    }

    /// The Voice lost this shift for a record: the sum of the dead's Voice, doubled for a bonded partner.
    pub fn death_voice_for(&self, record: &Record, died: &[Record]) -> i32 {
        died.iter().map(|d| d.stat(Stat::Voice) * if record.bonds.iter().any(|b| b.with == d.id && b.bonded) { 2 } else { 1 }).sum()
    }
}

fn weighted(rng: &mut Pcg32, w: &[u32]) -> usize {
    let total: u32 = w.iter().sum();
    let mut roll = rng.below(total.max(1));
    for (i, x) in w.iter().enumerate() {
        if roll < *x {
            return i;
        }
        roll -= x;
    }
    w.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roster(seed: u64) -> (Roster, TraitFile, NameFile) {
        let t = TraitFile::shipped();
        let n = NameFile::shipped();
        let mut r = Roster::new();
        r.found(&t, &n, seed, 1);
        (r, t, n)
    }

    #[test]
    fn a_run_is_the_shift_and_its_injury_and_the_trait_lean_reach_the_target() {
        let (mut r, t, _) = roster(11);
        let id = r.eligible()[0].id;
        {
            let rec = r.get_mut(id).unwrap();
            rec.traits = ["chromehead".into(), "quick".into()];
            rec.injuries.clear();
        }
        r.depart(id, &[]);
        r.apply_run(&t, &RunReport { runner: id, extracted: true, pulls: 1, items_taken: 2, kills: 1, hits_survived: 2, injuries: vec![Injury::FungalFever], ..Default::default() });
        let rec = r.get(id).unwrap();
        assert_eq!((rec.runs, rec.survived, rec.hauls, rec.kills), (1, 1, 1, 1));
        assert_eq!(rec.xp[Skill::Scav.index()], 4);
        assert_eq!(rec.xp[Skill::Fight.index()], 5);
        assert_eq!(rec.shift_job, Some(Job::Scav), "a run is the Scav job");
        assert_eq!(rec.injuries, vec![(Injury::FungalFever, 8)]);
        r.end_shift(&t, &ShiftWork::default(), |_| ShiftContext { on_mat: true, signalled: true, lit: true, ..Default::default() });
        let rec = r.get(id).unwrap();
        // Fed 85 and the rest at 88+ : five needs over 80 (+25); Chromehead -5; the fever -5
        assert_eq!(rec.need(Need::Fed), 85);
        assert_eq!(rec.need(Need::Rest), 88);
        assert_eq!(rec.mood_target, 75, "target 60 + 25 - 5 - 5");
        assert_eq!(rec.injuries, vec![(Injury::FungalFever, 7)]);
    }

    #[test]
    fn the_forty_traits_validate_and_no_excluded_pair_is_drawn() {
        let t = TraitFile::shipped();
        assert_eq!(t.traits.len(), 40);
        let n = NameFile::shipped();
        let mut r = Roster::new();
        let mut rng = Pcg32::new(5);
        for _ in 0..300 {
            r.create(&t, &n, &mut rng, 1, false);
        }
        for rec in &r.records {
            let a = t.get(&rec.traits[0]).unwrap();
            let b = t.get(&rec.traits[1]).unwrap();
            assert_ne!(a.id, b.id);
            assert!(!a.excludes.contains(&b.id) && !b.excludes.contains(&a.id), "{} with {}", a.id, b.id);
            for s in rec.stats {
                assert!((1..=10).contains(&s), "{:?}", rec.stats);
            }
            // Mat-sense is the depth's, plus at most a trait's lean (Mat-born)
            assert!((rec.stat(Stat::MatSense) - rec.depth.matsense()).abs() <= 1 || rec.depth == Depth::D5);
        }
        // the depth table lands near its weights
        let d2 = r.records.iter().filter(|x| x.depth == Depth::D2).count();
        assert!((90..=150).contains(&d2), "D2 {d2} of 300");
        // three of the first ten names differ at least
        let names: std::collections::HashSet<&str> = r.records.iter().take(10).map(|x| x.name.as_str()).collect();
        assert!(names.len() >= 3);
    }

    #[test]
    fn founders_can_run_and_the_runner_setup_reads_the_body() {
        let (r, t, _) = roster(11);
        let e = r.eligible();
        assert!(e.len() >= 3, "{} eligible of {}", e.len(), r.records.len());
        let s = e[0].runner_setup(&t);
        assert_eq!(s.body as i32, e[0].stat(Stat::Body));
        assert_eq!(s.veins, e[0].depth.veins());
        assert!(s.rig_rows >= 5 && s.rig_rows <= 7);
    }

    #[test]
    fn needs_move_by_the_rates_and_mood_chases_its_target_at_bounded_speed() {
        let (mut r, t, _) = roster(3);
        let id = r.records.iter().find(|x| x.depth == Depth::D2).map(|x| x.id).unwrap_or(r.records[0].id);
        let rec = r.get_mut(id).unwrap();
        rec.traits = ["loud".into(), "brawler".into()]; // no need rates, no Mood lean
        rec.injuries.clear();
        // a full shift of work on signalled, lit, dry Mat: Fed -15, Rest -12, the rest up
        rec.needs = [100, 100, 100, 100, 100, 60];
        rec.shift_job = Some(Job::Scav);
        rec.end_shift(&t, ShiftContext { on_mat: true, signalled: true, lit: true, ..Default::default() });
        assert_eq!(rec.need(Need::Fed), 85);
        assert_eq!(rec.need(Need::Rest), 88);
        // every need above 80: target 60 + 5 x 5 = 85, and the gauge climbs 4
        assert_eq!(rec.mood_target, 85);
        assert_eq!(rec.mood(), 64);
        // hungry, wet and off the Mat: Fed 10 (-20), Dry 50 -> 25 (-10), Signal 50 -> 30 (-10): target 20,
        // the gauge falls 3
        rec.needs = [10, 50, 50, 50, 50, 64];
        rec.shift_job = None;
        rec.end_shift(&t, ShiftContext { wet: true, ..Default::default() });
        assert_eq!(rec.need(Need::Dry), 25);
        assert_eq!(rec.need(Need::Signal), 30);
        assert_eq!(rec.mood_target, 20);
        assert_eq!(rec.mood(), 61);
        // Fed at 0 costs a point of Body
        let body = rec.stat(Stat::Body);
        rec.needs[Need::Fed.index()] = 5;
        rec.end_shift(&t, ShiftContext::default());
        assert_eq!(rec.need(Need::Fed), 0);
        assert_eq!(rec.stat(Stat::Body), body - 1);
        // the unrest line reads the gauge
        rec.needs[Need::Mood.index()] = 25;
        assert_eq!(rec.unrest_line(&t), 1, "strike");
        rec.needs[Need::Mood.index()] = 15;
        assert_eq!(rec.unrest_line(&t), 2, "defection");
        rec.needs[Need::Mood.index()] = 5;
        assert_eq!(rec.unrest_line(&t), 3, "petition");
        rec.traits[0] = "devout".into();
        rec.needs[Need::Mood.index()] = 22;
        assert_eq!(rec.unrest_line(&t), 2, "a Devout defects at 25");
    }

    #[test]
    fn utility_follows_priority_skill_and_traits_and_the_body_forbids() {
        let (mut r, t, n) = roster(7);
        let mut rng = Pcg32::new(99);
        // force a D4 and a D1 into the roster for the forbids
        let mut d4 = None;
        let mut d1 = None;
        for _ in 0..200 {
            let id = r.create(&t, &n, &mut rng, 1, true);
            match r.get(id).unwrap().depth {
                Depth::D4 if d4.is_none() => d4 = Some(id),
                Depth::D1 if d1.is_none() => d1 = Some(id),
                _ => {}
            }
            if d4.is_some() && d1.is_some() {
                break;
            }
        }
        let d4 = r.get(d4.unwrap()).unwrap();
        assert!(d4.utility(&t, Job::Scav).is_none() && d4.utility(&t, Job::Tend).is_some());
        let d1 = r.get(d1.unwrap()).unwrap();
        assert!(d1.utility(&t, Job::Tend).is_none());
        // a plain D2: priority 4 beats 2, skill raises it, a loved job raises it 1.4x
        let id = r.records.iter().find(|x| x.depth == Depth::D2).unwrap().id;
        let rec = r.get_mut(id).unwrap();
        rec.traits = ["hoarder".into(), "iron_gut".into()];
        rec.priorities[Job::Scav.index()] = 2;
        rec.priorities[Job::Haul.index()] = 2;
        rec.skills[Skill::Scav.index()] = 4;
        let scav = rec.utility(&t, Job::Scav).unwrap();
        let haul = rec.utility(&t, Job::Haul).unwrap();
        assert!((scav - haul * 1.4).abs() < 1e-4, "scav {scav} haul {haul}");
        rec.priorities[Job::Haul.index()] = 4;
        assert!(rec.utility(&t, Job::Haul).unwrap() > scav);
        rec.needs[Need::Mood.index()] = 15;
        assert!(rec.utility(&t, Job::Scav).is_none(), "a slimo under Mood 20 refuses");
        rec.needs[Need::Mood.index()] = 60;
        assert!(rec.utility(&t, Job::Research).is_none(), "Research needs Tweak 5");
        rec.skills[Skill::Tweak.index()] = 5;
        assert!(rec.utility(&t, Job::Research).is_some());
    }

    #[test]
    fn a_run_pays_xp_forms_bonds_and_buries_the_dead() {
        let (mut r, t, _) = roster(21);
        let e: Vec<u32> = r.eligible().iter().map(|x| x.id).collect();
        let (runner, f1, f2) = (e[0], e[1], e[2]);
        let scav0 = r.get(runner).unwrap().skill(Skill::Scav);
        let xp0 = r.get(runner).unwrap().xp[Skill::Scav.index()];
        r.depart(runner, &[f1, f2]);
        assert_eq!(r.get(runner).unwrap().status, Status::OnRun);
        let rep = RunReport { runner, followers: vec![f1, f2], extracted: true, pulls: 12, items_taken: 20, kills: 2, hits_survived: 3, follower_strikes: 4, followers_extracted: vec![f1, f2], injuries: vec![Injury::Fracture], ..Default::default() };
        for _ in 0..3 {
            r.apply_run(&t, &rep);
        }
        let rr = r.get(runner).unwrap();
        assert!(rr.skill(Skill::Scav) > scav0 || rr.xp[Skill::Scav.index()] > xp0);
        assert_eq!((rr.runs, rr.survived, rr.kills, rr.hauls), (3, 3, 6, 36));
        assert_eq!(rr.injuries.len(), 1);
        assert!(rr.bonds.iter().any(|b| b.with == f1 && b.bonded), "three runs together bond: {:?}", rr.bonds);
        // the fourth run kills a follower
        let rep2 = RunReport { runner, followers: vec![f1, f2], extracted: true, followers_extracted: vec![f1], followers_dead: vec![f2], ..Default::default() };
        r.apply_run(&t, &rep2);
        assert!(r.get(f2).is_none() && r.memorial.iter().any(|m| m.id == f2));
        let runner_rec = r.get(runner).unwrap().clone();
        let voice = r.death_voice_for(&runner_rec, &r.memorial);
        assert_eq!(voice, r.memorial[0].stat(Stat::Voice) * 2, "a bonded partner's death counts double");
    }
}
