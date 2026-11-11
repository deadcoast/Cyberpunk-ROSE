//! Route loops (Full Design v3.18, Route loops; Pass 2: "a route that returned twice can be assigned to a crew on a
//! loop; it then runs every shift without orders, with yield decaying 10% per repeat"): at each shift's end, before
//! the shift's work, every loop's slimo plays its route headless with the run bot, a kit from the Stash, and the
//! decay as a chance of nothing from each source; the run lands as any run does. The bridge and the twenty-shift city
//! both call `play`.

use colony::Colony;
use inventory::{Rig, Stash};
use items::{ItemInstance, ItemTable};
use people::{Depth, Injury, Past, RunReport};
use run::{ChunkLib, CreatureFile, LootTables, RouteMap, RunResult, RunSetup};

use crate::log::{CityLog, Class};
use crate::shift::Folk;

/// The decay a repeat adds, a hundredth a source: 10, 5 with a Nomad or a D2 on the loop, to 90 (Pass 2).
pub fn decay(repeats: u32, slow: bool) -> u32 {
    (repeats * if slow { 5 } else { 10 }).min(90)
}

fn take(stash: &mut Stash, table: &ItemTable, id: &str) -> Option<ItemInstance> {
    let idx = table.id_of(id)?;
    let h = stash.handles().into_iter().find(|(_, _, it)| it.def == idx && !it.pinned).map(|(h, _, _)| h)?;
    stash.take(h)
}

/// The loops' runs at a shift's end. `city_seed` names the routes (run::route_seed); `threats` gives the Flood. Returns
/// the lines for the report.
#[allow(clippy::too_many_arguments)]
pub fn play(colony: &mut Colony, folk: &mut Folk, stash: &mut Stash, table: &ItemTable, threats: Option<&threat::Threats>, sim: &mat::MatSim, city_seed: u64, day: u32, tick: u64, log: &mut CityLog) -> Vec<String> {
    let mut lines = Vec::new();
    if colony.loops.is_empty() || !colony.progress.tech.flag("route_loops") {
        return lines;
    }
    let lib = ChunkLib::from_toml(run::CHUNKS_TOML).expect("chunks.toml");
    let (tables, creatures) = (LootTables::shipped(), CreatureFile::shipped());
    let flags = crate::grow::run_flags(colony, threats, sim);
    let mut ended = Vec::new();
    for k in 0..colony.loops.len() {
        let l = colony.loops[k].clone();
        let Some(r) = folk.roster.get(l.slimo) else {
            ended.push(l.stratum);
            continue;
        };
        let able = r.can_run() || (folk.roster.d4_runs && r.depth == Depth::D4 && r.is_up());
        if !able || r.shift_job.is_some() {
            continue;
        }
        let slow = r.past == Past::Nomad || r.depth == Depth::D2;
        let cut = decay(l.repeats, slow);
        let mut setup = r.runner_setup(folk.traits);
        setup.flags.extend(flags.iter().cloned());
        setup.flags.push(format!("yield_cut_{cut}"));
        let name = r.full_name();
        let Ok(layout) = run::route(&lib, run::route_seed(city_seed, l.stratum), l.stratum) else { continue };
        let map = RouteMap::compile(&lib, &layout);
        let mut rig = Rig::new(8, setup.rig_rows.clamp(3, 8), 4, setup.carry_kg());
        rig.gear.hands[0] = take(stash, table, crate::city::KIT_HAND);
        for (i, id) in crate::city::KIT_QUICK.iter().enumerate() {
            rig.quick[i] = take(stash, table, id);
        }
        folk.roster.depart(l.slimo, &[]);
        let nonce = tick.wrapping_mul(131) ^ ((day as u64) << 20) ^ ((l.stratum as u64) << 8) ^ 0x100F;
        let rs = RunSetup { mat_lume_gene: sim.genome.0[mat::genome::LUME], memory: None, run_nonce: nonce, route_runs: 2 + l.repeats as u16, day, rig, runner: setup, followers: Vec::new(), corpses: Vec::new(), flood: threats.and_then(|t| t.flood_roll()), next_flood: None };
        let o = run::bot::play_setup(map, table.clone(), tables.clone(), creatures.clone(), rs);
        let extracted = matches!(o.result, RunResult::Extracted { .. });
        let report = RunReport {
            runner: l.slimo,
            followers: Vec::new(),
            extracted,
            runner_died: !extracted,
            followers_extracted: Vec::new(),
            followers_stranded: Vec::new(),
            followers_dead: Vec::new(),
            pulls: o.report.pulls as u32,
            items_taken: o.items_taken,
            kills: o.kills,
            hits_survived: o.hits_survived,
            follower_strikes: o.follower_strikes,
            injuries: o.injuries.iter().filter_map(|s| Injury::from_status(s)).collect(),
        };
        let mut haul: Vec<ItemInstance> = o.rig.as_ref().filter(|_| extracted).map(|rig| crate::store::rig_items(rig, table)).unwrap_or_default();
        haul.extend(o.mat_haul.iter().cloned());
        haul.extend(o.regrown.iter().cloned());
        let n = haul.len();
        for kk in &o.killed {
            if let Some(t) = colony.progress.collections.trophy("kill", kk, day) {
                log.push(tick, Class::Landing, format!("Trophy: {t}"));
            }
        }
        crate::shift::run_home(folk, stash, table, &report, haul);
        colony.loops[k].repeats += 1;
        let line = if extracted {
            format!("{name}'s loop on the S{} route came home with {n} item{}, its sources at {cut} % decay", l.stratum, if n == 1 { "" } else { "s" })
        } else {
            format!("{name} died on the S{} route's loop", l.stratum)
        };
        log.push(tick, if extracted { Class::Landing } else { Class::Loss }, line.clone());
        lines.push(line);
        if !extracted {
            ended.push(l.stratum);
        }
    }
    colony.loops.retain(|l| !ended.contains(&l.stratum));
    lines
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_decay_is_ten_a_repeat_five_for_a_nomad_or_a_d2_to_ninety() {
        assert_eq!(super::decay(0, false), 0);
        assert_eq!(super::decay(3, false), 30);
        assert_eq!(super::decay(3, true), 15);
        assert_eq!(super::decay(20, false), 90);
    }
}
