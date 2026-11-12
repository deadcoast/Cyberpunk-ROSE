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

    /// A run home (Sim's extraction): the stratum, whether it extracted, the best tier it brought (0 Common .. 4 Relic,
    /// -1 none) — chapter 5 reads the Vault's.
    #[func]
    fn campaign_note_run(&mut self, stratum: i64, extracted: bool, best_tier: i64) {
        let tier = match best_tier {
            0 => Some(items::Tier::Common),
            1 => Some(items::Tier::Uncommon),
            2 => Some(items::Tier::Rare),
            3 => Some(items::Tier::Blackline),
            4 => Some(items::Tier::Relic),
            _ => None,
        };
        game::campaign::note_run(&mut self.campaign, stratum.clamp(0, 255) as u8, extracted, tier);
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
