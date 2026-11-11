//! The handoff (Full Design v3.18; Pass 2: "the Mat proposes its own reroutes", bred, never bought): on a strain with
//! Plasticity and Conductance above 0.7, a jam that has stood a minute makes the Mat propose a bypass — the shortest
//! path of growable empty ground, at most 16 tiles, from the cord before the jam to the cord after it — and, confirmed,
//! the Mat lays it a tile a step from the Heart's nutrient. One stands at a time.

use std::collections::VecDeque;

use colony::Colony;
use logistics::City;
use mat::genome::{CONDUCTANCE, PLASTICITY};
use mat::sim::{hy, is_hyphal, sub};

use crate::log::{CityLog, Class};

/// Genes the handoff is bred on, and the line (Pass 2).
pub const HANDOFF_LINE: f32 = 0.7;
/// The longest bypass the Mat proposes.
pub const MOST_TILES: usize = 16;

pub fn bred(city: &City) -> bool {
    let g = &city.sim.genome.0;
    g[PLASTICITY] > HANDOFF_LINE && g[CONDUCTANCE] > HANDOFF_LINE
}

/// A bypass round the jammed tiles: from the unjammed cord beside the jam with the most signal to the one with the
/// least, over empty growable ground, the tiles in order. None when there is none within MOST_TILES.
pub fn bypass(city: &City) -> Option<Vec<u32>> {
    let sim = &city.sim;
    let (w, h) = (sim.w, sim.h);
    let nb = |i: usize| -> Vec<usize> {
        let (x, y) = (i % w, i / w);
        let mut v = Vec::with_capacity(4);
        if x > 0 {
            v.push(i - 1);
        }
        if x + 1 < w {
            v.push(i + 1);
        }
        if y > 0 {
            v.push(i - w);
        }
        if y + 1 < h {
            v.push(i + w);
        }
        v
    };
    // the cord beside the jam: hyphal, not jammed, touching a jammed tile
    let mut ends: Vec<usize> = (0..w * h).filter(|&i| is_hyphal(sim.hy[i]) && !city.jam[i] && nb(i).iter().any(|&j| city.jam[j])).collect();
    if ends.len() < 2 {
        return None;
    }
    ends.sort_by(|&a, &b| sim.sig[b].total_cmp(&sim.sig[a]).then(a.cmp(&b)));
    let (from, to) = (ends[0], *ends.last().unwrap());
    let open = |i: usize| sim.hy[i] == hy::EMPTY && !sim.salt[i] && sub::GROWABLE[sim.sub[i] as usize];
    // breadth-first over open ground from the tiles beside `from` to a tile beside `to`
    let mut prev = vec![u32::MAX; w * h];
    let mut dist = vec![u16::MAX; w * h];
    let mut q = VecDeque::new();
    for j in nb(from) {
        if open(j) {
            dist[j] = 1;
            prev[j] = from as u32;
            q.push_back(j);
        }
    }
    while let Some(i) = q.pop_front() {
        if nb(i).contains(&to) {
            let mut path = vec![i as u32];
            let mut k = i;
            while prev[k] as usize != from {
                k = prev[k] as usize;
                path.push(k as u32);
            }
            path.reverse();
            return Some(path);
        }
        if dist[i] as usize >= MOST_TILES {
            continue;
        }
        for j in nb(i) {
            if open(j) && dist[j] == u16::MAX {
                dist[j] = dist[i] + 1;
                prev[j] = i as u32;
                q.push_back(j);
            }
        }
    }
    None
}

/// A tick of the handoff: a jam standing a minute on a bred strain proposes a bypass; a confirmed one lays a tile
/// each Mat step. `jam_since` is the tick the standing jam began.
pub fn tick(city: &mut City, colony: &mut Colony, jam_since: Option<u64>, day: u32, log: &mut CityLog) {
    let t = city.tick;
    if colony.proposal_confirmed {
        if !colony.proposal.is_empty() && t.is_multiple_of(city.mat_every.max(1)) {
            let i = colony.proposal[0] as usize;
            if city.sim.lay(i) || city.sim.hy[i] != hy::EMPTY {
                colony.proposal.remove(0);
            }
            if colony.proposal.is_empty() {
                colony.proposal_confirmed = false;
                log.push(t, Class::Landing, "the Mat's bypass is laid".into());
            }
        }
        return;
    }
    if !colony.proposal.is_empty() || !bred(city) {
        return;
    }
    if jam_since.is_some_and(|s| t.saturating_sub(s) >= crate::alerts::JAM_ALERT_TICKS) {
        if let Some(path) = bypass(city) {
            let w = city.sim.w as u32;
            let at = path[0];
            log.push(t, Class::Decision, format!("the Mat proposes a cord of {} tile{} round the jam at {},{}", path.len(), if path.len() == 1 { "" } else { "s" }, at % w, at / w));
            colony.proposal = path;
            if !colony.progress.strain.has_card("handoff") {
                colony.progress.strain.book.push(("handoff".into(), day));
                log.push(t, Class::Landing, "Strainbook: The handoff".into());
            }
        }
    }
}

/// The player's answer: confirm lays it, sever drops it.
pub fn answer(colony: &mut Colony, confirm: bool) {
    if confirm && !colony.proposal.is_empty() {
        colony.proposal_confirmed = true;
    } else {
        colony.proposal.clear();
        colony.proposal_confirmed = false;
    }
}
