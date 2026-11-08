//! Followers: up to two colony slimos who fight without being told, take three orders (Follow, Hold, Guard), and
//! carry a downed Runner toward the way out (Full Design v3.2, Followers). One HP pool, the Runner's timers.

use crate::creature::{distance_field, speed_units, ticks, Creature, FP, NO_PATH};
use crate::map::RouteMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    Follow,
    Hold,
    /// guard the tile they stood on when ordered
    Guard,
}

impl Order {
    pub fn name(self) -> &'static str {
        match self {
            Order::Follow => "follow",
            Order::Hold => "hold",
            Order::Guard => "guard",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FollowerLife {
    Up,
    Down { left: i32 },
    Dead,
}

#[derive(Clone, Debug)]
pub struct Follower {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub facing: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub order: Order,
    pub post: (i32, i32),
    pub life: FollowerLife,
    /// melee damage (fists 4, or the weapon they carry)
    pub damage: i32,
    pub windup_left: i32,
    pub cooldown_left: i32,
    /// carrying the downed Runner: tiles of carry left
    pub carrying: Option<i32>,
    pub carried_units: i32,
    field_target: Option<(i32, i32)>,
    field: Vec<u16>,
}

pub const FOLLOW_DISTANCE: i32 = 3;
pub const GUARD_REACH: i32 = 3;
pub const PICKUP_RANGE: i32 = 6;
pub const CARRY_TILES: i32 = 20;
pub const CARRY_SPEED_MULT: f32 = 0.6;
pub const EXTRACT_RANGE: i32 = 4;

/// What a follower did this tick.
#[derive(Default)]
pub struct FollowerOutcome {
    /// (creature index, damage, from behind)
    pub strike: Option<(usize, i32, bool)>,
    pub picked_up: bool,
    pub set_down: bool,
}

impl Follower {
    pub fn new(id: u32, name: &str, tile: (i32, i32), max_hp: i32, damage: i32) -> Follower {
        Follower {
            id,
            name: name.to_string(),
            x: tile.0 * FP + FP / 2,
            y: tile.1 * FP + FP / 2,
            facing: 1,
            hp: max_hp,
            max_hp,
            order: Order::Follow,
            post: tile,
            life: FollowerLife::Up,
            damage,
            windup_left: 0,
            cooldown_left: 0,
            carrying: None,
            carried_units: 0,
            field_target: None,
            field: Vec::new(),
        }
    }

    #[inline]
    pub fn tile(&self) -> (i32, i32) {
        (self.x.div_euclid(FP), self.y.div_euclid(FP))
    }

    pub fn is_up(&self) -> bool {
        matches!(self.life, FollowerLife::Up)
    }

    pub fn give(&mut self, order: Order) {
        self.order = order;
        if order == Order::Guard {
            self.post = self.tile();
        }
    }

    fn ensure_field(&mut self, map: &RouteMap, target: (i32, i32)) {
        if self.field_target == Some(target) && !self.field.is_empty() {
            return;
        }
        self.field_target = Some(target);
        self.field = distance_field(map, target);
    }

    /// Step along the distance field toward `target`. Returns units moved.
    fn step_toward(&mut self, map: &RouteMap, target: (i32, i32), speed: i32) -> i32 {
        self.ensure_field(map, target);
        let (tx, ty) = self.tile();
        if (tx, ty) == target {
            return 0;
        }
        let w = map.w as i32;
        let here = self.field[(ty * w + tx) as usize];
        if here == NO_PATH {
            return 0;
        }
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
            return 0;
        }
        let (gx, gy) = (best.1 .0 * FP + FP / 2, best.1 .1 * FP + FP / 2);
        let (cx, cy) = (tx * FP + FP / 2, ty * FP + FP / 2);
        let before = (self.x, self.y);
        if best.1 .1 == ty && (self.y - cy).abs() > 8 {
            self.y += (cy - self.y).clamp(-speed, speed);
        } else if best.1 .0 == tx && (self.x - cx).abs() > 8 {
            self.x += (cx - self.x).clamp(-speed, speed);
        } else {
            let dx = (gx - self.x).clamp(-speed, speed);
            let dy = (gy - self.y).clamp(-speed, speed);
            if dx != 0 {
                self.facing = dx.signum();
            }
            self.x += dx;
            self.y += dy;
        }
        (self.x - before.0).abs() + (self.y - before.1).abs()
    }

    /// Carry the Runner one tick toward `exit`; the Runner's position is moved with the follower.
    /// Returns true while still carrying.
    fn carry_tick(&mut self, map: &RouteMap, exit: (i32, i32), walk_speed: i32, runner_pos: &mut (i32, i32)) -> bool {
        let Some(left) = self.carrying else { return false };
        let speed = (walk_speed as f32 * CARRY_SPEED_MULT) as i32;
        let moved = self.step_toward(map, exit, speed);
        self.carried_units += moved;
        *runner_pos = (self.x, self.y);
        if self.carried_units >= FP {
            self.carried_units -= FP;
            let l = left - 1;
            if l <= 0 {
                self.carrying = None;
                return false;
            }
            self.carrying = Some(l);
        }
        if moved == 0 && self.tile() != exit {
            // no way on: set the Runner down here
            self.carrying = None;
            return false;
        }
        true
    }
}

pub struct FollowerContext<'a> {
    pub map: &'a RouteMap,
    pub runner_tile: (i32, i32),
    pub runner_down: bool,
    pub exit: (i32, i32),
    pub walk_speed: i32,
    pub creatures: &'a [Creature],
    pub windup_ticks: i32,
    pub cooldown_ticks: i32,
}

/// One tick of a follower. `runner_pos` is written when carrying.
pub fn update(f: &mut Follower, cx: &FollowerContext, runner_pos: &mut (i32, i32), already_carried: bool) -> FollowerOutcome {
    let mut out = FollowerOutcome::default();
    match f.life {
        FollowerLife::Dead => return out,
        FollowerLife::Down { left } => {
            f.life = if left <= 1 { FollowerLife::Dead } else { FollowerLife::Down { left: left - 1 } };
            return out;
        }
        FollowerLife::Up => {}
    }
    f.cooldown_left = (f.cooldown_left - 1).max(0);
    let me = f.tile();
    // ---- carrying a downed Runner
    if f.carrying.is_some() {
        if !cx.runner_down {
            f.carrying = None;
        } else if !f.carry_tick(cx.map, cx.exit, cx.walk_speed, runner_pos) {
            out.set_down = true;
        }
        return out;
    }
    if cx.runner_down && !already_carried && f.order == Order::Follow {
        let d2 = (cx.runner_tile.0 - me.0).pow(2) + (cx.runner_tile.1 - me.1).pow(2);
        if d2 <= PICKUP_RANGE * PICKUP_RANGE {
            if d2 <= 2 {
                f.carrying = Some(CARRY_TILES);
                f.carried_units = 0;
                out.picked_up = true;
                return out;
            }
            f.step_toward(cx.map, cx.runner_tile, cx.walk_speed);
            return out;
        }
    }
    // ---- fight: the nearest living creature within reach (Guard: within GUARD_REACH of the post)
    let reach_from = if f.order == Order::Guard { f.post } else { me };
    let mut nearest: Option<(usize, i32)> = None;
    for (i, c) in cx.creatures.iter().enumerate() {
        if c.dead {
            continue;
        }
        let ct = c.tile();
        let d2 = (ct.0 - me.0).pow(2) + (ct.1 - me.1).pow(2);
        let from_post = (ct.0 - reach_from.0).pow(2) + (ct.1 - reach_from.1).pow(2);
        let engage = match f.order {
            Order::Guard => from_post <= GUARD_REACH * GUARD_REACH,
            Order::Hold => d2 <= 2,
            Order::Follow => d2 <= 4 * 4,
        };
        if engage && nearest.is_none_or(|(_, bd)| d2 < bd) {
            nearest = Some((i, d2));
        }
    }
    if let Some((i, d2)) = nearest {
        let ct = cx.creatures[i].tile();
        if ct.0 != me.0 {
            f.facing = (ct.0 - me.0).signum();
        }
        if d2 <= 2 {
            if f.windup_left == 0 && f.cooldown_left == 0 {
                f.windup_left = cx.windup_ticks.max(1);
            } else if f.windup_left > 0 {
                f.windup_left -= 1;
                if f.windup_left == 0 {
                    f.cooldown_left = cx.cooldown_ticks;
                    let behind = cx.creatures[i].facing == f.facing && ct.0 != me.0;
                    out.strike = Some((i, f.damage, behind));
                }
            }
            return out;
        }
        f.windup_left = 0;
        if f.order != Order::Hold {
            f.step_toward(cx.map, ct, cx.walk_speed);
            return out;
        }
    }
    f.windup_left = 0;
    // ---- orders
    match f.order {
        Order::Follow => {
            let d2 = (cx.runner_tile.0 - me.0).pow(2) + (cx.runner_tile.1 - me.1).pow(2);
            if d2 > FOLLOW_DISTANCE * FOLLOW_DISTANCE {
                f.step_toward(cx.map, cx.runner_tile, cx.walk_speed);
            }
        }
        Order::Hold => {}
        Order::Guard => {
            if me != f.post {
                f.step_toward(cx.map, f.post, cx.walk_speed);
            }
        }
    }
    out
}

/// The follower's HP from the Body stat: 60 % of a Runner's total at the same Body.
pub fn hp_for(body: u8, runner_total: i32) -> i32 {
    let _ = body;
    (runner_total as f32 * 0.6).round() as i32
}

pub fn down_ticks(down_s: f32) -> i32 {
    ticks(down_s)
}

pub fn walk_units(tiles_per_s: f32) -> i32 {
    speed_units(tiles_per_s)
}
