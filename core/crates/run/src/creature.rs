//! Creatures of the run: definitions from data/creatures.toml, the awareness model (research 09: one 0-1 value,
//! sight by distance inside the Runner's notice range and the creature's own symmetric FOV, sound as a floor with
//! a remembered position, slow decay through every phase), the four phases, pathing by distance field, attacks.
//! All integer / fixed-point; the sim's RNG is the only randomness.

use crate::chunk::substrate_passable;
use crate::fov;
use crate::map::{RouteMap, SpawnKind};
use mat::rng::Pcg32;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};

/// fixed point: 256 units per tile
pub const FP: i32 = 256;
pub const TICK_HZ: i32 = 20;
/// half-width of every body for collision, in units (0.375 tile)
pub const HALF: i32 = 96;

#[derive(Clone, Debug, Deserialize)]
pub struct CreatureDef {
    pub sprite: String,
    pub hp: i32,
    pub speed: f32,
    pub prowl_speed: f32,
    pub hear: u8,
    #[serde(default)]
    pub sees: bool,
    #[serde(default)]
    pub lamp: u8,
    #[serde(default)]
    pub damage: i32,
    #[serde(default)]
    pub windup: f32,
    #[serde(default)]
    pub cooldown: f32,
    #[serde(default = "one")]
    pub reach: i32,
    pub section_odds: [u32; 4],
    #[serde(default)]
    pub bleed_chance: f32,
    #[serde(default)]
    pub flee_below: f32,
    #[serde(default)]
    pub fuse: f32,
    #[serde(default)]
    pub burst_damage: i32,
    #[serde(default)]
    pub burst_radius: i32,
    #[serde(default)]
    pub death_burst: i32,
    #[serde(default)]
    pub grab: f32,
    #[serde(default)]
    pub hold_chunk: bool,
    #[serde(default)]
    pub ranged_damage: i32,
    #[serde(default)]
    pub shot_interval: f32,
    #[serde(default)]
    pub mag: i32,
    #[serde(default)]
    pub reload: f32,
    #[serde(default)]
    pub shot_noise: u8,
    #[serde(default)]
    pub range: i32,
    #[serde(default)]
    pub leave_after: f32,
    #[serde(default)]
    pub feral: bool,
    pub salvage: String,
}

fn one() -> i32 {
    1
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpawnRules {
    pub burrow_s1: [u32; 3],
    pub burrow_s2: [u32; 3],
    pub burrow_s3: [u32; 3],
    pub burrow_s4: [u32; 3],
    pub burrow_s5: [u32; 3],
    pub burrow_s6: [u32; 3],
    pub pack: [u8; 2],
    pub patrol_s1: f32,
    pub patrol_s2: f32,
    pub patrol_s3: f32,
    pub patrol_s4: f32,
    pub patrol_s5: f32,
    pub patrol_s6: f32,
}

impl SpawnRules {
    pub fn burrow(&self, stratum: u8) -> [u32; 3] {
        match stratum {
            1 => self.burrow_s1,
            2 => self.burrow_s2,
            3 => self.burrow_s3,
            4 => self.burrow_s4,
            5 => self.burrow_s5,
            _ => self.burrow_s6,
        }
    }
    pub fn patrol(&self, stratum: u8) -> f32 {
        match stratum {
            1 => self.patrol_s1,
            2 => self.patrol_s2,
            3 => self.patrol_s3,
            4 => self.patrol_s4,
            5 => self.patrol_s5,
            _ => self.patrol_s6,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Awareness {
    pub suspicious: f32,
    pub alert: f32,
    pub sound_floor: f32,
    pub decay_per_s: f32,
    pub sight_gain_min_per_s: f32,
    pub sight_gain_max_per_s: f32,
    pub search_s: f32,
    pub look_s: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RunnerRules {
    pub sections: [i32; 4],
    pub stamina: i32,
    pub sprint_per_s: i32,
    pub swing: i32,
    pub dodge: i32,
    pub dim_per_s: i32,
    pub regen_per_s: i32,
    pub regen_delay_s: f32,
    pub walk: f32,
    pub sneak: f32,
    pub sprint: f32,
    pub dodge_s: f32,
    pub dodge_tiles: i32,
    pub bleed_per_4s: i32,
    pub bleed_s: f32,
    pub blind_s: f32,
    pub soaked_s: f32,
    pub down_s: f32,
    pub dig_s: f32,
    pub extract_s: f32,
    pub melee_windup: f32,
    pub melee_active: f32,
    pub behind_mult: f32,
    pub shot_odds: [u32; 4],
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreatureFile {
    pub creature: HashMap<String, CreatureDef>,
    pub spawn: SpawnRules,
    pub awareness: Awareness,
    pub runner: RunnerRules,
}

pub const CREATURES_TOML: &str = include_str!("../../../../data/creatures.toml");

impl CreatureFile {
    pub fn from_toml(text: &str) -> Result<CreatureFile, String> {
        let f: CreatureFile = toml::from_str(text).map_err(|e| format!("creatures.toml: {e}"))?;
        for k in ["crawler", "burster", "maw", "scout"] {
            if !f.creature.contains_key(k) {
                return Err(format!("creatures.toml: missing {k}"));
            }
        }
        Ok(f)
    }
    pub fn shipped() -> CreatureFile {
        CreatureFile::from_toml(CREATURES_TOML).expect("shipped creatures")
    }
}

pub fn ticks(seconds: f32) -> i32 {
    (seconds * TICK_HZ as f32).round().max(0.0) as i32
}

/// tiles per second -> units per tick
pub fn speed_units(tiles_per_s: f32) -> i32 {
    (tiles_per_s * FP as f32 / TICK_HZ as f32).round() as i32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Crawler,
    Burster,
    Maw,
    Scout,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Crawler => "crawler",
            Kind::Burster => "burster",
            Kind::Maw => "maw",
            Kind::Scout => "scout",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Calm,
    Suspicious,
    Alert,
    Search,
    Flee,
    Leaving,
}

impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Phase::Calm => "calm",
            Phase::Suspicious => "suspicious",
            Phase::Alert => "alert",
            Phase::Search => "search",
            Phase::Flee => "flee",
            Phase::Leaving => "leaving",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub id: u32,
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub facing: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub phase: Phase,
    pub awareness: f32,
    /// last place a stimulus put the Runner, in tiles
    pub last_known: Option<(i32, i32)>,
    pub saw_runner: bool,
    pub ticks_without_stimulus: i32,
    pub home: (i32, i32),
    pub chunk: u16,
    /// attack state
    pub windup_left: i32,
    pub cooldown_left: i32,
    pub fuse_left: i32,
    pub grab_left: i32,
    /// scouts
    pub patrol: Vec<(i32, i32)>,
    pub patrol_i: usize,
    pub laps: u32,
    pub age: i32,
    pub mag: i32,
    pub reload_left: i32,
    pub shot_cd: i32,
    pub look_left: i32,
    pub dead: bool,
    /// cached distance field to the current target tile
    field_target: Option<(i32, i32)>,
    field: Vec<u16>,
}

pub const NO_PATH: u16 = u16::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    /// melee hit on the Runner: damage, section odds index into the creature's table, bleed roll
    Bite { damage: i32, grab_ticks: i32 },
    Burst { damage: i32, radius: i32 },
    Shot { damage: i32 },
}

impl Creature {
    pub fn new(id: u32, kind: Kind, def: &CreatureDef, tile: (i32, i32), chunk: u16) -> Creature {
        Creature {
            id,
            kind,
            x: tile.0 * FP + FP / 2,
            y: tile.1 * FP + FP / 2,
            facing: 1,
            hp: def.hp,
            max_hp: def.hp,
            phase: Phase::Calm,
            awareness: 0.0,
            last_known: None,
            saw_runner: false,
            ticks_without_stimulus: 0,
            home: tile,
            chunk,
            windup_left: 0,
            cooldown_left: 0,
            fuse_left: 0,
            grab_left: 0,
            patrol: Vec::new(),
            patrol_i: 0,
            laps: 0,
            age: 0,
            mag: def.mag,
            reload_left: 0,
            shot_cd: 0,
            look_left: 0,
            dead: false,
            field_target: None,
            field: Vec::new(),
        }
    }

    #[inline]
    pub fn tile(&self) -> (i32, i32) {
        (self.x.div_euclid(FP), self.y.div_euclid(FP))
    }

    fn ensure_field(&mut self, map: &RouteMap, target: (i32, i32)) {
        if self.field_target == Some(target) && !self.field.is_empty() {
            return;
        }
        self.field_target = Some(target);
        self.field = if self.kind == Kind::Scout { distance_field_walk(map, target) } else { distance_field(map, target) };
    }

    /// Step toward `target` along the distance field at `speed` units per tick. Returns true when moved.
    /// With `salt`, a salted next tile is never entered (ferals). Scouts walk floors: a target on the back wall is
    /// taken as the floor beneath it.
    fn step_toward(&mut self, map: &RouteMap, target: (i32, i32), speed: i32, salt: Option<&[bool]>) -> bool {
        let target = if self.kind == Kind::Scout { floor_under(map, target) } else { target };
        self.ensure_field(map, target);
        let (tx, ty) = self.tile();
        if (tx, ty) == target {
            // centre on the tile
            let (cx, cy) = (tx * FP + FP / 2, ty * FP + FP / 2);
            self.x += (cx - self.x).clamp(-speed, speed);
            self.y += (cy - self.y).clamp(-speed, speed);
            return false;
        }
        let w = map.w as i32;
        let here = self.field[(ty * w + tx) as usize];
        if here == NO_PATH {
            return false;
        }
        // the neighbour with the smallest distance
        let mut best = (here, (tx, ty));
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (tx + dx, ty + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= map.h as i32 {
                continue;
            }
            let d = self.field[(ny * w + nx) as usize];
            if d < best.0 {
                best = (d, (nx, ny));
            }
        }
        if best.1 == (tx, ty) {
            return false;
        }
        if let Some(salt) = salt {
            if salt[(best.1 .1 * w + best.1 .0) as usize] {
                return false;
            }
        }
        // move toward the centre of the chosen neighbour; axis-aligned so bodies slide along walls
        let (gx, gy) = (best.1 .0 * FP + FP / 2, best.1 .1 * FP + FP / 2);
        let dx = (gx - self.x).clamp(-speed, speed);
        let dy = (gy - self.y).clamp(-speed, speed);
        if dx != 0 {
            self.facing = dx.signum();
        }
        // if the other axis is off-centre, correct it first so the body fits the corridor
        let (cx, cy) = (tx * FP + FP / 2, ty * FP + FP / 2);
        if best.1 .1 == ty && (self.y - cy).abs() > 8 {
            self.y += (cy - self.y).clamp(-speed, speed);
        } else if best.1 .0 == tx && (self.x - cx).abs() > 8 {
            self.x += (cx - self.x).clamp(-speed, speed);
        } else {
            self.x += dx;
            self.y += dy;
        }
        true
    }
}

/// Where a walker (a scout) can stand: a passable tile over a solid one, or water. Ferals crawl anything.
pub fn walkable(map: &RouteMap, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x as usize >= map.w || y as usize >= map.h {
        return false;
    }
    let i = y as usize * map.w + x as usize;
    if !substrate_passable(map.substrate[i]) {
        return false;
    }
    if map.substrate[i] == mat::sim::sub::WATER {
        return true;
    }
    (y as usize) + 1 < map.h && !substrate_passable(map.substrate[i + map.w])
}

/// The floor under a tile: the first walkable tile at or below it in its column (a walker's way to reach a
/// Runner on the back wall), or the tile itself when nothing is below.
pub fn floor_under(map: &RouteMap, t: (i32, i32)) -> (i32, i32) {
    let mut y = t.1;
    while (y as usize) + 1 < map.h && substrate_passable(map.substrate[y as usize * map.w + t.0 as usize]) {
        if walkable(map, t.0, y) {
            return (t.0, y);
        }
        y += 1;
    }
    t
}

/// 4-connected BFS distance (in tiles) to `target` over passable tiles; NO_PATH where unreachable.
pub fn distance_field(map: &RouteMap, target: (i32, i32)) -> Vec<u16> {
    distance_field_by(map, target, &|x, y| map.passable(x as usize, y as usize))
}

/// The same over tiles a walker can stand on.
pub fn distance_field_walk(map: &RouteMap, target: (i32, i32)) -> Vec<u16> {
    distance_field_by(map, target, &|x, y| walkable(map, x, y))
}

fn distance_field_by(map: &RouteMap, target: (i32, i32), ok: &dyn Fn(i32, i32) -> bool) -> Vec<u16> {
    let (w, h) = (map.w, map.h);
    let mut d = vec![NO_PATH; w * h];
    if target.0 < 0 || target.1 < 0 || target.0 as usize >= w || target.1 as usize >= h {
        return d;
    }
    let ti = target.1 as usize * w + target.0 as usize;
    if !ok(target.0, target.1) {
        return d;
    }
    d[ti] = 0;
    let mut q = VecDeque::from([ti]);
    while let Some(i) = q.pop_front() {
        let (x, y) = (i % w, i / w);
        let dist = d[i];
        let mut push = |nx: usize, ny: usize| {
            let j = ny * w + nx;
            if d[j] == NO_PATH && ok(nx as i32, ny as i32) {
                d[j] = dist + 1;
                q.push_back(j);
            }
        };
        if x > 0 {
            push(x - 1, y);
        }
        if x + 1 < w {
            push(x + 1, y);
        }
        if y > 0 {
            push(x, y - 1);
        }
        if y + 1 < h {
            push(x, y + 1);
        }
    }
    d
}

/// Can `from` see `to` within `radius`, by the same symmetric shadowcast the Runner uses? `front` restricts to the
/// half-plane the creature faces (scouts); ferals see all round.
pub fn can_see(map: &RouteMap, from: (i32, i32), to: (i32, i32), radius: i32, facing: Option<i32>) -> bool {
    if let Some(f) = facing {
        let dx = to.0 - from.0;
        if dx != 0 && dx.signum() != f {
            return false;
        }
    }
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    if dx * dx + dy * dy > radius * radius {
        return false;
    }
    let (w, h) = (map.w as i32, map.h as i32);
    let mut seen = false;
    fov::compute(from, radius, w, h, |x, y| !substrate_passable(map.substrate[(y * w + x) as usize]), |x, y| {
        if (x, y) == to {
            seen = true;
        }
    });
    seen
}

/// What the senses tell a creature this tick.
pub struct Stimuli<'a> {
    pub runner_tile: (i32, i32),
    pub runner_dodging: bool,
    pub notice_range: i32,
    pub noise: &'a [u8],
    /// most recent sound position this tick, if any
    pub last_sound: Option<(i32, i32)>,
    pub salt: &'a [bool],
}

/// Everything a creature did this tick that the sim must apply.
#[derive(Default)]
pub struct Outcome {
    pub attack: Option<Attack>,
    pub became: Option<Phase>,
    pub shot_from: Option<(i32, i32)>,
    pub left: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn update(c: &mut Creature, def: &CreatureDef, aw: &Awareness, map: &RouteMap, st: &Stimuli, rng: &mut Pcg32) -> Outcome {
    let mut out = Outcome::default();
    if c.dead {
        return out;
    }
    c.age += 1;
    let prev = c.phase;
    let me = c.tile();
    let w = map.w as i32;
    let dist2 = {
        let dx = st.runner_tile.0 - me.0;
        let dy = st.runner_tile.1 - me.1;
        dx * dx + dy * dy
    };
    // ---- stimuli
    let mut stimulated = false;
    if def.sees && c.phase != Phase::Flee {
        let facing = if c.kind == Kind::Scout { Some(c.facing) } else { None };
        if can_see(map, me, st.runner_tile, st.notice_range, facing) {
            let d = (dist2 as f32).sqrt();
            let t = 1.0 - (d / st.notice_range.max(1) as f32).clamp(0.0, 1.0);
            let gain = (aw.sight_gain_min_per_s + (aw.sight_gain_max_per_s - aw.sight_gain_min_per_s) * t) / TICK_HZ as f32;
            c.awareness = (c.awareness + gain).min(1.0);
            c.last_known = Some(st.runner_tile);
            c.saw_runner = true;
            stimulated = true;
        } else {
            c.saw_runner = false;
        }
    }
    let heard = st.noise[(me.1 * w + me.0) as usize] >= def.hear;
    if heard && c.phase != Phase::Flee {
        c.awareness = c.awareness.max(aw.sound_floor);
        if let Some(p) = st.last_sound {
            if !c.saw_runner {
                c.last_known = Some(p);
            }
        }
        stimulated = true;
    }
    if stimulated {
        c.ticks_without_stimulus = 0;
    } else {
        c.ticks_without_stimulus += 1;
        c.awareness = (c.awareness - aw.decay_per_s / TICK_HZ as f32).max(0.0);
    }
    // ---- phase
    if c.phase != Phase::Flee && c.phase != Phase::Leaving {
        let hunt_at = if def.feral { aw.suspicious } else { aw.alert };
        if c.awareness >= hunt_at {
            c.phase = Phase::Alert;
        } else if c.awareness >= aw.suspicious {
            c.phase = if c.phase == Phase::Alert && c.ticks_without_stimulus < ticks(aw.search_s) { Phase::Search } else { Phase::Suspicious };
        } else if c.phase == Phase::Alert || c.phase == Phase::Search {
            c.phase = if c.ticks_without_stimulus < ticks(aw.search_s) { Phase::Search } else { Phase::Calm };
        } else if c.phase == Phase::Suspicious && c.awareness < aw.suspicious * 0.5 {
            c.phase = Phase::Calm;
        }
    }
    if def.flee_below > 0.0 && (c.hp as f32) < def.flee_below * c.max_hp as f32 && c.phase != Phase::Flee {
        c.phase = Phase::Flee;
        c.awareness = 0.0;
    }
    if c.kind == Kind::Scout && c.phase == Phase::Calm && (c.laps >= 2 || c.age >= ticks(def.leave_after)) {
        c.phase = Phase::Leaving;
    }
    // ---- cooldowns
    c.cooldown_left = (c.cooldown_left - 1).max(0);
    c.shot_cd = (c.shot_cd - 1).max(0);
    if c.reload_left > 0 {
        c.reload_left -= 1;
        if c.reload_left == 0 {
            c.mag = def.mag;
        }
    }
    if c.grab_left > 0 {
        c.grab_left -= 1;
    }
    // ---- act
    let hunt_speed = speed_units(def.speed);
    let prowl = speed_units(def.prowl_speed);
    let adjacent = dist2 <= def.reach * def.reach * 2; // reach tiles, diagonals allowed
    match c.phase {
        Phase::Alert => {
            let target = if c.saw_runner { st.runner_tile } else { c.last_known.unwrap_or(st.runner_tile) };
            if c.kind == Kind::Burster {
                if adjacent {
                    if c.fuse_left == 0 {
                        c.fuse_left = ticks(def.fuse).max(1);
                    }
                } else {
                    c.fuse_left = 0;
                }
                if c.fuse_left > 0 {
                    c.fuse_left -= 1;
                    if c.fuse_left == 0 {
                        out.attack = Some(Attack::Burst { damage: def.burst_damage, radius: def.burst_radius });
                        c.dead = true;
                        c.hp = 0;
                    }
                } else {
                    c.step_toward(map, target, hunt_speed, Some(st.salt));
                }
            } else if c.kind == Kind::Scout && c.saw_runner && def.ranged_damage > 0 && dist2 <= def.range * def.range && !adjacent {
                // hold and shoot
                if st.runner_tile.0 != me.0 {
                    c.facing = (st.runner_tile.0 - me.0).signum();
                }
                if c.mag == 0 && c.reload_left == 0 {
                    c.reload_left = ticks(def.reload);
                } else if c.mag > 0 && c.shot_cd == 0 && c.reload_left == 0 {
                    c.mag -= 1;
                    c.shot_cd = ticks(def.shot_interval);
                    out.shot_from = Some(me);
                    if !st.runner_dodging {
                        out.attack = Some(Attack::Shot { damage: def.ranged_damage });
                    }
                }
            } else if adjacent && def.damage > 0 {
                // melee with a wind-up
                if c.windup_left == 0 && c.cooldown_left == 0 {
                    c.windup_left = ticks(def.windup).max(1);
                } else if c.windup_left > 0 {
                    c.windup_left -= 1;
                    if c.windup_left == 0 {
                        c.cooldown_left = ticks(def.cooldown);
                        if !st.runner_dodging {
                            let grab = if def.grab > 0.0 { ticks(def.grab) } else { 0 };
                            out.attack = Some(Attack::Bite { damage: def.damage, grab_ticks: grab });
                        }
                    }
                }
                if st.runner_tile.0 != me.0 {
                    c.facing = (st.runner_tile.0 - me.0).signum();
                }
            } else {
                c.windup_left = 0;
                let leash = def.hold_chunk && map.chunk_of[(target.1 * w + target.0) as usize] != c.chunk;
                if !leash {
                    c.step_toward(map, target, hunt_speed, if def.feral { Some(st.salt) } else { None });
                }
            }
        }
        Phase::Suspicious | Phase::Search => {
            let target = c.last_known.unwrap_or(me);
            if me == target {
                if c.look_left == 0 {
                    c.look_left = ticks(aw.look_s);
                } else {
                    c.look_left -= 1;
                    if c.look_left == 0 {
                        // looked, found nothing: wander one tile at random to search
                        let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)][rng.below(4) as usize];
                        let n = (me.0 + dx, me.1 + dy);
                        if map.passable(n.0.max(0) as usize, n.1.max(0) as usize) {
                            c.last_known = Some(n);
                        }
                        c.awareness = (c.awareness - 0.05).max(0.0);
                    }
                }
            } else {
                c.step_toward(map, target, prowl, if def.feral { Some(st.salt) } else { None });
            }
        }
        Phase::Calm => {
            if c.kind == Kind::Scout && !c.patrol.is_empty() {
                let target = c.patrol[c.patrol_i % c.patrol.len()];
                if me == target {
                    if c.look_left == 0 {
                        c.look_left = ticks(aw.look_s);
                    } else {
                        c.look_left -= 1;
                        if c.look_left == 0 {
                            c.patrol_i += 1;
                            if c.patrol_i % c.patrol.len() == 0 {
                                c.laps += 1;
                            }
                        }
                    }
                } else {
                    c.step_toward(map, target, prowl, None);
                }
            } else if c.kind != Kind::Scout {
                // ferals prowl around the burrow: every few seconds pick a tile within 3 of home
                if c.age % (TICK_HZ * 3) == 0 {
                    let n = (c.home.0 + rng.range(-3, 3), c.home.1 + rng.range(-2, 2));
                    if n.0 >= 0 && n.1 >= 0 && map.passable(n.0 as usize, n.1 as usize) {
                        c.last_known = Some(n);
                    }
                }
                if let Some(t) = c.last_known {
                    c.step_toward(map, t, prowl, Some(st.salt));
                }
            }
        }
        Phase::Flee => {
            // run for the burrow; if the Runner is close to it, run for the tile of this chunk farthest from them
            let runner_near_home = {
                let dx = st.runner_tile.0 - c.home.0;
                let dy = st.runner_tile.1 - c.home.1;
                dx * dx + dy * dy <= 9
            };
            let goal = if runner_near_home { farthest_in_chunk(map, c.chunk, st.runner_tile) } else { c.home };
            if me == goal || !c.step_toward(map, goal, hunt_speed, None) {
                // hiding: recover only out of the Runner's notice, calm down at 60 %
                if dist2 > st.notice_range * st.notice_range && c.age % TICK_HZ == 0 {
                    c.hp = (c.hp + 1).min(c.max_hp);
                }
                if (c.hp as f32) >= 0.6 * c.max_hp as f32 {
                    c.phase = Phase::Calm;
                }
            }
        }
        Phase::Leaving => {
            // walk to the far east edge of the route and leave
            let exit = east_exit(map);
            if me == exit || !c.step_toward(map, exit, hunt_speed, None) {
                out.left = true;
                c.dead = true;
            }
        }
    }
    if c.phase != prev {
        out.became = Some(c.phase);
    }
    out
}

/// The passable tile of `chunk` farthest (by path) from `from`.
pub fn farthest_in_chunk(map: &RouteMap, chunk: u16, from: (i32, i32)) -> (i32, i32) {
    let d = distance_field(map, from);
    let mut best = (0u16, from);
    for (i, &c) in map.chunk_of.iter().enumerate() {
        if c == chunk && d[i] != NO_PATH && d[i] > best.0 {
            best = (d[i], ((i % map.w) as i32, (i / map.w) as i32));
        }
    }
    best.1
}

/// The passable tile nearest the route's east edge on the entry row band.
pub fn east_exit(map: &RouteMap) -> (i32, i32) {
    let w = map.w as i32;
    for x in (0..w).rev() {
        for y in 0..map.h as i32 {
            if map.passable(x as usize, y as usize) {
                return (x, y);
            }
        }
    }
    (0, 0)
}

/// Spawn the route's creatures: packs at burrows, a patrol of two scouts per chunk with chalk marks (by stratum
/// chance). Deterministic in `rng`.
pub fn spawn_all(map: &RouteMap, defs: &CreatureFile, rng: &mut Pcg32) -> Vec<Creature> {
    let mut out = Vec::new();
    let mut next_id = 1u32;
    let weights = defs.spawn.burrow(map.stratum);
    for s in map.spawns.iter().filter(|s| s.kind == SpawnKind::Sewerat) {
        let nest = map.chunks.get(s.chunk as usize).is_some_and(|c| c.kind == "nest" || c.kind == "vault");
        let n = rng.range(defs.spawn.pack[0] as i32, defs.spawn.pack[1] as i32);
        for _ in 0..n {
            let mut w = weights;
            if !nest {
                w[2] = 0;
            }
            let total: u32 = w.iter().sum();
            if total == 0 {
                continue;
            }
            let mut roll = rng.below(total);
            let kind = if roll < w[0] {
                Kind::Crawler
            } else {
                roll -= w[0];
                if roll < w[1] { Kind::Burster } else { Kind::Maw }
            };
            let def = &defs.creature[kind.key()];
            let mut c = Creature::new(next_id, kind, def, (s.x as i32, s.y as i32), s.chunk);
            c.facing = if rng.below(2) == 0 { -1 } else { 1 };
            next_id += 1;
            out.push(c);
        }
    }
    // patrols: group chalk marks by chunk
    let mut marks: HashMap<u16, Vec<(i32, i32)>> = HashMap::new();
    for s in map.spawns.iter().filter(|s| s.kind == SpawnKind::Patrol) {
        marks.entry(s.chunk).or_default().push((s.x as i32, s.y as i32));
    }
    let mut chunks: Vec<u16> = marks.keys().copied().collect();
    chunks.sort_unstable();
    let p = defs.spawn.patrol(map.stratum);
    for ch in chunks {
        if rng.f32() >= p {
            continue;
        }
        let pts = &marks[&ch];
        let def = &defs.creature["scout"];
        for k in 0..2 {
            let start = pts[k % pts.len()];
            let mut c = Creature::new(next_id, Kind::Scout, def, start, ch);
            c.patrol = pts.clone();
            c.patrol_i = (k + 1) % pts.len();
            next_id += 1;
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;

    fn map(seed: u64, stratum: u8) -> RouteMap {
        let l = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        RouteMap::compile(&l, &route(&l, seed, stratum).unwrap())
    }

    #[test]
    fn defs_parse_and_spawns_match_the_route() {
        let defs = CreatureFile::shipped();
        assert_eq!(defs.creature.len(), 4);
        let m = map(7, 2);
        let mut rng = Pcg32::new(1);
        let cs = spawn_all(&m, &defs, &mut rng);
        let burrows = m.spawns.iter().filter(|s| s.kind == SpawnKind::Sewerat).count();
        let ferals = cs.iter().filter(|c| c.kind != Kind::Scout).count();
        assert!(ferals >= burrows && ferals <= burrows * 3, "{ferals} ferals from {burrows} burrows");
        assert!(cs.iter().filter(|c| c.kind == Kind::Scout).count() % 2 == 0);
        for c in &cs {
            assert!(m.passable(c.tile().0 as usize, c.tile().1 as usize));
            if c.kind == Kind::Maw {
                assert!(m.chunks[c.chunk as usize].kind == "nest" || m.chunks[c.chunk as usize].kind == "vault");
            }
        }
        // S1 never has patrols
        let m1 = map(3, 1);
        let cs1 = spawn_all(&m1, &defs, &mut rng);
        assert!(cs1.iter().all(|c| c.kind != Kind::Scout));
    }

    #[test]
    fn a_noise_wakes_a_crawler_and_it_paths_to_the_source() {
        let defs = CreatureFile::shipped();
        let m = map(11, 1);
        let def = &defs.creature["crawler"];
        // a crawler at the entry corridor, the runner 8 tiles east behind a corner-less stretch
        let start = (m.entry[1].0 as i32 + 3, m.entry[1].1 as i32);
        let runner = (start.0 + 8, start.1);
        assert!(m.passable(runner.0 as usize, runner.1 as usize));
        let mut c = Creature::new(1, Kind::Crawler, def, start, 0);
        let silence = vec![0u8; m.w * m.h];
        let mut noise = vec![0u8; m.w * m.h];
        noise[(start.1 as usize) * m.w + start.0 as usize] = 5;
        let mut rng = Pcg32::new(2);
        let salt = vec![false; m.w * m.h];
        // silent and dark: notice range 1 -> stays calm
        let quiet = Stimuli { runner_tile: runner, runner_dodging: false, notice_range: 1, noise: &silence, last_sound: None, salt: &salt };
        for _ in 0..40 {
            update(&mut c, def, &defs.awareness, &m, &quiet, &mut rng);
        }
        assert_eq!(c.phase, Phase::Calm);
        // a sprint step's noise reaches it: feral -> Alert at once and it closes the distance
        let loud = Stimuli { runner_tile: runner, runner_dodging: false, notice_range: 1, noise: &noise, last_sound: Some(runner), salt: &salt };
        update(&mut c, def, &defs.awareness, &m, &loud, &mut rng);
        assert_eq!(c.phase, Phase::Alert, "awareness {}", c.awareness);
        let d0 = (runner.0 - c.tile().0).abs();
        for _ in 0..30 {
            update(&mut c, def, &defs.awareness, &m, &quiet, &mut rng);
        }
        let d1 = (runner.0 - c.tile().0).abs();
        assert!(d1 < d0, "did not close: {d0} -> {d1}");
    }

    #[test]
    fn sight_ramps_by_distance_and_decays_through_search_to_calm() {
        let defs = CreatureFile::shipped();
        let m = map(11, 1);
        let def = &defs.creature["crawler"];
        let start = (m.entry[1].0 as i32 + 3, m.entry[1].1 as i32);
        let runner = (start.0 + 4, start.1);
        let mut c = Creature::new(1, Kind::Crawler, def, start, 0);
        let noise = vec![0u8; m.w * m.h];
        let salt = vec![false; m.w * m.h];
        let mut rng = Pcg32::new(3);
        let lit = Stimuli { runner_tile: runner, runner_dodging: false, notice_range: 12, noise: &noise, last_sound: None, salt: &salt };
        let mut t = 0;
        while c.phase != Phase::Alert && t < 200 {
            update(&mut c, def, &defs.awareness, &m, &lit, &mut rng);
            t += 1;
        }
        assert!(t < 40, "took {t} ticks to notice a lit runner 4 tiles away");
        // the runner vanishes: Alert -> Search -> Calm over the search window
        let far = (m.w as i32 - 2, 0);
        let gone = Stimuli { runner_tile: far, runner_dodging: false, notice_range: 1, noise: &noise, last_sound: None, salt: &salt };
        let mut saw_search = false;
        for _ in 0..(ticks(defs.awareness.search_s) + 200) {
            update(&mut c, def, &defs.awareness, &m, &gone, &mut rng);
            if c.phase == Phase::Search {
                saw_search = true;
            }
        }
        assert!(saw_search);
        assert_eq!(c.phase, Phase::Calm);
    }

    #[test]
    fn scouts_walk_their_marks_and_leave_when_nothing_happens() {
        let defs = CreatureFile::shipped();
        // find a route with a patrol
        let mut found = None;
        for seed in 0..60u64 {
            let m = map(seed, 3);
            let mut rng = Pcg32::new(seed);
            let cs = spawn_all(&m, &defs, &mut rng);
            if let Some(c) = cs.into_iter().find(|c| c.kind == Kind::Scout && c.patrol.len() >= 2) {
                found = Some((m, c));
                break;
            }
        }
        let (m, mut c) = found.expect("a patrol in 60 S3 routes");
        let def = &defs.creature["scout"];
        let noise = vec![0u8; m.w * m.h];
        let salt = vec![false; m.w * m.h];
        let mut rng = Pcg32::new(9);
        let nobody = Stimuli { runner_tile: (0, 0), runner_dodging: false, notice_range: 1, noise: &noise, last_sound: None, salt: &salt };
        let mut left = false;
        let mut visited = std::collections::HashSet::new();
        for _ in 0..ticks(def.leave_after) + 20 * 60 {
            let o = update(&mut c, def, &defs.awareness, &m, &nobody, &mut rng);
            visited.insert(c.tile());
            if o.left {
                left = true;
                break;
            }
        }
        assert!(left, "scout never left (phase {:?}, laps {})", c.phase, c.laps);
        assert!(c.patrol.iter().filter(|p| visited.contains(p)).count() >= 1);
    }

    #[test]
    fn bursters_fuse_when_adjacent_and_salt_stops_a_crawler() {
        let defs = CreatureFile::shipped();
        let m = map(11, 1);
        let bdef = &defs.creature["burster"];
        let start = (m.entry[1].0 as i32 + 3, m.entry[1].1 as i32);
        let mut b = Creature::new(1, Kind::Burster, bdef, start, 0);
        b.phase = Phase::Alert;
        b.awareness = 1.0;
        let noise = vec![0u8; m.w * m.h];
        let salt = vec![false; m.w * m.h];
        let mut rng = Pcg32::new(4);
        let close = Stimuli { runner_tile: (start.0 + 1, start.1), runner_dodging: false, notice_range: 12, noise: &noise, last_sound: None, salt: &salt };
        let mut burst = None;
        for _ in 0..40 {
            let o = update(&mut b, bdef, &defs.awareness, &m, &close, &mut rng);
            if let Some(Attack::Burst { damage, radius }) = o.attack {
                burst = Some((damage, radius));
                break;
            }
        }
        assert_eq!(burst, Some((10, 1)));
        assert!(b.dead);
        // salt line between a hunting crawler and the runner
        let cdef = &defs.creature["crawler"];
        let mut c = Creature::new(2, Kind::Crawler, cdef, start, 0);
        c.phase = Phase::Alert;
        c.awareness = 1.0;
        c.last_known = Some((start.0 + 6, start.1));
        let mut salted = vec![false; m.w * m.h];
        for dy in -2..=2 {
            let y = start.1 + dy;
            if y >= 0 && (y as usize) < m.h {
                salted[(y as usize) * m.w + (start.0 + 2) as usize] = true;
            }
        }
        let behind_salt = Stimuli { runner_tile: (start.0 + 6, start.1), runner_dodging: false, notice_range: 12, noise: &noise, last_sound: None, salt: &salted };
        for _ in 0..60 {
            update(&mut c, cdef, &defs.awareness, &m, &behind_salt, &mut rng);
        }
        assert!(c.tile().0 <= start.0 + 1, "crawler crossed the salt: at {:?}", c.tile());
    }
}
