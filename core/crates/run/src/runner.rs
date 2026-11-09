//! The Runner's body and hands: four sections, statuses, stamina, actions with wind-ups, the rig. Numbers from
//! data/creatures.toml `[runner]` (Full Design v3.1, Combat and The Runner's body).

use crate::creature::{speed_units, ticks, RunnerRules, FP};
use inventory::Rig;
use items::{ItemInstance, ItemTable};
use mat::rng::Pcg32;

pub const HEAD: usize = 0;
pub const TORSO: usize = 1;
pub const ARMS: usize = 2;
pub const LEGS: usize = 3;
pub const SECTION_NAMES: [&str; 4] = ["head", "torso", "arms", "legs"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Bleeding,
    Blinded,
    ChromeShock,
    Soaked,
    FungalFever,
    Grabbed,
}

impl StatusKind {
    pub fn name(self) -> &'static str {
        match self {
            StatusKind::Bleeding => "bleeding",
            StatusKind::Blinded => "blinded",
            StatusKind::ChromeShock => "chrome shock",
            StatusKind::Soaked => "soaked",
            StatusKind::FungalFever => "fungal fever",
            StatusKind::Grabbed => "grabbed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub kind: StatusKind,
    /// ticks left; i32::MAX = until treated
    pub left: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Idle,
    /// melee: wind-up then an active window on the tile ahead
    Windup { left: i32 },
    Swing { left: i32 },
    Dodge { left: i32, dx: i32, dy: i32 },
    Reload { left: i32 },
    Dig { x: i32, y: i32, progress: i32 },
    Extract { progress: i32 },
}

/// How the Runner holds the world this tick: on a floor, clinging to the back wall or ceiling, in shallow water,
/// or swimming deep water.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    Walk,
    Crawl,
    Wade,
    Swim,
}

impl Pose {
    pub fn name(self) -> &'static str {
        match self {
            Pose::Walk => "walk",
            Pose::Crawl => "crawl",
            Pose::Wade => "wade",
            Pose::Swim => "swim",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Life {
    Up,
    Down { left: i32 },
    Dead,
}

#[derive(Clone, Debug)]
pub struct Runner {
    pub x: i32,
    pub y: i32,
    pub facing: i32,
    pub sections: [i32; 4],
    pub max_sections: [i32; 4],
    /// hundredths
    pub stamina: i32,
    pub max_stamina: i32,
    pub since_spend: i32,
    pub statuses: Vec<Status>,
    pub action: Action,
    pub life: Life,
    pub rig: Rig,
    pub veins: u8,
    pub eye: u8,
    pub body: u8,
    pub hands: u8,
    pub nerve: u8,
    /// the trait flags the run reads (Full Design, The record in a run)
    pub quirks: Quirks,
    /// stamina without the bond bonus
    base_stamina: i32,
    pub lantern_on: bool,
    pub dimming: bool,
    pub sneaking: bool,
    pub sprinting: bool,
    pub moved: bool,
    pub ammo: i32,
    pub melee_cd: i32,
    /// stamina spent this tick, for the HUD
    pub last_spend: i32,
    pub pose: Pose,
}

/// The trait flags a run reads, parsed once from the record's flag list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Quirks {
    pub see_plus_1: bool,
    pub noise_plus_1: bool,
    pub melee_noise_minus_1: bool,
    pub melee_plus_1: bool,
    pub ranged_plus_2: bool,
    pub sprint_plus: bool,
    pub regen_x1_2: bool,
    pub walk_minus: bool,
    pub water_at_walk: bool,
    pub deposit_plus_1: bool,
    pub eye_plus_1_dark: bool,
    pub fever_immune: bool,
}

impl Quirks {
    pub fn from_flags(flags: &[String]) -> Quirks {
        let has = |f: &str| flags.iter().any(|x| x == f);
        Quirks {
            see_plus_1: has("see_plus_1"),
            noise_plus_1: has("noise_plus_1"),
            melee_noise_minus_1: has("melee_noise_minus_1"),
            melee_plus_1: has("melee_plus_1"),
            ranged_plus_2: has("ranged_plus_2"),
            sprint_plus: has("sprint_plus_0_5"),
            regen_x1_2: has("stamina_regen_x1_2"),
            walk_minus: has("walk_minus_0_3"),
            water_at_walk: has("water_at_walk"),
            deposit_plus_1: has("deposit_plus_1"),
            eye_plus_1_dark: has("eye_plus_1_dark"),
            fever_immune: has("fever_immune"),
        }
    }
}

/// The stat scale (Full Design, The record in a run): 0.8 + 0.04 x stat, 1.0 at 5.
pub fn stat_scale(stat: u8) -> f32 {
    0.8 + 0.04 * stat as f32
}

/// Damage lean from a stat: (stat - 5) / 2 toward zero.
pub fn stat_lean(stat: u8) -> i32 {
    (stat as i32 - 5) / 2
}

impl Runner {
    pub fn new(rules: &RunnerRules, tile: (i32, i32), rig: Rig, setup: &people::RunnerSetup) -> Runner {
        let scale = stat_scale(setup.body);
        let max: [i32; 4] = [0, 1, 2, 3].map(|i| (rules.sections[i] as f32 * scale).round() as i32);
        let stamina = (rules.stamina as f32 * stat_scale(setup.nerve)).round() as i32 * 100;
        Runner {
            x: tile.0 * FP + FP / 2,
            y: tile.1 * FP + FP / 2,
            facing: 1,
            sections: max,
            max_sections: max,
            stamina,
            max_stamina: stamina,
            since_spend: 0,
            statuses: Vec::new(),
            action: Action::Idle,
            life: Life::Up,
            rig,
            veins: setup.veins,
            eye: setup.eye,
            body: setup.body,
            hands: setup.hands,
            nerve: setup.nerve,
            quirks: Quirks::from_flags(&setup.flags),
            base_stamina: stamina,
            lantern_on: true,
            dimming: false,
            sneaking: false,
            sprinting: false,
            moved: false,
            ammo: 0,
            melee_cd: 0,
            last_spend: 0,
            pose: Pose::Walk,
        }
    }

    /// A bonded partner within 6 tiles is +1 Nerve: +4 stamina on the pool (Full Design, Bonds).
    pub fn set_bond_near(&mut self, near: bool) {
        let want = self.base_stamina + if near { 400 } else { 0 };
        if want != self.max_stamina {
            self.max_stamina = want;
            self.stamina = self.stamina.min(self.max_stamina);
        }
    }

    #[inline]
    pub fn tile(&self) -> (i32, i32) {
        (self.x.div_euclid(FP), self.y.div_euclid(FP))
    }

    pub fn has(&self, k: StatusKind) -> bool {
        self.statuses.iter().any(|s| s.kind == k)
    }

    pub fn add_status(&mut self, k: StatusKind, left: i32) {
        if let Some(s) = self.statuses.iter_mut().find(|s| s.kind == k) {
            s.left = s.left.max(left);
        } else {
            self.statuses.push(Status { kind: k, left });
        }
    }

    pub fn cure(&mut self, k: StatusKind) -> bool {
        let n = self.statuses.len();
        self.statuses.retain(|s| s.kind != k);
        self.statuses.len() != n
    }

    pub fn is_up(&self) -> bool {
        matches!(self.life, Life::Up)
    }

    pub fn dodging(&self) -> bool {
        matches!(self.action, Action::Dodge { .. })
    }

    pub fn legs_out(&self) -> bool {
        self.sections[LEGS] == 0
    }

    pub fn arms_out(&self) -> bool {
        self.sections[ARMS] == 0
    }

    /// Spend stamina; false when there is not enough.
    pub fn spend(&mut self, hundredths: i32) -> bool {
        if self.stamina < hundredths {
            return false;
        }
        self.stamina -= hundredths;
        self.since_spend = 0;
        self.last_spend += hundredths;
        true
    }

    /// Movement speed this tick in units per tick, from pace, surface, legs and weight (Full Design v3.1,
    /// Traversal): walking the floor is the base; crawling the back wall or ceiling is x0.7 and never a sprint;
    /// wading shallow water x0.75; swimming deep water is sneak speed.
    pub fn speed(&self, rules: &RunnerRules, overweight: bool, pose: Pose) -> i32 {
        let sprint_ok = self.sprinting && !self.legs_out() && pose == Pose::Walk;
        let walk = if self.quirks.walk_minus { rules.walk - 0.3 } else { rules.walk };
        let sprint = if self.quirks.sprint_plus { rules.sprint + 0.5 } else { rules.sprint };
        let mut tps = if pose == Pose::Swim {
            if self.quirks.water_at_walk { walk } else { rules.sneak }
        } else if self.sneaking {
            rules.sneak
        } else if sprint_ok {
            if overweight { sprint * 0.5 } else { sprint }
        } else {
            walk
        };
        match pose {
            Pose::Crawl => tps *= 0.7,
            Pose::Wade if !self.quirks.water_at_walk => tps *= 0.75,
            _ => {}
        }
        if self.legs_out() {
            tps *= 0.5;
        }
        if self.has(StatusKind::Grabbed) {
            tps = 0.0;
        }
        speed_units(tps)
    }

    /// Damage to a section with Tarkov's transfer rule; returns (section hit, damage applied, went down).
    pub fn take_damage(&mut self, rules: &RunnerRules, amount: i32, odds: &[u32; 4], armour: i32, rng: &mut Pcg32) -> (usize, i32, bool) {
        let total: u32 = odds.iter().sum::<u32>().max(1);
        let mut roll = rng.below(total);
        let mut sec = TORSO;
        for (i, &w) in odds.iter().enumerate() {
            if roll < w {
                sec = i;
                break;
            }
            roll -= w;
        }
        let mut dmg = (amount - armour).max(1);
        // a section at zero transfers to the torso
        if self.sections[sec] == 0 && sec != TORSO && sec != HEAD {
            sec = TORSO;
        }
        let before = self.sections[sec];
        self.sections[sec] = (self.sections[sec] - dmg).max(0);
        let overflow = dmg - (before - self.sections[sec]);
        if overflow > 0 && sec != TORSO {
            self.sections[TORSO] = (self.sections[TORSO] - overflow).max(0);
        }
        dmg = dmg.min(before + self.sections[TORSO].max(0));
        let down = (self.sections[HEAD] == 0 || self.sections[TORSO] == 0) && self.is_up();
        if down {
            self.life = Life::Down { left: ticks(rules.down_s) };
            self.action = Action::Idle;
        }
        (sec, dmg, down)
    }

    /// Per-tick body upkeep: statuses, bleeding, down timer, stamina regen. Returns bleed damage dealt.
    pub fn upkeep(&mut self, rules: &RunnerRules, tick: u64) -> i32 {
        let mut bled = 0;
        for s in self.statuses.iter_mut() {
            if s.left != i32::MAX {
                s.left -= 1;
            }
        }
        if self.has(StatusKind::Bleeding) && tick % (4 * crate::creature::TICK_HZ as u64) == 0 {
            self.sections[TORSO] = (self.sections[TORSO] - rules.bleed_per_4s).max(0);
            bled = rules.bleed_per_4s;
            if self.sections[TORSO] == 0 && self.is_up() {
                self.life = Life::Down { left: ticks(rules.down_s) };
            }
        }
        self.statuses.retain(|s| s.left > 0);
        if let Life::Down { left } = self.life {
            if left <= 1 {
                self.life = Life::Dead;
            } else {
                self.life = Life::Down { left: left - 1 };
            }
        }
        self.since_spend += 1;
        if self.since_spend > ticks(rules.regen_delay_s) && self.stamina < self.max_stamina {
            let per_tick = rules.regen_per_s * 100 / crate::creature::TICK_HZ;
            let per_tick = if self.quirks.regen_x1_2 { per_tick * 12 / 10 } else { per_tick };
            self.stamina = (self.stamina + per_tick).min(self.max_stamina);
        }
        self.last_spend = 0;
        bled
    }

    /// The melee weapon in hand: (damage, noise, reach), the fist when nothing is wielded.
    pub fn melee(&self, table: &ItemTable) -> (i32, i32, i32) {
        let (dmg, noise, reach) = self.melee_gear(table);
        let dmg = (dmg + stat_lean(self.hands) + if self.quirks.melee_plus_1 { 1 } else { 0 }).max(1);
        let noise = (noise - if self.quirks.melee_noise_minus_1 { 1 } else { 0 }).max(0);
        (dmg, noise, reach)
    }

    fn melee_gear(&self, table: &ItemTable) -> (i32, i32, i32) {
        for h in self.rig.gear.hands.iter().flatten() {
            let d = table.def(h.def);
            if let (Some(g), Some(st)) = (&d.gear, h.gear_stats(d)) {
                if g.kind == "melee" {
                    let reach = if d.id == "ger_sword" || d.id == "ger_monowire" { 2 } else { 1 };
                    return (st.dmg, st.noise, reach);
                }
            }
        }
        (3, 1, 1)
    }

    /// Dig time in seconds for this body: the table's x (1.25 - 0.05 x Hands).
    pub fn dig_s(&self, rules: &RunnerRules) -> f32 {
        rules.dig_s * (1.25 - 0.05 * self.hands as f32)
    }

    /// The ranged weapon in hand, if any: (damage, noise, mag).
    pub fn ranged(&self, table: &ItemTable) -> Option<(i32, i32, i32)> {
        for h in self.rig.gear.hands.iter().flatten() {
            let d = table.def(h.def);
            if let (Some(g), Some(st)) = (&d.gear, h.gear_stats(d)) {
                if g.kind == "ranged" {
                    let dmg = (st.dmg + stat_lean(self.eye) + if self.quirks.ranged_plus_2 { 2 } else { 0 }).max(1);
                    return Some((dmg, st.noise, st.mag.max(1)));
                }
            }
        }
        None
    }

    /// Sum of equipped gear light (lantern in a quick slot counts while on).
    pub fn gear_light(&self, table: &ItemTable) -> i32 {
        let mut l = 0;
        let g = &self.rig.gear;
        for it in [&g.head, &g.body, &g.feet, &g.back].into_iter().flatten().chain(g.hands.iter().flatten()) {
            if let Some(st) = it.gear_stats(table.def(it.def)) {
                l += st.light;
            }
        }
        if self.lantern_on && self.rig.quick.iter().flatten().any(|q| table.def(q.def).id == "ger_lantern") {
            l += 6;
        }
        l
    }

    pub fn has_quick(&self, table: &ItemTable, id: &str) -> Option<usize> {
        self.rig.quick.iter().position(|q| q.as_ref().is_some_and(|q| table.def(q.def).id == id))
    }

    /// Armour on the slot a section maps to.
    pub fn armour_for(&self, table: &ItemTable, section: usize) -> i32 {
        let g = &self.rig.gear;
        let slot = match section {
            HEAD => &g.head,
            TORSO => &g.body,
            LEGS => &g.feet,
            _ => &g.body,
        };
        slot.as_ref().and_then(|it| it.gear_stats(table.def(it.def))).map_or(0, |st| st.armour / if section == ARMS { 2 } else { 1 })
    }

    pub fn put_quick(&mut self, slot: usize, item: ItemInstance) {
        if slot < self.rig.quick.len() {
            self.rig.quick[slot] = Some(item);
        }
    }
}
