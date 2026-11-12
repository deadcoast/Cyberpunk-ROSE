//! The threats in the shift (Full Design v3.14, Colony mode: Pressure): what the World and the game's bridge both call
//! each city tick and at each shift's end, so the city plays the same whoever holds it.

use colony::{Colony, ShiftOutcome, StructureFile};
use inventory::Stash;
use items::{ItemInstance, ItemTable};
use logistics::City;
use people::{Record, Roster};
use threat::Threats;

use crate::log::{CityLog, Class};

/// Lines from the threats into the city's log.
pub fn note(log: &mut CityLog, tick: u64, notes: Vec<threat::Note>) {
    for n in notes {
        let class = if n.loss { Class::Loss } else if n.decision { Class::Decision } else { Class::Landing };
        log.push(tick, class, n.what);
    }
}

/// One city tick of the threats, after the cargo landed: the sales and the Blackline sold count toward their grate's
/// Attention.
pub fn tick(threats: &mut Threats, city: &mut City, colony: &mut Colony, defs: &StructureFile, posts: &threat::Posts, log: &mut CityLog, landed: &crate::cargo::Landed) {
    for s in &landed.sold {
        if let Some(&g) = city.markets.get(s.market as usize) {
            if let Some(z) = threats.zones.iter_mut().find(|z| z.grate == g) {
                z.sold += s.eddies;
            }
        }
    }
    for &m in &landed.blackline {
        if let Some(&g) = city.markets.get(m as usize) {
            if let Some(z) = threats.zones.iter_mut().find(|z| z.grate == g) {
                z.blackline += 1;
            }
        }
    }
    let conscription = colony.charter.flag_at("d2_militia", 0);
    let notes = threats.tick(city, colony, defs, posts, conscription);
    note(log, city.tick, notes);
}

/// The threats' side of the shift's end (Full Design v3.14, Pressure): the gauge, the flood's toll, the Blooms,
/// Quarantine, salvage into the Stash; the slimos the bodies hurt, the white light panicked; defectors to a Bloom or
/// to the Oathkeepers. `run_kills`: Oathkeepers the shift's run killed.
#[allow(clippy::too_many_arguments)]
pub fn end(threats: &mut Threats, city: &mut City, colony: &mut Colony, defs: &StructureFile, roster: &mut Roster, stash: &mut Stash, table: &ItemTable, log: &mut CityLog, died: &mut Vec<Record>, o: &ShiftOutcome, day: u32, run_kills: u32) {
    let tick = city.tick;
    // Relic containment: what the Stash holds counts as vaulted (Full Design v3.18, Research); else the built Vaults
    // hold their capacity of it out of the light (v3.20)
    let vaulted: u32 = colony.structures.iter().filter(|s| s.built && defs.structure[s.def].id == "vault").map(|s| defs.structure[s.def].value_at("capacity", s.tier).max(0) as u32).sum();
    let exposed = if colony.progress.tech.flag("stash_vaulted") { 0 } else { (stash.exposed_contraband(table) as u32).saturating_sub(vaulted) };
    let shown = threat::Shown { exposed, run_kills };
    let open = colony.charter.flag_at("attention_x1_5", 0);
    let curfew = colony.charter.flag_at("curfew", 0);
    let quarantine = colony.charter.flag_at("relay_auto_sever", 0);
    let ended = threats.end_shift(city, colony, defs, &shown, open, curfew, quarantine);
    deeds(colony, &ended, day, tick, log);
    note(log, tick, ended.notes);
    // salvage: Blackline and Relics, the first of their tier the seed picks
    let mut r = mat::rng::Pcg32::new(threats.seed ^ 0x5A1_0000 ^ day as u64);
    for (tier, n) in [(items::Tier::Blackline, ended.blackline), (items::Tier::Relic, ended.relics)] {
        let pool: Vec<usize> = (0..table.len()).filter(|&k| table.defs[k].tier == tier).collect();
        for _ in 0..n {
            if pool.is_empty() {
                break;
            }
            let idx = items::ItemId(pool[r.below(pool.len() as u32) as usize] as u16);
            let it = ItemInstance::new(idx);
            stash.place_or_floor(table, it);
        }
        if n > 0 {
            log.push(tick, Class::Landing, format!("{n} {} salvaged into the Stash", if tier == items::Tier::Blackline { "Blackline" } else { "Relics" }));
        }
    }
    // the hurt and the panicked
    let hurt: Vec<u32> = threats.hurt.keys().copied().collect();
    for id in hurt {
        let Some(rec) = roster.get(id) else { continue };
        let hp = 130.0 * (0.8 + 0.04 * rec.stat(people::Stat::Body) as f32);
        let (dead, bleeding) = threats.verdict(hp, id);
        if dead {
            if let Some(r) = roster.dissolve(id) {
                log.push(tick, Class::Loss, format!("{} died to the Oathkeepers; to the memorial", r.full_name()));
                died.push(r);
            }
        } else if bleeding {
            if let Some(r) = roster.get_mut(id) {
                if !r.injuries.iter().any(|(i, _)| *i == people::Injury::Bleeding) {
                    r.injuries.push((people::Injury::Bleeding, 1));
                }
                log.push(tick, Class::Loss, format!("{} is bleeding from the fight", r.full_name()));
            }
        }
    }
    let panic = threats.rules.defence.panic_mood;
    for id in threats.panicked.clone() {
        if let Some(r) = roster.get_mut(id) {
            r.needs[5] = (r.needs[5] - panic).max(0);
        }
    }
    // defection (Full Design, Unrest): a member under 20 in a district ready to defect leaves, 1 in 4, to a Bloom
    // that borders it or to the Oathkeepers if a Patrol or worse is down in its zone; never from a Pod in Dark
    let mut gone = Vec::new();
    for did in &o.ready_to_defect {
        let Some(d) = colony.districts.iter().find(|d| d.id == *did) else { continue };
        let (hub, members) = (d.hub, d.members.clone());
        let bloom = threats.bloom_near(&city.sim, hub.0, hub.1);
        let zone = threats.zone_at(&city.sim, hub.0, hub.1);
        let patrol = zone.is_some_and(|z| threats.down_in(z, 0));
        if !bloom && !patrol {
            continue;
        }
        for id in members {
            let asleep_in_pod = o.contexts.get(&id).is_some_and(|c| c.pod_dark);
            let low = roster.get(id).is_some_and(|r| r.is_up() && r.needs[5] < 20);
            if low && !asleep_in_pod && r.below(4) == 0 {
                gone.push((id, bloom, zone));
            }
        }
    }
    for (id, bloom, zone) in gone {
        let Some(i) = roster.records.iter().position(|r| r.id == id) else { continue };
        let rec = roster.records.remove(i);
        for other in roster.records.iter_mut() {
            other.bonds.retain(|b| b.with != id);
        }
        colony.homes.remove(&id);
        if bloom {
            log.push(tick, Class::Loss, format!("{} defected to the Bloom", rec.full_name()));
        } else {
            log.push(tick, Class::Loss, format!("{} defected to the Oathkeepers", rec.full_name()));
            if let Some(z) = zone {
                let add = threats.rules.attention.defector;
                threats.zones[z].attention = (threats.zones[z].attention + add).min(threats.rules.attention.max);
            }
        }
    }
}

/// Authority's deeds and the trophies from the threats' shift (Full Design v3.18, Authority, Collections): a Patrol
/// repelled +1, a Sweep +2, a Cleansing +3; the first of each body down, each hazard lived through.
pub fn deeds(colony: &mut Colony, ended: &threat::Ended, day: u32, tick: u64, log: &mut CityLog) {
    for st in &ended.repelled {
        let n = repel_authority(st);
        if n > 0 {
            colony.charter.earn(n);
            log.push(tick, Class::Landing, format!("the {st} repelled: Authority +{n}"));
        }
    }
    for k in &ended.killed {
        if let Some(name) = colony.progress.collections.trophy("kill", k, day) {
            log.push(tick, Class::Landing, format!("Trophy: {name}"));
        }
    }
    for k in &ended.lived {
        if let Some(name) = colony.progress.collections.trophy("hazard", k, day) {
            log.push(tick, Class::Landing, format!("Trophy: {name}"));
        }
    }
}

/// Authority a stage repelled is worth (Full Design v3.18, Authority).
pub fn repel_authority(stage: &str) -> u32 {
    match stage {
        "patrol" => 1,
        "sweep" => 2,
        "cleansing" => 3,
        _ => 0,
    }
}
