//! The Charter (Full Design v3.10, The Charter; Pass 2): the eight laws from `data/laws.toml`, Authority, signing
//! and repeal as a signal wave walking the Mat from the Heart, the vote, and the per-slimo question every system
//! asks: which laws have reached this tile?

use std::collections::VecDeque;

use mat::sim::hy;
use mat::MatSim;
use serde::{Deserialize, Serialize};

pub const LAWS_TOML: &str = include_str!("../../../../data/laws.toml");

/// Flags the crates read; anything else in the data is refused.
pub const KNOWN_FLAGS: [&str; 13] = [
    "meal_draw_half",
    "curfew",
    "dissolution_rite",
    "any_faction_plus_1",
    "attention_x1_5",
    "d2_militia",
    "scav_alarm_minus_20",
    "relay_auto_sever",
    "false_positive_sever",
    "tithe_t1_10",
    "foreman_cut_5",
    "self_repair_x2",
    "quotas_slower",
];

/// Tiles of cord the wave walks a shift: 16 + 32 × Conductance.
pub fn wave_speed(mat: &MatSim) -> u32 {
    (16.0 + 32.0 * mat.genome.0[mat::genome::CONDUCTANCE]).round() as u32
}

/// D5 dissolved: nutrient at the Heart's Pit (Pass 2: 0.30 of Street grade) and the per-event hit without the Rite.
pub const DISSOLUTION_NUTRIENT: f32 = 0.30;
pub const DISSOLUTION_HIT: i32 = -10;

#[derive(Clone, Debug, Deserialize)]
pub struct LawDef {
    pub id: String,
    pub name: String,
    pub authority: u32,
    #[serde(default)]
    pub mood: i32,
    pub effect: String,
    pub cost: String,
    #[serde(default)]
    pub live: String,
    #[serde(default)]
    pub offered_by: String,
    #[serde(default)]
    pub flags: Vec<String>,
}

impl LawDef {
    pub fn is_live(&self) -> bool {
        self.live.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LawFile {
    pub law: Vec<LawDef>,
}

impl LawFile {
    pub fn from_toml(s: &str) -> Result<Self, String> {
        let f: LawFile = toml::from_str(s).map_err(|e| e.to_string())?;
        if f.law.len() != 8 {
            return Err(format!("{} laws, the Charter has eight", f.law.len()));
        }
        for want in ["rationing", "open_grates", "conscription", "quarantine", "dissolution_rite", "tithe", "curfew", "foremans_cut"] {
            if !f.law.iter().any(|l| l.id == want) {
                return Err(format!("law {want} missing"));
            }
        }
        for l in &f.law {
            if l.flags.is_empty() {
                return Err(format!("{}: no effect", l.id));
            }
            for fl in &l.flags {
                if !KNOWN_FLAGS.contains(&fl.as_str()) {
                    return Err(format!("{}: unknown flag {fl}", l.id));
                }
            }
        }
        Ok(f)
    }
    pub fn shipped() -> Self {
        Self::from_toml(LAWS_TOML).expect("shipped laws")
    }
    pub fn get(&self, id: &str) -> Option<&LawDef> {
        self.law.iter().find(|l| l.id == id)
    }
}

/// A law on the Charter: the wave's reach out from the Heart, and a repeal's reach behind it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignedLaw {
    pub id: String,
    pub name: String,
    pub shift: u32,
    pub mood: i32,
    pub flags: Vec<String>,
    /// tiles of cord the signing wave has walked
    pub reach: u32,
    /// tiles of cord the repeal wave has walked
    pub repeal_reach: Option<u32>,
    pub votes_for: u32,
    pub votes_against: u32,
    /// districts it has reached (each hears it once)
    pub reached: Vec<u32>,
}

impl SignedLaw {
    /// In force at a cord distance from the Heart.
    pub fn at(&self, dist: u32) -> bool {
        self.reach >= dist && !self.repeal_reach.is_some_and(|r| r >= dist)
    }
    pub fn has(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Charter {
    pub authority: u32,
    /// Authority earned over the run (Ivan's state reads it, never the balance)
    pub earned: u32,
    pub laws: Vec<SignedLaw>,
    /// "shift N: Rationing signed, for 1 against 0" — the board at the Heart
    pub history: Vec<String>,
    /// laws an event has put on the table (the accord offers Tithe)
    pub offered: Vec<String>,
    /// a D5 was dissolved without the Rite: every slimo's target takes the hit this shift
    pub dissolution_hit: bool,
}

impl Charter {
    /// Authority by a deed.
    pub fn earn(&mut self, n: u32) {
        self.authority += n;
        self.earned += n;
    }

    /// Ivan's Authority state 1-4 from what he has earned: 0-2, 3-6, 7-12, 13+.
    pub fn ivan_state(&self) -> u8 {
        match self.earned {
            0..=2 => 1,
            3..=6 => 2,
            7..=12 => 3,
            _ => 4,
        }
    }

    /// The law on the Charter (signed, spreading or being repealed), if any.
    pub fn law(&self, id: &str) -> Option<&SignedLaw> {
        self.laws.iter().find(|l| l.id == id)
    }

    /// Why a law cannot be signed now, or None.
    pub fn cannot_sign(&self, def: &LawDef) -> Option<String> {
        if let Some(l) = self.law(&def.id) {
            if l.repeal_reach.is_none() {
                return Some("already signed".into());
            }
            return Some("its repeal is still travelling".into());
        }
        if !def.offered_by.is_empty() && !self.offered.contains(&def.id) {
            return Some(format!("only the {} offers it", def.offered_by));
        }
        if self.authority < def.authority {
            return Some(format!("needs {} Authority", def.authority));
        }
        None
    }

    /// Sign: pay the Authority, start the wave with this shift's reach, record the vote.
    pub fn sign(&mut self, def: &LawDef, shift: u32, speed: u32, votes: (u32, u32)) -> Result<(), String> {
        if let Some(why) = self.cannot_sign(def) {
            return Err(why);
        }
        self.authority -= def.authority;
        self.laws.push(SignedLaw { id: def.id.clone(), name: def.name.clone(), shift, mood: def.mood, flags: def.flags.clone(), reach: speed, repeal_reach: None, votes_for: votes.0, votes_against: votes.1, reached: Vec::new() });
        self.history.push(format!("shift {shift}: {} signed, for {} against {}", def.name, votes.0, votes.1));
        Ok(())
    }

    /// Repeal: free, no refund, the same wave behind the law.
    pub fn repeal(&mut self, id: &str, shift: u32, speed: u32) -> Result<(), String> {
        let l = self.laws.iter_mut().find(|l| l.id == id).ok_or("not on the Charter")?;
        if l.repeal_reach.is_some() {
            return Err("already being repealed".into());
        }
        l.repeal_reach = Some(speed);
        self.history.push(format!("shift {shift}: {} repealed", l.name));
        Ok(())
    }

    /// Shift end: every wave walks on; a repeal that has passed the farthest tile takes the law off the Charter.
    pub fn advance(&mut self, speed: u32, farthest: u32) {
        for l in self.laws.iter_mut() {
            l.reach = l.reach.saturating_add(speed);
            if let Some(r) = l.repeal_reach.as_mut() {
                *r = r.saturating_add(speed);
            }
        }
        self.laws.retain(|l| !l.repeal_reach.is_some_and(|r| r > farthest && r > l.reach.min(farthest)));
    }

    /// The laws in force at a cord distance.
    pub fn in_force(&self, dist: u32) -> impl Iterator<Item = &SignedLaw> {
        self.laws.iter().filter(move |l| l.at(dist))
    }

    pub fn flag_at(&self, flag: &str, dist: u32) -> bool {
        self.in_force(dist).any(|l| l.has(flag))
    }

    pub fn mood_at(&self, dist: u32) -> i32 {
        self.in_force(dist).map(|l| l.mood).sum()
    }
}

/// Cord distance from a set of start tiles over Cord and Node (u32::MAX unreachable), out to `max` steps.
pub fn cord_distances(mat: &MatSim, starts: &[usize], max: u32) -> Vec<u32> {
    let mut d = vec![u32::MAX; mat.w * mat.h];
    let mut q = VecDeque::new();
    for &s in starts {
        if s < d.len() {
            d[s] = 0;
            q.push_back(s);
        }
    }
    while let Some(i) = q.pop_front() {
        let di = d[i];
        if di >= max {
            continue;
        }
        let (x, y) = (i % mat.w, i / mat.w);
        let mut push = |n: usize| {
            if d[n] == u32::MAX && (mat.hy[n] == hy::CORD || mat.hy[n] == hy::NODE) {
                d[n] = di + 1;
                q.push_back(n);
            }
        };
        if x > 0 {
            push(i - 1);
        }
        if x + 1 < mat.w {
            push(i + 1);
        }
        if y > 0 {
            push(i - mat.w);
        }
        if y + 1 < mat.h {
            push(i + mat.w);
        }
    }
    d
}

/// The tiles within `max` cords of the starts, with their distance (a bounded search for district reach).
pub fn cord_reach(mat: &MatSim, starts: &[usize], max: u32) -> std::collections::HashMap<usize, u32> {
    let mut d = std::collections::HashMap::new();
    let mut q = VecDeque::new();
    for &s in starts {
        if s < mat.hy.len() {
            d.insert(s, 0);
            q.push_back(s);
        }
    }
    while let Some(i) = q.pop_front() {
        let di = d[&i];
        if di >= max {
            continue;
        }
        let (x, y) = (i % mat.w, i / mat.w);
        let mut next = Vec::with_capacity(4);
        if x > 0 {
            next.push(i - 1);
        }
        if x + 1 < mat.w {
            next.push(i + 1);
        }
        if y > 0 {
            next.push(i - mat.w);
        }
        if y + 1 < mat.h {
            next.push(i + mat.w);
        }
        for n in next {
            if !d.contains_key(&n) && (mat.hy[n] == hy::CORD || mat.hy[n] == hy::NODE) {
                d.insert(n, di + 1);
                q.push_back(n);
            }
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_eight_load_from_pass_2_and_a_bad_flag_is_refused() {
        let f = LawFile::shipped();
        assert_eq!(f.law.len(), 8);
        let r = f.get("rationing").unwrap();
        assert_eq!((r.authority, r.mood), (1, -5));
        assert!(r.is_live());
        assert!(!f.get("open_grates").unwrap().is_live());
        assert_eq!(f.get("tithe").unwrap().offered_by, "accord");
        let bad = LAWS_TOML.replacen("\"meal_draw_half\"", "\"free_lunch\"", 1);
        assert!(LawFile::from_toml(&bad).unwrap_err().contains("unknown flag"));
    }

    #[test]
    fn a_signed_law_walks_out_and_its_repeal_walks_after_it() {
        let f = LawFile::shipped();
        let mut c = Charter { authority: 2, ..Default::default() };
        let rat = f.get("rationing").unwrap();
        assert_eq!(c.cannot_sign(f.get("tithe").unwrap()), Some("only the accord offers it".into()));
        c.sign(rat, 3, 32, (1, 0)).unwrap();
        assert_eq!(c.authority, 1);
        assert_eq!(c.cannot_sign(rat), Some("already signed".into()));
        // the Heart and a near district have it this shift; a district 50 cords out a shift later
        assert!(c.law("rationing").unwrap().at(0) && c.law("rationing").unwrap().at(32));
        assert!(!c.law("rationing").unwrap().at(50));
        c.advance(32, 80);
        assert!(c.law("rationing").unwrap().at(50));
        assert_eq!(c.mood_at(50), -5);
        // the repeal: near first, far a shift later, then off the board
        c.repeal("rationing", 5, 32).unwrap();
        assert!(!c.law("rationing").unwrap().at(10));
        assert!(c.law("rationing").unwrap().at(50));
        c.advance(32, 80);
        assert!(!c.law("rationing").unwrap().at(50));
        c.advance(32, 80);
        assert!(c.law("rationing").is_none());
        assert_eq!(c.history.len(), 2);
        // no refund
        assert_eq!(c.authority, 1);
    }
}
