//! The progression's reads and verbs (Full Design v3.18, Progression; UI and screens: Bench, Strain Lab, Codex). Every
//! rule is in the progress crate, the mat crate's organs and `game::grow`; this file shapes their answers for Godot.

use godot::prelude::*;

use crate::{RosePeople, RoseSim};
use progress::tech::{BRANCHES, BRANCH_NAMES};
use progress::{CollectionsFile, Project, StrainFile, TechFile};

fn strings(v: &[String]) -> PackedStringArray {
    let mut a = PackedStringArray::new();
    for s in v {
        a.push(s.as_str());
    }
    a
}

#[godot_api(secondary)]
impl RoseSim {
    /// The tree for the Bench's Research tab: every node with its state (researched, open, locked) and cost, the
    /// project standing, the blueprints the Stash holds, the history.
    #[func]
    fn tech(&self) -> VarDictionary {
        let f = TechFile::shipped();
        let t = &self.colony.progress.tech;
        let mut d = VarDictionary::new();
        d.set("open", &PackedInt32Array::from(t.open.iter().map(|&x| x as i32).collect::<Vec<i32>>().as_slice()));
        d.set("branches", &strings(&BRANCH_NAMES.iter().map(|s| s.to_string()).collect::<Vec<_>>()));
        let mut nodes = VarArray::new();
        for n in &f.node {
            let p = Project::Node(n.id.clone());
            let mut e = VarDictionary::new();
            e.set("id", n.id.as_str());
            e.set("name", n.name.as_str());
            e.set("branch", n.branch_index() as i64);
            e.set("tier", n.tier as i64);
            e.set("text", n.text.as_str());
            let state = if t.has(&n.id) { "researched" } else if n.tier <= t.open[n.branch_index()] { "open" } else { "locked" };
            e.set("state", state);
            let (parts, chrome) = f.cost_of(&p);
            e.set("points", f.points_of(&p) as i64);
            e.set("parts", parts as i64);
            e.set("chrome", chrome as i64);
            e.set("paid", t.was_paid(&p));
            e.set("why", t.cannot_pick(&p).unwrap_or_default().as_str());
            e.set("current", t.project.as_ref() == Some(&p));
            nodes.push(&e.to_variant());
        }
        d.set("nodes", &nodes);
        let mut proj = VarDictionary::new();
        if let Some(p) = &t.project {
            proj.set("name", t.project_name().unwrap_or_default().as_str());
            proj.set("got", t.got as i64);
            proj.set("need", f.points_of(p) as i64);
            proj.set("study", matches!(p, Project::Study(_)));
            let workers: Vec<String> = self.plan.iter().filter(|a| a.1 == ::people::Job::Research).map(|a| a.0.to_string()).collect();
            proj.set("workers", workers.len() as i64);
        }
        d.set("project", &proj);
        let mut bps = VarArray::new();
        if let Some(st) = self.stash.as_ref() {
            let st = st.bind();
            if let (Some(stash), Some(table)) = (st.stash.as_ref(), st.table.as_ref()) {
                let mut seen: Vec<String> = Vec::new();
                for (_, _, it) in stash.handles() {
                    let id = table.def(it.def).id.clone();
                    let Some((b, tier)) = f.blueprint(&id) else { continue };
                    if seen.contains(&id) {
                        continue;
                    }
                    seen.push(id.clone());
                    let mut e = VarDictionary::new();
                    e.set("item", id.as_str());
                    e.set("name", table.def(it.def).name.as_str());
                    e.set("branch", b as i64);
                    e.set("tier", tier as i64);
                    e.set("points", f.study[(tier as usize - 2).min(2)] as i64);
                    e.set("why", t.cannot_pick(&Project::Study(id.clone())).unwrap_or_default().as_str());
                    bps.push(&e.to_variant());
                }
            }
        }
        d.set("blueprints", &bps);
        let mut hist = VarArray::new();
        for (day, what) in t.history.iter().rev().take(6) {
            hist.push(&format!("shift {day}: {what}").to_variant());
        }
        d.set("history", &hist);
        d
    }

    /// Pick a project: `kind` "node" or "study", `id` the node's or the blueprint item's. Pays for it out of the
    /// Stash. Returns why not, or "" when it stands.
    #[func]
    fn research_pick(&mut self, kind: GString, id: GString) -> GString {
        let p = if kind.to_string() == "study" { Project::Study(id.to_string()) } else { Project::Node(id.to_string()) };
        let Some(st) = self.stash.as_mut() else { return "no Stash".into() };
        let mut st = st.bind_mut();
        st.ensure();
        let st = &mut *st;
        let (Some(stash), Some(table)) = (st.stash.as_mut(), st.table.as_ref()) else { return "no Stash".into() };
        match game::grow::pick_project(&mut self.colony, stash, &mut st.ledger, table, p) {
            Ok(()) => {
                let name = self.colony.progress.tech.project_name().unwrap_or_default();
                self.log.push(self.city.as_ref().map_or(0, |c| c.tick), game::Class::Decision, format!("research: {name}"));
                GString::new()
            }
            Err(why) => why.as_str().into(),
        }
    }

    /// Set the project aside: its points stay with it.
    #[func]
    fn research_drop(&mut self) {
        self.colony.progress.tech.drop_project();
    }

    /// The strain for the Strain Lab: the organs with their lines, grown and cap; the classes and which are held;
    /// the Strainbook's count; the lineage as each gene's values over the shifts.
    #[func]
    fn strain(&self) -> VarDictionary {
        let f = StrainFile::shipped();
        let mut d = VarDictionary::new();
        let Some(city) = self.city.as_ref() else { return d };
        let g = city.sim.genome.0;
        let cap = 1 + city.sim.node_count() / f.organ_per_nodes.max(1);
        let mut organs = VarArray::new();
        for (k, o) in f.organ.iter().enumerate() {
            let gi = progress::strain::gene_index(&o.gene).unwrap_or(0);
            let mut e = VarDictionary::new();
            e.set("id", o.id.as_str());
            e.set("name", o.name.as_str());
            e.set("art", o.art.as_str());
            e.set("gene", mat::genome::GENE_NAMES[gi]);
            e.set("line", o.line);
            e.set("value", g[gi]);
            e.set("crossed", g[gi] > o.line);
            e.set("count", city.sim.organs_of(k as u8).count() as i64);
            e.set("cap", cap as i64);
            e.set("seen", self.colony.progress.strain.has_card(&o.id));
            e.set("text", o.text.as_str());
            organs.push(&e.to_variant());
        }
        d.set("organs", &organs);
        let mut classes = VarArray::new();
        for c in &f.class {
            let mut e = VarDictionary::new();
            e.set("id", c.id.as_str());
            e.set("name", c.name.as_str());
            let names: Vec<String> = c.genes.iter().map(|k| mat::genome::GENE_NAMES[progress::strain::gene_index(k).unwrap_or(0)].to_string()).collect();
            e.set("genes", &strings(&names));
            e.set("line", c.line);
            e.set("held", self.colony.progress.strain.holds(&c.id));
            e.set("text", c.text.as_str());
            classes.push(&e.to_variant());
        }
        d.set("classes", &classes);
        d.set("book", self.colony.progress.strain.book.len() as i64);
        d.set("book_total", f.cards().len() as i64);
        // the lineage: ten rows of the shifts' values
        let lin = &self.colony.progress.strain.lineage;
        let mut rows = VarArray::new();
        for gi in 0..10 {
            rows.push(&PackedFloat32Array::from(lin.iter().map(|(_, v)| v[gi]).collect::<Vec<f32>>().as_slice()).to_variant());
        }
        d.set("lineage", &rows);
        d
    }

    /// The organs on the Mat for the World view: {i, x, y, id, name, art, rule, dark, housed}.
    #[func]
    fn organs(&mut self) -> VarArray {
        let f = StrainFile::shipped();
        let defs = self.defs().clone();
        let defs = &defs;
        let mut a = VarArray::new();
        let Some(city) = self.city.as_ref() else { return a };
        let w = city.sim.w;
        for o in &city.sim.organs {
            let Some(od) = f.organ.get(o.kind as usize) else { continue };
            let (x, y) = ((o.i as usize % w) as u16, (o.i as usize / w) as u16);
            let mut e = VarDictionary::new();
            e.set("i", o.i as i64);
            e.set("x", x as i64);
            e.set("y", y as i64);
            e.set("id", od.id.as_str());
            e.set("name", od.name.as_str());
            e.set("art", od.art.as_str());
            e.set("rule", o.rule as i64);
            e.set("dark", self.colony.dark_bulbs.contains(&o.i));
            e.set("housed", self.colony.occupied(defs, x, y).is_some_and(|sid| self.colony.get(sid).is_some_and(|s| defs.structure[s.def].category != "organ")));
            a.push(&e.to_variant());
        }
        a
    }

    /// Set an organ's rule (Organ rules, Mycelium tier 3): the Bulb's Attention line 0-3 (50, 25, 75, never), the
    /// Sphincter's 0-4 (8, 4, 12 tiles, always shut, always open), the Spore Sac's 0-1 (release, hold).
    #[func]
    fn organ_rule(&mut self, tile: i64, rule: i64) -> GString {
        if !self.colony.progress.tech.flag("organ_rules") {
            return "needs Organ rules researched".into();
        }
        let f = StrainFile::shipped();
        let Some(city) = self.city.as_mut() else { return "no city".into() };
        let Some(o) = city.sim.organs.iter_mut().find(|o| o.i as i64 == tile) else { return "no organ there".into() };
        let most = match f.organ.get(o.kind as usize).map(|d| d.id.as_str()) {
            Some("bulb") => 3,
            Some("sphincter") => 4,
            Some("sac") => 1,
            _ => return "this organ keeps no rule".into(),
        };
        o.rule = rule.clamp(0, most) as u8;
        GString::new()
    }

    /// The Codex's collections: the Scrapdex (every entry, seen or a silhouette), the Strainbook's cards, the curio
    /// sets, the trophies; the Memorial is the roster's.
    #[func]
    fn codex(&self, people: Gd<RosePeople>) -> VarDictionary {
        let mut d = VarDictionary::new();
        let c = &self.colony.progress.collections;
        let cf = CollectionsFile::shipped();
        let table = items::ItemTable::from_toml(items::ITEMS_TOML).expect("items");
        let mut dex = VarArray::new();
        for id in cf.scrapdex(&table) {
            let def = table.def(table.id_of(id).unwrap());
            let mut e = VarDictionary::new();
            e.set("id", id);
            e.set("name", def.name.as_str());
            e.set("tier", format!("{:?}", def.tier).to_lowercase().as_str());
            e.set("seen", c.seen(id));
            e.set("codex", def.codex.as_str());
            dex.push(&e.to_variant());
        }
        d.set("scrapdex", &dex);
        let sf = StrainFile::shipped();
        let mut book = VarArray::new();
        for (id, name, text) in sf.cards() {
            let mut e = VarDictionary::new();
            e.set("id", id);
            e.set("name", name);
            e.set("text", text);
            e.set("art", sf.organ.iter().find(|o| o.id == id).map_or("", |o| o.art.as_str()));
            e.set("earned", self.colony.progress.strain.book.iter().find(|b| b.0 == id).map_or(-1, |b| b.1 as i64));
            book.push(&e.to_variant());
        }
        d.set("strainbook", &book);
        let owned: Vec<items::SetBonus> = self.stash.as_ref().map(|st| {
            let st = st.bind();
            st.stash.as_ref().map(|s| items::set_bonuses(&table, s.all_items())).unwrap_or_default()
        }).unwrap_or_default();
        let mut sets = VarArray::new();
        for s in &cf.set {
            let mut e = VarDictionary::new();
            e.set("id", s.id.as_str());
            e.set("name", s.name.as_str());
            e.set("text", s.text.as_str());
            let n = owned.iter().find(|b| b.set == s.id).map_or(0, |b| b.owned);
            e.set("owned", n as i64);
            e.set("complete", c.set_flag(&s.flag));
            let members: Vec<String> = table.defs().filter(|x| x.set.as_deref() == Some(s.id.as_str())).map(|x| x.id.clone()).collect();
            e.set("items", &strings(&members));
            sets.push(&e.to_variant());
        }
        d.set("sets", &sets);
        let mut trophies = VarArray::new();
        for t in &cf.trophy {
            let mut e = VarDictionary::new();
            e.set("id", t.id.as_str());
            e.set("name", t.name.as_str());
            e.set("text", t.text.as_str());
            e.set("won", c.trophies.iter().find(|x| x.0 == t.id).map_or(-1, |x| x.1 as i64));
            trophies.push(&e.to_variant());
        }
        d.set("trophies", &trophies);
        let p = people.bind();
        let mut memorial = VarArray::new();
        for r in p.roster.memorial.iter().rev() {
            let mut e = VarDictionary::new();
            e.set("name", r.full_name().as_str());
            e.set("depth", format!("D{}", r.depth as u8 + 1).as_str());
            e.set("past", r.past.name());
            e.set("line", format!("{} runs survived, {} crafts", r.survived, r.crafts).as_str());
            memorial.push(&e.to_variant());
        }
        d.set("memorial", &memorial);
        let _ = BRANCHES;
        d
    }
}
