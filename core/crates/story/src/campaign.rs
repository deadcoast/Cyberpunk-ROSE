//! The campaign's state as a save keeps it (Full Design v3.19, Narrative and Ivan): the mode and difficulty, the open
//! chapter, the accord, the chapter's Bloom, the alliance, the Great Cleansing, the ending and the warchest, the panel
//! sets waiting to be played and those seen. The game crate moves it (`game::campaign`); these are its words.

use serde::{Deserialize, Serialize};

use crate::chapters::{ChapterFile, Panel};
use crate::strings::fill;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// the seven chapters to an ending
    #[default]
    Campaign,
    /// no chapters: the stack, the threats and growth with the Mat until the Heart burns
    Endless,
}

/// The difficulty (Full Design v3.19, the Game shell): Sump, Culvert (the design's values), Undercroft.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Sump,
    #[default]
    Culvert,
    Undercroft,
}

impl Difficulty {
    /// Attention a zone loses a shift.
    pub fn decay(self) -> f32 {
        match self {
            Difficulty::Sump => 3.0,
            Difficulty::Culvert => 2.0,
            Difficulty::Undercroft => 1.0,
        }
    }

    /// Raiders roll for a Runner's corpse on the routes: never in Sump.
    pub fn raid_corpses(self) -> bool {
        self != Difficulty::Sump
    }

    /// A dead Runner's Spored gear: only the first piece regrows in Undercroft.
    pub fn spored_first_only(self) -> bool {
        self == Difficulty::Undercroft
    }

    /// The day's first shift floods as well as its third in Undercroft.
    pub fn dawn_flood(self) -> bool {
        self == Difficulty::Undercroft
    }

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Sump => "Sump",
            Difficulty::Culvert => "Culvert",
            Difficulty::Undercroft => "Undercroft",
        }
    }

    pub fn from_name(s: &str) -> Option<Difficulty> {
        [Difficulty::Sump, Difficulty::Culvert, Difficulty::Undercroft].into_iter().find(|d| d.name().eq_ignore_ascii_case(s))
    }
}

/// Ivan on the map (Full Design v3.19, Narrative and Ivan): where he stands and walks, his HP, the shift each verb was
/// last used. He walks the Mat only, a tile every two ticks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ivan {
    pub x: u16,
    pub y: u16,
    pub goal: Option<(u16, u16)>,
    pub hp: f32,
    pub fell: bool,
    /// the shift each verb was last used (0 never)
    pub rally: u32,
    pub parley: u32,
    pub feed: u32,
    pub ooze: u32,
}

/// Ivan's HP: a Runner's 130 twice, a D4's body (this design's).
pub const IVAN_HP: f32 = 260.0;
/// The threats know him by this id among the city's slimos.
pub const IVAN_ID: u32 = u32::MAX - 1;

impl Default for Ivan {
    fn default() -> Self {
        Ivan { x: 0, y: 0, goal: None, hp: IVAN_HP, fell: false, rally: 0, parley: 0, feed: 0, ooze: 0 }
    }
}

/// Chapter 3's envoy, answered or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accord {
    Waiting { since: u32 },
    Accepted { on: u32 },
    Refused { on: u32 },
}

/// The shifts an envoy waits before its offer counts as refused (Full Design v3.19: "refused if it stands unanswered at
/// the second shift's end").
pub const ENVOY_WAITS: u32 = 2;
/// The accepted accord's calm before its Cleansing comes.
pub const ACCORD_CALM: u32 = 2;
/// The Great Cleansing comes this many shifts after chapter 7 opens.
pub const GREAT_IN: u32 = 6;
/// The chapter's Bloom: its tiles at the start.
pub const BLOOM_TILES: usize = 40;
/// The goal of chapter 4 by taming: crawlers standing tamed.
pub const TAMED_GOAL: usize = 2;
/// Chapter 1's goal: a Hamlet (Pass 2's first tier, 20), every one with a bed.
pub const SEEPAGE_SLIMOS: usize = 20;
/// A chapter's beat: Authority (the Charter's deeds).
pub const BEAT_AUTHORITY: i32 = 2;
/// Up to this many Undercroft grates send the Great Cleansing.
pub const GREAT_GRATES: usize = 3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Campaign {
    pub mode: Mode,
    pub difficulty: Difficulty,
    /// the run's seed (the strain code carries it) and the strain code it was founded on (New Game+), if any
    pub seed: u32,
    pub founded_on: Option<String>,
    /// the open chapter, 1-7; 8 once an ending is reached (none in Endless)
    pub chapter: u8,
    pub open_since: u32,
    /// (chapter, the shift it closed)
    pub closed: Vec<(u8, u32)>,
    pub accord: Option<Accord>,
    /// the envoy's zone (its grate)
    pub envoy_zone: Option<u16>,
    /// a scripted Cleansing: the shift it comes, its share, its zones; whether it has been sent
    pub cleansing_at: Option<u32>,
    pub cleansing_share: f32,
    pub cleansing_zones: Vec<u16>,
    pub cleansing_sent: bool,
    /// chapter 4's Bloom has been grown; the newcomer it took
    pub bloom: bool,
    pub feral_name: String,
    /// chapter 5's teller: a Corpo-Past slimo's name, or "a Runner"
    pub teller: String,
    /// a Runner home from the Vault (S5) with a Rare or better
    pub vault_home: bool,
    /// the Great Cleansing's shift
    pub great_at: Option<u32>,
    pub ending: Option<String>,
    pub warchest: Option<u32>,
    /// the shift the ending came (an Endless city: the shift its Heart burned)
    pub ended_at: Option<u32>,
    /// Endless Sump's rank: the shift the Empire's card landed on
    pub empire_at: Option<u32>,
    pub ivan_fell: bool,
    /// panel sets waiting for the screen ("ch3.open", "ch3.close", "end.garden"), and those seen
    pub panels: Vec<String>,
    pub seen: Vec<String>,
    /// the Tithe's measure: Junk in the Stash at the last shift's end
    pub junk_last: u32,
    /// Ivan on the map and his voice
    #[serde(default)]
    pub ivan: Ivan,
    #[serde(default)]
    pub barker: crate::barks::Barker,
    /// the first ten minutes' beats (Full Design v3.21, Onboarding)
    #[serde(default)]
    pub onboarding: crate::onboarding::Onboarding,
}

impl Default for Campaign {
    fn default() -> Self {
        Campaign::new(Mode::Campaign, Difficulty::Culvert)
    }
}

impl Campaign {
    pub fn new(mode: Mode, difficulty: Difficulty) -> Campaign {
        let campaign = mode == Mode::Campaign;
        Campaign {
            mode,
            difficulty,
            seed: 0,
            founded_on: None,
            chapter: if campaign { 1 } else { 0 },
            open_since: 1,
            closed: Vec::new(),
            accord: None,
            envoy_zone: None,
            cleansing_at: None,
            cleansing_share: 1.0,
            cleansing_zones: Vec::new(),
            cleansing_sent: false,
            bloom: false,
            feral_name: String::new(),
            teller: String::new(),
            vault_home: false,
            great_at: None,
            ending: None,
            warchest: None,
            ended_at: None,
            empire_at: None,
            ivan_fell: false,
            panels: if campaign { vec!["ch1.open".into()] } else { Vec::new() },
            seen: Vec::new(),
            junk_last: 0,
            ivan: Ivan::default(),
            barker: crate::barks::Barker::new(0),
            onboarding: crate::onboarding::Onboarding::default(),
        }
    }

    pub fn over(&self) -> bool {
        self.ending.is_some()
    }

    /// The HUD's chapter card ("CH 3 THE HAND: ANSWER THE ENVOY"); the Great Cleansing's count in chapter 7.
    pub fn card(&self, day: u32) -> String {
        if self.mode == Mode::Endless {
            return match (self.over(), self.empire_at) {
                (true, _) => "ENDLESS SUMP: THE HEART BURNED".into(),
                (false, Some(at)) => format!("ENDLESS SUMP: THE EMPIRE AT SHIFT {at}"),
                (false, None) => "ENDLESS SUMP".into(),
            };
        }
        if let Some(e) = &self.ending {
            return format!("THE END: {}", ChapterFile::shipped().ending(e).map_or(e.to_uppercase(), |d| d.name.to_uppercase()));
        }
        let f = ChapterFile::shipped();
        let Some(c) = f.get(self.chapter) else { return String::new() };
        let goal = match (self.chapter, self.great_at) {
            (3, _) if matches!(self.accord, Some(Accord::Waiting { .. })) => "ANSWER THE ENVOY".to_string(),
            (3, _) if self.accord.is_some() => "THE ACCORD'S CLEANSING".to_string(),
            (7, Some(at)) if day < at => format!("THE GREAT CLEANSING IN {} SHIFT{}", at - day, if at - day == 1 { "" } else { "S" }),
            (7, Some(_)) => "THE GREAT CLEANSING".to_string(),
            _ => c.card.clone(),
        };
        format!("CH {} {}: {}", c.n, c.name.to_uppercase(), goal)
    }

    /// A panel set's panels with their captions filled ("ch3.close" reads the accord's answer).
    pub fn set(&self, key: &str) -> Vec<Panel> {
        let f = ChapterFile::shipped();
        let mut parts = key.split('.');
        let (Some(a), Some(b)) = (parts.next(), parts.next()) else { return Vec::new() };
        let raw: Vec<Panel> = if a == "end" {
            f.ending(b).map(|e| e.panels.clone()).unwrap_or_default()
        } else {
            let n: u8 = a.trim_start_matches("ch").parse().unwrap_or(0);
            let Some(c) = f.get(n) else { return Vec::new() };
            match b {
                "open" => c.open.clone(),
                "close" if matches!(self.accord, Some(Accord::Refused { .. })) && !c.close_refused.is_empty() => c.close_refused.clone(),
                _ => c.close.clone(),
            }
        };
        let teller = if self.teller.is_empty() { "A Runner" } else { self.teller.as_str() };
        let name = if self.feral_name.is_empty() { "one of ours" } else { self.feral_name.as_str() };
        raw.into_iter().map(|p| Panel { slot: p.slot, caption: fill(&p.caption, &[("name", name), ("teller", teller)]) }).collect()
    }

    /// The next waiting set for the screen, marked seen.
    pub fn take_panels(&mut self) -> Option<String> {
        if self.panels.is_empty() {
            return None;
        }
        let k = self.panels.remove(0);
        if !self.seen.contains(&k) {
            self.seen.push(k.clone());
        }
        Some(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_campaign_opens_on_chapter_one_with_its_panels_and_its_card() {
        let mut c = Campaign::default();
        assert_eq!(c.chapter, 1);
        assert_eq!(c.card(1), "CH 1 SEEPAGE: A HAMLET: 20 SLIMOS WITH BEDS");
        assert_eq!(c.take_panels().as_deref(), Some("ch1.open"));
        assert_eq!(c.set("ch1.open").len(), 4);
        c.chapter = 3;
        c.accord = Some(Accord::Refused { on: 9 });
        assert!(c.set("ch3.close")[1].caption.contains("said no"));
        c.chapter = 7;
        c.great_at = Some(20);
        assert_eq!(c.card(17), "CH 7 RISE: THE GREAT CLEANSING IN 3 SHIFTS");
        c.feral_name = "Kerr".into();
        assert!(c.set("ch4.close")[2].caption.starts_with("Kerr"));
        assert_eq!(Campaign::new(Mode::Endless, Difficulty::Sump).card(5), "ENDLESS SUMP");
    }
}
