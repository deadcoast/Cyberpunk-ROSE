//! The Runner's lume and exposure, and `Senses`: the per-tick aggregate of light, sight, fog and noise that a run is
//! played against. Everything here is integer and deterministic so a recorded run replays bit-exact.
//!
//! Research 07, decisions 2-4: lume = veins + gear light + Mat under the feet, -3 while dimming, halved while
//! Soaked; a tile is visible when in the symmetric FOV and (lit >= dim or within 2 + lume + Eye/2);
//! exposure = max(light here, lume) and notice range = 2 + exposure (+2 sprinting, +1 digging, -1 still).

use crate::fog::Fog;
use crate::fov;
use crate::light::{LightField, Source, DIM_MIN, MAX_LIGHT};
use crate::map::RouteMap;
use crate::noise::{surface_add, NoiseField, Sound};

/// How far the FOV sweep goes; a lit pool further than this is beyond any chunk.
pub const FOV_RADIUS: i32 = 40;
pub const DIM_PENALTY: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Still,
    Sneak,
    Walk,
    Sprint,
    Dig,
}

impl Motion {
    pub fn notice_mod(self) -> i32 {
        match self {
            Motion::Still => -1,
            Motion::Sneak => 0,
            Motion::Walk => 0,
            Motion::Sprint => 2,
            Motion::Dig => 1,
        }
    }

    /// The footstep this motion makes, if it moves.
    pub fn step_sound(self) -> Option<Sound> {
        match self {
            Motion::Sneak => Some(Sound::SneakStep),
            Motion::Walk => Some(Sound::WalkStep),
            Motion::Sprint => Some(Sound::SprintStep),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lume {
    /// vein glow by Mod-Depth, 1-5
    pub veins: u8,
    /// sum of equipped gear `light` (lantern 6, lamp pole 8, visor -1)
    pub gear: i32,
    /// light of the Mat tile under the feet (0 off the Mat)
    pub mat_bonus: u8,
    pub dimmed: bool,
    pub soaked: bool,
}

impl Lume {
    pub fn emit(&self) -> u8 {
        let mut v = self.veins as i32 + self.gear + self.mat_bonus as i32;
        if self.dimmed {
            v -= DIM_PENALTY as i32;
        }
        if self.soaked {
            v /= 2;
        }
        // the ooze never goes fully out: dimmed, soaked, visored, a slimo still shows at least one step
        let floor = if self.veins > 0 { 1 } else { 0 };
        v.clamp(floor, MAX_LIGHT as i32) as u8
    }

    /// How far into the dark the Runner sees (Brogue's miner's light): 2 + lume + Eye/2.
    pub fn see_radius(&self, eye: u8) -> i32 {
        2 + self.emit() as i32 + (eye as i32) / 2
    }
}

/// How far anything can notice the Runner, in tiles.
pub fn notice_range(exposure: u8, motion: Motion) -> i32 {
    (2 + exposure as i32 + motion.notice_mod()).clamp(1, 20)
}

#[derive(Clone, Debug)]
pub struct Senses {
    pub light: LightField,
    pub fog: Fog,
    pub noise: NoiseField,
    pub visible: Vec<bool>,
    pub tick: u64,
    pub runner: (usize, usize),
    pub lume: Lume,
    pub eye: u8,
    /// Night Eyes: tiles added to the see radius
    pub see_bonus: i32,
    pub motion: Motion,
    /// transient light sources for this tick (muzzle flashes, carried lamps of others)
    pub flashes: Vec<Source>,
    pub exposure: u8,
}

impl Senses {
    pub fn new(map: &RouteMap, mat_lume_gene: f32, memory: Option<&[u8]>) -> Senses {
        let fog = match memory {
            Some(bits) => Fog::with_memory(map.w, map.h, bits),
            None => Fog::new(map.w, map.h),
        };
        let start = map.entry.get(1).or(map.entry.first()).copied().unwrap_or((0, 0));
        Senses {
            light: LightField::new(map, mat_lume_gene),
            fog,
            noise: NoiseField::new(map.w, map.h),
            visible: vec![false; map.w * map.h],
            tick: 0,
            runner: (start.0 as usize, start.1 as usize),
            lume: Lume { veins: 2, gear: 0, mat_bonus: 0, dimmed: false, soaked: false },
            eye: 3,
            see_bonus: 0,
            motion: Motion::Still,
            flashes: Vec::new(),
            exposure: 0,
        }
    }

    /// Advance one tick (20 Hz): dynamic light, sight, fog, noise decay. The caller sets `runner`, `lume`, `eye`,
    /// `motion` and pushes `flashes` before calling; flashes are consumed.
    pub fn tick(&mut self, map: &RouteMap) {
        self.tick += 1;
        let (rx, ry) = self.runner;
        // the Mat under the feet
        self.lume.mat_bonus = if map.entry.contains(&(rx as u16, ry as u16)) { self.light.static_[ry * map.w + rx] } else { 0 };
        let emit = self.lume.emit();
        let mut sources: Vec<Source> = Vec::with_capacity(1 + self.flashes.len());
        if emit > 0 {
            sources.push(Source { x: rx as u16, y: ry as u16, emit });
        }
        sources.append(&mut self.flashes);
        self.light.set_dynamic(map, &sources);
        self.compute_visible(map);
        self.fog.update(&self.visible);
        self.noise.decay();
        self.exposure = self.light.at(rx, ry).max(emit);
    }

    fn compute_visible(&mut self, map: &RouteMap) {
        self.visible.iter_mut().for_each(|v| *v = false);
        let (rx, ry) = (self.runner.0 as i32, self.runner.1 as i32);
        let see = self.lume.see_radius(self.eye) + self.see_bonus;
        let see2 = see * see;
        let (w, h) = (map.w as i32, map.h as i32);
        let light = &self.light.level;
        let vis = &mut self.visible;
        let sub_ = &map.substrate;
        fov::compute(
            (rx, ry),
            FOV_RADIUS,
            w,
            h,
            |x, y| !crate::chunk::substrate_passable(sub_[(y * w + x) as usize]),
            |x, y| {
                let i = (y * w + x) as usize;
                let dx = x - rx;
                let dy = y - ry;
                if light[i] >= DIM_MIN || dx * dx + dy * dy <= see2 {
                    vis[i] = true;
                }
            },
        );
    }

    /// A footstep or action by the Runner at their tile, with the surface add. Returns tiles reached.
    pub fn runner_sound(&mut self, map: &RouteMap, sound: Sound, extra: u8) -> usize {
        let (x, y) = self.runner;
        let loud = (sound.loudness() + surface_add(map.substrate[y * map.w + x]) + extra).min(crate::noise::MAX_NOISE);
        self.noise.emit(map, x, y, loud, self.tick, 0)
    }

    pub fn notice_range(&self) -> i32 {
        notice_range(self.exposure, self.motion)
    }

    #[inline]
    pub fn is_visible(&self, x: usize, y: usize) -> bool {
        self.visible[y * self.fog.w + x]
    }

    /// Packed RGB8 per tile for the fog shader: R light, G fog state (0/128/255), B noise.
    pub fn overlay_rgb(&self) -> Vec<u8> {
        let n = self.fog.w * self.fog.h;
        let mut out = vec![0u8; n * 3];
        for i in 0..n {
            out[i * 3] = self.light.level[i] * 17;
            out[i * 3 + 1] = match self.fog.state[i] {
                crate::fog::VISIBLE => 255,
                crate::fog::REMEMBERED => 128,
                _ => 0,
            };
            out[i * 3 + 2] = self.noise.level[i] * 17;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assemble::route;
    use crate::chunk::ChunkLib;
    use crate::light::LIT_MIN;

    fn map() -> RouteMap {
        let l = ChunkLib::from_toml(crate::CHUNKS_TOML).unwrap();
        RouteMap::compile(&l, &route(&l, 11, 1).unwrap())
    }

    #[test]
    fn lume_adds_up_and_clamps() {
        let mut l = Lume { veins: 2, gear: 6, mat_bonus: 0, dimmed: false, soaked: false };
        assert_eq!(l.emit(), 8);
        l.dimmed = true;
        assert_eq!(l.emit(), 5);
        l.soaked = true;
        assert_eq!(l.emit(), 2);
        l.gear = -1;
        l.dimmed = false;
        assert_eq!(l.emit(), 1, "the ooze never goes fully out");
        assert_eq!(Lume { veins: 0, gear: 0, mat_bonus: 0, dimmed: false, soaked: false }.emit(), 0);
        l.gear = 30;
        l.soaked = false;
        assert_eq!(l.emit(), MAX_LIGHT);
        assert_eq!(Lume { veins: 2, gear: 0, mat_bonus: 0, dimmed: false, soaked: false }.see_radius(4), 6);
    }

    #[test]
    fn notice_range_tracks_brogue_bands() {
        assert_eq!(notice_range(1, Motion::Walk), 3);
        assert_eq!(notice_range(5, Motion::Walk), 7);
        assert_eq!(notice_range(12, Motion::Walk), 14);
        assert_eq!(notice_range(5, Motion::Sprint), 9);
        assert_eq!(notice_range(0, Motion::Still), 1);
    }

    #[test]
    fn dark_runner_sees_little_and_lit_runner_is_exposed() {
        let m = map();
        let mut s = Senses::new(&m, 0.0, None);
        // step off the Mat edge into the culvert so the Mat glow does not count
        s.runner = (m.entry[1].0 as usize + 6, m.entry[1].1 as usize);
        s.lume = Lume { veins: 1, gear: 0, mat_bonus: 0, dimmed: true, soaked: false };
        s.eye = 0;
        s.tick(&m);
        let dark_seen = s.visible.iter().filter(|&&v| v).count();
        let dark_exposure = s.exposure;
        s.lume = Lume { veins: 2, gear: 8, mat_bonus: 0, dimmed: false, soaked: false };
        s.tick(&m);
        let lit_seen = s.visible.iter().filter(|&&v| v).count();
        assert!(lit_seen > dark_seen * 2, "lit {lit_seen} dark {dark_seen}");
        assert!(s.exposure >= LIT_MIN && s.exposure > dark_exposure);
        assert!(s.notice_range() > notice_range(dark_exposure, Motion::Still));
        // what was seen stays remembered after moving away
        let before = s.fog.seen_count();
        s.runner = (s.runner.0 + 1, s.runner.1);
        s.tick(&m);
        assert!(s.fog.seen_count() >= before);
        assert!(s.fog.state.iter().any(|&st| st == crate::fog::REMEMBERED));
    }

    #[test]
    fn mat_edge_lights_the_runner_standing_on_it() {
        let m = map();
        let mut s = Senses::new(&m, 1.0, None);
        s.lume = Lume { veins: 1, gear: 0, mat_bonus: 0, dimmed: false, soaked: false };
        s.tick(&m);
        assert_eq!(s.lume.mat_bonus, crate::light::mat_emit(1.0));
        assert!(s.exposure >= crate::light::mat_emit(1.0));
    }

    #[test]
    fn sounds_carry_surface_and_overlay_packs() {
        let m = map();
        let mut s = Senses::new(&m, 0.5, None);
        s.runner = (m.entry[1].0 as usize + 6, m.entry[1].1 as usize);
        s.tick(&m);
        let reached = s.runner_sound(&m, Sound::SprintStep, 0);
        assert!(reached > 20);
        assert_eq!(s.noise.at(s.runner.0, s.runner.1), 7 + surface_add(m.substrate[m.idx(s.runner.0, s.runner.1)]));
        let rgb = s.overlay_rgb();
        assert_eq!(rgb.len(), m.w * m.h * 3);
        let i = s.runner.1 * m.w + s.runner.0;
        assert_eq!(rgb[i * 3 + 1], 255);
        assert!(rgb[i * 3 + 2] > 0);
        // memory survives a fresh Senses built from the bits
        let bits = s.fog.memory_bits();
        let t = Senses::new(&m, 0.5, Some(&bits));
        assert_eq!(t.fog.seen_count(), s.fog.seen_count());
    }
}
