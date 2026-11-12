//! Ivan's barks (Full Design v3.19, Narrative and Ivan: Bark pools; research 22). A line is tagged by event and
//! optionally by Ivan's Authority state, the strain's dominant class, the Dissolution Rite or a chapter beat. The pool
//! for a moment is every line of the event whose tags all hold; it is a shuffled bag that empties before it repeats,
//! its most specific lines first in each round (Valve's rule). Ivan speaks at most once in BARK_GAP ticks; a death or
//! a chapter beat takes the place of a waiting line; a line shows for BARK_SHOWS ticks.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::strings::fill;

const BARKS_TOML: &str = include_str!("../../../../data/barks.toml");

/// Twenty seconds of the city's clock (2 Hz) between lines.
pub const BARK_GAP: u64 = 40;
/// Six seconds on screen.
pub const BARK_SHOWS: u64 = 12;

/// The fourteen event classes, then the chapter beat.
pub const EVENTS: [&str; 15] = ["haul", "rare", "blackline", "runner_down", "runner_dead", "newcomer", "d5_fed", "patrol", "sweep", "cleansing", "bloom", "flood", "law", "stratum", "beat"];

#[derive(Clone, Debug, Deserialize)]
pub struct BarkDef {
    pub event: String,
    #[serde(default)]
    pub state: u8,
    #[serde(default)]
    pub class: String,
    #[serde(default)]
    pub rite: bool,
    #[serde(default)]
    pub beat: u8,
    pub text: String,
}

impl BarkDef {
    /// How many of its tags it carries: the more, the sooner in a round.
    fn specificity(&self) -> u8 {
        (self.state > 0) as u8 + !self.class.is_empty() as u8 + self.rite as u8 + (self.beat > 0) as u8
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BarkFile {
    pub bark: Vec<BarkDef>,
}

impl BarkFile {
    pub fn shipped() -> &'static BarkFile {
        static F: OnceLock<BarkFile> = OnceLock::new();
        F.get_or_init(|| {
            let f: BarkFile = toml::from_str(BARKS_TOML).expect("data/barks.toml");
            assert!(f.bark.iter().all(|b| EVENTS.contains(&b.event.as_str())), "a bark of an unknown event");
            f
        })
    }
}

/// The moment a line is chosen for: Ivan's Authority state (1-4), the strain's dominant class, the Rite, the beat.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ctx {
    pub state: u8,
    pub class: Option<String>,
    pub rite: bool,
    pub beat: u8,
}

impl Ctx {
    fn holds(&self, b: &BarkDef) -> bool {
        (b.state == 0 || b.state == self.state) && (b.class.is_empty() || self.class.as_deref() == Some(b.class.as_str())) && (!b.rite || self.rite) && (b.beat == 0 || b.beat == self.beat)
    }

    fn key(&self, event: &str) -> String {
        format!("{event}|{}|{}|{}|{}", self.state, self.class.as_deref().unwrap_or("-"), self.rite as u8, self.beat)
    }
}

/// A death or a chapter beat takes the place of a waiting line.
fn urgent(event: &str) -> bool {
    matches!(event, "runner_down" | "runner_dead" | "beat")
}

/// A line waiting for the gap: its event, tokens and moment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Pending {
    event: String,
    extra: Vec<(String, String)>,
    state: u8,
    class: Option<String>,
    rite: bool,
    beat: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Barker {
    /// each pool's bag: the lines left this round, in the order they come
    bags: BTreeMap<String, Vec<u16>>,
    /// the line on screen and the tick it leaves
    pub line: Option<(String, u64)>,
    /// the tick of the last line said
    last: Option<u64>,
    /// a line waiting for the gap: (event, extra tokens, its moment)
    waiting: Option<Pending>,
    /// the bags' shuffle
    seed: u64,
    /// lines said over the run
    pub said: u32,
    /// Ivan has fallen: he says nothing more (Pass 1)
    pub silent: bool,
}

impl Barker {
    pub fn new(seed: u64) -> Barker {
        Barker { seed: seed ^ 0xBA2C, ..Default::default() }
    }

    fn next_rand(&mut self) -> u64 {
        // splitmix64
        self.seed = self.seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// The next line of an event's pool for a moment, filling and drawing its bag.
    pub fn draw(&mut self, event: &str, ctx: &Ctx) -> Option<String> {
        let f = BarkFile::shipped();
        let key = ctx.key(event);
        if self.bags.get(&key).is_none_or(|b| b.is_empty()) {
            let pool: Vec<u16> = f.bark.iter().enumerate().filter(|(_, b)| b.event == event && ctx.holds(b)).map(|(k, _)| k as u16).collect();
            if pool.is_empty() {
                return None;
            }
            let mut tiers: BTreeMap<u8, Vec<u16>> = BTreeMap::new();
            for k in pool {
                tiers.entry(f.bark[k as usize].specificity()).or_default().push(k);
            }
            let mut bag = Vec::new();
            for (_, mut t) in tiers.into_iter().rev() {
                for i in (1..t.len()).rev() {
                    let j = (self.next_rand() % (i as u64 + 1)) as usize;
                    t.swap(i, j);
                }
                bag.extend(t);
            }
            // the bag is drawn from its end: the most specific last in, first out
            bag.reverse();
            self.bags.insert(key.clone(), bag);
        }
        let k = self.bags.get_mut(&key)?.pop()?;
        Some(f.bark[k as usize].text.clone())
    }

    /// Ivan has something to say about `event`: said now if the gap allows, else it waits (a death or a beat taking the
    /// place of what waits; otherwise the newest waits).
    pub fn say(&mut self, event: &str, ctx: &Ctx, extra: &[(&str, &str)], tick: u64) {
        if self.silent {
            return;
        }
        let ready = self.last.is_none_or(|t| tick >= t + BARK_GAP);
        if ready {
            if let Some(text) = self.draw(event, ctx) {
                self.show(&text, extra, tick);
            }
            return;
        }
        let keep = self.waiting.as_ref().is_some_and(|w| urgent(&w.event) && !urgent(event));
        if !keep {
            self.waiting = Some(Pending { event: event.to_string(), extra: extra.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(), state: ctx.state, class: ctx.class.clone(), rite: ctx.rite, beat: ctx.beat });
        }
    }

    fn show(&mut self, text: &str, extra: &[(&str, &str)], tick: u64) {
        self.line = Some((fill(text, extra), tick + BARK_SHOWS));
        self.last = Some(tick);
        self.said += 1;
    }

    /// The clock: a waiting line is said once the gap has passed; a shown line leaves when its time is up.
    pub fn tick(&mut self, tick: u64) {
        if self.line.as_ref().is_some_and(|l| tick >= l.1) {
            self.line = None;
        }
        if self.silent || self.waiting.is_none() || self.last.is_some_and(|t| tick < t + BARK_GAP) {
            return;
        }
        let p = self.waiting.take().unwrap();
        let ctx = Ctx { state: p.state, class: p.class, rite: p.rite, beat: p.beat };
        if let Some(text) = self.draw(&p.event, &ctx) {
            let ex: Vec<(&str, &str)> = p.extra.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            self.show(&text, &ex, tick);
        }
    }

    /// The line on screen now, if any.
    pub fn current(&self) -> Option<&str> {
        self.line.as_ref().map(|l| l.0.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_event_has_its_twenty_six_and_a_moment_draws_twelve_before_a_repeat_the_most_specific_first() {
        let f = BarkFile::shipped();
        for e in &EVENTS[..14] {
            assert_eq!(f.bark.iter().filter(|b| b.event == *e && !b.rite).count(), 26, "{e}");
        }
        assert_eq!(f.bark.len(), 379);
        let ctx = Ctx { state: 2, class: Some("hive".into()), rite: false, beat: 0 };
        let mut b = Barker::new(7);
        let round: Vec<String> = (0..12).map(|_| b.draw("runner_dead", &ctx).unwrap()).collect();
        let mut uniq = round.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), 12, "a round of twelve, none twice");
        // the class's and the state's lines (one tag each) come before the untagged eight
        let tagged: Vec<&str> = f.bark.iter().filter(|x| x.event == "runner_dead" && (x.class == "hive" || x.state == 2)).map(|x| x.text.as_str()).collect();
        assert_eq!(tagged.len(), 4);
        assert!(round[..4].iter().all(|r| tagged.contains(&r.as_str())), "the most specific first: {:?}", &round[..4]);
        // no class held: ten lines
        let plain = Ctx { state: 1, class: None, rite: false, beat: 0 };
        let mut c = Barker::new(3);
        let ten: Vec<String> = (0..10).map(|_| c.draw("flood", &plain).unwrap()).collect();
        let mut u = ten.clone();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), 10);
    }

    #[test]
    fn ivan_waits_twenty_seconds_between_lines_and_a_death_takes_the_waiting_place() {
        let ctx = Ctx { state: 1, ..Default::default() };
        let mut b = Barker::new(1);
        b.say("haul", &ctx, &[], 100);
        assert!(b.current().is_some());
        b.say("flood", &ctx, &[], 105);
        b.say("runner_dead", &ctx, &[("name", "Kerr")], 106);
        b.say("law", &ctx, &[("name", "Curfew")], 107);
        b.tick(120);
        assert!(b.current().is_none(), "the haul's line left after six seconds");
        b.tick(140);
        let line = b.current().unwrap().to_string();
        assert!(BarkFile::shipped().bark.iter().filter(|x| x.event == "runner_dead").any(|x| fill(&x.text, &[("name", "Kerr")]) == line), "the death, not the law: {line}");
        assert_eq!(b.said, 2);
        b.silent = true;
        b.say("haul", &ctx, &[], 300);
        assert_eq!(b.said, 2, "a fallen Ivan says nothing");
    }
}
