//! A cautious bot that plays a run for the cadence instrument (Full Design v3.6, The cadence budget and its log):
//! it opens what it sees, takes what fits, fights what is adjacent while over 40 % HP and backs off under it, digs
//! a deposit it stands beside, and extracts when the rig is nearly full, the shift is under 90 s, or the body is
//! under 30 %. It knows only what the Runner has seen (the fog), paths over known passable tiles, and leaves the
//! lantern on: it is a yardstick for the route, not a player. Under a Flood's water half the drowning time, it climbs
//! for air before anything else (the HUD's UNDER tag is the one thing no player misses).

use std::collections::VecDeque;

use crate::creature::{CreatureFile, Phase, FP, TICK_HZ};
use crate::fog::UNSEEN;
use crate::loot::{LootTables, SourceState};
use crate::map::RouteMap;
use crate::sim::{Outcome, RunInput, RunSetup, RunSim};
use inventory::Rig;
use items::ItemTable;
use mat::sim::sub;

pub const FIGHT_HP: f32 = 0.40;
pub const EXTRACT_HP: f32 = 0.30;
pub const EXTRACT_FREE_CELLS: usize = 3;
pub const EXTRACT_SHIFT_LEFT_S: f32 = 90.0;
/// a run the bot cannot finish in this many ticks is cut off (a stuck bot is a bug, not a result)
pub const MAX_TICKS: u64 = 40 * 60 * TICK_HZ as u64;

#[derive(Clone, Debug, Default)]
pub struct Bot {
    toggle: bool,
    going_home: bool,
    /// source indices the rig had no room for
    skip: Vec<usize>,
    /// deposit tiles already dug or given up
    dug: Vec<(i32, i32)>,
    stuck: u32,
    last_tile: (i32, i32),
}

impl Bot {
    pub fn new() -> Self {
        Bot::default()
    }

    fn hp_frac(s: &RunSim) -> f32 {
        let hp: i32 = s.runner.sections.iter().sum();
        let max: i32 = s.runner.max_sections.iter().sum();
        hp as f32 / max.max(1) as f32
    }

    /// Breadth-first over known passable tiles from `from`; returns the first step toward the nearest tile
    /// satisfying `goal`, and that tile.
    fn path_step(s: &RunSim, from: (i32, i32), passable: &[bool], goal: &dyn Fn((i32, i32)) -> bool) -> Option<((i32, i32), (i32, i32))> {
        let (w, h) = (s.map.w as i32, s.map.h as i32);
        let idx = |x: i32, y: i32| (y * w + x) as usize;
        let known = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && passable[idx(x, y)] && s.senses.fog.state[idx(x, y)] != UNSEEN;
        let mut prev = vec![u32::MAX; (w * h) as usize];
        let mut q = VecDeque::new();
        prev[idx(from.0, from.1)] = idx(from.0, from.1) as u32;
        q.push_back(from);
        while let Some((x, y)) = q.pop_front() {
            if (x, y) != from && goal((x, y)) {
                // walk back to the first step
                let mut cur = (x, y);
                loop {
                    let p = prev[idx(cur.0, cur.1)] as usize;
                    let pt = ((p as i32) % w, (p as i32) / w);
                    if pt == from {
                        return Some((cur, (x, y)));
                    }
                    cur = pt;
                }
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if known(nx, ny) && prev[idx(nx, ny)] == u32::MAX {
                    prev[idx(nx, ny)] = idx(x, y) as u32;
                    q.push_back((nx, ny));
                }
            }
        }
        None
    }

    /// Steer toward a neighbouring tile: the body (HALF wide) must sit near the lane's centre on the other axis or a
    /// wall corner stops it, so the perpendicular axis nudges toward the tile centre while the main axis moves.
    fn steer(s: &RunSim, step: (i32, i32), input: &mut RunInput) {
        let rt = s.runner.tile();
        let (cx, cy) = (rt.0 * FP + FP / 2, rt.1 * FP + FP / 2);
        let (dx, dy) = ((step.0 - rt.0).signum(), (step.1 - rt.1).signum());
        input.dx = dx as i8;
        input.dy = dy as i8;
        if dx != 0 && (s.runner.y - cy).abs() > 20 {
            input.dy = (cy - s.runner.y).signum() as i8;
        }
        if dy != 0 && (s.runner.x - cx).abs() > 20 {
            input.dx = (cx - s.runner.x).signum() as i8;
        }
    }

    /// The input for this tick.
    pub fn decide(&mut self, s: &RunSim) -> RunInput {
        let mut input = RunInput { lantern: false, ..Default::default() };
        if s.result.is_some() || !s.runner.is_up() {
            return input;
        }
        // the senses tick with the sim: nothing is known before the first step
        if s.tick < 2 {
            return input;
        }
        self.toggle = !self.toggle;
        let press = self.toggle;
        let rt = s.runner.tile();
        if rt == self.last_tile {
            self.stuck += 1;
        } else {
            self.stuck = 0;
            self.last_tile = rt;
        }
        let passable = s.map.passable_mask();
        let hp = Self::hp_frac(s);
        // bleeding: antiseptic from the quick slots (a press, once per tick pair)
        if press && s.runner.statuses.iter().any(|st| st.kind == crate::runner::StatusKind::Bleeding) {
            if let Some(slot) = s.runner.has_quick(&s.table, "org_antiseptic") {
                input.quick = slot as u8 + 1;
                return input;
            }
        }
        let (w, h) = (s.map.w as i32, s.map.h as i32);
        let idx = |x: i32, y: i32| (y * w + x) as usize;
        // under water half the time it takes to go Down: up, or sideways when the way up is shut
        if s.under * 2 >= crate::creature::ticks(s.defs.flood.drown_s) {
            if rt.1 > 0 && passable[idx(rt.0, rt.1 - 1)] {
                input.dy = -1;
            } else {
                input.dx = if rt.0 + 1 < w && passable[idx(rt.0 + 1, rt.1)] { 1 } else { -1 };
            }
            return input;
        }

        // ---- threats within two tiles
        let mut nearest: Option<(i32, i32, i32)> = None; // (dist, cx, cy)
        for c in s.creatures.iter().filter(|c| !c.dead && !matches!(c.phase, Phase::Leaving | Phase::Flee)) {
            let (cx, cy) = c.tile();
            let d = (cx - rt.0).abs().max((cy - rt.1).abs());
            if d <= 2 && nearest.is_none_or(|(nd, _, _)| d < nd) {
                nearest = Some((d, cx, cy));
            }
        }
        if let Some((d, cx, cy)) = nearest {
            if hp > FIGHT_HP {
                // face it and swing; step in when it sits a tile beyond reach
                input.dx = (cx - rt.0).signum() as i8;
                if input.dx == 0 {
                    input.dx = s.runner.facing as i8;
                }
                if d > 1 || (cy - rt.1).abs() > 1 {
                    input.dy = (cy - rt.1).signum() as i8;
                } else {
                    input.dx = if (cx - rt.0) == 0 { s.runner.facing as i8 } else { (cx - rt.0).signum() as i8 };
                    input.attack = press;
                    // do not walk into it while swinging
                    if (cx - rt.0).abs() <= 1 {
                        input.dx = 0;
                        if (cx - rt.0) != 0 {
                            // keep facing without moving: the sim sets facing from dx, so nudge then stop next tick
                            input.dx = if press { (cx - rt.0).signum() as i8 } else { 0 };
                        }
                    }
                }
                return input;
            } else {
                input.dx = -(cx - rt.0).signum() as i8;
                input.dy = -(cy - rt.1).signum() as i8;
                input.sprint = true;
                self.going_home = true;
                return input;
            }
        }

        // ---- standing on a source (five seconds on one that gives nothing more — the rig too heavy for what is
        // left, a lock it cannot open — and it is skipped)
        if let Some(i) = s.loot.source_at(rt.0 as usize, rt.1 as usize) {
            if self.stuck > 5 * TICK_HZ as u32 && !self.skip.contains(&i) {
                self.skip.push(i);
            }
            let src = &s.loot.sources[i];
            match src.state {
                SourceState::Closed if !self.skip.contains(&i) => {
                    input.interact = press;
                    return input;
                }
                SourceState::Open if !src.items.is_empty() && !self.skip.contains(&i) => {
                    let fits = src.items.iter().any(|it| {
                        let d = s.table.def(it.def);
                        (d.size[0] as usize * d.size[1] as usize) <= s.runner.rig.backpack.free_cells()
                    });
                    if fits {
                        input.take = press;
                        return input;
                    }
                    self.skip.push(i);
                }
                _ => {}
            }
        }

        // ---- time to go home?
        let shift_left = s.shift_ticks.saturating_sub(s.tick) as f32 / TICK_HZ as f32;
        if s.runner.rig.backpack.free_cells() < EXTRACT_FREE_CELLS || shift_left < EXTRACT_SHIFT_LEFT_S || hp < EXTRACT_HP || s.tick > MAX_TICKS / 2 {
            self.going_home = true;
        }
        if self.going_home {
            if s.map.entry.contains(&(rt.0 as u16, rt.1 as u16)) {
                input.interact = true;
                return input;
            }
            let entry: Vec<(i32, i32)> = s.map.entry.iter().map(|&(x, y)| (x as i32, y as i32)).collect();
            if let Some((step, _)) = Self::path_step(s, rt, &passable, &|t| entry.contains(&t)) {
                Self::steer(s, step, &mut input);
                input.sprint = hp > FIGHT_HP;
            }
            return input;
        }

        // ---- a deposit beside us: dig it
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (rt.0 + dx, rt.1 + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            if s.map.substrate[idx(nx, ny)] == sub::DEPOSIT && !self.dug.contains(&(nx, ny)) {
                if self.stuck > 15 * TICK_HZ as u32 {
                    self.dug.push((nx, ny));
                    continue;
                }
                input.dx = dx as i8;
                input.dy = dy as i8;
                input.dig = true;
                return input;
            }
        }

        // ---- the nearest known source worth a visit
        let skip = self.skip.clone();
        let want = |t: (i32, i32)| -> bool {
            s.loot.source_at(t.0 as usize, t.1 as usize).is_some_and(|i| {
                let src = &s.loot.sources[i];
                !skip.contains(&i) && (src.state == SourceState::Closed || (src.state == SourceState::Open && !src.items.is_empty()))
            })
        };
        if let Some((step, _)) = Self::path_step(s, rt, &passable, &want) {
            Self::steer(s, step, &mut input);
            return input;
        }
        // ---- a known deposit: stand beside it
        let dug = self.dug.clone();
        let beside_deposit = |t: (i32, i32)| -> bool {
            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (t.0 + dx, t.1 + dy);
                nx >= 0 && ny >= 0 && nx < w && ny < h && s.map.substrate[idx(nx, ny)] == sub::DEPOSIT && s.senses.fog.state[idx(nx, ny)] != UNSEEN && !dug.contains(&(nx, ny))
            })
        };
        if let Some((step, _)) = Self::path_step(s, rt, &passable, &beside_deposit) {
            Self::steer(s, step, &mut input);
            return input;
        }
        // ---- the frontier: a known tile beside the unseen
        let frontier = |t: (i32, i32)| -> bool {
            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (t.0 + dx, t.1 + dy);
                nx >= 0 && ny >= 0 && nx < w && ny < h && passable[idx(nx, ny)] && s.senses.fog.state[idx(nx, ny)] == UNSEEN
            })
        };
        if self.stuck < 10 * TICK_HZ as u32 {
            if let Some((step, _)) = Self::path_step(s, rt, &passable, &frontier) {
                Self::steer(s, step, &mut input);
                return input;
            }
        }
        // nothing left to see: home
        self.going_home = true;
        input
    }
}

/// The standard kit the bot carries: a pipe in hand; lantern, pry bar, antiseptic and salt in the quick slots.
pub fn bot_rig(table: &ItemTable) -> Rig {
    let mut rig = Rig::new(8, 5, 4, 18.0);
    let mut quick = Vec::new();
    for id in ["ger_lantern", "ger_pry_bar", "org_antiseptic", "prt_salt"] {
        quick.push(inventory::instance(table, id).map(|(_, it)| it));
    }
    rig.quick = quick;
    if let Some((_, pipe)) = inventory::instance(table, "ger_pipe") {
        rig.gear.hands[0] = Some(pipe);
    }
    rig
}

/// Play one seeded run with the bot and return its outcome (with the cadence log and report).
pub fn play(map: RouteMap, table: ItemTable, tables: LootTables, defs: CreatureFile, nonce: u64, route_runs: u16, day: u32, memory: Option<&[u8]>) -> Outcome {
    let rig = bot_rig(&table);
    let setup = RunSetup { mat_lume_gene: 0.5, memory, run_nonce: nonce, route_runs, day, rig, runner: people::RunnerSetup::plain(5), followers: Vec::new(), corpses: Vec::new(), flood: None, next_flood: None };
    play_setup(map, table, tables, defs, setup)
}

/// The bot plays a run the city set up: its Runner's record, the rig packed from the Stash (`rose city`).
pub fn play_setup(map: RouteMap, table: ItemTable, tables: LootTables, defs: CreatureFile, setup: RunSetup) -> Outcome {
    let mut sim = RunSim::new(map, table, tables, defs, setup);
    let mut bot = Bot::new();
    while sim.result.is_none() && sim.tick < MAX_TICKS {
        let input = bot.decide(&sim);
        sim.step(input);
    }
    if sim.result.is_none() {
        // cut off: count it as a death for the instrument, with the log as it stands
        sim.abandon();
    }
    sim.outcome.take().expect("a finished run has an outcome")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;

    const ITEMS: &str = include_str!("../../../../data/items.toml");

    #[test]
    #[ignore]
    fn probe() {
        let lib = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        let m = RouteMap::compile(&lib, &route(&lib, 11, 1).unwrap());
        let table = ItemTable::from_toml(ITEMS).unwrap();
        let rig = bot_rig(&table);
        let setup = RunSetup { mat_lume_gene: 0.5, memory: None, run_nonce: 7, route_runs: 1, day: 1, rig, runner: people::RunnerSetup::plain(5), followers: Vec::new(), corpses: Vec::new(), flood: None, next_flood: None };
        let mut sim = RunSim::new(m, table, LootTables::shipped(), CreatureFile::shipped(), setup);
        let mut bot = Bot::new();
        while sim.result.is_none() && sim.tick < 6000 {
            let input = bot.decide(&sim);
            if sim.tick % 100 == 0 {
                println!("t {} tile {:?} action {:?} input dx{} dy{} i{} t{} d{} a{} home {} stuck {} free {} hp {:.2}", sim.tick, sim.runner.tile(), sim.runner.action, input.dx, input.dy, input.interact, input.take, input.dig, input.attack, bot.going_home, bot.stuck, sim.runner.rig.backpack.free_cells(), Bot::hp_frac(&sim));
            }
            sim.step(input);
        }
        for e in &sim.log.entries {
            println!("{} {} {} {} {}", crate::telemetry::clock(e.tick), e.class.tag(), e.kind, e.value, e.detail);
        }
    }

    #[test]
    fn a_seed_replays_bit_exact_and_the_cadence_stays_in_its_guard() {
        let lib = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        let t = ItemTable::from_toml(ITEMS).unwrap();
        for stratum in 1..=6u8 {
            let m = RouteMap::compile(&lib, &route(&lib, 3, stratum).unwrap());
            let a = play(m.clone(), t.clone(), LootTables::shipped(), CreatureFile::shipped(), 9, 1, 1, None);
            let b = play(m, t.clone(), LootTables::shipped(), CreatureFile::shipped(), 9, 1, 1, None);
            assert_eq!(a.report, b.report, "S{stratum}: two plays of one seed disagree");
            assert_eq!(a.log.entries.len(), b.log.entries.len());
            let r = &a.report;
            // the guard is wide on purpose: it catches a route that stops paying or a bot that stops moving, not
            // the budget, which rose cadence judges on twenty seeds
            // a Runner who dies early in a deep stratum is the stratum's danger, not a route that stopped paying
            assert!(r.pulls >= 4 || r.result == "died", "S{stratum}: {} pulls\n{}", r.pulls, r.text());
            assert!(r.pull_gap_median >= 4.0 && r.pull_gap_median <= 60.0, "S{stratum}: pull gap {}\n{}", r.pull_gap_median, r.text());
            assert!(r.ticks <= 15 * 60 * TICK_HZ as u64, "S{stratum}: the bot took {}", crate::telemetry::clock(r.ticks));
        }
    }

    #[test]
    fn the_bot_finishes_a_run_and_the_log_has_pulls_and_decisions() {
        let lib = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        let m = RouteMap::compile(&lib, &route(&lib, 11, 1).unwrap());
        let o = play(m, ItemTable::from_toml(ITEMS).unwrap(), LootTables::shipped(), CreatureFile::shipped(), 7, 1, 1, None);
        let r = &o.report;
        assert!(o.ticks < MAX_TICKS, "the bot never finished");
        assert!(r.pulls >= 2, "pulls {} in {}", r.pulls, r.text());
        assert!(r.landings > r.pulls && r.decisions >= r.pulls, "{}", r.text());
        assert!(r.landing_gap_median > 0.0 && r.pull_gap_median > 0.0);
        let text = o.log.text("test", r);
        assert!(text.contains("LAND") && text.contains("pull") && text.contains("budget"));
    }
}
