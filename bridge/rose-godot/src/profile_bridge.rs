//! The profile (Full Design v3.19, Meta): what outlives a run, kept by the game in `user://profile` as the save
//! crate's bytes. The menu shows its endings and Daily best, New Run offers its last code, the Codex reads it.

use godot::prelude::*;

use crate::RoseSim;

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseProfile {
    base: Base<RefCounted>,
    pub(crate) p: save::Profile,
}

#[godot_api]
impl RoseProfile {
    /// The profile from its file's bytes; false (and a fresh profile) when they do not read.
    #[func]
    fn load_bytes(&mut self, bytes: PackedByteArray) -> bool {
        match save::Profile::decode(bytes.as_slice()) {
            Some(p) => {
                self.p = p;
                true
            }
            None => {
                self.p = save::Profile::default();
                false
            }
        }
    }

    #[func]
    fn to_bytes(&self) -> PackedByteArray {
        PackedByteArray::from(self.p.encode().as_slice())
    }

    /// {endings: [{ending, name, warchest, code, shift, difficulty}], daily_best: {date, score} or {}, dailies,
    /// endless: [{seed, empire_at (-1 never), burned_at}], strainbook, scrapdex, laws, last_code}
    #[func]
    fn summary(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let chapters = story::ChapterFile::shipped();
        let mut endings = VarArray::new();
        for e in &self.p.endings {
            let mut v = VarDictionary::new();
            v.set("ending", e.ending.as_str());
            v.set("name", chapters.ending(&e.ending).map_or(e.ending.clone(), |x| x.name.clone()).as_str());
            v.set("warchest", e.warchest as i64);
            v.set("code", e.code.as_str());
            v.set("shift", e.shift as i64);
            v.set("difficulty", e.difficulty.as_str());
            endings.push(&v.to_variant());
        }
        d.set("endings", &endings);
        let mut best = VarDictionary::new();
        if let Some((date, score)) = self.p.daily_best() {
            best.set("date", date as i64);
            best.set("score", score as i64);
        }
        d.set("daily_best", &best);
        d.set("dailies", self.p.daily.len() as i64);
        let mut endless = VarArray::new();
        for e in &self.p.endless {
            let mut v = VarDictionary::new();
            v.set("seed", e.seed as i64);
            v.set("empire_at", e.empire_at.map_or(-1, |x| x as i64));
            v.set("burned_at", e.burned_at as i64);
            endless.push(&v.to_variant());
        }
        d.set("endless", &endless);
        d.set("strainbook", self.p.strainbook.len() as i64);
        d.set("scrapdex", self.p.scrapdex.len() as i64);
        d.set("laws", self.p.laws.len() as i64);
        d.set("last_code", self.p.last_code.clone().unwrap_or_default().as_str());
        d
    }

    /// The Scrapdex entries ever seen (the Codex marks them).
    #[func]
    fn scrapdex(&self) -> PackedStringArray {
        PackedStringArray::from(self.p.scrapdex.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice())
    }

    /// The Strainbook cards ever earned.
    #[func]
    fn strainbook(&self) -> PackedStringArray {
        PackedStringArray::from(self.p.strainbook.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice())
    }

    /// The day's score, or -1 when the day has not been tried.
    #[func]
    fn daily_of(&self, date: i64) -> i64 {
        self.p.daily.iter().find(|(d, _)| *d as i64 == date).map_or(-1, |(_, s)| *s as i64)
    }

    /// The day's one try; false when the day was already tried.
    #[func]
    fn daily_score(&mut self, date: i64, score: i64) -> bool {
        self.p.daily_score(date.max(0) as u32, score.max(0) as u32)
    }

    /// The city's Strainbook and Scrapdex into the profile (each shift's end): the new entries.
    #[func]
    fn absorb(&mut self, sim: Gd<RoseSim>) -> i64 {
        let s = sim.bind();
        game::meta::absorb(&mut self.p, &s.colony) as i64
    }

    /// A finished run into the profile: the ending with its warchest and code and the laws it signed, or an Endless
    /// city's rank. False while the run is not over.
    #[func]
    fn record_end(&mut self, sim: Gd<RoseSim>) -> bool {
        let s = sim.bind();
        let Some(c) = s.city.as_ref() else { return false };
        game::meta::record_end(&mut self.p, &s.campaign, &c.sim, &s.colony)
    }
}
