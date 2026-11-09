//! The colony side of `RoseSim` (Full Design v3.9, Colony mode): structures, zones, districts, the shift. The
//! `#[func]`s live on `RoseSim` in lib.rs and call these; the Stash and the roster are other Godot objects, bound
//! for the shift.

use colony::{Colony, Cost, Filter, ShiftOutcome, StructureFile, ZoneKind};
use godot::prelude::*;

use crate::data_text;
use crate::people::RosePeople;
use crate::RoseStash;

pub fn structure_file() -> StructureFile {
    StructureFile::from_toml(&data_text("structures.toml", colony::STRUCTURES_TOML)).unwrap_or_else(|_| StructureFile::shipped())
}

fn cost_dict(c: Cost) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("parts", c.parts as i64);
    d.set("chrome", c.chrome as i64);
    d.set("eddies", c.eddies as i64);
    d.set("text", c.text().as_str());
    d
}

fn ints(v: &[u32]) -> PackedInt32Array {
    PackedInt32Array::from(v.iter().map(|x| *x as i32).collect::<Vec<_>>().as_slice())
}

/// Every def: what the Build menu lists.
pub fn defs_array(defs: &StructureFile) -> VarArray {
    let mut out = VarArray::new();
    for d in &defs.structure {
        let mut e = VarDictionary::new();
        e.set("id", d.id.as_str());
        e.set("name", d.name.as_str());
        e.set("w", d.footprint[0] as i64);
        e.set("h", d.footprint[1] as i64);
        e.set("sigma", d.sigma as f64);
        e.set("node", d.node);
        e.set("grate", d.grate);
        e.set("relay", d.relay);
        e.set("prebuilt", d.prebuilt);
        e.set("heart_tier", d.heart_tier as i64);
        e.set("tiers", d.tiers as i64);
        e.set("cost", &cost_dict(d.cost));
        e.set("build", d.build as i64);
        e.set("hosts", d.hosts.as_str());
        e.set("gate", d.gate as i64);
        e.set("slots", &PackedInt32Array::from(d.slots.iter().map(|s| *s as i32).collect::<Vec<_>>().as_slice()));
        e.set("hp", d.hp as i64);
        let mut eff = VarArray::new();
        for (name, values) in d.effects() {
            let mut f = VarDictionary::new();
            f.set("name", name.as_str());
            f.set("values", &PackedInt32Array::from(values.as_slice()));
            eff.push(&f.to_variant());
        }
        e.set("effects", &eff);
        e.set("text", d.text.as_str());
        e.set("art", d.art.as_str());
        out.push(&e.to_variant());
    }
    out
}

/// Every placed structure, ghosts included.
pub fn structures_array(colony: &Colony, defs: &StructureFile) -> VarArray {
    let mut out = VarArray::new();
    for s in &colony.structures {
        let d = &defs.structure[s.def];
        let mut e = VarDictionary::new();
        e.set("id", s.id as i64);
        e.set("def", d.id.as_str());
        e.set("name", d.name.as_str());
        e.set("x", s.x as i64);
        e.set("y", s.y as i64);
        e.set("w", d.footprint[0] as i64);
        e.set("h", d.footprint[1] as i64);
        e.set("tier", s.tier as i64);
        e.set("tiers", d.tiers as i64);
        e.set("hp", s.hp as i64);
        e.set("hp_max", d.hp_at(s.tier) as i64);
        e.set("built", s.built);
        e.set("progress", s.progress as i64);
        e.set("build_total", d.build_at(s.upgrading_to.unwrap_or(1)) as i64);
        e.set("paid", s.paid);
        e.set("upgrading_to", s.upgrading_to.map(|t| t as i64).unwrap_or(0));
        e.set("district", s.district.map(|d| d as i64).unwrap_or(-1));
        e.set("hosts", d.hosts.as_str());
        e.set("slots", d.slots_at(s.tier) as i64);
        e.set("art", d.art.as_str());
        e.set("cost", &cost_dict(d.cost_at(s.upgrading_to.unwrap_or(1))));
        out.push(&e.to_variant());
    }
    out
}

pub fn zones_array(colony: &Colony) -> VarArray {
    let mut out = VarArray::new();
    for z in &colony.zones {
        let mut e = VarDictionary::new();
        e.set("id", z.id as i64);
        e.set("kind", z.kind.name());
        e.set("cells", &ints(&z.cells));
        e.set("categories", &PackedStringArray::from(z.filter.categories.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
        e.set("min_tier", z.filter.min_tier as i64);
        e.set("min_condition", z.filter.min_condition as i64);
        e.set("anchor", z.anchor.map(|a| a as i64).unwrap_or(-1));
        out.push(&e.to_variant());
    }
    out
}

pub fn districts_array(colony: &Colony, people: Option<&RosePeople>, defs: Option<&StructureFile>, mat: Option<&mat::MatSim>) -> VarArray {
    let mut out = VarArray::new();
    for d in &colony.districts {
        let mut e = VarDictionary::new();
        e.set("id", d.id as i64);
        e.set("name", d.name.as_str());
        e.set("hub_x", d.hub.0 as i64);
        e.set("hub_y", d.hub.1 as i64);
        e.set("structures", &ints(&d.structures));
        e.set("members", &ints(&d.members));
        e.set("foreman", d.foreman.map(|f| f as i64).unwrap_or(-1));
        e.set("foreman_name", d.foreman.and_then(|f| people.and_then(|p| p.roster.get(f)).map(|r| r.full_name())).unwrap_or_default().as_str());
        e.set("term_left", d.term_left as i64);
        e.set("priorities", &PackedInt32Array::from(d.priorities.iter().map(|p| *p as i32).collect::<Vec<_>>().as_slice()));
        e.set("boon_shifts", d.boon_shifts as i64);
        e.set("wake", d.wake);
        e.set("mood", d.mood as i64);
        e.set("strike", d.strike);
        e.set("ready_to_defect", d.ready_to_defect);
        e.set("refusals", d.refusals as i64);
        // the foreman's cargo demand and stock (v3.11): what the members ate, what the Cut took
        e.set("ate", d.ate as f64);
        e.set("cut", d.cut as i64);
        // the reach: the bounding box of the tiles within 12 cords of the hub
        if let Some(m) = mat {
            let reach = colony::charter::cord_reach(m, &[d.hub.1 as usize * m.w + d.hub.0 as usize], colony::DISTRICT_CORDS);
            let (mut x0, mut y0, mut x1, mut y1) = (d.hub.0 as i64, d.hub.1 as i64, d.hub.0 as i64, d.hub.1 as i64);
            for i in reach.keys() {
                let (x, y) = ((i % m.w) as i64, (i / m.w) as i64);
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
            e.set("reach", &PackedInt32Array::from([x0 as i32, y0 as i32, x1 as i32, y1 as i32].as_slice()));
        }
        e.set("board", defs.is_some_and(|defs| colony.has_board(defs, d)));
        let mut p = VarDictionary::new();
        if let Some(pet) = &d.petition {
            p.set("need", pet.need.name());
            p.set("asks", &PackedStringArray::from(pet.asks.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
            p.set("repeal", pet.repeal.clone().unwrap_or_default().as_str());
            p.set("repeal_name", pet.repeal.as_deref().and_then(|l| colony.charter.law(l)).map(|l| l.name.as_str()).unwrap_or(""));
            p.set("shifts_left", pet.shifts_left as i64);
            p.set("missed", pet.missed);
        }
        e.set("petition", &p);
        out.push(&e.to_variant());
    }
    out
}

pub fn parse_zone(kind: &str) -> Option<ZoneKind> {
    ZoneKind::from_key(kind)
}

pub fn filter_from(categories: &PackedStringArray, min_tier: i64, min_condition: i64) -> Filter {
    Filter { categories: categories.as_slice().iter().map(|s| s.to_string()).collect(), min_tier: min_tier.clamp(0, 6) as u8, min_condition: min_condition.clamp(0, 100) as u8 }
}

/// The shift's outcome as the game reads it: the ledger report, the colony's lines and the roster's, who worked
/// what, what waits for Parts.
pub fn outcome_dict(o: &ShiftOutcome, people_lines: &[String], report: VarDictionary, defs: &StructureFile, colony: &Colony) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("report", &report);
    let mut lines: Vec<GString> = o.lines.iter().map(|s| GString::from(s.as_str())).collect();
    lines.extend(people_lines.iter().map(|s| GString::from(s.as_str())));
    d.set("lines", &PackedStringArray::from(lines.as_slice()));
    let mut jobs = VarArray::new();
    for (slimo, job, sid) in &o.assignments {
        let mut e = VarDictionary::new();
        e.set("slimo", *slimo as i64);
        e.set("job", job.name());
        e.set("structure", *sid as i64);
        e.set("structure_name", colony.get(*sid).map(|s| defs.structure[s.def].name.as_str()).unwrap_or(""));
        jobs.push(&e.to_variant());
    }
    d.set("assignments", &jobs);
    let mut waiting = VarArray::new();
    for (sid, cost) in &o.waiting {
        let mut e = VarDictionary::new();
        e.set("structure", *sid as i64);
        e.set("name", colony.get(*sid).map(|s| defs.structure[s.def].name.as_str()).unwrap_or(""));
        e.set("cost", &cost_dict(*cost));
        waiting.push(&e.to_variant());
    }
    d.set("waiting", &waiting);
    d.set("built", &ints(&o.built));
    d.set("upgraded", &ints(&o.upgraded));
    d.set("bench_slots", o.bench_slots as i64);
    d.set("identified", o.identifies as i64);
    d.set("salt", o.salt as i64);
    d.set("doses", o.doses as i64);
    d.set("brine", o.brine as i64);
    d.set("upkeep", o.upkeep as i64);
    d.set("paste_meals", o.paste_meals as i64);
    d.set("treated", &ints(&o.treated));
    d.set("districts_formed", &ints(&o.districts_formed));
    let mut el = VarArray::new();
    for (did, who) in &o.elections {
        el.push(&PackedInt32Array::from([*did as i32, *who as i32].as_slice()).to_variant());
    }
    d.set("elections", &el);
    let mut pr = VarArray::new();
    for (did, need) in &o.petitions_raised {
        let mut e = VarDictionary::new();
        e.set("district", *did as i64);
        e.set("need", need.name());
        pr.push(&e.to_variant());
    }
    d.set("petitions_raised", &pr);
    d.set("petitions_met", &ints(&o.petitions_met));
    d.set("petitions_missed", &ints(&o.petitions_missed));
    d.set("strikes", &ints(&o.strikes));
    d.set("ready_to_defect", &ints(&o.ready_to_defect));
    d.set("authority_gained", o.authority_gained as i64);
    let mut la = VarArray::new();
    for (law, did) in &o.laws_arrived {
        let mut e = VarDictionary::new();
        e.set("law", law.as_str());
        e.set("district", *did as i64);
        la.push(&e.to_variant());
    }
    d.set("laws_arrived", &la);
    d
}

/// The eight laws with their state on the Charter: {id, name, authority, mood, effect, cost, live, offered_by,
/// state: "signed" | "spreading" | "repealing" | "open" | "locked", why (when it cannot be signed), shift, votes,
/// reached (district names)}.
pub fn laws_array(colony: &Colony, laws: &colony::LawFile, mat: &mat::MatSim) -> VarArray {
    let mat_farthest = colony.heart_dist.iter().copied().filter(|d| *d != u32::MAX).max().unwrap_or(0);
    let mut out = VarArray::new();
    for l in &laws.law {
        let mut e = VarDictionary::new();
        e.set("id", l.id.as_str());
        e.set("name", l.name.as_str());
        e.set("authority", l.authority as i64);
        e.set("mood", l.mood as i64);
        e.set("effect", l.effect.as_str());
        e.set("cost", l.cost.as_str());
        e.set("live", l.live.as_str());
        e.set("offered_by", l.offered_by.as_str());
        let why = colony.charter.cannot_sign(l);
        let state = match colony.charter.law(&l.id) {
            Some(s) if s.repeal_reach.is_some() => "repealing",
            Some(s) if s.reach >= mat_farthest => "signed",
            Some(_) => "spreading",
            None if why.is_none() => "open",
            None => "locked",
        };
        e.set("state", state);
        e.set("why", why.unwrap_or_default().as_str());
        if let Some(s) = colony.charter.law(&l.id) {
            e.set("shift", s.shift as i64);
            e.set("votes_for", s.votes_for as i64);
            e.set("votes_against", s.votes_against as i64);
            e.set("reach", s.reach as i64);
            // where the wave stands now, whether or not the shift has announced it yet
            let names: Vec<GString> = colony.districts.iter().filter(|d| s.at(colony.district_dist(mat, d))).map(|d| GString::from(d.name.as_str())).collect();
            e.set("reached", &PackedStringArray::from(names.as_slice()));
        }
        out.push(&e.to_variant());
    }
    out
}

/// Apply the shift's Stash-side effects: identifications, the lanterns' oil and the paste from the ledger. What the
/// Stills and Salt Works make rides home as cargo (cargo_bridge::dispatch_shift). Returns the lines.
pub fn apply_to_stash(o: &ShiftOutcome, stash: &mut RoseStash) -> Vec<String> {
    let mut lines = Vec::new();
    if o.identifies > 0 {
        let n = stash.identify_n(o.identifies as usize);
        if n > 0 {
            lines.push(format!("the Appraisal Table named {n} item{}", if n == 1 { "" } else { "s" }));
        }
    }
    let spend = o.upkeep + o.paste_eddies;
    if spend > 0 {
        stash.ledger.eddies = stash.ledger.eddies.saturating_sub(spend);
        lines.push(format!("{} e for oil and paste", spend));
    }
    lines
}
