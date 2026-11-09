//! The cadence log (Full Design v3.6, The cadence budget and its log): one entry per event in three classes —
//! landings (a reward arrives; an open is a *pull*), losses, decisions — with the run clock, and the report that
//! checks a run against the budget. `RunSim` feeds it every tick; the Outcome carries it; the game writes it to
//! the save folder and `rose cadence` aggregates bot runs.

use crate::creature::TICK_HZ;
use crate::sim::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Landing,
    Loss,
    Decision,
}

impl Class {
    pub fn tag(self) -> &'static str {
        match self {
            Class::Landing => "LAND",
            Class::Loss => "LOSS",
            Class::Decision => "DECI",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub tick: u64,
    pub class: Class,
    pub kind: &'static str,
    pub value: i32,
    pub detail: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CadenceLog {
    pub entries: Vec<Entry>,
    /// tick of the last engagement decision, so a fight is one decision and not every swing
    last_engage: Option<u64>,
}

/// The budget, per route at launch, for a player (Full Design, The cadence budget).
pub const PULL_GAP_S: (f32, f32) = (60.0, 120.0);
pub const LANDING_GAP_S: (f32, f32) = (10.0, 30.0);
pub const LOSS_GAP_MAX_S: f32 = 120.0;
pub const DEATH_RATE: (f32, f32) = (0.10, 0.30);
pub const ENGAGE_WINDOW_S: f32 = 10.0;
/// The yardstick bot walks the lit route from source to source and never waits, reads or hesitates; the design
/// takes it as three times faster than a cautious player until a recorded human run calibrates the factor (Phase
/// F playtest). The bot's bands are the player's divided by this.
pub const BOT_FACTOR: f32 = 3.0;

/// The bot's bands: (pull gap, landing gap, longest quiet) in seconds.
pub fn bot_bands() -> ((f32, f32), (f32, f32), f32) {
    ((PULL_GAP_S.0 / BOT_FACTOR, PULL_GAP_S.1 / BOT_FACTOR), (LANDING_GAP_S.0 / BOT_FACTOR, LANDING_GAP_S.1 / BOT_FACTOR), LOSS_GAP_MAX_S / BOT_FACTOR)
}

impl CadenceLog {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, tick: u64, class: Class, kind: &'static str, value: i32, detail: String) {
        self.entries.push(Entry { tick, class, kind, value, detail });
    }

    /// Record this tick's events. `source_kind` names a loot source by index for the detail column.
    pub fn record(&mut self, tick: u64, events: &[Event], source_kind: impl Fn(usize) -> String) {
        for e in events {
            match e {
                Event::Opened { source, revealed, notable, forced, generous, .. } => {
                    let mut d = source_kind(*source);
                    if *forced {
                        d.push_str(" forced");
                    }
                    if *generous {
                        d.push_str(" first-run");
                    }
                    self.push(tick, Class::Decision, "open", *revealed as i32, d.clone());
                    self.push(tick, Class::Landing, "pull", *revealed as i32, d);
                    if *notable > 0 {
                        self.push(tick, Class::Landing, "notable", *notable as i32, source_kind(*source));
                    }
                }
                Event::Took { source, taken, left } => {
                    if *taken > 0 {
                        self.push(tick, Class::Landing, "take", *taken as i32, format!("{}{}", source_kind(*source), if *left > 0 { format!(" {left} left") } else { String::new() }));
                    }
                }
                Event::Dug { pile, .. } => {
                    self.push(tick, Class::Decision, "dig", 0, String::new());
                    if pile.is_some() {
                        self.push(tick, Class::Landing, "pull", 1, "deposit pile".to_string());
                    }
                }
                Event::CreatureDied { kind, .. } => {
                    self.push(tick, Class::Landing, "kill", 1, format!("{kind:?}").to_lowercase());
                }
                Event::Dropped { .. } => {
                    self.push(tick, Class::Decision, "drop", 0, String::new());
                }
                Event::Extracted => {
                    self.push(tick, Class::Decision, "extract", 0, String::new());
                    self.push(tick, Class::Landing, "extract", 0, String::new());
                }
                Event::Swing { .. } | Event::Shot { by_runner: true, .. } => {
                    let window = (ENGAGE_WINDOW_S * TICK_HZ as f32) as u64;
                    if self.last_engage.is_none_or(|t| tick.saturating_sub(t) > window) {
                        self.push(tick, Class::Decision, "engage", 0, if matches!(e, Event::Swing { .. }) { "melee".into() } else { "shot".into() });
                    }
                    self.last_engage = Some(tick);
                }
                Event::Salted { .. } => self.push(tick, Class::Decision, "salt", 0, String::new()),
                Event::Ordered { order } => self.push(tick, Class::Decision, "order", 0, format!("{order:?}").to_lowercase()),
                Event::Used { id, .. } => self.push(tick, Class::Decision, "use", 0, id.clone()),
                Event::RunnerHit { section, damage, .. } => {
                    self.push(tick, Class::Loss, "hit", *damage, ["head", "torso", "arms", "legs"].get(*section).unwrap_or(&"?").to_string());
                }
                Event::Bleed { damage } => self.push(tick, Class::Loss, "bleed", *damage, String::new()),
                Event::Down => self.push(tick, Class::Loss, "down", 0, String::new()),
                Event::Dead => self.push(tick, Class::Loss, "dead", 0, String::new()),
                Event::FollowerDown { id } => self.push(tick, Class::Loss, "follower_down", 0, format!("#{id}")),
                Event::FollowerDead { id } => self.push(tick, Class::Loss, "follower_dead", 0, format!("#{id}")),
                Event::Overtime { minute, .. } => self.push(tick, Class::Loss, "overtime", *minute as i32, String::new()),
                _ => {}
            }
        }
    }

    /// Mark the end of the run (so a death closes the log with the final value).
    pub fn report(&self, ticks: u64, blind_value: u32, result: &str) -> Report {
        // entries within a second of each other are one moment (an open and its take, a kill and its salvage)
        let gaps = |pred: &dyn Fn(&Entry) -> bool| -> Vec<f32> {
            let mut moments: Vec<u64> = Vec::new();
            for e in self.entries.iter().filter(|e| pred(e)) {
                if moments.last().is_none_or(|&m| e.tick.saturating_sub(m) > TICK_HZ as u64) {
                    moments.push(e.tick);
                }
            }
            moments.windows(2).map(|w| (w[1] - w[0]) as f32 / TICK_HZ as f32).collect()
        };
        let landing_gaps = gaps(&|e| e.class == Class::Landing);
        let pull_gaps = gaps(&|e| e.kind == "pull");
        let loss_gaps = gaps(&|e| e.class == Class::Loss);
        let minutes = ticks as f32 / TICK_HZ as f32 / 60.0;
        Report {
            ticks,
            result: result.to_string(),
            landings: self.entries.iter().filter(|e| e.class == Class::Landing).count(),
            pulls: self.entries.iter().filter(|e| e.kind == "pull").count(),
            notables: self.entries.iter().filter(|e| e.kind == "notable").map(|e| e.value).sum::<i32>() as usize,
            kills: self.entries.iter().filter(|e| e.kind == "kill").count(),
            losses: self.entries.iter().filter(|e| e.class == Class::Loss).count(),
            damage: self.entries.iter().filter(|e| e.kind == "hit" || e.kind == "bleed").map(|e| e.value).sum(),
            decisions: self.entries.iter().filter(|e| e.class == Class::Decision).count(),
            landing_gap_median: median(&landing_gaps),
            landing_gap_p90: percentile(&landing_gaps, 0.9),
            pull_gap_median: median(&pull_gaps),
            pull_gap_p90: percentile(&pull_gaps, 0.9),
            loss_gap_max: loss_gaps.iter().cloned().fold(0.0, f32::max),
            blind_value,
            value_per_min: if minutes > 0.0 { blind_value as f32 / minutes } else { 0.0 },
        }
    }

    /// The log as text: a header, one line per entry, the report.
    pub fn text(&self, header: &str, report: &Report) -> String {
        let mut s = String::new();
        s.push_str(header);
        s.push('\n');
        s.push_str("  clock   class  kind           value  detail\n");
        for e in &self.entries {
            s.push_str(&format!("  {}  {}   {:<13}  {:>5}  {}\n", clock(e.tick), e.class.tag(), e.kind, e.value, e.detail));
        }
        s.push('\n');
        s.push_str(&report.text());
        s
    }
}

pub fn clock(tick: u64) -> String {
    let s = tick as f32 / TICK_HZ as f32;
    format!("{:02}:{:04.1}", (s / 60.0) as u32, s % 60.0)
}

fn median(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut w = v.to_vec();
    w.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = w.len();
    if n % 2 == 1 { w[n / 2] } else { (w[n / 2 - 1] + w[n / 2]) / 2.0 }
}

fn percentile(v: &[f32], p: f32) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut w = v.to_vec();
    w.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let i = ((w.len() - 1) as f32 * p).round() as usize;
    w[i.min(w.len() - 1)]
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub ticks: u64,
    pub result: String,
    pub landings: usize,
    pub pulls: usize,
    pub notables: usize,
    pub kills: usize,
    pub losses: usize,
    pub damage: i32,
    pub decisions: usize,
    pub landing_gap_median: f32,
    pub landing_gap_p90: f32,
    pub pull_gap_median: f32,
    pub pull_gap_p90: f32,
    pub loss_gap_max: f32,
    pub blind_value: u32,
    pub value_per_min: f32,
}

impl Report {
    pub fn text(&self) -> String {
        format!(
            "  run {}  {}  |  landings {} (gap median {:.0} s, p90 {:.0} s)  pulls {} (gap median {:.0} s, p90 {:.0} s)  notables {}  kills {}\n  losses {} ({} damage, longest quiet {:.0} s)  decisions {}  blind value {} e  {:.1} e/min\n  budget: pull gap {:.0}-{:.0} s  landing gap {:.0}-{:.0} s  loss gap under {:.0} s",
            clock(self.ticks), self.result, self.landings, self.landing_gap_median, self.landing_gap_p90, self.pulls, self.pull_gap_median, self.pull_gap_p90,
            self.notables, self.kills, self.losses, self.damage, self.loss_gap_max, self.decisions, self.blind_value, self.value_per_min,
            PULL_GAP_S.0, PULL_GAP_S.1, LANDING_GAP_S.0, LANDING_GAP_S.1, LOSS_GAP_MAX_S
        )
    }
}
