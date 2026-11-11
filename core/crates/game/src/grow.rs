//! The city's progression at the shift's end (Full Design v3.18, Progression): the Mat's mods from the tree and the
//! classes, the organs grown and the Mat's own structures stood on them, the Graft Buds, the strain's classes and
//! Strainbook, the Scrapdex, the curio sets' passives and the title cards. The bridge and the twenty-shift city both
//! call it through `shift::end_shift`.

use colony::{Colony, StructureFile};
use inventory::Stash;
use items::{ItemTable, Tier};
use logistics::City;
use progress::{StrainFile, StrainShift};

use crate::log::{CityLog, Class};

/// What the tree and the classes change of the Mat's step, from what stands now.
pub fn mods_of(colony: &Colony, genome: &[f32; 10]) -> mat::Mods {
    let t = &colony.progress.tech;
    let s = &colony.progress.strain;
    mat::Mods {
        range: if t.flag("conduit") && genome[mat::genome::CHROME] >= 0.6 { 1.5 } else { 1.0 },
        maint: if t.flag("deep_tissue") { 0.75 } else { 1.0 },
        release: if t.flag("deep_pump") { 1.5 } else { 1.0 },
        runner: s.holds("runner"),
        hive: s.holds("hive"),
        grafting: t.flag("grafting"),
    }
}

/// The Mat's mods and organ kinds set from what stands (the founding and every shift's end).
pub fn refresh_mat(city: &mut City, colony: &Colony) {
    let f = StrainFile::shipped();
    city.sim.kinds = f.kinds();
    city.sim.mods = mods_of(colony, &city.sim.genome.0);
    city.sim.refresh_organ_cache(&f.organ_defs());
}

/// The structure the Mat stands on an organ of a kind, if it stands one (the Brood Knot, the Sphincter, the Maw,
/// the Venom Gland).
fn own_def(defs: &StructureFile, organ: &str) -> Option<usize> {
    defs.structure.iter().position(|d| d.category == "organ" && d.organ == organ)
}

/// The shift's end for the city's progression. Returns the lines for the shift's report.
#[allow(clippy::too_many_arguments)]
pub fn shift_end(city: &mut City, colony: &mut Colony, defs: &StructureFile, stash: &Stash, table: &ItemTable, population: usize, day: u32, log: &mut CityLog) -> Vec<String> {
    let f = StrainFile::shipped();
    let tick = city.tick;
    let mut lines = Vec::new();
    let organ_defs = f.organ_defs();
    city.sim.kinds = f.kinds();
    // the Mat's own that a threat ruined take their organ with them; organs whose Node died take their structure
    let own: Vec<(usize, &str)> = f.organ.iter().filter_map(|o| own_def(defs, &o.id).map(|d| (d, o.id.as_str()))).collect();
    let w = city.sim.w;
    city.sim.organs.retain(|o| {
        let kind = &f.organ[o.kind as usize].id;
        match own.iter().find(|(_, id)| *id == kind) {
            Some((d, _)) => colony.structures.iter().any(|s| s.def == *d && s.y as usize * w + s.x as usize == o.i as usize),
            None => true,
        }
    });
    let lost = city.sim.prune_organs();
    for (k, i) in &lost {
        let name = &f.organ[*k as usize].name;
        let (x, y) = ((i % w) as u16, (i / w) as u16);
        log.push(tick, Class::Loss, format!("the {name} at {x},{y} died with its Node"));
        if let Some(sid) = colony.structures.iter().find(|s| defs.structure[s.def].category == "organ" && s.x == x && s.y == y).map(|s| s.id) {
            colony.ruin(defs, sid);
        }
    }
    // the organs grow (Full Design v3.18, Organs), then the Mat's own stand on theirs
    let covered = |i: usize| colony.occupied(defs, (i % w) as u16, (i / w) as u16).is_some();
    let grown = city.sim.grow_organs(&organ_defs, f.organ_spacing, f.organ_per_nodes, &covered);
    for &(k, i) in &grown {
        let o = &f.organ[k as usize];
        let (x, y) = ((i % w) as u16, (i / w) as u16);
        log.push(tick, Class::Landing, format!("a {} grew at {x},{y}", o.name));
        lines.push(format!("a {} grew at {x},{y}", o.name));
        if let Some(d) = own_def(defs, &o.id) {
            colony.stand_own(defs, d, x, y);
        }
    }
    city.sim.refresh_organ_cache(&organ_defs);
    city.bellows = f.organ_index("bellows").map(|k| city.sim.organs_of(k).map(|o| o.i as usize).collect()).unwrap_or_default();
    // the Graft Buds take feral tissue back
    city.sim.graft_buds();
    // the strain: classes, the Strainbook, the lineage
    let mut s: StrainShift = std::mem::take(&mut colony.strain_shift);
    s.grown = grown.iter().map(|g| g.0).collect();
    s.reclaimed = std::mem::take(&mut city.sim.reclaimed);
    s.bloom = city.sim.blooms() && (0..city.sim.hy.len()).any(|i| city.sim.is_feral(i));
    s.hyphal = city.sim.hyphal_count();
    let genome = city.sim.genome.0;
    for l in colony.progress.strain.end_shift(f, &genome, day, &s) {
        let class = if l.starts_with("the strain lost") || l.starts_with("the strain let") { Class::Loss } else { Class::Landing };
        log.push(tick, class, l.clone());
        lines.push(l);
    }
    // the Scrapdex: every item identified in the Stash is seen
    let mut seen = 0;
    for it in stash.all_items() {
        if colony.progress.collections.see(table, it, day) {
            seen += 1;
        }
    }
    if seen > 0 {
        let total = progress::CollectionsFile::shipped().scrapdex(table).len();
        let n = colony.progress.collections.scrapdex.len();
        let l = format!("Scrapdex: {seen} new, {n} of {total}");
        log.push(tick, Class::Landing, l.clone());
        lines.push(l);
    }
    // the curio sets' passives
    let sets = progress::CollectionsFile::shipped().set_flags(table, stash.all_items());
    for flag in sets.iter().filter(|f| !colony.progress.collections.sets.contains(f)) {
        if let Some(d) = progress::CollectionsFile::shipped().set.iter().find(|d| &d.flag == flag) {
            let l = format!("the {} is complete: {}", d.name, d.text);
            log.push(tick, Class::Landing, l.clone());
            lines.push(l);
        }
    }
    colony.progress.collections.sets = sets;
    // the title cards
    while let Some((n, name)) = colony.progress.collections.title(population) {
        let l = format!("the {name}: {n} slimos");
        log.push(tick, Class::Landing, l.clone());
        lines.push(l);
    }
    // a World-Eater strain digests the city's own structures at half rate: 2 % of their HP a shift (Pass 2; v3.18)
    if colony.progress.strain.holds("world_eater") {
        let mut ruined = Vec::new();
        for k in 0..colony.structures.len() {
            let s = &colony.structures[k];
            if !s.built || defs.structure[s.def].id == "heart" {
                continue;
            }
            let bite = (colony.hp_max(defs, s) / 50).max(1);
            let s = &mut colony.structures[k];
            s.hp -= bite;
            if s.hp <= 0 {
                ruined.push(s.id);
            }
        }
        for id in ruined {
            if let Some(s) = colony.ruin(defs, id) {
                log.push(tick, Class::Loss, format!("the Mat digested the {} at {},{}", defs.structure[s.def].name, s.x, s.y));
            }
        }
    }
    city.sim.mods = mods_of(colony, &city.sim.genome.0);
    lines
}

/// A feed landed at a Pit: steered drift, the reflections and a Relic fed, for the Strainbook (v3.18).
pub fn fed(city: &mut City, colony: &mut Colony, table: &ItemTable, item: items::ItemId, drift: [f32; 10]) {
    let mut d = drift;
    if colony.progress.tech.flag("steered_drift") {
        for v in d.iter_mut() {
            *v *= 1.5;
        }
    }
    let bounced = city.sim.drift_genome(d);
    colony.strain_shift.bounced.extend(bounced);
    if table.def(item).tier == Tier::Relic {
        colony.strain_shift.relic_fed = true;
    }
}

/// The run's flags from the city (Full Design v3.18, Collections: Scav yield): the Street set's +1, the Arasaka set's
/// locks, the Feral set's notice, and Conscription's cost while the Heart's zone is under a Patrol or worse.
pub fn run_flags(colony: &Colony, threats: Option<&threat::Threats>, sim: &mat::MatSim) -> Vec<String> {
    let c = &colony.progress.collections;
    let mut v = Vec::new();
    for (set, flag) in [("scav_yield_plus_1", "scav_yield_plus_1"), ("locks_open", "locks_open"), ("notice_minus_1", "notice_minus_1")] {
        if c.set_flag(set) {
            v.push(flag.to_string());
        }
    }
    let alarm = threats.is_some_and(|t| t.heart_zone(sim).is_some_and(|z| t.down_in(z, 0)));
    if alarm && colony.charter.flag_at("d2_militia", 0) {
        v.push("alarm_yield_cut".into());
    }
    v
}

/// Pick a research project and pay for it (Full Design v3.18, Research): a study spends its blueprint from the Stash;
/// a node its Parts and Chrome, unless it was paid for and dropped before. Refused while the Stash is short, with
/// what it lacks.
pub fn pick_project(colony: &mut Colony, stash: &mut Stash, ledger: &mut inventory::Ledger, table: &ItemTable, p: progress::Project) -> Result<(), String> {
    if let Some(why) = colony.progress.tech.cannot_pick(&p) {
        return Err(why);
    }
    let f = progress::TechFile::shipped();
    match &p {
        progress::Project::Study(item) => {
            let h = stash.handles().into_iter().find(|(_, _, it)| !it.pinned && table.def(it.def).id == *item).map(|(h, _, _)| h);
            let Some(h) = h else { return Err("the blueprint is not in the Stash".into()) };
            stash.take(h);
        }
        progress::Project::Node(_) if colony.progress.tech.was_paid(&p) => {}
        progress::Project::Node(_) => {
            let (parts, chrome) = f.cost_of(&p);
            if !crate::store::take_cost(stash, ledger, table, colony::Cost { parts, chrome, eddies: 0 }) {
                return Err(format!("needs {parts} Parts and {chrome} Chrome"));
            }
        }
    }
    colony.progress.tech.pick(p)
}
