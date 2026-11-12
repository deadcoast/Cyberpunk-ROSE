//! The city's log (Full Design v3.12, The city's log): landings, losses and decisions on the city's clock (a tick
//! is half a second; the run's clock and the city's are one).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Landing,
    Loss,
    Decision,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Landing => "landing",
            Class::Loss => "loss",
            Class::Decision => "decision",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub tick: u64,
    pub class: Class,
    pub what: String,
}

impl Event {
    pub fn seconds(&self) -> f32 {
        self.tick as f32 / 2.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct CityLog {
    pub events: Vec<Event>,
    /// how many the view has already taken
    pub shown: usize,
    /// what Ivan has something to say about, since he last spoke: (event, its tokens) — Full Design v3.19, barks
    pub barks: Vec<(String, Vec<(String, String)>)>,
}

impl CityLog {
    pub fn push(&mut self, tick: u64, class: Class, what: String) {
        self.events.push(Event { tick, class, what });
    }

    /// An event for Ivan's barks, with its tokens ({name}, {stratum}, beat).
    pub fn bark(&mut self, event: &str, extra: &[(&str, &str)]) {
        self.barks.push((event.to_string(), extra.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()));
    }

    /// The events the view has not taken yet.
    pub fn take_new(&mut self) -> Vec<Event> {
        let v = self.events[self.shown..].to_vec();
        self.shown = self.events.len();
        v
    }

    pub fn count(&self, class: Class) -> usize {
        self.events.iter().filter(|e| e.class == class).count()
    }
}
