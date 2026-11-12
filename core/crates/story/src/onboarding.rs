//! The first ten minutes (Full Design v3.21, Onboarding): Pass 2's opening played as beats. Each beat is one line of
//! Ivan's and one thing on screen to touch; it fires when the city reaches it and ends when the player does it (or
//! the city does it for them). No modal text, no pause. The beats and their order are this design's.

use serde::{Deserialize, Serialize};

/// What the beats read from the city: the game crate fills it from the live state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Facts {
    /// a Pod or Bunkhouse placed (ghost or standing) beyond the founders'
    pub home_placed: bool,
    /// a Pod or Bunkhouse standing beyond the founders'
    pub home_built: bool,
    /// newcomers waiting at a grate
    pub waiting: bool,
    /// more slimos than the founders
    pub joined: bool,
    /// a feed queued, riding or landed
    pub fed: bool,
    /// runs the city has made
    pub runs: u32,
    /// a Runner out on a run now
    pub out: bool,
    /// a sale queued, riding or paid
    pub sale: bool,
    /// a sale landed and paid at a stall
    pub sold: bool,
    /// an Oathkeeper Patrol on the map
    pub patrol: bool,
    /// a district formed
    pub district: bool,
    /// the water up
    pub flood: bool,
    /// the work cycle (minutes of city time)
    pub cycle: u32,
}

/// A beat: its id (the Codex's Systems entry and the string table's), the screen element it lights, and Ivan's line.
#[derive(Debug, PartialEq)]
pub struct Beat {
    pub id: &'static str,
    /// the Codex's Systems entry's title
    pub title: &'static str,
    pub highlight: &'static str,
    pub line: &'static str,
}

/// The beats in order (this design's lines, in Ivan's voice).
pub const BEATS: [Beat; 10] = [
    Beat { id: "homes", title: "Homes", highlight: "tool:build", line: "Bodies flush down that grate soon. They need beds or they rot. BUILD a Bunkhouse on a cord." },
    Beat { id: "building", title: "Building", highlight: "ghost", line: "My tweakers walk over and raise it. The Stash pays: scrap is the only coin down here." },
    Beat { id: "newcomers", title: "Newcomers", highlight: "alert:newcomers", line: "Fresh flesh at the grate. Ten minutes to find them a bed, then the drain keeps them." },
    Beat { id: "feeding", title: "Feeding the Heart", highlight: "nav:stash", line: "Feed me, darling. STASH, pick a Chrome, FEED: the ground grows under your feet." },
    Beat { id: "runs", title: "Runs", highlight: "nav:route", line: "The scrap is up the tunnels. ROUTE: pick a run, put a weapon in a hand, go." },
    Beat { id: "hauls", title: "The haul", highlight: "nav:stash", line: "A haul! In the STASH: sell it, feed it to me, or keep it. Every piece is a choice." },
    Beat { id: "markets", title: "Markets", highlight: "nav:market", line: "The stall pays the moment it lands. Eddies buy what scrap can't: the MARKET." },
    Beat { id: "attention", title: "Attention", highlight: "alert:patrol", line: "White light at the grate: a Patrol. Being seen costs. Keep Lume and trade off that gauge." },
    Beat { id: "districts", title: "Districts", highlight: "nav:jobs", line: "Three structures on one hub: a district. They choose their own work by your JOBS rules." },
    Beat { id: "floods", title: "Floods", highlight: "clock", line: "Every third shift it rains topside and it all comes down here. Build off the low ground." },
];

/// Where the beats stand: the one showing (an index into BEATS, or BEATS.len() once done) and the cycle it fired.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Onboarding {
    pub beat: u8,
    pub since: u32,
    /// the beats that fired (the Codex's Systems entries), by id
    pub seen: Vec<String>,
    /// the beat showing has fired (its line and light are up)
    pub live: bool,
}

impl Onboarding {
    /// The beat showing, if one is live.
    pub fn showing(&self) -> Option<&'static Beat> {
        BEATS.get(self.beat as usize).filter(|_| self.live)
    }

    /// Advance on the city's facts: the live beat ends when its deed is done, and the next fires when the city has
    /// reached it. Returns the beat that fired now, if one did.
    pub fn step(&mut self, f: &Facts) -> Option<&'static Beat> {
        // the tour, in order: a beat whose deed the city has already done is skipped whole, so a player ahead of the
        // tour never waits
        while (self.beat as usize) < NOTICES {
            let k = self.beat as usize;
            if done(k, f, self.since) {
                // the system fired, shown or not: its Codex entry is the player's
                self.mark(k);
                self.beat += 1;
                self.live = false;
                continue;
            }
            if !self.live && fires(k, f) {
                self.live = true;
                self.since = f.cycle;
                self.mark(k);
                return Some(&BEATS[k]);
            }
            return None;
        }
        // the notices: each the first time its event comes, in whatever order, one on the screen at a time
        if self.live {
            if !done(self.beat as usize, f, self.since) {
                return None;
            }
            self.live = false;
        }
        for (k, b) in BEATS.iter().enumerate().skip(NOTICES) {
            if !self.seen.iter().any(|s| s == b.id) && fires(k, f) {
                self.beat = k as u8;
                self.live = true;
                self.since = f.cycle;
                self.mark(k);
                return Some(b);
            }
        }
        if BEATS[NOTICES..].iter().all(|b| self.seen.iter().any(|s| s == b.id)) {
            self.beat = BEATS.len() as u8;
        }
        None
    }

    fn mark(&mut self, k: usize) {
        if !self.seen.iter().any(|s| s == BEATS[k].id) {
            self.seen.push(BEATS[k].id.to_string());
        }
    }
}

/// The beats from here on are notices: each shows the first time its event comes, in whatever order.
const NOTICES: usize = 7;

/// Whether beat `k` can fire.
fn fires(k: usize, f: &Facts) -> bool {
    match k {
        0 => true,
        1 => f.home_placed,
        2 => f.waiting,
        3 => f.joined,
        4 => true,
        5 => f.runs >= 1,
        6 => f.sale,
        7 => f.patrol,
        8 => f.district,
        9 => f.flood,
        _ => false,
    }
}

/// Whether beat `k`'s deed is done.
fn done(k: usize, f: &Facts, since: u32) -> bool {
    match k {
        0 => f.home_placed,
        1 => f.home_built,
        2 => f.joined,
        3 => f.fed,
        4 => f.runs >= 1 || f.out,
        5 => f.sale || f.fed && f.runs >= 2,
        6 => f.sold,
        // a notice: two cycles on the screen
        7..=9 => f.cycle >= since + 2,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_beats_run_in_order_and_one_already_done_is_skipped() {
        let mut o = Onboarding::default();
        let mut f = Facts { cycle: 1, ..Default::default() };
        assert_eq!(o.step(&f).map(|b| b.id), Some("homes"));
        assert_eq!(o.step(&f), None, "a live beat fires once");
        f.home_placed = true;
        assert_eq!(o.step(&f).map(|b| b.id), Some("building"));
        f.home_built = true;
        // nobody waits yet: no beat shows
        assert_eq!(o.step(&f), None);
        assert!(o.showing().is_none());
        // a player who fed before the newcomers came: the feeding beat is skipped when its turn comes
        f.fed = true;
        f.waiting = true;
        assert_eq!(o.step(&f).map(|b| b.id), Some("newcomers"));
        f.joined = true;
        assert_eq!(o.step(&f).map(|b| b.id), Some("runs"), "feeding done already: straight to the runs");
        assert!(o.seen.contains(&"feeding".to_string()), "a system used before its beat is still the Codex's");
        // the tour ends with a sale paid; the notices come in the order their events do
        f.runs = 1;
        f.sale = true;
        f.sold = true;
        f.district = true;
        assert_eq!(o.step(&f).map(|b| b.id), Some("districts"), "no Patrol yet: the district's notice does not wait for one");
        f.patrol = true;
        assert_eq!(o.step(&f), None, "one notice on the screen at a time");
        f.cycle = 3;
        assert_eq!(o.step(&f).map(|b| b.id), Some("attention"));
        f.cycle = 5;
        f.flood = true;
        assert_eq!(o.step(&f).map(|b| b.id), Some("floods"));
        f.cycle = 7;
        assert_eq!(o.step(&f), None);
        assert_eq!(o.beat as usize, BEATS.len(), "every beat seen: the tour is over");
    }
}
