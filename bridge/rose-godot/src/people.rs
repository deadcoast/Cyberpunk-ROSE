//! `RosePeople`: the roster (Full Design, Runners and slimos) held by the `Sim` autoload for the session. The Slimo
//! card and the Route Board are views of it; the run takes a `RunnerSetup` from it and hands back a `RunReport`.

use godot::prelude::*;
pub use people::RunnerSetup;
use people::{Injury, Job, Need, Record, Roster, RunReport, Skill, Stat, Status, TraitFile};

use crate::data_text;

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RosePeople {
    base: Base<RefCounted>,
    pub(crate) traits: Option<TraitFile>,
    pub(crate) names: Option<people::NameFile>,
    pub(crate) roster: Roster,
    /// the dead since the last shift end, for the survivors' Mood
    pub(crate) died_this_shift: Vec<Record>,
}

fn stats_dict(r: &Record) -> VarDictionary {
    let mut d = VarDictionary::new();
    for s in Stat::all() {
        d.set(s.name().to_lowercase().replace('-', "").as_str(), r.stat(s) as i64);
    }
    d
}

pub(crate) fn status_text(s: &Status) -> String {
    match s {
        Status::Idle => "idle".into(),
        Status::Working(j) => format!("working {}", j.name().to_lowercase()),
        Status::Resting => "resting".into(),
        Status::OnRun => "on a run".into(),
        Status::Stranded => "stranded".into(),
        Status::Down => "down".into(),
        Status::Dead => "dead".into(),
        Status::Feral => "feral".into(),
    }
}

fn pack_i32(v: &[u32]) -> PackedInt32Array {
    PackedInt32Array::from(v.iter().map(|x| *x as i32).collect::<Vec<_>>().as_slice())
}

fn strings(v: &[String]) -> PackedStringArray {
    PackedStringArray::from(v.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice())
}

#[godot_api]
impl RosePeople {
    pub(crate) fn ensure(&mut self) {
        if self.traits.is_none() {
            self.traits = Some(TraitFile::from_toml(&data_text("traits.toml", people::TRAITS_TOML)).unwrap_or_else(|_| TraitFile::shipped()));
        }
        if self.names.is_none() {
            self.names = Some(toml::from_str(&data_text("names.toml", people::NAMES_TOML)).unwrap_or_else(|_| people::NameFile::shipped()));
        }
    }

    fn t(&self) -> &TraitFile {
        self.traits.as_ref().expect("ensure")
    }

    /// Roll the six founders from the save's seed (Full Design, Creation). Does nothing when the roster is peopled.
    #[func]
    fn found(&mut self, seed: i64, day: i64) {
        self.ensure();
        if self.roster.records.is_empty() {
            let names = self.names.clone().unwrap();
            let traits = self.traits.clone().unwrap();
            self.roster.found(&traits, &names, seed as u64, day.max(0) as u32);
        }
    }

    /// A newcomer at a drain.
    #[func]
    fn newcomer(&mut self, seed: i64, day: i64) -> i64 {
        self.ensure();
        let mut rng = mat::rng::Pcg32::new(seed as u64 ^ 0xD3A1_4000 ^ (self.roster.next_id as u64) << 8);
        let names = self.names.clone().unwrap();
        let traits = self.traits.clone().unwrap();
        self.roster.create(&traits, &names, &mut rng, day.max(0) as u32, false) as i64
    }

    #[func]
    fn count(&self) -> i64 {
        self.roster.records.len() as i64
    }

    /// Every living record's id, the Runners first (the roster's order).
    #[func]
    fn ids(&self) -> PackedInt32Array {
        pack_i32(&self.roster.records.iter().map(|r| r.id).collect::<Vec<_>>())
    }

    /// Who may go on a run now: D1-D3, up, not already out; by Scav then Fight.
    #[func]
    fn eligible(&self) -> PackedInt32Array {
        pack_i32(&self.roster.eligible().iter().map(|r| r.id).collect::<Vec<_>>())
    }

    /// The nine jobs in priority order.
    #[func]
    fn jobs(&self) -> PackedStringArray {
        PackedStringArray::from(Job::all().iter().map(|j| GString::from(j.name())).collect::<Vec<_>>().as_slice())
    }

    /// The full record as the Slimo card reads it.
    #[func]
    fn record(&mut self, id: i64) -> VarDictionary {
        self.ensure();
        let Some(r) = self.roster.get(id as u32).cloned() else { return VarDictionary::new() };
        self.record_dict(&r)
    }

    /// A memorial entry, by index.
    #[func]
    fn memorial(&mut self) -> VarArray {
        self.ensure();
        let mut out = VarArray::new();
        for r in self.roster.memorial.clone() {
            out.push(&self.record_dict(&r).to_variant());
        }
        out
    }

    fn record_dict(&self, r: &Record) -> VarDictionary {
        let traits = self.t();
        let mut d = VarDictionary::new();
        d.set("id", r.id as i64);
        d.set("name", r.name.as_str());
        d.set("epithet", r.epithet.as_str());
        d.set("full_name", r.full_name().as_str());
        d.set("depth", r.depth.tag());
        d.set("depth_name", r.depth.name());
        d.set("veins", r.depth.veins() as i64);
        d.set("past", r.past.name());
        d.set("stats", &stats_dict(r));
        let mut skills = VarArray::new();
        for s in Skill::all() {
            let mut e = VarDictionary::new();
            e.set("id", s.name().to_lowercase().as_str());
            e.set("name", s.name());
            e.set("level", r.skill(s) as i64);
            e.set("xp", r.xp[s.index()] as i64);
            e.set("next", people::xp_to_next(r.skill(s)) as i64);
            e.set("mastery", s.mastery());
            skills.push(&e.to_variant());
        }
        d.set("skills", &skills);
        let mut tr = VarArray::new();
        for id in &r.traits {
            let mut e = VarDictionary::new();
            e.set("id", id.as_str());
            if let Some(t) = traits.get(id) {
                e.set("name", t.name.as_str());
                e.set("text", t.text.as_str());
                e.set("loves", t.loves.as_str());
                e.set("hates", t.hates.as_str());
            }
            tr.push(&e.to_variant());
        }
        d.set("traits", &tr);
        let mut needs = VarDictionary::new();
        for n in Need::all() {
            needs.set(n.name().to_lowercase().as_str(), r.need(n) as i64);
        }
        d.set("needs", &needs);
        d.set("mood_target", r.mood_target as i64);
        d.set("unrest_line", r.unrest_line(traits) as i64);
        let mut inj = VarArray::new();
        for (i, shifts) in &r.injuries {
            let mut e = VarDictionary::new();
            e.set("name", i.name());
            e.set("shifts", *shifts as i64);
            inj.push(&e.to_variant());
        }
        d.set("injuries", &inj);
        d.set("priorities", &PackedInt32Array::from(r.priorities.iter().map(|p| *p as i32).collect::<Vec<_>>().as_slice()));
        d.set("schedule", &PackedStringArray::from(r.schedule.iter().map(|x| GString::from(x.name())).collect::<Vec<_>>().as_slice()));
        d.set("status", status_text(&r.status).as_str());
        d.set("can_run", r.can_run());
        d.set("runs", r.runs as i64);
        d.set("survived", r.survived as i64);
        d.set("kills", r.kills as i64);
        d.set("hauls", r.hauls as i64);
        d.set("crafts", r.crafts as i64);
        d.set("joined_day", r.joined_day as i64);
        let mut bonds = VarArray::new();
        for b in &r.bonds {
            let mut e = VarDictionary::new();
            e.set("with", b.with as i64);
            e.set("with_name", self.roster.get(b.with).map(|x| x.full_name()).unwrap_or_else(|| self.roster.memorial.iter().find(|x| x.id == b.with).map(|x| x.full_name()).unwrap_or_default()).as_str());
            e.set("runs", b.runs_together as i64);
            e.set("bonded", b.bonded);
            e.set("need", r.bond_runs(traits) as i64);
            bonds.push(&e.to_variant());
        }
        d.set("bonds", &bonds);
        // utility per job, -1 where the body, the gate or the gauges forbid
        let mut util = PackedFloat32Array::new();
        for j in Job::all() {
            util.push(r.utility(traits, j).unwrap_or(-1.0));
        }
        d.set("utility", &util);
        d
    }

    /// Set a job priority 0-4 (0 never).
    #[func]
    fn set_priority(&mut self, id: i64, job: i64, value: i64) {
        if let Some(r) = self.roster.get_mut(id as u32) {
            if (0..9).contains(&job) {
                r.priorities[job as usize] = value.clamp(0, 4) as u8;
            }
        }
    }

    /// Cycle one shift of a slimo's schedule row (0 Dim, 1 Dark, 2 Flood): Work, Rest, Any.
    #[func]
    fn cycle_schedule(&mut self, id: i64, shift: i64) -> GString {
        let Some(r) = self.roster.get_mut(id as u32) else { return GString::new() };
        let k = shift.clamp(0, 2) as usize;
        r.schedule[k] = r.schedule[k].next();
        GString::from(r.schedule[k].name())
    }

    /// What the run needs from the record: {id, name, body, nerve, hands, eye, veins, rig_rows, carry_kg, flags}.
    #[func]
    fn runner_setup(&mut self, id: i64) -> VarDictionary {
        self.ensure();
        let Some(r) = self.roster.get(id as u32) else { return VarDictionary::new() };
        let s = r.runner_setup(self.t());
        let mut d = VarDictionary::new();
        d.set("id", s.id as i64);
        d.set("name", s.name.as_str());
        d.set("body", s.body as i64);
        d.set("nerve", s.nerve as i64);
        d.set("hands", s.hands as i64);
        d.set("eye", s.eye as i64);
        d.set("veins", s.veins as i64);
        d.set("rig_rows", s.rig_rows as i64);
        d.set("carry_kg", s.carry_kg() as f64);
        d.set("back_off_pct", s.back_off_pct() as i64);
        d.set("flags", &strings(&s.flags));
        d
    }

    /// A follower's setup for `RoseRun.begin_run_rig`: {id, name, body, damage, back_off_pct, bonded}; `runner`
    /// decides the bond.
    #[func]
    fn follower_setup(&mut self, id: i64, runner: i64) -> VarDictionary {
        self.ensure();
        let Some(r) = self.roster.get(id as u32) else { return VarDictionary::new() };
        let s = r.runner_setup(self.t());
        let mut d = VarDictionary::new();
        d.set("id", s.id as i64);
        d.set("name", s.name.as_str());
        d.set("body", s.body as i64);
        // fists 4, +1 per two Hands above 5 as the Runner's melee (Full Design, The record in a run)
        d.set("damage", (4 + (s.hands as i64 - 5) / 2).max(1));
        d.set("back_off_pct", s.back_off_pct() as i64);
        d.set("bonded", r.bonds.iter().any(|b| b.with == runner as u32 && b.bonded));
        d
    }

    /// Mark the crew out on the run.
    #[func]
    fn depart(&mut self, runner: i64, followers: PackedInt32Array) {
        let fs: Vec<u32> = followers.as_slice().iter().map(|x| *x as u32).collect();
        self.roster.depart(runner as u32, &fs);
    }

    /// The run came home. `report` from `RoseRun.outcome()` plus the crew: {runner, followers, extracted,
    /// runner_died, followers_extracted, followers_stranded, followers_dead, pulls, items_taken, kills,
    /// hits_survived, follower_strikes, injuries: PackedStringArray}. Returns the record lines for the shift report.
    #[func]
    fn apply_run(&mut self, report: VarDictionary) -> PackedStringArray {
        self.ensure();
        let geti = |k: &str| report.get(k).and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0);
        let getb = |k: &str| report.get(k).and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);
        let getv = |k: &str| report.get(k).and_then(|v| v.try_to::<PackedInt32Array>().ok()).map(|a| a.as_slice().iter().map(|x| *x as u32).collect::<Vec<_>>()).unwrap_or_default();
        let injuries: Vec<Injury> = report
            .get("injuries")
            .and_then(|v| v.try_to::<PackedStringArray>().ok())
            .map(|a| {
                a.as_slice()
                    .iter()
                    .filter_map(|s| Injury::from_status(&s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let rr = RunReport {
            runner: geti("runner") as u32,
            followers: getv("followers"),
            extracted: getb("extracted"),
            runner_died: getb("runner_died"),
            followers_extracted: getv("followers_extracted"),
            followers_stranded: getv("followers_stranded"),
            followers_dead: getv("followers_dead"),
            pulls: geti("pulls") as u32,
            items_taken: geti("items_taken") as u32,
            kills: geti("kills") as u32,
            hits_survived: geti("hits_survived") as u32,
            follower_strikes: geti("follower_strikes") as u32,
            injuries,
        };
        let traits = self.traits.clone().unwrap();
        let names = self.names.clone().unwrap();
        let mut folk = game::Folk { roster: &mut self.roster, traits: &traits, names: &names, died: &mut self.died_this_shift };
        let lines = game::run_report(&mut folk, &rr);
        strings(&lines)
    }

    /// Serialise the roster (postcard) for the save; and back.
    #[func]
    fn to_bytes(&self) -> PackedByteArray {
        PackedByteArray::from(postcard::to_allocvec(&self.roster).unwrap_or_default().as_slice())
    }

    #[func]
    fn from_bytes(&mut self, bytes: PackedByteArray) -> bool {
        match postcard::from_bytes::<Roster>(bytes.to_vec().as_slice()) {
            Ok(r) => {
                self.roster = r;
                true
            }
            Err(_) => false,
        }
    }

    /// A plain Runner setup dict for a run without a record (tests, the bot).
    #[func]
    fn plain_runner(body: i64) -> VarDictionary {
        let s = RunnerSetup::plain(body.clamp(1, 10) as u8);
        let mut d = VarDictionary::new();
        d.set("id", 0i64);
        d.set("name", s.name.as_str());
        d.set("body", s.body as i64);
        d.set("nerve", s.nerve as i64);
        d.set("hands", s.hands as i64);
        d.set("eye", s.eye as i64);
        d.set("veins", s.veins as i64);
        d.set("rig_rows", s.rig_rows as i64);
        d.set("carry_kg", s.carry_kg() as f64);
        d.set("back_off_pct", 30i64);
        d.set("flags", &PackedStringArray::new());
        d
    }
}
