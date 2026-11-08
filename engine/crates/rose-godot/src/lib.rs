//! GDExtension: exposes the ROSE core to Godot as `RoseSim`.
//! Godot owns the clock and the picture; this class owns every number.

use godot::prelude::*;
use matsim::city::state;
use matsim::genome::Genome;
use matsim::sim::{hy, sub};
use matsim::{City, MatSim};

struct RoseExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RoseExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseSim {
    base: Base<RefCounted>,
    city: Option<City>,
    last_step_us: i64,
}

#[godot_api]
impl RoseSim {
    /// Build the reference scale scenario: grow a stratum for `grow_ticks`, then populate it with `agents` slimos.
    #[func]
    fn start_scale(&mut self, seed: i64, agents: i64, grow_ticks: i64) {
        self.city = Some(matsim::scale_scenario(seed as u64, agents.max(0) as usize, grow_ticks.max(0) as u64));
    }

    /// Fresh stratum with a Heart at (x, y) and no city; feed it to make it grow.
    #[func]
    fn start_empty(&mut self, seed: i64, w: i64, h: i64, heart_x: i64, heart_y: i64) {
        let mut sim = MatSim::new(w.max(16) as usize, h.max(8) as usize, seed as u64, Genome::default());
        sim.seed_heart(heart_x as usize, heart_y as usize);
        self.city = Some(City::new(sim, 0, &[], seed as u64));
    }

    #[func]
    fn step(&mut self) -> i64 {
        let t0 = std::time::Instant::now();
        let changed = match self.city.as_mut() {
            Some(c) => {
                c.step();
                c.sim.changed.len() as i64
            }
            None => 0,
        };
        self.last_step_us = t0.elapsed().as_micros() as i64;
        changed
    }

    #[func]
    fn width(&self) -> i64 {
        self.city.as_ref().map_or(0, |c| c.sim.w as i64)
    }

    #[func]
    fn height(&self) -> i64 {
        self.city.as_ref().map_or(0, |c| c.sim.h as i64)
    }

    #[func]
    fn tick(&self) -> i64 {
        self.city.as_ref().map_or(0, |c| c.tick as i64)
    }

    /// Substrate per cell (see matsim::sim::sub), row-major.
    #[func]
    fn substrate(&self) -> PackedByteArray {
        self.city.as_ref().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.sim.sub.as_slice()))
    }

    /// Hyphal state per cell (0 empty, 1 tip, 2 cord, 3 node, 4 necrotic).
    #[func]
    fn hyphae(&self) -> PackedByteArray {
        self.city.as_ref().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.sim.hy.as_slice()))
    }

    /// Signal 0..1 per cell.
    #[func]
    fn signal(&self) -> PackedFloat32Array {
        self.city.as_ref().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.sig.as_slice()))
    }

    /// Nutrient 0..1 per cell.
    #[func]
    fn nutrient(&self) -> PackedFloat32Array {
        self.city.as_ref().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.n.as_slice()))
    }

    /// Cell indices whose hyphal state changed on the last step.
    #[func]
    fn changed_cells(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.sim.changed.iter().map(|&i| i as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Slimo positions as cell indices.
    #[func]
    fn agent_positions(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.pos.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Slimo states (0 home, 1 to market, 2 carrying to pit, 3 going home).
    #[func]
    fn agent_states(&self) -> PackedByteArray {
        self.city.as_ref().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.state.as_slice()))
    }

    /// Cargo packet positions as cell indices.
    #[func]
    fn packet_positions(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.pk_pos.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Traffic per cell (slimos + 2 x packets), what the Stack view draws as light.
    #[func]
    fn traffic(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.traffic().iter().map(|&t| t as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    #[func]
    fn sites(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        if let Some(c) = &self.city {
            let to_arr = |v: &Vec<usize>| PackedInt32Array::from(v.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice());
            let (m, p, h) = (to_arr(&c.markets), to_arr(&c.pits), to_arr(&c.pods));
            d.set("markets", &m);
            d.set("pits", &p);
            d.set("pods", &h);
        }
        d
    }

    // ------------------------------------------------------------ player verbs
    #[func]
    fn feed(&mut self, x: i64, y: i64, amount: f64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.feed(x as usize, y as usize, amount as f32);
            }
        }
    }

    #[func]
    fn bait(&mut self, x: i64, y: i64, amount: f64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.bait(x as usize, y as usize, amount as f32);
            }
        }
    }

    #[func]
    fn sever(&mut self, x: i64, y: i64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.sever(x as usize, y as usize);
            }
        }
    }

    #[func]
    fn salt(&mut self, x: i64, y: i64, on: bool) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.salt_tile(x as usize, y as usize, on);
            }
        }
    }

    #[func]
    fn place_relay(&mut self, x: i64, y: i64) {
        if let Some(c) = self.city.as_mut() {
            if Self::inside(c, x, y) {
                c.sim.place_relay(x as usize, y as usize);
                c.pits.push(y as usize * c.sim.w + x as usize);
                c.rebuild_fields();
            }
        }
    }

    #[func]
    fn stats(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        if let Some(c) = &self.city {
            let s = c.sim.stats();
            let cs = c.stats();
            d.set("tick", s.tick as i64);
            d.set("tips", s.tips as i64);
            d.set("cords", s.cords as i64);
            d.set("nodes", s.nodes as i64);
            d.set("necrotic", s.necro as i64);
            d.set("cells", (s.tips + s.cords + s.nodes) as i64);
            d.set("reach", s.reach as i64);
            d.set("nutrient", s.nutrient as f64);
            d.set("agents", cs.agents as i64);
            d.set("carrying", cs.carrying as i64);
            d.set("packets", cs.packets as i64);
            d.set("deliveries", cs.deliveries as i64);
            d.set("hauls", cs.hauls as i64);
            d.set("jammed", cs.jammed_tiles as i64);
            d.set("step_us", self.last_step_us);
            let _ = (hy::EMPTY, sub::ROCK, state::HOME);
        }
        d
    }

    fn inside(c: &City, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < c.sim.w && (y as usize) < c.sim.h
    }
}
