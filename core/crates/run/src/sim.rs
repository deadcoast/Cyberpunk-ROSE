//! The run as one simulation: `RunSim::step(input)` at 20 Hz advances movement, senses, loot, creatures, combat,
//! digging and extraction on integer / fixed-point state, so a logged input word replays a run bit-exact
//! (Full Design v3.1, Technical architecture: Determinism). Godot gathers input and draws state and events.

use crate::chunk::substrate_passable;
use crate::creature::{self, Attack, Creature, CreatureFile, Kind, Phase, Stimuli, FP, HALF, TICK_HZ};
use crate::light::Source;
use crate::loot::{LootState, LootTables, OpenMethod, SourceKind, SourceState};
use crate::map::RouteMap;
use crate::noise::Sound;
use crate::runner::{Action, Life, Pose, Runner, StatusKind, ARMS, LEGS, TORSO};
use crate::senses::{Motion, Senses};
use inventory::Rig;
use items::ItemTable;
use mat::rng::Pcg32;
use mat::sim::sub;

/// One tick of player intent. Packs into (buttons u16, aim_x i16, aim_y i16) for the save's `Input::RunInput`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RunInput {
    pub dx: i8,
    pub dy: i8,
    pub sprint: bool,
    pub sneak: bool,
    pub dim: bool,
    pub attack: bool,
    pub shoot: bool,
    pub dodge: bool,
    pub interact: bool,
    pub dig: bool,
    pub take: bool,
    pub lantern: bool,
    /// 1-4 uses a quick slot this tick; 0 none
    pub quick: u8,
}

impl RunInput {
    pub fn pack(&self) -> (u16, i16, i16) {
        let mut b = 0u16;
        for (i, f) in [self.sprint, self.sneak, self.dim, self.attack, self.shoot, self.dodge, self.interact, self.dig, self.take, self.lantern].iter().enumerate() {
            if *f {
                b |= 1 << i;
            }
        }
        b |= ((self.quick & 7) as u16) << 10;
        (b, self.dx as i16, self.dy as i16)
    }

    pub fn unpack(b: u16, ax: i16, ay: i16) -> RunInput {
        let f = |i: u16| b & (1 << i) != 0;
        RunInput {
            dx: ax.clamp(-1, 1) as i8,
            dy: ay.clamp(-1, 1) as i8,
            sprint: f(0),
            sneak: f(1),
            dim: f(2),
            attack: f(3),
            shoot: f(4),
            dodge: f(5),
            interact: f(6),
            dig: f(7),
            take: f(8),
            lantern: f(9),
            quick: ((b >> 10) & 7) as u8,
        }
    }
}

/// What happened this tick, for the view (sounds, flashes, labels) and for tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    RunnerHit { section: usize, damage: i32, by: u32 },
    Bleed { damage: i32 },
    CreatureHit { id: u32, damage: i32, behind: bool },
    CreatureDied { id: u32, kind: Kind, x: i32, y: i32, source: Option<usize> },
    CreatureLeft { id: u32 },
    PhaseChange { id: u32, phase: Phase },
    Swing { x: i32, y: i32, hit: bool },
    Shot { x0: i32, y0: i32, x1: i32, y1: i32, by_runner: bool, hit: bool },
    Burst { x: i32, y: i32, radius: i32 },
    Opened { source: usize, revealed: usize, notable: usize, generous: bool, forced: bool, noise: u8 },
    Took { source: usize, taken: usize, left: usize },
    Dug { x: i32, y: i32, pile: Option<usize> },
    Salted { x: i32, y: i32 },
    Used { slot: usize, id: String },
    Down,
    Dead,
    Extracted,
    Grabbed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunResult {
    Extracted { tick: u64 },
    Died { tick: u64 },
}

#[derive(Clone, Debug)]
pub struct RunSim {
    pub map: RouteMap,
    pub table: ItemTable,
    pub tables: LootTables,
    pub defs: CreatureFile,
    pub senses: Senses,
    pub loot: LootState,
    pub runner: Runner,
    pub creatures: Vec<Creature>,
    pub salt: Vec<bool>,
    pub tick: u64,
    pub events: Vec<Event>,
    pub result: Option<RunResult>,
    rng: Pcg32,
    /// distance moved since the last footstep, in units
    step_accum: i32,
    prev_lantern: bool,
    prev_attack: bool,
    prev_dodge: bool,
    prev_interact: bool,
    prev_take: bool,
    prev_shoot: bool,
    prev_quick: u8,
    /// press buffers: a press during a wind-up, swing or dodge is kept for a few ticks and fires when Idle
    buf_attack: i32,
    buf_dodge: i32,
    buf_interact: i32,
    buf_take: i32,
    buf_shoot: i32,
    /// the open loot source under the Runner, if any
    pub open_source: Option<usize>,
}

/// how long a buffered press waits for the Runner to be free (ticks)
const PRESS_BUFFER: i32 = 8;

pub struct RunSetup<'a> {
    pub mat_lume_gene: f32,
    pub memory: Option<&'a [u8]>,
    pub run_nonce: u64,
    pub route_runs: u16,
    pub day: u32,
    pub rig: Rig,
    pub body: u8,
}

impl RunSim {
    pub fn new(map: RouteMap, table: ItemTable, tables: LootTables, defs: CreatureFile, setup: RunSetup) -> RunSim {
        let mut rng = Pcg32::new(map.seed ^ setup.run_nonce.rotate_left(17) ^ 0xC0FFEE);
        let senses = Senses::new(&map, setup.mat_lume_gene, setup.memory);
        let loot = LootState::new(&map, &tables, setup.run_nonce, setup.route_runs, setup.day);
        let creatures = creature::spawn_all(&map, &defs, &mut rng);
        let start = map.entry.get(1).or(map.entry.first()).copied().unwrap_or((1, 12));
        let mut runner = Runner::new(&defs.runner, (start.0 as i32, start.1 as i32), setup.rig, setup.body);
        if let Some((dmg, _, mag)) = runner.ranged(&table) {
            let _ = dmg;
            runner.ammo = mag;
        }
        let n = map.w * map.h;
        RunSim {
            map,
            table,
            tables,
            defs,
            senses,
            loot,
            runner,
            creatures,
            salt: vec![false; n],
            tick: 0,
            events: Vec::new(),
            result: None,
            rng,
            step_accum: 0,
            prev_lantern: false,
            prev_attack: false,
            prev_dodge: false,
            prev_interact: false,
            prev_take: false,
            prev_shoot: false,
            prev_quick: 0,
            buf_attack: 0,
            buf_dodge: 0,
            buf_interact: 0,
            buf_take: 0,
            buf_shoot: 0,
            open_source: None,
        }
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    #[inline]
    fn idx(&self, x: i32, y: i32) -> usize {
        y as usize * self.map.w + x as usize
    }

    fn passable_tile(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.map.w && (y as usize) < self.map.h && substrate_passable(self.map.substrate[self.idx(x, y)])
    }

    /// Is a body centred at (x, y) units free of solid tiles?
    fn body_free(&self, x: i32, y: i32) -> bool {
        let x0 = (x - HALF).div_euclid(FP);
        let x1 = (x + HALF - 1).div_euclid(FP);
        let y0 = (y - HALF).div_euclid(FP);
        let y1 = (y + HALF - 1).div_euclid(FP);
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !self.passable_tile(tx, ty) {
                    return false;
                }
            }
        }
        true
    }

    /// Axis-separated move with wall sliding. Returns units actually moved (for footsteps).
    fn move_body(&self, x: &mut i32, y: &mut i32, dx: i32, dy: i32) -> i32 {
        let mut moved = 0;
        if dx != 0 {
            let nx = *x + dx;
            if self.body_free(nx, *y) {
                *x = nx;
                moved += dx.abs();
            } else {
                // slide up to the wall
                let step = dx.signum();
                while self.body_free(*x + step, *y) {
                    *x += step;
                    moved += 1;
                }
            }
        }
        if dy != 0 {
            let ny = *y + dy;
            if self.body_free(*x, ny) {
                *y = ny;
                moved += dy.abs();
            } else {
                let step = dy.signum();
                while self.body_free(*x, *y + step) {
                    *y += step;
                    moved += 1;
                }
            }
        }
        moved
    }

    fn overweight(&self) -> bool {
        self.runner.rig.overweight(&self.table)
    }

    /// Walk on a floor (solid below), wade in water with air above, swim in water under water, crawl the back wall
    /// or ceiling anywhere else.
    pub fn pose_at(&self, x: i32, y: i32) -> Pose {
        let here = self.map.substrate[self.idx(x, y)];
        let above = if y > 0 { self.map.substrate[self.idx(x, y - 1)] } else { sub::ROCK };
        let below = if (y as usize) + 1 < self.map.h { self.map.substrate[self.idx(x, y + 1)] } else { sub::ROCK };
        if here == sub::WATER {
            if above == sub::WATER { Pose::Swim } else { Pose::Wade }
        } else if !substrate_passable(below) {
            Pose::Walk
        } else {
            Pose::Crawl
        }
    }



    fn on_extraction(&self) -> bool {
        let t = self.runner.tile();
        let t16 = (t.0 as u16, t.1 as u16);
        self.map.entry.contains(&t16) || self.map.grates.contains(&t16)
    }

    pub fn source_here(&self) -> Option<usize> {
        let t = self.runner.tile();
        self.loot.source_at(t.0 as usize, t.1 as usize)
    }

    fn runner_sound(&mut self, sound: Sound) {
        self.senses.runner_sound(&self.map, sound);
    }

    /// One tick.
    pub fn step(&mut self, input: RunInput) {
        if self.result.is_some() {
            return;
        }
        self.tick += 1;
        let rules = self.defs.runner.clone();
        // ---- body upkeep
        let bled = self.runner.upkeep(&rules, self.tick);
        if bled > 0 {
            self.events.push(Event::Bleed { damage: bled });
        }
        match self.runner.life {
            Life::Dead => {
                self.events.push(Event::Dead);
                self.result = Some(RunResult::Died { tick: self.tick });
                return;
            }
            Life::Down { .. } => {}
            Life::Up => self.runner_act(&rules, input),
        }
        // the surface under the Runner: deep water soaks
        let rt = self.runner.tile();
        self.runner.pose = self.pose_at(rt.0, rt.1);
        if self.runner.pose == Pose::Swim {
            self.runner.add_status(StatusKind::Soaked, creature::ticks(rules.soaked_s));
        }
        // ---- senses
        let motion = match self.runner.action {
            Action::Dig { .. } => Motion::Dig,
            _ if !self.runner.moved => Motion::Still,
            _ if self.runner.sneaking => Motion::Sneak,
            _ if self.runner.sprinting => Motion::Sprint,
            _ => Motion::Walk,
        };
        self.senses.runner = (rt.0 as usize, rt.1 as usize);
        self.senses.lume.veins = self.runner.veins;
        self.senses.lume.gear = self.runner.gear_light(&self.table);
        self.senses.lume.dimmed = self.runner.dimming;
        self.senses.lume.soaked = self.runner.has(StatusKind::Soaked);
        self.senses.eye = if self.runner.has(StatusKind::Blinded) { 0 } else { self.runner.eye };
        self.senses.motion = motion;
        for c in &self.creatures {
            if c.kind == Kind::Scout && !c.dead {
                let (tx, ty) = c.tile();
                self.senses.flashes.push(Source { x: tx as u16, y: ty as u16, emit: self.defs.creature["scout"].lamp });
            }
        }
        self.senses.tick(&self.map);
        if self.runner.has(StatusKind::Blinded) {
            // blinded: see only the tile you stand on and its neighbours
            let (rx, ry) = (rt.0, rt.1);
            for (i, v) in self.senses.visible.iter_mut().enumerate() {
                let (x, y) = ((i % self.map.w) as i32, (i / self.map.w) as i32);
                if (x - rx).abs() > 1 || (y - ry).abs() > 1 {
                    *v = false;
                }
            }
        }
        // ---- creatures
        self.creatures_act(&rules);
        // ---- life
        if let Life::Down { left } = self.runner.life {
            if left == creature::ticks(rules.down_s) - 1 {
                self.events.push(Event::Down);
            }
        }
        if self.runner.is_up() {
            self.open_source = self.source_here().filter(|&i| self.loot.sources[i].state == SourceState::Open);
        }
    }

    fn runner_act(&mut self, rules: &crate::creature::RunnerRules, input: RunInput) {
        let r = &mut self.runner;
        r.moved = false;
        r.sneaking = input.sneak && !input.sprint;
        r.dimming = input.dim && r.stamina > 0;
        if input.lantern && !self.prev_lantern {
            r.lantern_on = !r.lantern_on;
        }
        self.prev_lantern = input.lantern;
        // edges this tick, buffered so a press mid-action lands when the Runner is free
        if input.attack && !self.prev_attack {
            self.buf_attack = PRESS_BUFFER;
        }
        if input.shoot && !self.prev_shoot {
            self.buf_shoot = PRESS_BUFFER;
        }
        if input.dodge && !self.prev_dodge {
            self.buf_dodge = PRESS_BUFFER;
        }
        if input.interact && !self.prev_interact {
            self.buf_interact = PRESS_BUFFER;
        }
        if input.take && !self.prev_take {
            self.buf_take = PRESS_BUFFER;
        }
        let attack_press = self.buf_attack > 0;
        let shoot_press = self.buf_shoot > 0;
        let dodge_press = self.buf_dodge > 0;
        let interact_press = self.buf_interact > 0;
        let take_press = self.buf_take > 0;
        let quick_press = if input.quick != 0 && input.quick != self.prev_quick { input.quick } else { 0 };
        self.prev_attack = input.attack;
        self.prev_shoot = input.shoot;
        self.prev_dodge = input.dodge;
        self.prev_interact = input.interact;
        self.prev_take = input.take;
        self.prev_quick = input.quick;
        for b in [&mut self.buf_attack, &mut self.buf_shoot, &mut self.buf_dodge, &mut self.buf_interact, &mut self.buf_take] {
            *b = (*b - 1).max(0);
        }
        let dim_cost = rules.dim_per_s * 100 / TICK_HZ;
        if self.runner.dimming && !self.runner.spend(dim_cost) {
            self.runner.dimming = false;
        }
        let (dx, dy) = (input.dx as i32, input.dy as i32);
        if dx != 0 {
            self.runner.facing = dx.signum();
        }
        match self.runner.action {
            Action::Dodge { left, dx: ddx, dy: ddy } => {
                let speed = rules.dodge_tiles * FP / creature::ticks(rules.dodge_s).max(1);
                let (mut x, mut y) = (self.runner.x, self.runner.y);
                self.move_body(&mut x, &mut y, ddx * speed, ddy * speed);
                self.runner.x = x;
                self.runner.y = y;
                self.runner.moved = true;
                self.runner.action = if left <= 1 { Action::Idle } else { Action::Dodge { left: left - 1, dx: ddx, dy: ddy } };
            }
            Action::Windup { left } => {
                if left <= 1 {
                    self.resolve_swing(rules);
                    self.runner.action = Action::Swing { left: creature::ticks(rules.melee_active).max(1) };
                } else {
                    self.runner.action = Action::Windup { left: left - 1 };
                }
            }
            Action::Swing { left } => {
                self.runner.action = if left <= 1 { Action::Idle } else { Action::Swing { left: left - 1 } };
            }
            Action::Reload { left } => {
                if left <= 1 {
                    if let Some((_, _, mag)) = self.runner.ranged(&self.table) {
                        self.runner.ammo = mag;
                    }
                    self.runner.action = Action::Idle;
                } else {
                    self.runner.action = Action::Reload { left: left - 1 };
                }
            }
            Action::Dig { x, y, progress } => {
                let still = input.dig && self.is_diggable(x, y) && (self.runner.tile().0 - x).abs() + (self.runner.tile().1 - y).abs() == 1;
                if !still {
                    self.runner.action = Action::Idle;
                } else {
                    let claw = self.runner.has_quick(&self.table, "ger_dig_claw").is_some() || self.runner.rig.gear.hands.iter().flatten().any(|h| self.table.def(h.def).id == "ger_dig_claw");
                    let mut gain = if claw { 2 } else { 1 };
                    if self.runner.arms_out() && self.tick % 2 == 0 {
                        gain = 0;
                    }
                    let need = creature::ticks(rules.dig_s);
                    let p = progress + gain;
                    self.runner_sound(Sound::Dig);
                    if p >= need {
                        let was_deposit = self.map.substrate[self.idx(x, y)] == sub::DEPOSIT;
                        let chunk = self.map.chunk_of[self.idx(x, y)];
                        self.map.dig(x as usize, y as usize);
                        let pile = if was_deposit { Some(self.loot.add_pile(&self.tables, &self.table, x as usize, y as usize, if chunk == crate::map::NO_CHUNK { 0 } else { chunk })) } else { None };
                        self.events.push(Event::Dug { x, y, pile });
                        self.runner.action = Action::Idle;
                    } else {
                        self.runner.action = Action::Dig { x, y, progress: p };
                    }
                }
            }
            Action::Extract { progress } => {
                if !self.on_extraction() || !input.interact {
                    self.runner.action = Action::Idle;
                } else {
                    let gain = if self.runner.legs_out() { if self.tick % 2 == 0 { 1 } else { 0 } } else { 1 };
                    let p = progress + gain;
                    self.runner_sound(Sound::ExtractChannel);
                    if p >= creature::ticks(rules.extract_s) {
                        self.events.push(Event::Extracted);
                        self.result = Some(RunResult::Extracted { tick: self.tick });
                        self.runner.action = Action::Idle;
                    } else {
                        self.runner.action = Action::Extract { progress: p };
                    }
                }
            }
            Action::Idle => {
                // presses first
                if attack_press && self.runner.melee_cd == 0 && self.runner.spend(rules.swing * 100) {
                    self.buf_attack = 0;
                    self.runner.action = Action::Windup { left: creature::ticks(rules.melee_windup).max(1) };
                } else if shoot_press {
                    self.buf_shoot = 0;
                    self.fire(rules, dx, dy);
                } else if dodge_press && (dx != 0 || dy != 0) && !self.runner.legs_out() && self.runner.spend(rules.dodge * 100) {
                    self.buf_dodge = 0;
                    self.runner.action = Action::Dodge { left: creature::ticks(rules.dodge_s).max(1), dx: dx.signum(), dy: dy.signum() };
                } else if input.dig && (dx != 0 || dy != 0) {
                    let t = self.runner.tile();
                    let (tx, ty) = (t.0 + dx.signum(), t.1 + dy.signum());
                    // dig straight only: the tile ahead on one axis
                    let (tx, ty) = if dx != 0 { (t.0 + dx.signum(), t.1) } else { (tx, ty) };
                    if self.is_diggable(tx, ty) {
                        self.runner.action = Action::Dig { x: tx, y: ty, progress: 0 };
                    }
                } else if input.interact || interact_press {
                    // opening is a press; the extraction channel is a hold, so a hold that began mid-swing still counts
                    if let Some(i) = self.source_here() {
                        if interact_press && self.loot.sources[i].state == SourceState::Closed {
                            self.buf_interact = 0;
                            self.open(i);
                        }
                    } else if input.interact && self.on_extraction() {
                        self.buf_interact = 0;
                        self.runner.action = Action::Extract { progress: 0 };
                    }
                } else if take_press {
                    self.buf_take = 0;
                    if let Some(i) = self.source_here() {
                        if self.loot.sources[i].state == SourceState::Open {
                            let (taken, left) = self.loot.take_all(&self.table, i, &mut self.runner.rig);
                            self.events.push(Event::Took { source: i, taken, left });
                        }
                    }
                } else if quick_press != 0 {
                    self.use_quick(rules, quick_press as usize - 1);
                }
                // movement
                if matches!(self.runner.action, Action::Idle) && (dx != 0 || dy != 0) {
                    let overweight = self.overweight();
                    let sprint_cost = rules.sprint_per_s * 100 / TICK_HZ;
                    let t = self.runner.tile();
                    let pose = self.pose_at(t.0, t.1);
                    self.runner.sprinting = input.sprint && !self.runner.sneaking && !self.runner.legs_out() && pose == Pose::Walk && self.runner.stamina >= sprint_cost;
                    if self.runner.sprinting {
                        self.runner.spend(sprint_cost);
                    }
                    let speed = self.runner.speed(rules, overweight, pose);
                    // diagonal normalisation 181/256 ~ 1/sqrt2
                    let (vx, vy) = if dx != 0 && dy != 0 { (dx * speed * 181 / 256, dy * speed * 181 / 256) } else { (dx * speed, dy * speed) };
                    let (mut x, mut y) = (self.runner.x, self.runner.y);
                    let moved = self.move_body(&mut x, &mut y, vx, vy);
                    self.runner.x = x;
                    self.runner.y = y;
                    if moved > 0 {
                        self.runner.moved = true;
                        self.step_accum += moved;
                        if self.step_accum >= FP {
                            self.step_accum -= FP;
                            let s = if self.runner.sneaking {
                                Sound::SneakStep
                            } else if self.runner.sprinting {
                                Sound::SprintStep
                            } else {
                                Sound::WalkStep
                            };
                            self.runner_sound(s);
                        }
                    }
                } else {
                    self.runner.sprinting = false;
                }
            }
        }
        self.runner.melee_cd = (self.runner.melee_cd - 1).max(0);
    }

    fn is_diggable(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.map.w && (y as usize) < self.map.h && matches!(self.map.substrate[self.idx(x, y)], sub::DEPOSIT | sub::COLLAPSE)
    }

    fn open(&mut self, i: usize) {
        let has_pry = self.runner.has_quick(&self.table, "ger_pry_bar").is_some() || self.runner.rig.gear.hands.iter().flatten().any(|h| self.table.def(h.def).id == "ger_pry_bar");
        let method = if has_pry { OpenMethod::PryBar } else { OpenMethod::Hands };
        if let Ok(r) = self.loot.open(&self.tables, &self.table, i, method) {
            let (sx, sy) = (self.loot.sources[i].x as usize, self.loot.sources[i].y as usize);
            let tick = self.senses.tick;
            self.senses.noise.emit(&self.map, sx, sy, r.noise, tick, 0);
            self.events.push(Event::Opened { source: i, revealed: r.revealed, notable: r.notable, generous: r.generous, forced: r.forced, noise: r.noise });
        }
    }

    fn use_quick(&mut self, rules: &crate::creature::RunnerRules, slot: usize) {
        let Some(item) = self.runner.rig.quick.get(slot).cloned().flatten() else { return };
        let id = self.table.def(item.def).id.clone();
        let consumed = match id.as_str() {
            "org_antiseptic" => {
                self.runner.cure(StatusKind::Bleeding);
                true
            }
            "org_stim" => {
                self.runner.stamina = self.runner.max_stamina;
                true
            }
            "org_splint" => {
                for s in [ARMS, LEGS] {
                    if self.runner.sections[s] == 0 {
                        self.runner.sections[s] = self.runner.max_sections[s] / 4;
                    }
                }
                true
            }
            "prt_salt" => {
                // a line across the corridor: the Runner's column, every passable tile up and down to the walls
                let (x, y) = self.runner.tile();
                let w = self.map.w;
                let mut yy = y;
                while self.passable_tile(x, yy) {
                    self.salt[yy as usize * w + x as usize] = true;
                    yy -= 1;
                }
                yy = y + 1;
                while self.passable_tile(x, yy) {
                    self.salt[yy as usize * w + x as usize] = true;
                    yy += 1;
                }
                self.events.push(Event::Salted { x, y });
                true
            }
            _ => false,
        };
        let _ = rules;
        if consumed {
            self.runner.rig.quick[slot] = None;
            self.events.push(Event::Used { slot, id });
        }
    }

    fn resolve_swing(&mut self, rules: &crate::creature::RunnerRules) {
        let (dmg, noise, reach) = self.runner.melee(&self.table);
        let mut dmg = dmg;
        if self.runner.arms_out() {
            dmg = (dmg / 2).max(1);
        }
        self.runner_sound(Sound::MeleeSwing(noise));
        let (rx, ry) = self.runner.tile();
        let f = self.runner.facing;
        let mut hit_any = false;
        let mut hits: Vec<(usize, i32, bool)> = Vec::new();
        for (i, c) in self.creatures.iter().enumerate() {
            if c.dead {
                continue;
            }
            let (cx, cy) = c.tile();
            let ahead = (cx - rx) * f >= 0 && (cx - rx).abs() <= reach && (cy - ry).abs() <= 1 && !(cx == rx && cy == ry && reach == 1 && false);
            if ahead && ((cx - rx).abs() + (cy - ry).abs()) <= reach + 1 {
                let behind = c.facing == f && cx != rx;
                let d = if behind { (dmg as f32 * rules.behind_mult).round() as i32 } else { dmg };
                hits.push((i, d, behind));
            }
        }
        for (i, d, behind) in hits {
            hit_any = true;
            self.damage_creature(i, d, behind);
        }
        self.events.push(Event::Swing { x: rx + f, y: ry, hit: hit_any });
        self.runner.melee_cd = creature::ticks(0.2);
    }

    fn fire(&mut self, _rules: &crate::creature::RunnerRules, dx: i32, dy: i32) {
        let Some((dmg, noise, _mag)) = self.runner.ranged(&self.table) else { return };
        if self.runner.ammo <= 0 {
            self.runner.action = Action::Reload { left: creature::ticks(if self.runner.arms_out() { 4.0 } else { 2.0 }) };
            return;
        }
        self.runner.ammo -= 1;
        let (ax, ay) = if dx != 0 || dy != 0 { (dx.signum(), dy.signum()) } else { (self.runner.facing, 0) };
        let (rx, ry) = self.runner.tile();
        self.runner_sound(Sound::Gunshot(noise));
        self.senses.flashes.push(Source { x: rx as u16, y: ry as u16, emit: crate::light::FLASH_EMIT });
        let (mut x, mut y) = (rx, ry);
        let mut hit = None;
        for _ in 0..12 {
            x += ax;
            y += ay;
            if !self.passable_tile(x, y) {
                break;
            }
            if let Some(i) = self.creatures.iter().position(|c| !c.dead && c.tile() == (x, y)) {
                hit = Some(i);
                break;
            }
        }
        let hit_any = hit.is_some();
        if let Some(i) = hit {
            let behind = self.creatures[i].facing == ax && ax != 0;
            self.damage_creature(i, dmg, behind);
        }
        self.events.push(Event::Shot { x0: rx, y0: ry, x1: x, y1: y, by_runner: true, hit: hit_any });
        if self.runner.ammo == 0 {
            self.runner.action = Action::Reload { left: creature::ticks(if self.runner.arms_out() { 4.0 } else { 2.0 }) };
        }
    }

    fn damage_creature(&mut self, i: usize, dmg: i32, behind: bool) {
        let c = &mut self.creatures[i];
        c.hp -= dmg;
        let id = c.id;
        self.events.push(Event::CreatureHit { id, damage: dmg, behind });
        if c.hp <= 0 {
            c.dead = true;
            let kind = c.kind;
            let (x, y) = c.tile();
            let chunk = c.chunk;
            // a killed burster still bursts, smaller
            if kind == Kind::Burster {
                let d = self.defs.creature["burster"].death_burst;
                self.burst(x, y, d, 1, id);
            }
            let skind = if kind == Kind::Scout { SourceKind::ScoutBody } else { SourceKind::RatCorpse };
            let src = self.loot.add_open_source(&self.tables, &self.table, skind, x as usize, y as usize, chunk);
            self.events.push(Event::CreatureDied { id, kind, x, y, source: Some(src) });
        } else {
            // being hit is a stimulus: it knows where you are
            c.awareness = 1.0;
            c.last_known = Some(self.runner.tile());
        }
    }

    fn burst(&mut self, x: i32, y: i32, damage: i32, radius: i32, by: u32) {
        self.events.push(Event::Burst { x, y, radius });
        let (rx, ry) = self.runner.tile();
        if (rx - x).abs() <= radius && (ry - y).abs() <= radius && !self.runner.dodging() && self.runner.is_up() {
            let rules = self.defs.runner.clone();
            let odds = self.defs.creature["burster"].section_odds;
            let armour = 0;
            let (sec, dealt, down) = self.runner.take_damage(&rules, damage, &odds, armour, &mut self.rng);
            self.events.push(Event::RunnerHit { section: sec, damage: dealt, by });
            self.runner.add_status(StatusKind::Blinded, creature::ticks(rules.blind_s));
            self.runner.add_status(StatusKind::Soaked, creature::ticks(rules.soaked_s));
            self.runner.add_status(StatusKind::FungalFever, i32::MAX);
            if down {
                self.events.push(Event::Down);
            }
        }
        // ooze puddle is loud enough to wake the corridor
        let tick = self.senses.tick;
        self.senses.noise.emit(&self.map, x as usize, y as usize, 9, tick, 2);
    }

    fn creatures_act(&mut self, rules: &crate::creature::RunnerRules) {
        let runner_tile = self.runner.tile();
        let notice = self.senses.notice_range();
        let last_sound = self.senses.noise.since(self.senses.tick.saturating_sub(1)).filter(|e| e.actor == 0).last().map(|e| (e.x as i32, e.y as i32));
        let noise = self.senses.noise.level.clone();
        let salt = self.salt.clone();
        let st = Stimuli { runner_tile, runner_dodging: self.runner.dodging(), notice_range: notice, noise: &noise, last_sound, salt: &salt };
        let aw = self.defs.awareness.clone();
        let n = self.creatures.len();
        let mut attacks: Vec<(usize, Attack)> = Vec::new();
        let mut shots: Vec<(i32, i32, bool)> = Vec::new();
        for i in 0..n {
            let kind = self.creatures[i].kind;
            let def = self.defs.creature[kind.key()].clone();
            let mut c = std::mem::replace(&mut self.creatures[i], Creature::new(0, kind, &def, (0, 0), 0));
            let out = creature::update(&mut c, &def, &aw, &self.map, &st, &mut self.rng);
            let id = c.id;
            let tile = c.tile();
            self.creatures[i] = c;
            if let Some(p) = out.became {
                self.events.push(Event::PhaseChange { id, phase: p });
            }
            if out.left {
                self.events.push(Event::CreatureLeft { id });
            }
            if let Some(from) = out.shot_from {
                shots.push((from.0, from.1, out.attack.is_some()));
                let tick = self.senses.tick;
                self.senses.noise.emit(&self.map, from.0 as usize, from.1 as usize, def.shot_noise, tick, 2);
                self.senses.flashes.push(Source { x: from.0 as u16, y: from.1 as u16, emit: crate::light::FLASH_EMIT });
            }
            if let Some(a) = out.attack {
                attacks.push((i, a));
            }
            let _ = tile;
        }
        for (x0, y0, hit) in shots {
            self.events.push(Event::Shot { x0, y0, x1: runner_tile.0, y1: runner_tile.1, by_runner: false, hit });
        }
        for (i, a) in attacks {
            let id = self.creatures[i].id;
            let kind = self.creatures[i].kind;
            let def = self.defs.creature[kind.key()].clone();
            match a {
                Attack::Bite { damage, grab_ticks } => {
                    if !self.runner.is_up() {
                        continue;
                    }
                    let odds = def.section_odds;
                    let sec_guess = TORSO;
                    let armour = self.runner.armour_for(&self.table, sec_guess);
                    let (sec, dealt, down) = self.runner.take_damage(rules, damage, &odds, armour, &mut self.rng);
                    self.events.push(Event::RunnerHit { section: sec, damage: dealt, by: id });
                    if def.bleed_chance > 0.0 && self.rng.f32() < def.bleed_chance {
                        self.runner.add_status(StatusKind::Bleeding, creature::ticks(rules.bleed_s));
                    }
                    if grab_ticks > 0 {
                        self.runner.add_status(StatusKind::Grabbed, grab_ticks);
                        self.events.push(Event::Grabbed);
                    }
                    if down {
                        self.events.push(Event::Down);
                    }
                }
                Attack::Burst { damage, radius } => {
                    let (x, y) = self.creatures[i].tile();
                    self.burst(x, y, damage, radius, id);
                    let (cx, cy) = (x, y);
                    let chunk = self.creatures[i].chunk;
                    let src = self.loot.add_open_source(&self.tables, &self.table, SourceKind::RatCorpse, cx as usize, cy as usize, chunk);
                    self.events.push(Event::CreatureDied { id, kind, x: cx, y: cy, source: Some(src) });
                }
                Attack::Shot { damage } => {
                    if !self.runner.is_up() {
                        continue;
                    }
                    let odds = rules.shot_odds;
                    let armour = self.runner.armour_for(&self.table, TORSO);
                    let (sec, dealt, down) = self.runner.take_damage(rules, damage, &odds, armour, &mut self.rng);
                    self.events.push(Event::RunnerHit { section: sec, damage: dealt, by: id });
                    if down {
                        self.events.push(Event::Down);
                    }
                }
            }
        }
        self.creatures.retain(|c| !c.dead || c.kind == Kind::Scout && false);
        self.creatures.retain(|c| !c.dead);
    }

    /// A compact fingerprint of the state for replay tests.
    pub fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |v: i64| {
            h ^= v as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        };
        mix(self.tick as i64);
        mix(self.runner.x as i64);
        mix(self.runner.y as i64);
        for s in self.runner.sections {
            mix(s as i64);
        }
        mix(self.runner.stamina as i64);
        for c in &self.creatures {
            mix(c.id as i64);
            mix(c.x as i64);
            mix(c.y as i64);
            mix(c.hp as i64);
            mix((c.awareness * 1000.0) as i64);
        }
        mix(self.loot.sources.len() as i64);
        mix(self.senses.fog.seen_count() as i64);
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;
    use crate::creature::ticks;

    const ITEMS: &str = include_str!("../../../../data/items.toml");

    fn sim(seed: u64, stratum: u8) -> RunSim {
        let l = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        let m = RouteMap::compile(&l, &route(&l, seed, stratum).unwrap());
        let table = ItemTable::from_toml(ITEMS).unwrap();
        let mut rig = Rig::new(8, 5, 4, 18.0);
        let (_, lantern) = inventory::instance(&table, "ger_lantern").unwrap();
        let (_, pry) = inventory::instance(&table, "ger_pry_bar").unwrap();
        let (_, anti) = inventory::instance(&table, "org_antiseptic").unwrap();
        let (_, salt) = inventory::instance(&table, "prt_salt").unwrap();
        rig.quick = vec![Some(lantern), Some(pry), Some(anti), Some(salt)];
        let (_, pipe) = inventory::instance(&table, "ger_pipe").unwrap();
        rig.gear.hands[0] = Some(pipe);
        RunSim::new(m, table, LootTables::shipped(), CreatureFile::shipped(), RunSetup { mat_lume_gene: 0.5, memory: None, run_nonce: 7, route_runs: 1, day: 1, rig, body: 5 })
    }

    fn walk(s: &mut RunSim, dx: i8, dy: i8, n: usize, sprint: bool) {
        for _ in 0..n {
            s.step(RunInput { dx, dy, sprint, ..Default::default() });
        }
    }

    #[test]
    fn input_word_round_trips() {
        let i = RunInput { dx: -1, dy: 1, sprint: true, dodge: true, take: true, quick: 3, ..Default::default() };
        let (b, ax, ay) = i.pack();
        assert_eq!(RunInput::unpack(b, ax, ay), i);
    }

    #[test]
    fn movement_is_continuous_collides_and_makes_footsteps() {
        let mut s = sim(11, 1);
        let x0 = s.runner.x;
        walk(&mut s, 1, 0, 10, false);
        let moved = s.runner.x - x0;
        // 10 ticks at 4.5 tiles/s walking = 2.25 tiles; crawling the back wall x0.7
        let t = s.runner.tile();
        let expect = if s.pose_at(t.0, t.1) == Pose::Walk { 576 } else { 403 };
        assert!((moved - expect).abs() <= 24, "moved {moved} units, expected ~{expect} ({:?})", s.pose_at(t.0, t.1));
        assert!(s.senses.noise.since(0).any(|e| e.loudness >= 4), "no footstep");
        // sprint on a floor is faster and louder and costs stamina; put the Runner on the floor first
        let t = s.runner.tile();
        let mut fy = t.1;
        while s.pose_at(t.0, fy) != Pose::Walk && (fy as usize) + 1 < s.map.h && s.map.passable(t.0 as usize, (fy + 1) as usize) {
            fy += 1;
        }
        s.runner.y = fy * FP + FP / 2;
        let st0 = s.runner.stamina;
        let x1 = s.runner.x;
        walk(&mut s, 1, 0, 10, false);
        let walked = s.runner.x - x1;
        let x2 = s.runner.x;
        walk(&mut s, 1, 0, 10, true);
        let sprinted = s.runner.x - x2;
        assert!(s.pose_at(t.0, fy) != Pose::Walk || sprinted > walked, "sprint not faster: {walked} vs {sprinted} ({:?})", s.pose_at(t.0, fy));
        assert!(s.pose_at(t.0, fy) != Pose::Walk || s.runner.stamina < st0);
        assert!(s.pose_at(t.0, fy) != Pose::Walk || s.senses.noise.since(0).any(|e| e.loudness >= 7));
        // walls stop you: walk up into the ceiling
        for _ in 0..200 {
            s.step(RunInput { dy: -1, ..Default::default() });
        }
        let t = s.runner.tile();
        assert!(s.map.passable(t.0 as usize, t.1 as usize));
        assert!(!s.map.passable(t.0 as usize, (t.1 - 1) as usize) || s.runner.y % FP != FP / 2);
    }

    #[test]
    fn opening_taking_and_extracting_through_inputs() {
        let mut s = sim(11, 1);
        // find the nearest closed container and walk the Runner onto it by teleport (pathing is the view's job)
        let i = (0..s.loot.sources.len()).find(|&i| s.loot.sources[i].state == SourceState::Closed).unwrap();
        let (sx, sy) = (s.loot.sources[i].x as i32, s.loot.sources[i].y as i32);
        s.runner.x = sx * FP + FP / 2;
        s.runner.y = sy * FP + FP / 2;
        s.step(RunInput { interact: true, ..Default::default() });
        assert!(s.events.iter().any(|e| matches!(e, Event::Opened { source, .. } if *source == i)), "{:?}", s.events);
        s.drain_events();
        s.step(RunInput::default());
        s.step(RunInput { take: true, ..Default::default() });
        let took = s.events.iter().find_map(|e| if let Event::Took { taken, .. } = e { Some(*taken) } else { None }).unwrap();
        assert_eq!(s.runner.rig.backpack.count(), took);
        // extraction: back to the Mat edge and hold E for ten seconds
        let (ex, ey) = s.map.entry[1];
        s.runner.x = ex as i32 * FP + FP / 2;
        s.runner.y = ey as i32 * FP + FP / 2;
        for _ in 0..ticks(10.0) + 2 {
            s.step(RunInput { interact: true, ..Default::default() });
        }
        assert!(matches!(s.result, Some(RunResult::Extracted { .. })), "{:?}", s.runner.action);
    }

    #[test]
    fn a_crawler_bites_sections_and_a_pipe_kills_it() {
        let mut s = sim(11, 1);
        s.creatures.clear();
        let def = s.defs.creature["crawler"].clone();
        let (rx, ry) = s.runner.tile();
        let mut c = Creature::new(99, Kind::Crawler, &def, (rx + 1, ry), 0);
        c.phase = Phase::Alert;
        c.awareness = 1.0;
        c.last_known = Some((rx, ry));
        s.creatures.push(c);
        let hp0: i32 = s.runner.sections.iter().sum();
        for _ in 0..ticks(3.0) {
            s.step(RunInput::default());
        }
        let hp1: i32 = s.runner.sections.iter().sum();
        assert!(hp1 < hp0, "no bite landed");
        assert!(s.events.iter().any(|e| matches!(e, Event::RunnerHit { .. })));
        // swing: pipe 8 damage; under 30 % the crawler flees faster than a Runner walks
        s.runner.facing = 1;
        let mut swings = 0;
        while s.creatures[0].phase != Phase::Flee && swings < 12 {
            let face = (s.creatures[0].tile().0 - s.runner.tile().0).signum();
            s.step(RunInput { attack: true, dx: face as i8, ..Default::default() });
            for _ in 0..ticks(0.6) {
                s.step(RunInput::default());
            }
            swings += 1;
        }
        assert_eq!(s.creatures[0].phase, Phase::Flee, "crawler did not flee after {swings} swings (hp {})", s.creatures[0].hp);
        assert!(s.events.iter().any(|e| matches!(e, Event::CreatureHit { .. })));
        let d0 = (s.creatures[0].tile().0 - s.runner.tile().0).abs();
        for _ in 0..ticks(1.0) {
            s.step(RunInput::default());
        }
        let d1 = (s.creatures[0].tile().0 - s.runner.tile().0).abs();
        assert!(d1 > d0, "fleeing crawler did not open the distance ({d0} -> {d1})");
        // corner it: one more hit kills it and leaves a corpse to pick over
        let (rx, ry) = s.runner.tile();
        s.creatures[0].x = (rx + 1) * FP + FP / 2;
        s.creatures[0].y = ry * FP + FP / 2;
        s.creatures[0].hp = 6; // above the flee line, under one pipe blow
        s.creatures[0].phase = Phase::Alert;
        s.creatures[0].awareness = 1.0;
        s.runner.facing = 1;
        s.step(RunInput { attack: true, dx: 1, ..Default::default() });
        for _ in 0..ticks(0.6) {
            s.step(RunInput::default());
        }
        assert!(s.creatures.is_empty(), "cornered crawler survived (hp {:?})", s.creatures.first().map(|c| c.hp));
        assert!(s.events.iter().any(|e| matches!(e, Event::CreatureDied { kind: Kind::Crawler, source: Some(_), .. })));
        assert!(s.loot.sources.iter().any(|src| src.kind == SourceKind::RatCorpse));
    }

    #[test]
    fn legs_at_zero_halve_speed_and_the_runner_goes_down_then_dies() {
        let mut s = sim(11, 1);
        s.creatures.clear();
        let x0 = s.runner.x;
        walk(&mut s, 1, 0, 10, false);
        let full = s.runner.x - x0;
        s.runner.sections[LEGS] = 0;
        let x1 = s.runner.x;
        walk(&mut s, 1, 0, 10, false);
        let half = s.runner.x - x1;
        assert!(half * 2 <= full + 30 && half * 2 >= full - 30, "full {full} half {half}");
        // sprint refused on dead legs
        walk(&mut s, 1, 0, 1, true);
        assert!(!s.runner.sprinting);
        // torso to zero: Down, then Dead after down_s
        let rules = s.defs.runner.clone();
        let (_, _, down) = s.runner.take_damage(&rules, 999, &[0, 1, 0, 0], 0, &mut Pcg32::new(1));
        assert!(down);
        for _ in 0..ticks(rules.down_s) + 2 {
            s.step(RunInput::default());
        }
        assert!(matches!(s.result, Some(RunResult::Died { .. })));
    }

    #[test]
    fn same_inputs_replay_bit_exact_and_different_inputs_do_not() {
        let script: Vec<RunInput> = (0..400)
            .map(|t| RunInput { dx: if t % 50 < 30 { 1 } else { 0 }, dy: if t % 70 < 10 { -1 } else { 0 }, sprint: t % 90 < 20, attack: t % 37 == 0, interact: t % 61 == 0, ..Default::default() })
            .collect();
        let mut a = sim(21, 2);
        let mut b = sim(21, 2);
        for i in &script {
            a.step(*i);
            b.step(*i);
        }
        assert_eq!(a.fingerprint(), b.fingerprint());
        let mut c = sim(21, 2);
        for (t, i) in script.iter().enumerate() {
            let mut j = *i;
            if (100..110).contains(&t) {
                j.dx = 0;
                j.sprint = false;
            }
            c.step(j);
        }
        assert_ne!(a.fingerprint(), c.fingerprint());
    }

    #[test]
    fn salt_and_antiseptic_consume_their_quick_slots() {
        let mut s = sim(11, 1);
        s.runner.add_status(StatusKind::Bleeding, 1000);
        s.step(RunInput { quick: 3, ..Default::default() });
        assert!(!s.runner.has(StatusKind::Bleeding));
        assert!(s.runner.rig.quick[2].is_none());
        s.step(RunInput::default());
        s.step(RunInput { quick: 4, ..Default::default() });
        let (x, y) = s.runner.tile();
        assert!(s.salt[s.idx(x, y)]);
        assert!(s.events.iter().any(|e| matches!(e, Event::Salted { .. })));
    }
}
