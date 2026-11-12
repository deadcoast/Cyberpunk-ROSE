//! The campaign's reads and verbs (Full Design v3.19, Narrative and Ivan): the chapter's card, the panel sets waiting,
//! the envoy's answer, the alliance, a run's note. The rules are `story` and `game::campaign`; this file shapes them.

use godot::prelude::*;

use crate::{RosePeople, RoseSim};
use story::campaign::Accord;

#[godot_api(secondary)]
impl RoseSim {
    /// The campaign as the HUD shows it: {mode, chapter, card, accord ("" waiting accepted refused), envoy (the grate's
    /// tile, -1), great_at (-1), ending ("" until one), warchest (-1), waiting (a panel set waits), seen, difficulty}.
    #[func]
    fn campaign(&self) -> VarDictionary {
        let c = &self.campaign;
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        let mut d = VarDictionary::new();
        d.set("mode", if c.mode == story::Mode::Endless { "endless" } else { "campaign" });
        d.set("difficulty", c.difficulty.name());
        d.set("chapter", c.chapter as i64);
        d.set("card", c.card(day).as_str());
        d.set("accord", match c.accord {
            Some(Accord::Waiting { .. }) => "waiting",
            Some(Accord::Accepted { .. }) => "accepted",
            Some(Accord::Refused { .. }) => "refused",
            None => "",
        });
        let envoy = c.envoy_zone.and_then(|z| self.threats.as_ref().and_then(|t| t.zones.get(z as usize)).map(|z| z.grate as i64)).unwrap_or(-1);
        d.set("envoy", envoy);
        d.set("great_at", c.great_at.map_or(-1, |x| x as i64));
        d.set("ending", c.ending.clone().unwrap_or_default().as_str());
        d.set("warchest", c.warchest.map_or(-1, |x| x as i64));
        d.set("waiting", !c.panels.is_empty());
        let mut seen = PackedStringArray::new();
        for s in &c.seen {
            seen.push(s.as_str());
        }
        d.set("seen", &seen);
        d
    }

    /// The next panel set waiting, marked seen: {key, title, panels: [{slot, caption}]}; empty when none waits.
    #[func]
    fn take_panels(&mut self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(key) = self.campaign.take_panels() else { return d };
        d.set("key", key.as_str());
        d.set("title", panel_title(&key).as_str());
        d.set("panels", &panels_array(&self.campaign, &key));
        d
    }

    /// A panel set already seen (the Codex's Lore replays it): {key, title, panels}.
    #[func]
    fn panel_set(&self, key: GString) -> VarDictionary {
        let mut d = VarDictionary::new();
        let key = key.to_string();
        d.set("key", key.as_str());
        d.set("title", panel_title(&key).as_str());
        d.set("panels", &panels_array(&self.campaign, &key));
        d
    }

    /// The envoy's answer (chapter 3): "" when it stands, else why not.
    #[func]
    fn answer_accord(&mut self, people: Gd<RosePeople>, accept: bool) -> GString {
        let defs = self.defs().clone();
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        let (Some(city), Some(t), Some(e), Some(st)) = (self.city.as_mut(), self.threats.as_mut(), self.economy.as_mut(), self.stash.as_mut()) else { return "no city".into() };
        let mut p = people.clone();
        let mut p = p.bind_mut();
        p.ensure();
        let p = &mut *p;
        let mut st = st.bind_mut();
        st.ensure();
        let st = &mut *st;
        let guards = self.posts.guards.len();
        let mut parts = game::campaign::Parts { city, colony: &mut self.colony, defs: &defs, threats: t, economy: e, roster: &mut p.roster, died: &mut p.died_this_shift, stash: st.stash.as_mut().unwrap(), table: st.table.as_ref().unwrap(), log: &mut self.log, guards };
        match game::campaign::answer_accord(&mut self.campaign, &mut parts, accept, day) {
            Ok(()) => GString::new(),
            Err(why) => why.as_str().into(),
        }
    }

    /// Development only (the play script): Authority earned, as deeds would earn it.
    #[func]
    fn debug_earn(&mut self, n: i64) {
        self.colony.charter.earn(n.clamp(0, 99) as u32);
    }

    /// Development only (the play script): the campaign at chapter `n`, the earlier ones closed, opening at the next
    /// shift's end when its stratum stands; chapter 7 with its Great Cleansing come and gone, so the next shift's end
    /// reads the ending off the genome.
    #[func]
    fn debug_chapter(&mut self, n: i64) {
        let n = n.clamp(1, 7) as u8;
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        let c = &mut self.campaign;
        for k in 1..n {
            if !c.closed.iter().any(|x| x.0 == k) {
                c.closed.push((k, day));
            }
        }
        c.chapter = n;
        c.open_since = 0;
        if n == 7 {
            c.open_since = day;
            c.great_at = Some(day);
            c.cleansing_sent = true;
        }
    }
}

#[godot_api(secondary)]
impl RoseSim {
    /// Ivan on the map (Full Design v3.19): {x, y, goal_x, goal_y (-1), hp, hp_max, fell, state (his Authority state),
    /// the verbs' readiness rally, parley, feed, ooze (true when not yet spent this shift), at_market (its index, -1),
    /// at_pit (a Pit's structure, -1), hostile_near}.
    #[func]
    fn ivan(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let v = &self.campaign.ivan;
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        d.set("x", v.x as i64);
        d.set("y", v.y as i64);
        d.set("goal_x", v.goal.map_or(-1, |g| g.0 as i64));
        d.set("goal_y", v.goal.map_or(-1, |g| g.1 as i64));
        d.set("hp", v.hp as f64);
        d.set("hp_max", story::campaign::IVAN_HP as f64);
        d.set("fell", v.fell);
        d.set("state", self.colony.charter.ivan_state() as i64);
        d.set("rally", v.rally != day);
        d.set("parley", v.parley != day);
        d.set("feed", v.feed != day);
        d.set("ooze", v.ooze != day);
        if let Some(c) = self.city.as_ref() {
            d.set("at_market", game::ivan::market_here(&self.campaign, c).map_or(-1, |m| m as i64));
            let defs = self.defs.clone().unwrap_or_else(crate::colony_bridge::structure_file);
            d.set("at_pit", game::ivan::pit_here(&self.campaign, c, &self.colony, &defs).map_or(-1, |p| p.0 as i64));
        }
        let me = (v.x as i32, v.y as i32);
        let near = self.threats.as_ref().is_some_and(|t| t.bodies.iter().any(|b| !b.dead && !b.gone && b.role.hostile() && (b.x as i32 - me.0).abs().max((b.y as i32 - me.1).abs()) <= game::ivan::OOZE_REACH));
        d.set("hostile_near", near);
        d
    }

    /// Send Ivan to a tile of the Mat: "" or why not.
    #[func]
    fn ivan_order(&mut self, x: i64, y: i64) -> GString {
        let Some(c) = self.city.as_ref() else { return "no city".into() };
        match game::ivan::order(&mut self.campaign, &c.sim, x.max(0) as u16, y.max(0) as u16) {
            Ok(()) => GString::new(),
            Err(e) => e.as_str().into(),
        }
    }

    /// Ivan's verbs: "" or why not (Parley and Feed return what they did on success, prefixed "+").
    #[func]
    fn ivan_rally(&mut self, people: Gd<RosePeople>) -> GString {
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        let Some(c) = self.city.as_mut() else { return "no city".into() };
        let mut p = people.clone();
        let mut p = p.bind_mut();
        p.ensure();
        match game::ivan::rally(&mut self.campaign, c, &mut p.roster, day, &mut self.log) {
            Ok(n) => format!("+{n} rallied").as_str().into(),
            Err(e) => e.as_str().into(),
        }
    }

    #[func]
    fn ivan_parley(&mut self) -> GString {
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        let (Some(c), Some(e)) = (self.city.as_ref(), self.economy.as_mut()) else { return "no city".into() };
        match game::ivan::parley(&mut self.campaign, c, &mut self.colony, e, day, &mut self.log) {
            Ok(line) => format!("+{line}").as_str().into(),
            Err(e) => e.as_str().into(),
        }
    }

    #[func]
    fn ivan_feed(&mut self, handle: i64) -> GString {
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        let defs = self.defs().clone();
        let Some(st) = self.stash.as_mut() else { return "no Stash".into() };
        let mut st = st.bind_mut();
        st.ensure();
        let st = &mut *st;
        let Some(c) = self.city.as_mut() else { return "no city".into() };
        match game::ivan::feed(&mut self.campaign, c, &mut self.colony, &defs, st.stash.as_mut().unwrap(), st.table.as_ref().unwrap(), handle.max(0) as u32, day, &mut self.log) {
            Ok(line) => format!("+{line}").as_str().into(),
            Err(e) => e.as_str().into(),
        }
    }

    /// Ooze-time: "" when it holds (the view slows the clock to OOZE_SCALE for OOZE_SECONDS), else why not.
    #[func]
    fn ivan_ooze(&mut self) -> GString {
        let day = self.threats.as_ref().map_or(1, |t| t.shift);
        let Some(t) = self.threats.as_ref() else { return "no threats".into() };
        match game::ivan::ooze(&mut self.campaign, t, day) {
            Ok(()) => GString::new(),
            Err(e) => e.as_str().into(),
        }
    }

    /// Ivan's line on screen now, or "".
    #[func]
    fn bark(&self) -> GString {
        self.campaign.barker.current().unwrap_or("").into()
    }

    /// A bark from the run (the Run HUD: runner_down) with the runner's name.
    #[func]
    fn bark_event(&mut self, event: GString, name: GString) {
        let n = name.to_string();
        self.log.bark(&event.to_string(), &[("name", n.as_str())]);
        if let (Some(c), Some(t)) = (self.city.as_ref(), self.threats.as_mut()) {
            game::ivan::voice(&mut self.campaign, &self.colony, &c.sim, t, &mut self.log, c.tick);
        }
    }

    /// A run home (Sim's extraction): Ivan's line on it, and chapter 5 reads the Vault's. `rig` is the run's rig
    /// (postcard), whose best item names the run's tier.
    #[func]
    fn campaign_run_home(&mut self, stratum: i64, name: GString, extracted: bool, died: bool, rig: PackedByteArray) {
        let table = items::ItemTable::from_toml(items::ITEMS_TOML).ok();
        let best = postcard::from_bytes::<inventory::Rig>(rig.as_slice()).ok().and_then(|r| {
            let t = table.as_ref()?;
            r.backpack.items().chain(r.quick.iter().flatten()).map(|it| t.def(it.def).tier).max()
        });
        let hauled = !rig.is_empty();
        let n = name.to_string();
        if died {
            self.log.bark("runner_dead", &[("name", n.as_str())]);
        } else if extracted && hauled {
            self.log.bark("haul", &[("name", n.as_str())]);
        }
        game::campaign::note_run(&mut self.campaign, stratum.clamp(0, 255) as u8, extracted, best);
    }
}

fn panel_title(key: &str) -> String {
    let f = story::ChapterFile::shipped();
    let mut parts = key.split('.');
    match (parts.next(), parts.next()) {
        (Some("end"), Some(id)) => f.ending(id).map_or(String::new(), |e| format!("THE {} ENDING", e.name.to_uppercase())),
        (Some(ch), Some(when)) => {
            let n: u8 = ch.trim_start_matches("ch").parse().unwrap_or(0);
            f.get(n).map_or(String::new(), |c| format!("CHAPTER {} {}{}", c.n, c.name.to_uppercase(), if when == "close" { ": THE END OF IT" } else { "" }))
        }
        _ => String::new(),
    }
}

fn panels_array(c: &story::Campaign, key: &str) -> VarArray {
    let mut a = VarArray::new();
    for p in c.set(key) {
        let mut e = VarDictionary::new();
        e.set("slot", p.slot.as_str());
        e.set("caption", p.caption.as_str());
        a.push(&e.to_variant());
    }
    a
}
