//! The Colony HUD's reads and verbs (Full Design v3.12, Colony mode: The steering tools, Orders, Alerts, Overlays;
//! UI and screens: Colony HUD, Job grid). Every rule is in the crates; this file shapes their answers for Godot.

use godot::prelude::*;

use crate::{RoseSim, RoseStash};

fn level_name(l: game::Level) -> &'static str {
    match l {
        game::Level::Red => "red",
        game::Level::Yellow => "yellow",
        game::Level::White => "white",
    }
}

#[godot_api(secondary)]
impl RoseSim {
    /// The alerts standing now: [{level "red"|"yellow"|"white", key, text, x, y}] (x -1 when it has no tile).
    #[func]
    fn alerts(&self) -> VarArray {
        let Some(w) = self.world.as_ref() else { return VarArray::new() };
        let mut out = VarArray::new();
        let c = &w.city;
        for a in w.alerts() {
            let mut e = VarDictionary::new();
            e.set("level", level_name(a.level));
            e.set("key", a.key.as_str());
            e.set("text", a.text.as_str());
            e.set("x", a.cell.map_or(-1, |i| (i % c.sim.w) as i64));
            e.set("y", a.cell.map_or(-1, |i| (i / c.sim.w) as i64));
            out.push(&e.to_variant());
        }
        out
    }

    /// The city's log since the last call: [{t (seconds), class, what}].
    #[func]
    fn city_log_new(&mut self) -> VarArray {
        let Some(w) = self.world.as_mut() else { return VarArray::new() };
        let mut out = VarArray::new();
        for ev in w.log.take_new() {
            let mut e = VarDictionary::new();
            e.set("t", ev.seconds() as f64);
            e.set("class", ev.class.name());
            e.set("what", ev.what.as_str());
            out.push(&e.to_variant());
        }
        out
    }

    /// An overlay at one pixel a tile, RGBA: "nutrient", "signal", "light", "zones", "traffic". Empty for an
    /// overlay that waits for its system (noise, danger: Phase D).
    #[func]
    fn overlay(&mut self, kind: GString) -> PackedByteArray {
        let Some(w) = self.world.as_mut() else { return PackedByteArray::new() };
        let defs = &w.defs;
        let c = &w.city;
        let (mw, mh) = (c.sim.w, c.sim.h);
        let mut px = vec![0u8; mw * mh * 4];
        let mut put = |i: usize, rgb: [u8; 3], a: u8| {
            px[i * 4] = rgb[0];
            px[i * 4 + 1] = rgb[1];
            px[i * 4 + 2] = rgb[2];
            px[i * 4 + 3] = a;
        };
        // SEEPAGE: Ooze Glow for a fed or signalled tile, Acid for light, Cyan / Hot Pink / Acid / Chalk for zones
        const GLOW: [u8; 3] = [240, 166, 255];
        const ACID: [u8; 3] = [214, 255, 61];
        const CYAN: [u8; 3] = [61, 226, 255];
        const PINK: [u8; 3] = [255, 61, 166];
        const CHALK: [u8; 3] = [181, 180, 176];
        match kind.to_string().as_str() {
            "nutrient" => {
                for i in 0..mw * mh {
                    let n = c.sim.n[i];
                    if n > 0.02 {
                        put(i, GLOW, (40.0 + 200.0 * n.min(1.0)) as u8);
                    }
                }
            }
            "signal" => {
                for i in 0..mw * mh {
                    let s = c.sim.sig[i];
                    if s > 0.0 {
                        put(i, if s >= 0.3 { CYAN } else { PINK }, (60.0 + 160.0 * s.min(1.0)) as u8);
                    }
                }
            }
            "light" => {
                let lit = w.colony.lit_cells(&defs, &c.sim, true);
                for (i, l) in lit.iter().enumerate() {
                    if *l {
                        put(i, ACID, 90);
                    }
                }
            }
            "zones" => {
                for z in &w.colony.zones {
                    let col = match z.kind {
                        colony::ZoneKind::Stockpile => CYAN,
                        colony::ZoneKind::Hab => ACID,
                        colony::ZoneKind::Quarantine => PINK,
                        colony::ZoneKind::Kill => CHALK,
                    };
                    for &cell in &z.cells {
                        put(cell as usize, col, 140);
                    }
                }
            }
            "traffic" => {
                let t = c.traffic();
                for i in 0..mw * mh {
                    if c.jam[i] {
                        put(i, CHALK, 230);
                    } else if t[i] >= 3 {
                        put(i, GLOW, 200);
                    } else if t[i] >= 1 {
                        put(i, [164, 75, 224], 170);
                    }
                }
            }
            _ => return PackedByteArray::new(),
        }
        PackedByteArray::from(px.as_slice())
    }

    /// What stands on a tile: {x, y, mat ("tip"|"cord"|"node"|"necrotic"|""), sigma, nutrient, traffic, jam, salt,
    /// substrate, structure (id or -1), zone (id or -1), zone_kind, cut (bool), pit_stock}.
    #[func]
    fn tile_info(&mut self, x: i64, y: i64) -> VarDictionary {
        let Some(w) = self.world.as_mut() else { return VarDictionary::new() };
        let defs = &w.defs;
        let mut e = VarDictionary::new();
        let c = &w.city;
        if x < 0 || y < 0 || x as usize >= c.sim.w || y as usize >= c.sim.h {
            return e;
        }
        let i = y as usize * c.sim.w + x as usize;
        e.set("x", x);
        e.set("y", y);
        e.set("mat", match c.sim.hy[i] {
            mat::sim::hy::TIP => "tip",
            mat::sim::hy::CORD => "cord",
            mat::sim::hy::NODE => "node",
            mat::sim::hy::NECRO => "necrotic",
            _ => "",
        });
        e.set("sigma", c.sim.sig[i] as f64);
        e.set("nutrient", c.sim.n[i] as f64);
        e.set("traffic", c.traffic()[i] as i64);
        e.set("jam", c.jam[i]);
        e.set("salt", c.sim.salt[i]);
        e.set("substrate", ["rock", "open", "pipe", "brick", "deposit", "water", "grate", "concrete", "collapse", "slab"].get(c.sim.sub[i] as usize).copied().unwrap_or("?"));
        e.set("structure", w.colony.occupied(&defs, x as u16, y as u16).map_or(-1, |s| s as i64));
        let z = w.colony.zone_at(i as u32);
        e.set("zone", z.map_or(-1, |z| z.id as i64));
        e.set("zone_kind", z.map_or("", |z| z.kind.name()));
        e.set("cut", w.colony.cuts.iter().any(|k| k.x as i64 == x && k.y as i64 == y));
        // the organ on it (Full Design v3.18, Organs) and whether its rule can be set
        let f = progress::StrainFile::shipped();
        match c.sim.organ_at(i).and_then(|o| f.organ.get(o.kind as usize).map(|d| (d, o.rule))) {
            Some((d, rule)) => {
                e.set("organ", d.name.as_str());
                e.set("organ_id", d.id.as_str());
                e.set("organ_text", d.text.as_str());
                e.set("organ_rule", rule as i64);
                e.set("organ_dark", w.colony.dark_bulbs.contains(&(i as u32)) || w.colony.nerve.dark_bulbs.contains(&(i as u32)));
            }
            None => e.set("organ", ""),
        }
        e.set("organ_rules", w.colony.progress.tech.flag("organ_rules"));
        e.set("proposal", w.colony.proposal.contains(&(i as u32)));
        e.set("proposal_tiles", w.colony.proposal.len() as i64);
        e.set("proposal_confirmed", w.colony.proposal_confirmed);
        e
    }

    /// A structure's card: {id, def, name, tier, tiers, built, paid, progress, need, hp, hp_max, sigma, sigma_need,
    /// workers: [{slimo, name, job}], slots, hosts, upgrade: "" or why not, upgrade_cost, quota (bool), store (bool),
    /// zone (id or -1), rotated}.
    #[func]
    fn structure_info(&mut self, id: i64, population: i64) -> VarDictionary {
        let Some(w) = self.world.as_mut() else { return VarDictionary::new() };
        let defs = &w.defs;
        let mut e = VarDictionary::new();
        let c = &w.city;
        let Some(s) = w.colony.get(id as u32).cloned() else { return e };
        let d = &defs.structure[s.def];
        e.set("id", id);
        e.set("def", d.id.as_str());
        e.set("name", d.name.as_str());
        e.set("tier", s.tier as i64);
        e.set("tiers", d.tiers as i64);
        e.set("built", s.built);
        e.set("paid", s.paid);
        e.set("progress", s.progress as i64);
        e.set("need", d.build_at(s.upgrading_to.unwrap_or(1)) as i64);
        e.set("upgrading_to", s.upgrading_to.map_or(0, |t| t as i64));
        e.set("hp", s.hp as i64);
        e.set("hp_max", w.colony.hp_max(&defs, &s) as i64);
        let ai = s.y as usize * c.sim.w + s.x as usize;
        e.set("sigma", c.sim.sig.get(ai).copied().unwrap_or(0.0) as f64);
        e.set("sigma_need", w.colony.sigma_need(d) as f64);
        e.set("x", s.x as i64);
        e.set("y", s.y as i64);
        e.set("dark", s.dark);
        e.set("drowned", s.drowned);
        e.set("hosts", d.hosts.as_str());
        e.set("slots", d.slots_at(s.tier) as i64);
        e.set("rotated", s.rotated);
        let mut workers = VarArray::new();
        for (slimo, job, sid) in w.plan.iter().filter(|p| p.2 == s.id) {
            let mut w = VarDictionary::new();
            w.set("slimo", *slimo as i64);
            w.set("job", job.name());
            w.set("structure", *sid as i64);
            workers.push(&w.to_variant());
        }
        e.set("workers", &workers);
        // the upgrade: what it would cost and whether it may start (a dry run on a copy)
        let mut probe = w.colony.clone();
        let tier = colony::Colony::city_tier(population.max(0) as usize);
        let why = match probe.upgrade(&defs, s.id, tier) {
            Ok(_) => String::new(),
            Err(why) => why,
        };
        e.set("upgrade", why.as_str());
        e.set("upgrade_cost", d.cost_at((s.tier + 1).min(d.tiers)).text().as_str());
        e.set("quota", matches!(d.id.as_str(), "heart" | "pit" | "still"));
        e.set("store", matches!(d.id.as_str(), "rack" | "cold_store"));
        e.set("zone", w.colony.zones.iter().find(|z| z.anchor == Some(s.id)).map_or(-1, |z| z.id as i64));
        e
    }

    /// Mark a tile of Mat to be cut by Build at shift end. "" or why not.
    #[func]
    fn mark_cut(&mut self, x: i64, y: i64) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let c = &w.city;
        if x < 0 || y < 0 || x as usize >= c.sim.w || y as usize >= c.sim.h {
            return GString::from("off the map");
        }
        match w.colony.mark_cut(&c.sim, x as u16, y as u16) {
            Ok(_) => {
                crate::decide(w, format!("cut marked at {x},{y}"));
                GString::new()
            }
            Err(e) => GString::from(e),
        }
    }

    #[func]
    fn unmark_cut(&mut self, x: i64, y: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        x >= 0 && y >= 0 && w.colony.unmark_cut(x as u16, y as u16)
    }

    /// The tiles marked to be cut: [{x, y, progress}].
    #[func]
    fn cuts(&self) -> VarArray {
        let Some(w) = self.world.as_ref() else { return VarArray::new() };
        let mut out = VarArray::new();
        for k in &w.colony.cuts {
            let mut e = VarDictionary::new();
            e.set("x", k.x as i64);
            e.set("y", k.y as i64);
            e.set("progress", k.progress as i64);
            out.push(&e.to_variant());
        }
        out
    }

    /// Lay a tile of salt from the open bag, or a new bag from the Stash. "" or why not; `salt_left()` after.
    #[func]
    fn lay_salt(&mut self, x: i64, y: i64) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        if x < 0 || y < 0 || x as usize >= w.city.sim.w || y as usize >= w.city.sim.h {
            return GString::from("off the map");
        }
        let (stash, table) = (&mut w.stash, &w.table);
        let r = w.colony.lay_salt(&mut w.city.sim, x as u16, y as u16, &mut || game::store::take_salt_bag(stash, table));
        match r {
            Ok(_) => {
                crate::decide(w, format!("salt at {x},{y}"));
                GString::new()
            }
            Err(e) => GString::from(e),
        }
    }

    #[func]
    fn salt_left(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        w.colony.salt_left as i64
    }

    /// Place a ghost turned (or not); -1 when it cannot stand.
    #[func]
    fn place_turned(&mut self, def_id: GString, x: i64, y: i64, rotated: bool) -> i64 {
        let Some(w) = self.world.as_mut() else { return 0 };
        let defs = &w.defs;
        let c = &w.city;
        if x < 0 || y < 0 {
            return -1;
        }
        let id = w.colony.place_r(&defs, &c.sim, &def_id.to_string(), x as u16, y as u16, rotated).map(|id| id as i64).unwrap_or(-1);
        if id >= 0 {
            crate::decide(w, format!("{} placed at {x},{y}{}", defs.get(&def_id.to_string()).map_or("?", |d| d.name.as_str()), if rotated { " turned" } else { "" }));
        }
        id
    }

    /// "" when the def may stand here turned (or not), else why not.
    #[func]
    fn can_place_turned(&mut self, def_id: GString, x: i64, y: i64, rotated: bool) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let defs = &w.defs;
        let c = &w.city;
        if x < 0 || y < 0 {
            return GString::from("off the map");
        }
        match w.colony.can_place_r(&defs, &c.sim, &def_id.to_string(), x as u16, y as u16, rotated) {
            Ok(()) => GString::new(),
            Err(e) => GString::from(e.to_string().as_str()),
        }
    }

    /// The anchors in a rect of tiles where the def may stand (turned or not), as x, y pairs: the Build tool's map of
    /// where it can go (Full Design v3.21, this design's).
    #[func]
    fn place_spots(&mut self, def_id: GString, x0: i64, y0: i64, span_w: i64, span_h: i64, rotated: bool) -> PackedInt32Array {
        let Some(w) = self.world.as_ref() else { return PackedInt32Array::new() };
        let mut out = PackedInt32Array::new();
        let defs = &w.defs;
        let c = &w.city;
        let id = def_id.to_string();
        for y in y0.max(0)..(y0 + span_h).min(c.sim.h as i64) {
            for x in x0.max(0)..(x0 + span_w).min(c.sim.w as i64) {
                if w.colony.can_place_r(&defs, &c.sim, &id, x as u16, y as u16, rotated).is_ok() {
                    out.push(x as i32);
                    out.push(y as i32);
                }
            }
        }
        out
    }

    /// Order a slimo to work a structure the next shift (over its priorities). "" or why not.
    #[func]
    fn order(&mut self, slimo: i64, structure: i64) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let defs = &w.defs;
        match w.colony.order(&defs, slimo as u32, structure as u32) {
            Ok(()) => {
                let name = w.colony.get(structure as u32).map_or("?".into(), |s| defs.structure[s.def].name.clone());
                crate::decide(w, format!("slimo {slimo} ordered to the {name}"));
                GString::new()
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// slimo -> structure ordered for the next shift.
    #[func]
    fn orders(&self) -> VarDictionary {
        let Some(w) = self.world.as_ref() else { return VarDictionary::new() };
        let mut d = VarDictionary::new();
        for (a, b) in &w.colony.orders {
            d.set(*a as i64, *b as i64);
        }
        d
    }

    /// Paint a stockpile against a built Rack or Cold Store: "" or why not.
    #[func]
    fn paint_stockpile(&mut self, anchor: i64, cells: PackedInt32Array) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let defs = &w.defs;
        let c = &w.city;
        let cells: Vec<u32> = cells.as_slice().iter().filter(|x| **x >= 0).map(|x| *x as u32).collect();
        match w.colony.paint_stockpile(&defs, &c.sim, anchor as u32, &cells) {
            Ok(z) => {
                let n = w.colony.zones.iter().find(|q| q.id == z).map_or(0, |q| q.cells.len());
                crate::decide(w, format!("stockpile {z} painted, {n} cells"));
                w.publish();
                GString::new()
            }
            Err(e) => GString::from(e),
        }
    }

    #[func]
    fn set_zone_priority(&mut self, zone: i64, priority: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let ok = w.colony.set_zone_priority(zone as u32, priority.clamp(1, 4) as u8);
        if ok {
            crate::decide(w, format!("stockpile {zone} at priority {}", priority.clamp(1, 4)));
        }
        ok
    }

    /// The tiles salted (the Mat will not enter them).
    #[func]
    fn salt_cells(&self) -> PackedInt32Array {
        let Some(c) = self.city() else { return PackedInt32Array::new() };
        PackedInt32Array::from(c.sim.salt.iter().enumerate().filter(|(_, s)| **s).map(|(i, _)| i as i32).collect::<Vec<_>>().as_slice())
    }

    /// A job priority set on the Job grid or the Slimo card goes to the log (the roster's verb is RosePeople's).
    #[func]
    fn log_priority(&mut self, name: GString, job: GString, value: i64) {
        self.decide(format!("{name}: {job} at {value}"));
    }

    #[func]
    fn set_zone_filter(&mut self, zone: i64, categories: PackedStringArray, min_tier: i64, min_condition: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let ok = w.colony.set_zone_filter(zone as u32, crate::colony_bridge::filter_from(&categories, min_tier, min_condition));
        if ok {
            crate::decide(w, format!("stockpile {zone} filter set"));
        }
        ok
    }

    /// The zone holding a tile, or -1.
    #[func]
    fn zone_at(&self, x: i64, y: i64) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        let c = &w.city;
        if x < 0 || y < 0 {
            return -1;
        }
        w.colony.zone_at(y as u32 * c.sim.w as u32 + x as u32).map_or(-1, |z| z.id as i64)
    }

    /// The Job grid (Full Design v3.12): one row a slimo — {id, name, depth, district, district_name, priorities
    /// [9], effective [9] (with the foreman's where it stands over), job (this shift's plan), structure_name}.
    #[func]
    fn job_grid(&self) -> VarArray {
        let Some(w) = self.world.as_ref() else { return VarArray::new() };
        let defs = &w.defs;
        let mut out = VarArray::new();
        for r in &w.roster.records {
            let mut e = VarDictionary::new();
            e.set("id", r.id as i64);
            e.set("name", r.full_name().as_str());
            e.set("depth", r.depth.tag());
            let d = w.colony.district_of(r.id);
            e.set("district", d.map_or(-1, |d| d.id as i64));
            e.set("district_name", d.map_or("", |d| d.name.as_str()));
            e.set("priorities", &PackedInt32Array::from(r.priorities.iter().map(|x| *x as i32).collect::<Vec<_>>().as_slice()));
            let eff: Vec<i32> = ::people::Job::all().iter().map(|j| w.colony.priority_for(r, *j) as i32).collect();
            e.set("effective", &PackedInt32Array::from(eff.as_slice()));
            let plan = w.plan.iter().find(|x| x.0 == r.id);
            e.set("job", plan.map_or("", |x| x.1.name()));
            e.set("structure_name", plan.and_then(|x| w.colony.get(x.2)).map_or("", |s| defs.structure[s.def].name.as_str()));
            e.set("status", crate::people::status_text(&r.status).as_str());
            out.push(&e.to_variant());
        }
        out
    }
}

#[godot_api(secondary)]
impl RoseStash {
    /// The Heart's floor (what the grid has no room for) as the Stash screen draws it: items() fields, x, y their
    /// order, cell -2.
    #[func]
    fn floor_items(&mut self) -> VarArray {
        let Some(mut sim) = self.sim.clone() else { return VarArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarArray::new() };
        let mut out = VarArray::new();
        let t = w.table.clone();
        let floor = (&w.stash).floor.clone();
        for (k, s) in floor.iter().enumerate() {
            let mut e = crate::item_dict(&t, inventory::FLOOR + s.id, &s.item);
            e.set("x", (k % 8) as i64);
            e.set("y", (k / 8) as i64);
            e.set("w", 1);
            e.set("h", 1);
            e.set("cell", -2);
            out.push(&e.to_variant());
        }
        out
    }

    /// Items in the Heart's grid (count() is the whole Stash, the stockpiles with it).
    #[func]
    fn count_grid(&mut self) -> i64 {
        let Some(mut sim) = self.sim.clone() else { return 0 };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return 0 };
        (&w.stash).grid.placed().len() as i64
    }

    /// What the Stash can pay with now: {parts, chrome, eddies} (game::store::holds), for the Build menu's costs.
    #[func]
    fn holds(&mut self) -> VarDictionary {
        let Some(mut sim) = self.sim.clone() else { return VarDictionary::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarDictionary::new() };
        let h = game::store::holds(&w.stash, &w.ledger, &w.table);
        let mut d = VarDictionary::new();
        d.set("parts", h.parts as i64);
        d.set("chrome", h.chrome as i64);
        d.set("eddies", h.eddies as i64);
        d
    }

    /// The stockpiles, the Stash's other rooms: [{zone, anchor, name, cap, count, waiting, priority, cold,
    /// categories, min_tier, min_condition, mat_cells, off_cells}].
    #[func]
    fn rooms(&mut self) -> VarArray {
        let Some(mut sim) = self.sim.clone() else { return VarArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarArray::new() };
        let mut out = VarArray::new();
        for r in &(&w.stash).rooms {
            let mut e = VarDictionary::new();
            e.set("zone", r.zone as i64);
            e.set("anchor", r.anchor as i64);
            e.set("name", r.name.as_str());
            e.set("cap", r.cap as i64);
            e.set("count", r.items.len() as i64);
            e.set("waiting", r.items.iter().filter(|s| s.cell.is_none()).count() as i64);
            e.set("priority", r.priority as i64);
            e.set("cold", r.cold);
            e.set("categories", &PackedStringArray::from(r.categories.iter().map(|c| GString::from(c.as_str())).collect::<Vec<_>>().as_slice()));
            e.set("min_tier", r.min_tier as i64);
            e.set("min_condition", r.min_condition as i64);
            e.set("mat_cells", r.mat_cells.len() as i64);
            e.set("off_cells", r.off_cells.len() as i64);
            out.push(&e.to_variant());
        }
        out
    }

    /// A stockpile's items as the Stash screen draws them: the same fields as items() with x, y the item's order in
    /// the room and its cell (-1 waiting on the store's footprint).
    #[func]
    fn room_items(&mut self, zone: i64) -> VarArray {
        let Some(mut sim) = self.sim.clone() else { return VarArray::new() };
        let mut g = sim.bind_mut();
        let Some(w) = g.world.as_mut() else { return VarArray::new() };
        let mut out = VarArray::new();
        let t = w.table.clone();
        let Some(r) = (&w.stash).rooms.iter().find(|r| r.zone as i64 == zone).cloned() else { return out };
        for (k, s) in r.items.iter().enumerate() {
            let mut e = crate::item_dict(&t, inventory::SHELF + s.id, &s.item);
            e.set("x", (k % 8) as i64);
            e.set("y", (k / 8) as i64);
            e.set("w", 1);
            e.set("h", 1);
            e.set("cell", s.cell.map_or(-1, |c| c as i64));
            out.push(&e.to_variant());
        }
        out
    }
}
