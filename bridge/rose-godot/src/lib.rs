//! GDExtension: exposes the ROSE core to Godot as `RoseSim`.
//! Godot owns the clock and the picture; this class owns every number.

use godot::prelude::*;
use logistics::city::state;
use mat::genome::Genome;
use mat::sim::{hy, sub};
use logistics::City;
use mat::MatSim;

mod cargo_bridge;
mod hud_bridge;
mod market_bridge;
mod progress_bridge;
mod story_bridge;
mod colony_bridge;
mod people;
mod profile_bridge;

use crate::people::RosePeople;

struct RoseExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RoseExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseSim {
    base: Base<RefCounted>,
    city: Option<City>,
    last_step_us: i64,
    /// the city on the Mat (colony crate) and its structure defs
    colony: colony::Colony,
    defs: Option<colony::StructureFile>,
    /// the last shift's assignments: (slimo, job, structure), for the HUD
    last_jobs: Vec<(u32, ::people::Job, u32)>,
    /// the Charter's eight laws (data/laws.toml)
    laws: Option<colony::LawFile>,
    /// the city's one Stash, bound so cargo can land as it arrives (Full Design v3.11, Cargo)
    stash: Option<Gd<RoseStash>>,
    /// the city's log (landings, losses, decisions) and the last shift's outcome, for the alerts (v3.12)
    log: game::CityLog,
    last_outcome: Option<colony::ShiftOutcome>,
    /// the tick a jam began, while one stands
    jam_since: Option<u64>,
    /// the plan for the shift being worked: (slimo, job, structure), the movers' and the Job grid's
    plan: Vec<(u32, ::people::Job, u32)>,
    /// the surface's answer (Full Design v3.14, Pressure) and who stands Guard this shift
    threats: Option<threat::Threats>,
    posts: threat::Posts,
    /// Oathkeepers the shift's run killed
    run_kills: u32,
    /// the surface's factions (Full Design v3.15, Economy)
    economy: Option<economy::Economy>,
    /// the strata the Board offers (`set_strata_open`): raiders walk its routes from S2, and a recover's body lies on
    /// one of them
    strata: u8,
    raiders: bool,
    /// the campaign (Full Design v3.19, Narrative and Ivan)
    campaign: story::Campaign,
    /// New Run's choices waiting for `start_city` (mode, difficulty, strain code), and the profile New Game+ carries
    /// into `found_city`
    new_run: Option<(story::Mode, story::Difficulty, Option<save::StrainCode>)>,
    carry: Option<save::Profile>,
}

#[godot_api]
impl RoseSim {
    /// New Run (Full Design v3.19, the Game shell), before `start_city`: "campaign" or "endless", the difficulty by
    /// name, a strain code ("" none: New Game+ otherwise, carrying the profile's laws and Scrapdex). "" or why not.
    #[func]
    fn configure_run(&mut self, mode: GString, difficulty: GString, code: GString, profile: Option<Gd<profile_bridge::RoseProfile>>) -> GString {
        let mode = if mode.to_string() == "endless" { story::Mode::Endless } else { story::Mode::Campaign };
        let Some(diff) = story::Difficulty::from_name(&difficulty.to_string()) else { return GString::from("no such difficulty") };
        let code_s = code.to_string();
        let code = if code_s.trim().is_empty() {
            None
        } else {
            match save::StrainCode::parse(&code_s) {
                Some(c) => Some(c),
                None => return GString::from("that strain code does not read: 32 hex digits after ROSE-"),
            }
        };
        self.carry = code.and(profile.map(|p| p.bind().p.clone()));
        self.new_run = Some((mode, diff, code));
        GString::new()
    }

    /// The city's strain code as it stands: ROSE- and eight groups of four (Full Design v3.19, Meta).
    #[func]
    fn strain_code(&self) -> GString {
        let Some(c) = self.city.as_ref() else { return GString::new() };
        GString::from(game::meta::strain_code(&c.sim, &self.colony, self.campaign.seed).encode().as_str())
    }

    /// Whether a shift (the colony's day counter) is a Flood shift: the day's third, its first too in Undercroft.
    #[func]
    fn flood_shift(&self, day: i64) -> bool {
        self.threats.as_ref().is_some_and(|t| t.is_flood_shift(day.max(1) as u32))
    }

    /// The scale proof (Phase A): a grown stratum with `agents` synthetic slimos and synthetic market hauls. A load
    /// test of the arrays, not the game's city (Full Design v3.11, The city's clock).
    #[func]
    fn start_scale(&mut self, seed: i64, agents: i64, grow_ticks: i64) {
        self.city = Some(logistics::scale_scenario(seed as u64, agents.max(0) as usize, grow_ticks.max(0) as u64));
    }

    /// The game's city: the founding (the Mat steered to the first grate, logistics::founding), the Mat on its own
    /// clock, no phantoms; the movers are the roster and the cargo is the Stash's.
    #[func]
    fn start_city(&mut self, seed: i64) {
        let (mode, difficulty, code) = self.new_run.take().unwrap_or((story::Mode::Campaign, story::Difficulty::Culvert, None));
        let n = game::meta::NewRun { seed: seed as u32, mode, difficulty, code };
        self.campaign = n.campaign();
        let city = City::game(logistics::founding_with(seed as u64, n.genome()), seed as u64);
        self.threats = Some(threat::Threats::new(&city.sim, seed as u64));
        let table = items::ItemTable::from_toml(items::ITEMS_TOML).expect("items.toml");
        self.economy = Some(game::world::found_economy(&city.sim, &table, seed as u64));
        self.city = Some(city);
        if let (Some(t), Some(e)) = (self.threats.as_mut(), self.economy.as_mut()) {
            game::campaign::found(&self.campaign, t, e);
        }
        if let Some(c) = self.city.as_ref() {
            game::ivan::found(&mut self.campaign, &c.sim);
        }
        self.campaign.barker = story::Barker::new(seed as u64);
    }

    /// The Board's open strata: raiders walk its routes when one of them rolls for raiders (clear-raiders contracts).
    #[func]
    fn set_strata_open(&mut self, strata: i64) {
        self.strata = strata.clamp(1, 255) as u8;
        self.raiders = game::trade::raiders_walk(self.strata);
    }

    /// The city's stratum: the highest the Mat has reached (Full Design v3.19, The stack and the breach).
    #[func]
    fn city_stratum(&self) -> i64 {
        self.threats.as_ref().map_or(1, |t| t.stratum as i64)
    }

    /// The Route Board's three strata: one below the city's to one above (Full Design v3.19).
    #[func]
    fn board_strata(&self) -> PackedInt32Array {
        let n = self.city.as_ref().map_or(6, |c| c.sim.strata);
        PackedInt32Array::from(logistics::board_strata(self.city_stratum() as u8, n).map(|s| s as i32).collect::<Vec<_>>().as_slice())
    }

    /// The stack's strata for the views: [{n, name, short, region, ceiling (row), rows, reached}] from the Sump up.
    #[func]
    fn strata(&self) -> Array<VarDictionary> {
        let mut out = Array::new();
        let Some(c) = self.city.as_ref() else { return out };
        let top = self.city_stratum() as u8;
        for d in &logistics::StrataFile::shipped().stratum {
            if d.n > c.sim.strata {
                break;
            }
            let mut v = VarDictionary::new();
            v.set("n", d.n as i64);
            v.set("name", d.name.as_str());
            v.set("short", d.short.as_str());
            v.set("region", d.region.as_str());
            v.set("ceiling", c.sim.ceiling(d.n) as i64);
            v.set("rows", c.sim.strata_rows as i64);
            v.set("reached", d.n <= top);
            out.push(&v);
        }
        out
    }

    /// The crossings between strata: [{tile, x, y, below, sealed, key, key_name, has_key}] (a slab's card).
    #[func]
    fn crossings(&self) -> Array<VarDictionary> {
        let mut out = Array::new();
        let Some(c) = self.city.as_ref() else { return out };
        let held: Vec<String> = self.stash.as_ref().map(|st| {
            let b = st.bind();
            match (b.stash.as_ref(), b.table.as_ref()) {
                (Some(s), Some(t)) => s.handles().iter().map(|(_, _, it)| t.def(it.def).id.clone()).collect(),
                _ => Vec::new(),
            }
        }).unwrap_or_default();
        let table = items::ItemTable::from_toml(items::ITEMS_TOML).ok();
        for x in game::breach::crossings(&c.sim) {
            let mut v = VarDictionary::new();
            v.set("tile", x.tile as i64);
            v.set("x", (x.tile % c.sim.w) as i64);
            v.set("y", (x.tile / c.sim.w) as i64);
            v.set("below", x.below as i64);
            v.set("sealed", x.sealed);
            v.set("key", x.key.as_str());
            let name = table.as_ref().and_then(|t| t.id_of(&x.key).map(|k| t.def(k).name.clone())).unwrap_or_default();
            v.set("key_name", name.as_str());
            v.set("has_key", !x.key.is_empty() && held.contains(&x.key));
            out.push(&v);
        }
        out
    }

    /// Spend the grate key on a crossing (its card's OPEN): "" when it opened, else why not.
    #[func]
    fn open_crossing(&mut self, tile: i64) -> GString {
        let Some(st) = self.stash.as_mut() else { return "no Stash".into() };
        let mut st = st.bind_mut();
        st.ensure();
        let st = &mut *st;
        let Some(city) = self.city.as_mut() else { return "no city".into() };
        match game::breach::open_with_key(city, self.threats.as_mut(), st.stash.as_mut().unwrap(), st.table.as_ref().unwrap(), tile.max(0) as usize, &mut self.log) {
            Ok(_) => "".into(),
            Err(e) => e.as_str().into(),
        }
    }

    /// Fresh stratum with a Heart at (x, y) and no city; feed it to make it grow.
    #[func]
    fn start_empty(&mut self, seed: i64, w: i64, h: i64, heart_x: i64, heart_y: i64) {
        let mut sim = MatSim::new(w.max(16) as usize, h.max(8) as usize, seed as u64, Genome::default());
        sim.seed_heart(heart_x as usize, heart_y as usize);
        self.city = Some(City::game(sim, seed as u64));
    }

    /// Bind the city's Stash: cargo lands into it as it arrives. A ledger that comes with cargo this city is not
    /// carrying (a loaded save) puts it back on the Mat at the Heart (Full Design v3.11: it rides again on load).
    #[func]
    fn bind_stash(&mut self, stash: Gd<RoseStash>) {
        if let Some(c) = self.city.as_mut() {
            if let Some(heart) = c.heart_cell() {
                let riding: std::collections::HashSet<u32> = c.in_transit().iter().map(|t| t.0).collect();
                for s in stash.bind().ledger.in_flight.iter().filter(|s| !riding.contains(&s.id)) {
                    let (kind, key) = game::cargo::kind_of(s.to);
                    c.dispatch(heart, s.id, kind, key);
                }
            }
        }
        self.stash = Some(stash);
    }

    /// One city tick: slimos walk, cargo rides and lands, the Mat steps on its own clock.
    #[func]
    fn step(&mut self) -> i64 {
        let t0 = std::time::Instant::now();
        let defs = self.defs().clone();
        let changed = match self.city.as_mut() {
            Some(c) => {
                c.step();
                // a jam's standing time, for its alert
                self.jam_since = if c.last_jam > 0 { Some(self.jam_since.unwrap_or(c.tick)) } else { None };
                let shift = self.colony.shift.max(1);
                game::handoff::tick(c, &mut self.colony, self.jam_since, shift, &mut self.log);
                if let Some(st) = self.stash.as_mut() {
                    let mut st = st.bind_mut();
                    st.ensure();
                    let st = &mut *st;
                    let (stash, table) = (st.stash.as_mut().unwrap(), st.table.as_ref().unwrap());
                    if c.tick % logistics::city::REBUILD_EVERY == 0 && !c.synthetic() {
                        game::nerve::beat(c, &mut self.colony, &defs, self.threats.as_ref(), stash, &mut st.ledger, table, &mut self.log);
                        game::cargo::refresh_demands(&self.colony, &defs, c, stash, &st.ledger, table);
                    }
                    let landed = game::cargo::land(c, &mut self.colony, &defs, stash, &mut st.ledger, table, &mut self.log);
                    if let Some(e) = self.economy.as_mut() {
                        for (market, contract, item) in landed.delivered.iter().cloned() {
                            game::trade::delivered(e, &mut st.ledger, c, table, market, contract, item, &mut self.log);
                        }
                    }
                    if let Some(t) = self.threats.as_mut() {
                        game::ivan::tick(&mut self.campaign, c, t, &mut self.log);
                        game::pressure::tick(t, c, &mut self.colony, &defs, &self.posts, &mut self.log, &landed);
                        game::ivan::voice(&mut self.campaign, &self.colony, &c.sim, t, &mut self.log, c.tick);
                    }
                }
                c.sim.changed.len() as i64
            }
            None => 0,
        };
        self.last_step_us = t0.elapsed().as_micros() as i64;
        changed
    }

    // ------------------------------------------------------------ pressure (Full Design v3.14)

    /// The bodies on the map: role, x, y, hp, max_hp, lamp (tiles of white light), oathkeeper.
    #[func]
    fn threat_bodies(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(t) = &self.threats else { return out };
        for b in t.bodies.iter().filter(|b| !b.dead && !b.gone) {
            let def = &t.rules.body[b.role.key()];
            let mut d = VarDictionary::new();
            d.set("id", b.id as i64);
            d.set("role", if b.role == threat::Role::Tamed { "tamed" } else { b.role.key() });
            d.set("name", def.name.as_str());
            d.set("x", b.x as i64);
            d.set("y", b.y as i64);
            d.set("hp", b.hp as f64);
            d.set("max_hp", b.max_hp as f64);
            d.set("lamp", if b.role == threat::Role::Tamed { 0 } else { def.lamp as i64 });
            d.set("oathkeeper", b.role.oathkeeper());
            out.push(&d.to_variant());
        }
        out
    }

    /// Each grate's zone: grate (tile), x, y, number (on its stratum), stratum, label ("G5", "S2 G3"), attention, step
    /// (0 quiet, 1 Patrol, 2 Sweep, 3 Cleansing), reached, gained (last shift end), an event down.
    #[func]
    fn attention(&self) -> VarArray {
        let mut out = VarArray::new();
        let (Some(t), Some(c)) = (&self.threats, &self.city) else { return out };
        for (k, z) in t.zones.iter().enumerate() {
            let mut d = VarDictionary::new();
            d.set("grate", z.grate as i64);
            d.set("x", (z.grate % c.sim.w) as i64);
            d.set("y", (z.grate / c.sim.w) as i64);
            let (n, s) = (c.grate_number(z.grate), c.sim.stratum_at(z.grate));
            d.set("number", n as i64);
            d.set("stratum", s as i64);
            d.set("label", if s <= 1 { format!("G{n}") } else { format!("S{s} G{n}") }.as_str());
            d.set("attention", z.attention as f64);
            d.set("step", t.step_of(k) as i64);
            d.set("reached", c.markets.contains(&z.grate));
            d.set("gained", z.gained as f64);
            d.set("down", t.down_in(k, 0));
            // the street's faction, and whether Corpo intel shows the number (Economy)
            let e = self.economy.as_ref();
            d.set("faction", e.and_then(|e| e.faction_at(z.grate).map(|f| e.name(f).to_string())).unwrap_or_default().as_str());
            d.set("exact", e.is_some_and(|e| e.s.intel > 0));
            out.push(&d.to_variant());
        }
        out
    }

    /// Tiles the Flood has under water now.
    #[func]
    fn flooded_cells(&self) -> PackedInt32Array {
        self.threats.as_ref().map_or(PackedInt32Array::new(), |t| PackedInt32Array::from(t.flooded().map(|i| i as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// Tiles with bleach on them.
    #[func]
    fn stain_cells(&self) -> PackedInt32Array {
        self.threats.as_ref().map_or(PackedInt32Array::new(), |t| PackedInt32Array::from((0..t.stain.len()).filter(|&i| t.stain[i] > 0).map(|i| i as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// A Bloom's tissue.
    #[func]
    fn bloom_cells(&self) -> PackedInt32Array {
        self.city.as_ref().map_or(PackedInt32Array::new(), |c| PackedInt32Array::from((0..c.sim.hy.len()).filter(|&i| c.sim.is_bloom(i)).map(|i| i as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// Oathkeepers the shift's run killed: their Attention counts in the Heart's zone at the shift's end; raiders it
    /// killed count toward the clear-raiders contracts in hand (Economy).
    #[func]
    fn add_run_kills(&mut self, oathkeepers: i64, raiders: i64) {
        self.run_kills += oathkeepers.max(0) as u32;
        if let Some(e) = self.economy.as_mut() {
            e.raider_kills(raiders.max(0) as u32);
        }
    }

    /// What the run killed, by creature id: the first of each is a trophy (Full Design v3.18, Collections).
    #[func]
    fn add_run_trophies(&mut self, killed: PackedStringArray, day: i64) -> PackedStringArray {
        let mut won = PackedStringArray::new();
        for k in killed.as_slice() {
            if let Some(name) = self.colony.progress.collections.trophy("kill", &k.to_string(), day.max(1) as u32) {
                self.log.push(self.city.as_ref().map_or(0, |c| c.tick), game::Class::Landing, format!("Trophy: {name}"));
                won.push(name.as_str());
            }
        }
        won
    }

    /// The run's flags from the city (Full Design v3.18, Scav yield): the curio sets' and Conscription's cost.
    #[func]
    fn run_flags(&self) -> PackedStringArray {
        let Some(city) = self.city.as_ref() else { return PackedStringArray::new() };
        let mut a = PackedStringArray::new();
        for f in game::grow::run_flags(&self.colony, self.threats.as_ref(), &city.sim) {
            a.push(f.as_str());
        }
        a
    }

    /// Development only (the shots and the play script): set a gene, as a run of feeds would leave it.
    #[func]
    fn debug_gene(&mut self, gene: i64, value: f64) {
        if let Some(c) = self.city.as_mut() {
            if (0..10).contains(&gene) {
                c.sim.genome.0[gene as usize] = value.clamp(0.0, 1.0) as f32;
                c.sim.p = c.sim.genome.params();
            }
        }
    }

    /// Development only (the play script): the Mat up an open crossing, `up` tiles into the stratum above, as feeding
    /// it a few shifts would grow it: Cord from the Mat nearest the grate below to the grate, then up the shaft.
    #[func]
    fn debug_climb(&mut self, crossing: i64, up: i64) {
        use mat::sim::{hy, sub};
        let Some(c) = self.city.as_mut() else { return };
        let (sw, cross) = (c.sim.w, crossing.max(0) as usize);
        let g = cross + sw;
        if g >= c.sim.sub.len() || c.sim.sub[g] != sub::GRATE {
            return;
        }
        let way = logistics::ground_distance(&c.sim, g);
        let joined = logistics::joined_to_heart(&c.sim);
        let Some(mut at) = (0..joined.len()).filter(|&i| joined[i] && way[i] != usize::MAX).min_by_key(|&i| (way[i], i)) else { return };
        while way[at] > 0 {
            if !c.sim.is_hyphal_at(at) {
                c.sim.hy[at] = hy::CORD;
                c.sim.n[at] = 0.4;
            }
            let (x, y) = (at % sw, at / sw);
            let Some(n) = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)].into_iter().filter(|&(nx, ny)| nx < sw && ny < c.sim.h).map(|(nx, ny)| ny * sw + nx).filter(|&j| way[j] < way[at]).min_by_key(|&j| (way[j], j)) else { break };
            at = n;
        }
        for y in (cross / sw).saturating_sub(up.max(0) as usize)..=g / sw {
            for x in [cross % sw, cross % sw + 1] {
                let i = y * sw + x;
                if sub::GROWABLE[c.sim.sub[i] as usize] && !c.sim.is_hyphal_at(i) {
                    c.sim.hy[i] = hy::CORD;
                    c.sim.n[i] = 0.4;
                    c.sim.changed.push(i as u32);
                }
            }
        }
        c.sim.compute_signal();
    }

    /// Development only (the play script): every node researched and every tier open, as a long campaign leaves the
    /// Bench, so the later frames reach the systems the tree gates.
    #[func]
    fn debug_tech_all(&mut self) {
        // the HP nodes raise what stands by their share, as researching them does
        let defs = self.defs().clone();
        let before: Vec<i32> = self.colony.structures.iter().map(|s| self.colony.hp_max(&defs, s)).collect();
        self.colony.progress.tech = progress::Tech::all();
        let after: Vec<i32> = self.colony.structures.iter().map(|s| self.colony.hp_max(&defs, s)).collect();
        for (s, (b, a)) in self.colony.structures.iter_mut().zip(before.into_iter().zip(after)) {
            s.hp = s.hp * a / b.max(1);
        }
    }

    /// Development only (the play script): the Mat's proposal round a jam on the first strand tile that has ground
    /// round it, as a jam standing a minute on a bred strain makes it (game::handoff).
    #[func]
    fn debug_proposal(&mut self) -> i64 {
        let Some(c) = self.city.as_mut() else { return 0 };
        let (w, h) = (c.sim.w, c.sim.h);
        let saved = c.jam.clone();
        let mut path = None;
        for i in w..w * (h - 1) {
            let hy_at = |j: usize| matches!(c.sim.hy[j], mat::sim::hy::CORD | mat::sim::hy::NODE);
            if !hy_at(i) || c.sim.sig[i] <= 0.0 || i % w == 0 || i % w + 1 == w || !(hy_at(i - 1) && hy_at(i + 1)) {
                continue;
            }
            c.jam.iter_mut().for_each(|j| *j = false);
            c.jam[i] = true;
            if let Some(p) = game::handoff::bypass(c) {
                path = Some(p);
                break;
            }
        }
        c.jam = saved;
        let n = path.as_ref().map_or(0, |p| p.len() as i64);
        if let Some(p) = path {
            self.colony.proposal = p;
            self.colony.proposal_confirmed = false;
        }
        n
    }

    /// Development only (the play script): set a built structure's HP, as a flood or a flamer would leave it.
    #[func]
    fn debug_hp(&mut self, structure: i64, hp: i64) -> bool {
        let Some(s) = self.colony.get_mut(structure as u32) else { return false };
        s.hp = hp.clamp(1, i32::MAX as i64) as i32;
        true
    }

    /// Development only (the screenshot script): send a stage ("patrol", "sweep", "cleansing") down a zone's grate now.
    #[func]
    fn debug_send(&mut self, stage: GString, zone: i64) -> bool {
        let defs = self.defs().clone();
        let (Some(t), Some(c)) = (self.threats.as_mut(), self.city.as_ref()) else { return false };
        let Some(k) = t.rules.stage.iter().position(|st| st.id == stage.to_string()) else { return false };
        if zone < 0 || zone as usize >= t.zones.len() {
            return false;
        }
        let notes = t.send(c, &self.colony, &defs, k, zone as usize);
        game::pressure::note(&mut self.log, c.tick, notes);
        true
    }

    /// This shift's Flood roll for a run on it (0-3 tiles before the Lungs), -1 when it is not a Flood shift.
    #[func]
    fn flood_roll(&self) -> i64 {
        self.threats.as_ref().and_then(|t| t.flood_roll()).map_or(-1, |r| r as i64)
    }

    /// The next shift's Flood roll when it is a Flood shift (a run's overtime runs into it), else -1.
    #[func]
    fn next_flood_roll(&self) -> i64 {
        let Some(t) = self.threats.as_ref() else { return -1 };
        let next = t.shift + 1;
        if t.is_flood_shift(next) { t.roll_for(next) as i64 } else { -1 }
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

    /// Which slimo each mover is (roster ids; the scale proof's are indices).
    #[func]
    fn agent_ids(&self) -> PackedInt32Array {
        self.city.as_ref().map_or_else(PackedInt32Array::new, |c| PackedInt32Array::from(c.ids.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// Slimo states: the roster's 0 at home, 3 going home, 4 going to work, 5 at work (the scale proof's 1 to market,
    /// 2 carrying to a Pit).
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

    /// The tier rank (0 Common … 4 Relic) of the item each packet carries, in packet order; the Street view's pulse.
    #[func]
    fn packet_tiers(&self) -> PackedByteArray {
        let Some(c) = &self.city else { return PackedByteArray::new() };
        let Some(st) = &self.stash else { return PackedByteArray::from(vec![0u8; c.pk_pos.len()].as_slice()) };
        let st = st.bind();
        let Some(t) = st.table.as_ref() else { return PackedByteArray::from(vec![0u8; c.pk_pos.len()].as_slice()) };
        let v: Vec<u8> = c
            .pk_cargo
            .iter()
            .map(|id| st.ledger.in_flight.iter().find(|s| s.id == *id).map_or(0, |s| game::cargo::TIERS.iter().position(|x| *x == t.def(s.item.def).tier).unwrap_or(0) as u8))
            .collect();
        PackedByteArray::from(v.as_slice())
    }

    /// The Stack view, one pixel a tile, RGB (render::stack_view: the Mat dim, traffic as light, a jam — a full tile
    /// with cargo waiting to enter it — Chalk white, consumers Acid, markets Hot Pink).
    #[func]
    fn stack_image(&self) -> PackedByteArray {
        self.city.as_ref().map_or_else(PackedByteArray::new, |c| {
            let mut px = render::stack_view(c, 1).px;
            // a stratum not yet reached is Void, its slab row shown (Full Design v3.19)
            let first = c.sim.ceiling(self.city_stratum() as u8).saturating_sub(1);
            for v in px.iter_mut().take(first * c.sim.w * 3).collect::<Vec<_>>() {
                *v = 0x07;
            }
            PackedByteArray::from(px.as_slice())
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
            // the Pits: the Heart first, then every built Pit (the scale proof: the Heart and the Relays)
            let pits: Vec<usize> = if c.synthetic() {
                c.pits.clone()
            } else {
                let defs = self.defs.clone().unwrap_or_else(colony_bridge::structure_file);
                self.colony.pits(&defs).into_iter().filter_map(|sid| self.colony.cargo_cell(&defs, &c.sim, sid)).collect()
            };
            d.set("markets", &to_arr(&c.markets));
            d.set("markets_reached", &PackedByteArray::from(c.market_cells.iter().map(|m| m.is_some() as u8).collect::<Vec<u8>>().as_slice()));
            // each market by its grate's number and the faction whose street it is (Economy)
            let names: Vec<GString> = (0..c.markets.len())
                .map(|k| {
                    let who = self.economy.as_ref().and_then(|e| e.faction_at(c.markets[k]).map(|f| e.name(f).split(' ').next().unwrap_or("").to_uppercase())).unwrap_or_default();
                    GString::from(format!("GRATE {} {who}", c.market_grate(k)).trim_end())
                })
                .collect();
            d.set("market_names", &PackedStringArray::from(names.as_slice()));
            d.set("pits", &to_arr(&pits));
            d.set("pods", &to_arr(&c.crowd.as_ref().map_or(Vec::new(), |cr| cr.pods.clone())));
        }
        d
    }

    // ------------------------------------------------------------ player verbs
    /// Bait from the Stash (Full Design v3.12, The steering tools): the cheapest Common Chrome in any room is
    /// smeared onto the 3 × 3 around the tile, its nutrient spread over the nine (game::store::take_bait). "" or why
    /// not.
    #[func]
    fn bait_from_stash(&mut self, x: i64, y: i64) -> GString {
        let Some(c) = self.city.as_mut() else { return GString::from("no city") };
        if !Self::inside(c, x, y) {
            return GString::from("off the map");
        }
        let Some(st) = self.stash.as_mut() else { return GString::from("no Stash") };
        let mut st = st.bind_mut();
        st.ensure();
        let t = st.table.clone().unwrap();
        let Some(n) = game::store::take_bait(st.stash.as_mut().unwrap(), &t) else { return GString::from("bait takes a Common Chrome from the Stash: none") };
        c.sim.bait(x as usize, y as usize, n / 9.0);
        drop(st);
        self.decide(format!("bait at {x},{y}"));
        GString::new()
    }

    /// The colony strain's gene value (0..1) by index (mat::genome constants); the Run reads Lume (4).
    #[func]
    fn gene(&self, index: i64) -> f64 {
        self.city.as_ref().map_or(0.5, |c| c.sim.genome.0.get(index as usize).copied().unwrap_or(0.5) as f64)
    }

    /// All ten genes, 0..1.
    #[func]
    fn genes(&self) -> PackedFloat32Array {
        self.city.as_ref().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.genome.0.as_slice()))
    }

    /// Stock sitting in a Pit (nutrient units), by index among the city's Pits (0 is the Heart).
    #[func]
    fn pit_stock(&mut self, pit: i64) -> f64 {
        let cells = self.sites().get("pits").and_then(|v| v.try_to::<PackedInt32Array>().ok()).unwrap_or_default();
        let Some(c) = self.city.as_ref() else { return 0.0 };
        let Some(&i) = cells.as_slice().get(pit.max(0) as usize) else { return 0.0 };
        let (i, w) = (i as usize, c.sim.w);
        c.sim.pits.get(&(i % w, i / w)).copied().unwrap_or(0.0) as f64
    }

    /// Cells of growth one unit of nutrient buys at the current genome (the Stash's "feed now").
    #[func]
    fn cells_per_nutrient(&self) -> f64 {
        self.city.as_ref().map_or(4.0, |c| c.sim.cells_per_nutrient() as f64)
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
            d.set("riding", c.pk_pos.len() as i64);
            d.set("waiting", (cs.packets - c.pk_pos.len()) as i64);
            d.set("step_us", self.last_step_us);
            let _ = (hy::EMPTY, sub::ROCK, state::HOME);
        }
        d
    }

    /// A decision for the city's log (Full Design v3.12, The city's log): every verb that changes the city says so.
    pub(crate) fn decide(&mut self, what: String) {
        let t = self.city.as_ref().map_or(0, |c| c.tick);
        self.log.push(t, game::Class::Decision, what);
    }

    fn inside(c: &City, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < c.sim.w && (y as usize) < c.sim.h
    }

    // ------------------------------------------------------------ the colony (Full Design v3.9, Colony mode)

    fn defs(&mut self) -> &colony::StructureFile {
        if self.defs.is_none() {
            self.defs = Some(colony_bridge::structure_file());
        }
        self.defs.as_ref().unwrap()
    }

    /// Found the city on the grown Mat: the Heart, a Route Board, a tier-1 Bench, a Pit on every Relay. Once.
    #[func]
    fn found_city(&mut self) -> bool {
        if !self.colony.structures.is_empty() {
            return true;
        }
        let defs = self.defs().clone();
        match &self.city {
            Some(c) => match self.colony.found(&defs, &c.sim) {
                Ok(()) => {
                    // New Game+: the profile's laws unlocked and its Scrapdex kept
                    if let Some(p) = self.carry.take() {
                        game::meta::carry(&p, &mut self.colony);
                    }
                    if let (Some(t), Some(c)) = (self.threats.as_mut(), self.city.as_mut()) {
                        let notes = t.start_shift(c, &mut self.colony, &defs, 1, t.is_flood_shift(1));
                        game::pressure::note(&mut self.log, c.tick, notes);
                    }
                    true
                }
                Err(e) => {
                    godot_error!("found_city: {e}");
                    false
                }
            },
            None => false,
        }
    }

    /// The 24 structure defs for the Build menu.
    #[func]
    fn structure_defs(&mut self) -> VarArray {
        let defs = self.defs().clone();
        let mut a = colony_bridge::defs_array(&defs);
        // what the tree and the organs ask of each (Full Design v3.18): the node it waits on, the organ it stands on
        let f = progress::StrainFile::shipped();
        for (k, d) in defs.structure.iter().enumerate() {
            let mut e: VarDictionary = a.at(k).to();
            e.set("locked", self.colony.progress.tech.structure_locked(&d.id).map_or("", |n| n.name.as_str()));
            e.set("organ", f.organ.iter().find(|o| o.id == d.organ).map_or("", |o| o.name.as_str()));
            e.set("free_on", f.organ.iter().find(|o| o.id == d.free_on).map_or("", |o| o.name.as_str()));
            a.set(k, &e.to_variant());
        }
        a
    }

    /// Every placed structure, ghosts included.
    #[func]
    fn structures(&mut self) -> VarArray {
        let defs = self.defs().clone();
        colony_bridge::structures_array(&self.colony, &defs)
    }

    /// "" when the def may stand here, else why not.
    #[func]
    fn can_place(&mut self, def_id: GString, x: i64, y: i64) -> GString {
        let defs = self.defs().clone();
        let Some(c) = &self.city else { return GString::from("no city") };
        if x < 0 || y < 0 {
            return GString::from("off the map");
        }
        match self.colony.can_place(&defs, &c.sim, &def_id.to_string(), x as u16, y as u16) {
            Ok(()) => GString::new(),
            Err(e) => GString::from(e.to_string().as_str()),
        }
    }

    /// Place a ghost; -1 when it cannot stand.
    #[func]
    fn place(&mut self, def_id: GString, x: i64, y: i64) -> i64 {
        let defs = self.defs().clone();
        let Some(c) = &self.city else { return -1 };
        if x < 0 || y < 0 {
            return -1;
        }
        let id = self.colony.place(&defs, &c.sim, &def_id.to_string(), x as u16, y as u16).map(|id| id as i64).unwrap_or(-1);
        if id >= 0 {
            self.decide(format!("{} placed at {x},{y}", defs.get(&def_id.to_string()).map_or("?", |d| d.name.as_str())));
        }
        id
    }

    /// Cancel an unpaid ghost (a paid one is cut and welded already).
    #[func]
    fn cancel_ghost(&mut self, id: i64) -> bool {
        let defs = self.defs().clone();
        if self.colony.get(id as u32).is_some_and(|s| s.paid) {
            return false;
        }
        self.colony.cancel(&defs, id as u32).is_none() && self.colony.get(id as u32).is_none()
    }

    /// Start an upgrade in place; "" or why not. `population` sets the city tier.
    #[func]
    fn upgrade(&mut self, id: i64, population: i64) -> GString {
        let defs = self.defs().clone();
        match self.colony.upgrade(&defs, id as u32, colony::Colony::city_tier(population.max(0) as usize)) {
            Ok(_) => {
                let name = self.colony.get(id as u32).map_or("?".into(), |s| defs.structure[s.def].name.clone());
                self.decide(format!("{name} to be upgraded"));
                GString::new()
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    #[func]
    fn city_tier(&self, population: i64) -> i64 {
        colony::Colony::city_tier(population.max(0) as usize) as i64
    }

    #[func]
    fn city_tier_name(&self, population: i64) -> GString {
        GString::from(colony::CITY_TIER_NAMES[(colony::Colony::city_tier(population.max(0) as usize) as usize - 1).min(3)])
    }

    /// Beds in the city and beds taken.
    #[func]
    fn beds(&mut self) -> PackedInt32Array {
        let defs = self.defs().clone();
        let (a, b) = self.colony.bed_count(&defs);
        PackedInt32Array::from([a, b].as_slice())
    }

    /// slimo id -> structure id of its bed.
    #[func]
    fn homes(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        for (slimo, home) in &self.colony.homes {
            d.set(*slimo as i64, *home as i64);
        }
        d
    }

    #[func]
    fn zones(&self) -> VarArray {
        colony_bridge::zones_array(&self.colony)
    }

    /// Paint a zone: kind "stockpile" | "hab" | "quarantine" | "kill", cells as indices; -1 when no cell took.
    #[func]
    fn paint_zone(&mut self, kind: GString, cells: PackedInt32Array, categories: PackedStringArray, min_tier: i64, min_condition: i64, anchor: i64) -> i64 {
        let Some(k) = colony_bridge::parse_zone(&kind.to_string()) else { return -1 };
        let Some(c) = &self.city else { return -1 };
        let cells: Vec<u32> = cells.as_slice().iter().filter(|x| **x >= 0).map(|x| *x as u32).collect();
        let id = self.colony.paint_zone(&c.sim, k, &cells, colony_bridge::filter_from(&categories, min_tier, min_condition), if anchor >= 0 { Some(anchor as u32) } else { None }).map(|id| id as i64).unwrap_or(-1);
        if id >= 0 {
            self.decide(format!("{} zone {id} painted", kind));
        }
        id
    }

    #[func]
    fn erase_zone(&mut self, id: i64) -> bool {
        let ok = self.colony.erase_zone(id as u32);
        if ok {
            self.decide(format!("zone {id} erased"));
            self.republish();
        }
        ok
    }

    /// Whether a cell may hold a zone.
    #[func]
    fn zone_cell_ok(&self, cell: i64) -> bool {
        self.city.as_ref().is_some_and(|c| cell >= 0 && colony::Colony::zone_cell_ok(&c.sim, cell as usize))
    }

    #[func]
    fn districts(&mut self, people: Gd<RosePeople>) -> VarArray {
        let defs = self.defs().clone();
        let p = people.bind();
        colony_bridge::districts_array(&self.colony, Some(&p), Some(&defs), self.city.as_ref().map(|c| &c.sim))
    }

    // ------------------------------------------------------------ the Charter (Full Design v3.10)

    fn law_file(&mut self) -> &colony::LawFile {
        if self.laws.is_none() {
            self.laws = Some(colony::LawFile::from_toml(&data_text("laws.toml", colony::charter::LAWS_TOML)).unwrap_or_else(|_| colony::LawFile::shipped()));
        }
        self.laws.as_ref().unwrap()
    }

    /// The eight laws and their state on the Charter.
    #[func]
    fn laws(&mut self) -> VarArray {
        let laws = self.law_file().clone();
        if self.colony.heart_dist.is_empty() {
            if let Some(c) = &self.city {
                self.colony.refresh_heart_dist(&c.sim);
            }
        }
        match &self.city {
            Some(c) => colony_bridge::laws_array(&self.colony, &laws, &c.sim),
            None => VarArray::new(),
        }
    }

    #[func]
    fn authority(&self) -> i64 {
        self.colony.charter.authority as i64
    }

    /// Ivan's Authority state 1-4 (from the Authority earned over the run).
    #[func]
    fn ivan_state(&self) -> i64 {
        self.colony.charter.ivan_state() as i64
    }

    /// Cords the wave walks a shift (16 + 32 × Conductance).
    #[func]
    fn wave_speed(&self) -> i64 {
        self.city.as_ref().map(|c| colony::charter::wave_speed(&c.sim) as i64).unwrap_or(0)
    }

    /// The board at the Heart: every signing, repeal and refusal with its shift.
    #[func]
    fn charter_history(&self) -> PackedStringArray {
        PackedStringArray::from(self.colony.charter.history.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice())
    }

    /// Sign a law; "" or why not.
    #[func]
    fn sign_law(&mut self, id: GString) -> GString {
        let defs = self.defs().clone();
        let laws = self.law_file().clone();
        let Some(def) = laws.get(&id.to_string()).cloned() else { return GString::from("no such law") };
        let Some(c) = &self.city else { return GString::from("no city") };
        match self.colony.sign(&defs, &c.sim, &def) {
            Ok(()) => {
                self.decide(format!("{} signed", def.name));
                self.log.bark("law", &[("name", &def.name)]);
                GString::new()
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// Repeal a law; "" or why not.
    #[func]
    fn repeal_law(&mut self, id: GString) -> GString {
        let Some(c) = &self.city else { return GString::from("no city") };
        match self.colony.repeal(&c.sim, &id.to_string()) {
            Ok(()) => {
                self.decide(format!("{id} repealed"));
                GString::new()
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// Refuse a district's petition: "refused", "dissolved" or why not.
    #[func]
    fn refuse_petition(&mut self, district: i64) -> GString {
        match self.colony.refuse_petition(district as u32) {
            Ok(true) => {
                self.decide(format!("district {district}'s petition refused; it dissolved"));
                GString::from("dissolved")
            }
            Ok(false) => {
                self.decide(format!("district {district}'s petition refused"));
                GString::from("refused")
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// Dissolve a D5 into the Heart's Pit; the line for the report, or why not.
    #[func]
    fn dissolve(&mut self, people: Gd<RosePeople>, slimo: i64) -> GString {
        let mut people = people;
        let Some(c) = self.city.as_mut() else { return GString::from("no city") };
        let mut p = people.bind_mut();
        let name = p.roster.get(slimo as u32).map(|r| r.full_name()).unwrap_or_default();
        match self.colony.dissolve(&mut p.roster, &mut c.sim, slimo as u32) {
            Ok(line) => {
                self.log.bark("d5_fed", &[("name", &name)]);
                GString::from(line.as_str())
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// A district's job priority (the foreman's, over its members' where higher).
    #[func]
    fn set_district_priority(&mut self, district: i64, job: i64, value: i64) {
        if let Some(d) = self.colony.districts.iter_mut().find(|d| d.id == district as u32) {
            if (0..9).contains(&job) {
                d.priorities[job as usize] = value.clamp(0, 4) as u8;
            }
        }
    }

    /// The last shift's assignments: [{slimo, job, structure}].
    #[func]
    fn assignments(&mut self) -> VarArray {
        let defs = self.defs().clone();
        let mut out = VarArray::new();
        for (slimo, job, sid) in &self.last_jobs {
            let mut e = VarDictionary::new();
            e.set("slimo", *slimo as i64);
            e.set("job", job.name());
            e.set("structure", *sid as i64);
            e.set("structure_name", self.colony.get(*sid).map(|s| defs.structure[s.def].name.as_str()).unwrap_or(""));
            out.push(&e.to_variant());
        }
        out
    }

    /// The shift, whole (game::end_shift): the colony works and pays, the Stash's side, the cargo, the stockpiles,
    /// the roster's records; then the movers take the next shift's plan. `day` picks the shift of the day (Dim / Dark
    /// / Flood). The report is what landed since the last shift end and what just left.
    #[func]
    fn colony_end_shift(&mut self, people: Gd<RosePeople>, stash: Gd<RoseStash>, day: i64) -> VarDictionary {
        let defs = self.defs().clone();
        let mut people = people;
        let mut stash = stash;
        if self.stash.is_none() {
            self.stash = Some(stash.clone());
        }
        let Some(city) = self.city.as_mut() else { return VarDictionary::new() };
        let shift = {
            let mut p = people.bind_mut();
            p.ensure();
            let p = &mut *p;
            let traits = p.traits.clone().unwrap();
            let names = p.names.clone().unwrap();
            let mut st = stash.bind_mut();
            st.ensure();
            let st = &mut *st;
            // the route loops run first, as a run comes home before the shift ends (Full Design v3.18)
            let loop_lines = {
                let mut folk = game::Folk { roster: &mut p.roster, traits: &traits, names: &names, died: &mut p.died_this_shift };
                let city_seed = self.economy.as_ref().map_or(0, |e| e.s.seed);
                game::loops::play(&mut self.colony, &mut folk, st.stash.as_mut().unwrap(), st.table.as_ref().unwrap(), self.threats.as_ref(), &city.sim, city_seed, day.max(1) as u32, city.tick, &mut self.log)
            };
            let (flush_mult, gain_mult) = game::campaign::mults(&self.campaign, &city.sim);
            if let Some(t) = self.threats.as_mut() {
                t.campaign.gain_mult = gain_mult;
            }
            let town = game::Town { city, colony: &mut self.colony, defs: &defs, flush_mult };
            let store = game::Store { stash: st.stash.as_mut().unwrap(), ledger: &mut st.ledger, bench: &mut st.bench, table: st.table.as_ref().unwrap(), recipes: st.recipes.as_ref().unwrap() };
            let folk = game::Folk { roster: &mut p.roster, traits: &traits, names: &names, died: &mut p.died_this_shift };
            let mut shift = game::end_shift(town, store, folk, day.max(1) as u32, &mut self.log);
            shift.people_lines.splice(0..0, loop_lines);
            // the breach and the city's stratum before the threats choose what comes (Full Design v3.19)
            if let Some(top) = game::breach::shift_end(city, &self.colony, &defs, self.threats.as_mut(), &mut self.log) {
                let name = logistics::StrataFile::shipped().get(top).map(|d| d.name.clone()).unwrap_or_default();
                shift.people_lines.push(format!("the Mat reached {name} (S{top})"));
                self.strata = *logistics::board_strata(top, city.sim.strata).end();
                self.raiders = game::trade::raiders_walk(self.strata);
            }
            // the threats' side, the stalls' (Full Design v3.15, Economy), then the next shift's water (v3.14, Pressure)
            if let Some(t) = self.threats.as_mut() {
                let kills = std::mem::take(&mut self.run_kills);
                game::pressure::end(t, city, &mut self.colony, &defs, &mut p.roster, st.stash.as_mut().unwrap(), st.table.as_ref().unwrap(), &mut self.log, &mut p.died_this_shift, &shift.outcome, day.max(1) as u32, kills);
            }
            let mut laid = Vec::new();
            if let Some(e) = self.economy.as_mut() {
                let lib = run::ChunkLib::from_toml(run::CHUNKS_TOML).expect("chunks.toml");
                let board = game::trade::Board { lib: &lib, city_seed: e.s.seed, strata: self.strata.max(1) };
                let (lines, l) = game::trade::end_shift(e, &mut st.ledger, st.stash.as_mut().unwrap(), st.table.as_ref().unwrap(), city, &mut self.colony, &mut p.roster, &traits, &names, self.threats.as_mut(), Some(&board), &shift.closed.report.to_faction, day.max(1) as u32, self.raiders, &mut self.log);
                shift.stash_lines.extend(lines);
                laid = l;
            }
            // the campaign reads the shift: a chapter's goal, the accord, the Great Cleansing, the ending (v3.19)
            if let (Some(t), Some(e)) = (self.threats.as_mut(), self.economy.as_mut()) {
                let guards = self.posts.guards.len();
                let mut parts = game::campaign::Parts { city, colony: &mut self.colony, defs: &defs, threats: t, economy: e, roster: &mut p.roster, died: &mut p.died_this_shift, stash: st.stash.as_mut().unwrap(), table: st.table.as_ref().unwrap(), log: &mut self.log, guards };
                let story = game::campaign::shift_end(&mut self.campaign, &mut parts, day.max(1) as u32);
                shift.people_lines.extend(story);
            }
            if let Some(t) = self.threats.as_mut() {
                let next = day.max(1) as u32 + 1;
                let notes = t.start_shift(city, &mut self.colony, &defs, next, t.is_flood_shift(next));
                game::pressure::note(&mut self.log, city.tick, notes);
                if t.flood.level > 0 {
                    self.log.bark("flood", &[]);
                }
            }
            game::ivan::shift_end(&mut self.campaign, city);
            (shift, laid)
        };
        let (shift, laid) = shift;
        let mut report = stash.bind().closed_dict(&shift.closed);
        report.set("stash_lines", &PackedStringArray::from(shift.stash_lines.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
        self.last_jobs = shift.outcome.assignments.clone();
        let mut out = colony_bridge::outcome_dict(&shift.outcome, &shift.people_lines, report, &defs, &self.colony);
        // recover links begun at this shift's end: their bodies, for the Board's corpses
        out.set("laid", &market_bridge::laid_array(&laid));
        self.last_outcome = Some(shift.outcome);
        self.refresh_movers(people, day + 1);
        out
    }

    /// The roster on the map for shift `day`: everyone in the city at their bed, or walking to the structure the
    /// colony's plan for that shift gives them; a Runner on a run is off the map.
    #[func]
    fn refresh_movers(&mut self, people: Gd<RosePeople>, day: i64) {
        let defs = self.defs().clone();
        let Some(city) = self.city.as_mut() else { return };
        let mut people = people;
        let mut p = people.bind_mut();
        p.ensure();
        let traits = p.traits.clone().unwrap();
        self.plan = self.colony.assign_jobs(&defs, &p.roster, &traits, game::shift_of_day(day.max(1) as u32));
        self.posts = threat::Posts::from_plan(&self.plan, &p.roster);
        game::refresh_movers(city, &self.colony, &defs, &p.roster, &traits, day.max(1) as u32);
        // the surface's side of the shift's start: the day's clock, the offers, the stalls' books (Economy)
        if let (Some(e), Some(st)) = (self.economy.as_mut(), self.stash.as_mut()) {
            let names = p.names.clone().unwrap();
            let mut st = st.bind_mut();
            st.ensure();
            game::trade::start_shift(e, &mut st.ledger, city, &self.colony, &defs, &p.roster, &names, &self.plan, self.threats.as_ref(), day.max(1) as u32, self.raiders, &mut self.log);
        }
    }

    // ------------------------------------------------------------ quotas (Full Design v3.11, Cargo: Quotas)

    /// The Pit or Still whose footprint holds this tile, or -1.
    #[func]
    fn consumer_at(&mut self, x: i64, y: i64) -> i64 {
        let defs = self.defs().clone();
        for s in &self.colony.structures {
            let d = &defs.structure[s.def];
            if !matches!(d.id.as_str(), "heart" | "pit" | "still") || !(s.built || d.id == "heart") {
                continue;
            }
            if x >= s.x as i64 && x < s.x as i64 + d.footprint[0] as i64 && y >= s.y as i64 && y < s.y as i64 + d.footprint[1] as i64 {
                return s.id as i64;
            }
        }
        -1
    }

    /// A Pit's or Still's quota for its card: {structure, name, kind "pit"|"still", target, ceiling, ceiling_name,
    /// from "yours"|"the foreman's"|"its slots"|"off", stock, riding, cap, reached}; empty for anything else.
    #[func]
    fn quota(&mut self, structure: i64) -> VarDictionary {
        let defs = self.defs().clone();
        let (Some(c), Some(st)) = (self.city.as_ref(), self.stash.as_ref()) else { return VarDictionary::new() };
        let mut st = st.clone();
        st.bind_mut().ensure();
        let st = st.bind();
        cargo_bridge::quota_view(&self.colony, &defs, c, &st, structure as u32).unwrap_or_default()
    }

    /// Set a Pit's or Still's quota (a Pit's target in nutrient, a Still's in Parts; ceiling 0 Common … 4 Relic).
    #[func]
    fn set_quota(&mut self, structure: i64, target: f64, ceiling: i64) -> bool {
        let defs = self.defs().clone();
        let ok = self.colony.set_quota(&defs, structure as u32, Some(colony::QuotaSet { target: target.max(0.0) as f32, ceiling: ceiling.clamp(0, 4) as u8 }));
        self.republish();
        if ok {
            self.decide(format!("quota at structure {structure}: keep {target:.1}"));
        }
        ok
    }

    /// Back to the default: off, the foreman's, or a Still's slots.
    #[func]
    fn clear_quota(&mut self, structure: i64) -> bool {
        let defs = self.defs().clone();
        let ok = self.colony.set_quota(&defs, structure as u32, None);
        self.republish();
        ok
    }

    fn republish(&mut self) {
        let defs = self.defs().clone();
        if let (Some(c), Some(st)) = (self.city.as_mut(), self.stash.as_ref()) {
            let mut st = st.clone();
            st.bind_mut().ensure();
            let st = st.bind();
            let consumers = game::cargo::consumers(&self.colony, &defs, c, st.stash.as_ref().unwrap(), &st.ledger, st.table.as_ref().unwrap());
            c.set_consumers(consumers);
        }
    }
}

// ======================================================================= items and the Stash
use godot::classes::ProjectSettings;
use items::{ItemInstance, ItemTable, PriceContext};
use inventory::Stash;

const ITEMS_TOML: &str = include_str!("../../../data/items.toml");

fn table() -> ItemTable {
    // data/items.toml is compiled in so a built game carries its table; in debug the file beside the project wins
    let path = ProjectSettings::singleton().globalize_path("res://../data/items.toml").to_string();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| ITEMS_TOML.to_string());
    ItemTable::from_toml(&text).unwrap_or_else(|_| ItemTable::from_toml(ITEMS_TOML).expect("built-in item table"))
}

fn def_dict(t: &ItemTable, idx: usize) -> VarDictionary {
    let d = &t.defs[idx];
    let mut out = VarDictionary::new();
    out.set("index", idx as i64);
    out.set("id", d.id.as_str());
    out.set("name", d.name.as_str());
    out.set("category", format!("{:?}", d.category).to_lowercase().as_str());
    out.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
    out.set("w", d.size[0] as i64);
    out.set("h", d.size[1] as i64);
    out.set("weight", d.weight as f64);
    out.set("value", d.value as i64);
    out.set("feed", d.feed as f64);
    out.set("codex", d.codex.as_str());
    let tags = PackedStringArray::from(d.tags.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice());
    out.set("tags", &tags);
    // gear: slot ("hand", "rig", "head", ...), kind ("melee", "ranged", "armour", "tool", "implant"), the numbers
    if let Some(g) = &d.gear {
        out.set("gear_slot", g.slot.as_str());
        out.set("gear_kind", g.kind.as_str());
        out.set("dmg", g.dmg as i64);
        out.set("armour", g.armour as i64);
        out.set("light", g.light as i64);
        out.set("gear_noise", g.noise as i64);
    }
    out
}

/// The item table, read-only.
#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseItems {
    base: Base<RefCounted>,
    table: Option<ItemTable>,
}

#[godot_api]
impl RoseItems {
    fn t(&mut self) -> &ItemTable {
        if self.table.is_none() {
            self.table = Some(table());
        }
        self.table.as_ref().unwrap()
    }

    #[func]
    fn count(&mut self) -> i64 {
        self.t().len() as i64
    }

    #[func]
    fn get(&mut self, index: i64) -> VarDictionary {
        let t = self.t();
        if index < 0 || index as usize >= t.len() {
            return VarDictionary::new();
        }
        def_dict(t, index as usize)
    }

    #[func]
    fn find(&mut self, id: GString) -> i64 {
        self.t().id_of(&id.to_string()).map_or(-1, |i| i.0 as i64)
    }

    /// Indices of every item in a category ("chrome", "gear", ...).
    #[func]
    fn by_category(&mut self, category: GString) -> PackedInt32Array {
        let t = self.t();
        let want = category.to_string();
        let v: Vec<i32> = (0..t.len()).filter(|&i| format!("{:?}", t.defs[i].category).to_lowercase() == want).map(|i| i as i32).collect();
        PackedInt32Array::from(v.as_slice())
    }
}

/// A Stash backed by the inventory crate; the Stash screen is a view of this.
#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseStash {
    base: Base<RefCounted>,
    pub(crate) table: Option<ItemTable>,
    pub(crate) stash: Option<Stash>,
    pub(crate) ledger: inventory::Ledger,
    recipes: Option<bench::RecipeFile>,
    bench: bench::BenchQueue,
}

impl RoseStash {
    /// One item as the Stash screen and the pickers read it (no placement: the caller adds x, y, w, h).
    pub(crate) fn item_dict(&self, t: &ItemTable, h: u32, item: &ItemInstance) -> VarDictionary {
        let d = t.def(item.def);
        let mut e = VarDictionary::new();
        e.set("handle", h as i64);
        e.set("index", item.def.0 as i64);
        e.set("id", d.id.as_str());
        e.set("name", d.name.as_str());
        e.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
        e.set("condition", item.condition as i64);
        e.set("identified", item.ident == items::Ident::Full);
        e.set("pinned", item.pinned);
        // a recover contract's item (Economy): sent from the Market screen, it counts where it lands
        e.set("bound", item.bound.map_or(-1, |c| c as i64));
        e.set("sell", items::price(d, item, PriceContext::default()) as i64);
        e.set("feed", items::feed_value(d, item).0 as f64);
        e.set("category", format!("{:?}", d.category).to_lowercase().as_str());
        e.set("weight", d.weight as f64);
        e.set("found", item.found_in_run);
        e.set("spored", item.spored);
        if let Some(c) = &item.crafted {
            e.set("crafted_by", format!("{} on day {}", c.by, c.day).as_str());
            e.set("crafted_from", &PackedStringArray::from(c.parts.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
        }
        if let Some(st) = item.gear_stats(d) {
            let mut sd = VarDictionary::new();
            sd.set("dmg", st.dmg as i64);
            sd.set("armour", st.armour as i64);
            sd.set("noise", st.noise as i64);
            sd.set("light", st.light as i64);
            sd.set("mag", st.mag as i64);
            e.set("stats", &sd);
        }
        e.set("tags", &PackedStringArray::from(d.tags.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
        e.set("codex", d.codex.as_str());
        e
    }

    /// The Stash's side of a closed shift as the report the screens read.
    pub(crate) fn closed_dict(&self, c: &game::store::Closed) -> VarDictionary {
        let mut d = Self::report_dict(&c.report);
        d.set("crafted", c.placed as i64);
        d.set("crafted_names", &PackedStringArray::from(c.crafted.iter().map(|n| GString::from(n.as_str())).collect::<Vec<_>>().as_slice()));
        d.set("waiting", c.waiting as i64);
        d.set("eddies", self.ledger.eddies as i64);
        d.set("held", self.stash.as_ref().map_or(0, |s| s.floor.len()) as i64);
        d
    }

    fn report_dict(r: &inventory::ShiftReport) -> VarDictionary {
        let mut d = VarDictionary::new();
        d.set("sold", r.sold as i64);
        d.set("earned", r.earned as i64);
        d.set("fed", r.fed as i64);
        d.set("stored", r.stored as i64);
        d.set("home", r.home as i64);
        d.set("spilled", r.spilled as i64);
        d.set("cut", r.cut as i64);
        d.set("left", r.left as i64);
        let mut dl = VarArray::new();
        for de in &r.deliveries {
            let mut e = VarDictionary::new();
            e.set("pit", de.pit as i64);
            e.set("nutrient", de.nutrient as f64);
            e.set("drift", &PackedFloat32Array::from(de.drift.as_slice()));
            dl.push(&e.to_variant());
        }
        d.set("deliveries", &dl);
        d
    }
}

#[godot_api]
impl RoseStash {
    fn ensure(&mut self) {
        if self.table.is_none() {
            self.table = Some(table());
        }
        if self.stash.is_none() {
            self.stash = Some(Stash::new());
        }
        if self.recipes.is_none() {
            self.recipes = Some(bench::RecipeFile::from_toml(&data_text("recipes.toml", bench::RECIPES_TOML)).unwrap_or_else(|_| bench::RecipeFile::shipped()));
        }
        if self.bench.tier_open == 0 {
            self.bench = bench::BenchQueue::new();
        }
    }

    // ------------------------------------------------------------ the Bench

    /// Every recipe: [{id, name, tier, product, product_name, count, shifts, open, slots: [{tag, drives}]}].
    #[func]
    fn recipes(&mut self) -> VarArray {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let f = self.recipes.as_ref().unwrap();
        let mut out = VarArray::new();
        for r in &f.recipe {
            let mut d = VarDictionary::new();
            d.set("id", r.id.as_str());
            d.set("name", r.name.as_str());
            d.set("tier", r.tier as i64);
            d.set("product", r.product.as_str());
            let pname = if let Some(job) = r.product.strip_prefix('@') {
                match job {
                    "repair" => "the gear, +40 condition".to_string(),
                    "identify" => "the item, identified".to_string(),
                    _ => "the gear, Spored".to_string(),
                }
            } else {
                t.get(&r.product).map(|d| d.name.clone()).unwrap_or_default()
            };
            d.set("product_name", pname.as_str());
            d.set("count", r.count as i64);
            d.set("shifts", f.shifts(r) as i64);
            d.set("open", self.bench.is_open(r));
            let mut slots = VarArray::new();
            for sl in &r.slots {
                let mut e = VarDictionary::new();
                e.set("tag", sl.tag.as_str());
                e.set("drives", sl.drives.clone().unwrap_or_default().as_str());
                slots.push(&e.to_variant());
            }
            d.set("slots", &slots);
            out.push(&d.to_variant());
        }
        out
    }

    /// Grid handles that fill a slot with this tag, best tier first.
    #[func]
    fn candidates(&mut self, tag: GString) -> PackedInt32Array {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let slot = bench::Slot { tag: tag.to_string(), drives: None };
        let mut v: Vec<(i32, u32)> = self
            .stash
            .as_ref()
            .unwrap()
            .grid
            .placed()
            .into_iter()
            .filter(|(_, p)| bench::accepts(&slot, t.def(p.item.def), &p.item))
            .map(|(h, p)| (h as i32, t.def(p.item.def).tier as u32))
            .collect();
        v.sort_by_key(|(h, tier)| (std::cmp::Reverse(*tier), *h));
        PackedInt32Array::from(v.iter().map(|(h, _)| *h).collect::<Vec<i32>>().as_slice())
    }

    fn parts_for(&self, handles: &PackedInt32Array) -> Option<Vec<ItemInstance>> {
        let stash = self.stash.as_ref()?;
        let mut parts = Vec::new();
        for h in handles.as_slice() {
            parts.push(stash.get_any(*h as u32)?.clone());
        }
        Some(parts)
    }

    fn outcome_dict(&self, o: &bench::Outcome) -> VarDictionary {
        let t = self.table.as_ref().unwrap();
        let d = t.def(o.product.def);
        let mut out = VarDictionary::new();
        out.set("ok", true);
        out.set("product_id", d.id.as_str());
        out.set("product_name", d.name.as_str());
        out.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
        out.set("condition", o.product.condition as i64);
        out.set("count", o.count as i64);
        out.set("shifts", o.shifts as i64);
        out.set("spored", o.product.spored);
        out.set("identified", o.product.ident == items::Ident::Full);
        let stat_dict = |g: &items::GearStats| {
            let mut sd = VarDictionary::new();
            sd.set("dmg", g.dmg as i64);
            sd.set("armour", g.armour as i64);
            sd.set("noise", g.noise as i64);
            sd.set("light", g.light as i64);
            sd.set("mag", g.mag as i64);
            sd
        };
        if let Some(b) = &o.base {
            out.set("base", &stat_dict(b));
        }
        if let Some(st) = &o.stats {
            out.set("stats", &stat_dict(st));
        }
        if let Some(c) = &o.product.crafted {
            out.set("by", c.by.as_str());
            out.set("parts", &PackedStringArray::from(c.parts.iter().map(|p| GString::from(p.as_str())).collect::<Vec<_>>().as_slice()));
        }
        out
    }

    /// The outcome of a recipe on these grid handles (one per slot), without crafting: {ok, error?, product_id,
    /// product_name, tier, condition, count, shifts, base {dmg, armour, noise, light, mag}, stats {...}, by, parts}.
    #[func]
    fn preview(&mut self, recipe: GString, handles: PackedInt32Array, by: GString, day: i64) -> VarDictionary {
        self.ensure();
        let mut err = VarDictionary::new();
        err.set("ok", false);
        let Some(r) = self.recipes.as_ref().unwrap().get(&recipe.to_string()).cloned() else {
            err.set("error", "no such recipe");
            return err;
        };
        let Some(parts) = self.parts_for(&handles) else {
            err.set("error", "a part is missing from the Stash");
            return err;
        };
        let t = self.table.as_ref().unwrap();
        match bench::outcome(t, self.recipes.as_ref().unwrap(), &r, &parts, &by.to_string(), day.max(0) as u32) {
            Ok(o) => self.outcome_dict(&o),
            Err(e) => {
                let msg = match e {
                    bench::CraftError::WrongPartCount { want, got } => format!("{want} slots, {got} parts"),
                    bench::CraftError::SlotRefuses { slot, tag, item } => format!("slot {} ({}) refuses {}", slot + 1, tag, t.get(&item).map(|d| d.name.as_str()).unwrap_or(&item)),
                    bench::CraftError::UnknownProduct(p) => format!("no such product {p}"),
                    bench::CraftError::IdentifyNeedsUnidentified => "that item is already identified".to_string(),
                };
                err.set("error", msg.as_str());
                err
            }
        }
    }

    /// Queue the craft: the parts leave the grid. False when the recipe is closed or the slotting is wrong.
    #[func]
    fn craft(&mut self, recipe: GString, handles: PackedInt32Array, by: GString, day: i64) -> bool {
        self.ensure();
        let Some(r) = self.recipes.as_ref().unwrap().get(&recipe.to_string()).cloned() else { return false };
        if !self.bench.is_open(&r) {
            return false;
        }
        let Some(parts) = self.parts_for(&handles) else { return false };
        let t = self.table.as_ref().unwrap().clone();
        if bench::outcome(&t, self.recipes.as_ref().unwrap(), &r, &parts, &by.to_string(), day.max(0) as u32).is_err() {
            return false;
        }
        let stash = self.stash.as_mut().unwrap();
        for h in handles.as_slice() {
            stash.take(*h as u32);
        }
        let f = self.recipes.as_ref().unwrap();
        self.bench.queue(f, &r, parts, &by.to_string(), day.max(0) as u32);
        true
    }

    /// The Bench queue: [{index, recipe, name, shifts_left, parts: [names]}], then whatever waits for Stash room.
    #[func]
    fn bench_jobs(&mut self) -> VarArray {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let f = self.recipes.as_ref().unwrap();
        let mut out = VarArray::new();
        for (i, j) in self.bench.jobs.iter().enumerate() {
            let mut d = VarDictionary::new();
            d.set("index", i as i64);
            d.set("recipe", j.recipe.as_str());
            d.set("name", f.get(&j.recipe).map(|r| r.name.as_str()).unwrap_or("?"));
            d.set("shifts_left", j.shifts_left as i64);
            d.set("parts", &PackedStringArray::from(j.parts.iter().map(|p| GString::from(t.def(p.def).name.as_str())).collect::<Vec<_>>().as_slice()));
            out.push(&d.to_variant());
        }
        out
    }

    /// The best open recipe this grid item can go into (highest product value), for the Stash card's third number:
    /// {id, name, tier, product_name, slot_tag} or empty.
    #[func]
    fn best_recipe(&mut self, handle: i64) -> VarDictionary {
        self.ensure();
        let mut out = VarDictionary::new();
        let t = self.table.as_ref().unwrap();
        let Some(item) = self.stash.as_ref().unwrap().get_any(handle as u32) else { return out };
        let d = t.def(item.def);
        let mut best: Option<(&bench::Recipe, &bench::Slot, u32)> = None;
        for r in &self.recipes.as_ref().unwrap().recipe {
            if !self.bench.is_open(r) {
                continue;
            }
            let Some(slot) = r.slots.iter().find(|s| bench::accepts(s, d, item)) else { continue };
            let value = t.get(&r.product).map(|pd| pd.value).unwrap_or(d.value);
            if best.is_none_or(|(_, _, v)| value > v) {
                best = Some((r, slot, value));
            }
        }
        if let Some((r, slot, _)) = best {
            out.set("id", r.id.as_str());
            out.set("name", r.name.as_str());
            out.set("tier", r.tier as i64);
            out.set("product_name", t.get(&r.product).map(|pd| pd.name.as_str()).unwrap_or(r.name.as_str()));
            out.set("slot_tag", slot.tag.as_str());
        }
        out
    }

    #[func]
    fn bench_waiting(&self) -> i64 {
        self.bench.waiting.len() as i64
    }

    #[func]
    fn tier_open(&self) -> i64 {
        self.bench.tier_open.max(1) as i64
    }

    /// Cancel a job: its parts come back to the grid (those that fit; the rest wait at the Bench).
    #[func]
    fn cancel_job(&mut self, index: i64) -> bool {
        self.ensure();
        let parts = self.bench.cancel(index.max(0) as usize);
        if parts.is_empty() {
            return false;
        }
        let t = self.table.as_ref().unwrap().clone();
        let stash = self.stash.as_mut().unwrap();
        for p in parts {
            if stash.grid.auto_place(t.def(p.def), p.clone()).is_err() {
                self.bench.waiting.push(p);
            }
        }
        true
    }

    // ------------------------------------------------------------ the four verbs and the ledger

    /// Queue an item to sell at a market (index among the colony's market sites). It leaves the grid.
    #[func]
    fn queue_sell(&mut self, handle: i64, market: i64, day: i64) -> bool {
        self.ensure();
        let stash = self.stash.as_mut().unwrap();
        self.ledger.queue(stash, handle as u32, inventory::Verb::Sell { market: market.clamp(0, 255) as u8 }, day.max(0) as u32)
    }

    /// Queue an item to feed a Pit (0 is the Heart). Only items with a feed value go; returns false otherwise.
    #[func]
    fn queue_feed(&mut self, handle: i64, pit: i64, day: i64) -> bool {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let stash = self.stash.as_mut().unwrap();
        let Some(item) = stash.get_any(handle as u32) else { return false };
        if t.def(item.def).feed <= 0.0 {
            return false;
        }
        self.ledger.queue(stash, handle as u32, inventory::Verb::Feed { pit: pit.clamp(0, 255) as u8 }, day.max(0) as u32)
    }

    /// Pull a queued item back into the grid; -1 when it is missing or there is no room.
    #[func]
    fn unqueue(&mut self, index: i64) -> i64 {
        self.ensure();
        let t = self.table.as_ref().unwrap().clone();
        let stash = self.stash.as_mut().unwrap();
        self.ledger.unqueue(stash, &t, index.max(0) as usize).map_or(-1, |h| h as i64)
    }

    /// The queue: [{index, id, name, tier, identified, verb: "sell"|"feed", target, quote, nutrient, day}]
    #[func]
    fn queued(&mut self) -> VarArray {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let mut out = VarArray::new();
        for (i, q) in self.ledger.queue.iter().enumerate() {
            let d = t.def(q.item.def);
            let mut e = VarDictionary::new();
            e.set("index", i as i64);
            e.set("id", d.id.as_str());
            e.set("name", d.name.as_str());
            e.set("tier", format!("{:?}", d.tier).to_lowercase().as_str());
            e.set("identified", q.item.ident == items::Ident::Full);
            e.set("w", d.size[0] as i64);
            e.set("h", d.size[1] as i64);
            match q.verb {
                inventory::Verb::Sell { market } => {
                    e.set("verb", "sell");
                    e.set("target", market as i64);
                    // the price it will fetch given what of its category is queued ahead of it at that market; -1 when
                    // no stall there buys it
                    let cat = d.category;
                    let ahead = self.ledger.queue[..i].iter().filter(|p| p.verb == q.verb && t.def(p.item.def).category == cat).count() as u32;
                    let glut = self.ledger.glut_units(market, cat as usize) + ahead;
                    e.set("quote", self.ledger.best(t, &q.item, market, glut).map_or(-1, |(p, _)| p as i64));
                    e.set("nutrient", 0.0);
                }
                inventory::Verb::Deliver { market, contract } => {
                    e.set("verb", "deliver");
                    e.set("target", market as i64);
                    e.set("contract", contract as i64);
                    e.set("quote", 0i64);
                    e.set("nutrient", 0.0);
                }
                inventory::Verb::Feed { pit } => {
                    e.set("verb", "feed");
                    e.set("target", pit as i64);
                    e.set("quote", 0i64);
                    e.set("nutrient", items::feed_value(d, &q.item).0 as f64);
                }
            }
            e.set("day", q.day as i64);
            out.push(&e.to_variant());
        }
        out
    }

    /// What an item in the grid would fetch at a market now (glut and the queue ahead included); -1 when no stall
    /// there buys it (a Relic; Blackline outside the Corpo remnants' tier 3; a faction closed to the city).
    #[func]
    fn quote(&mut self, handle: i64, market: i64) -> i64 {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let stash = self.stash.as_ref().unwrap();
        stash.get_any(handle as u32).map_or(0, |it| self.ledger.quote(t, it, market.clamp(0, 255) as u8).map_or(-1, |p| p as i64))
    }

    #[func]
    fn eddies(&self) -> i64 {
        self.ledger.eddies as i64
    }

    /// Re-pack the grid by "tier", "value", "weight", "condition" or "category".
    #[func]
    fn sort_by(&mut self, key: GString) {
        self.ensure();
        let t = self.table.as_ref().unwrap().clone();
        self.stash.as_mut().unwrap().sort_by(&t, &key.to_string());
    }

    #[func]
    fn queue_count(&self) -> i64 {
        self.ledger.queue.len() as i64
    }

    /// Unpinned items in the grid that match the filters: category ("" any), tier ("" any), condition below
    /// `max_condition` (101 any), unidentified only. Pinned items never match.
    #[func]
    fn bulk_handles(&mut self, category: GString, tier: GString, max_condition: i64, unidentified: bool) -> PackedInt32Array {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        let stash = self.stash.as_ref().unwrap();
        let (cat, tr) = (category.to_string(), tier.to_string());
        let mut v: Vec<i32> = stash
            .grid
            .placed()
            .into_iter()
            .filter(|(_, p)| !p.item.pinned)
            .filter(|(_, p)| {
                let d = t.def(p.item.def);
                (cat.is_empty() || format!("{:?}", d.category).to_lowercase() == cat)
                    && (tr.is_empty() || format!("{:?}", d.tier).to_lowercase() == tr)
                    && (p.item.condition as i64) < max_condition
                    && (!unidentified || p.item.ident != items::Ident::Full)
            })
            .map(|(h, _)| h as i32)
            .collect();
        v.sort_unstable();
        PackedInt32Array::from(v.as_slice())
    }

    /// What has landed since the last shift end (the report so far) and what the Stash has on the Mat:
    /// {sold, earned, fed, stored, home, spilled, cut, left, deliveries, riding, held}.
    #[func]
    fn landed(&self) -> VarDictionary {
        let mut d = Self::report_dict(&self.ledger.landed);
        d.set("riding", self.ledger.in_flight.len() as i64);
        d.set("held", self.stash.as_ref().map_or(0, |s| s.floor.len()) as i64);
        d
    }

    #[func]
    fn width(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().grid.w as i64
    }

    #[func]
    fn height(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().grid.h as i64
    }

    #[func]
    fn tier(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().tier as i64
    }

    /// Auto-place one item by table index; returns the handle or -1.
    #[func]
    fn add(&mut self, index: i64, condition: i64, found_in_run: bool) -> i64 {
        self.ensure();
        let t = self.table.as_ref().unwrap();
        if index < 0 || index as usize >= t.len() {
            return -1;
        }
        let def = &t.defs[index as usize];
        let mut inst = ItemInstance::new(items::ItemId(index as u16));
        inst.condition = condition.clamp(0, 100) as u8;
        inst.found_in_run = found_in_run;
        if found_in_run {
            inst.ident = items::Ident::Unknown;
        }
        match self.stash.as_mut().unwrap().grid.auto_place(def, inst) {
            Ok(h) => h as i64,
            Err(_) => -1,
        }
    }

    #[func]
    fn remove(&mut self, handle: i64) -> bool {
        self.ensure();
        self.stash.as_mut().unwrap().take(handle as u32).is_some()
    }

    #[func]
    fn pin(&mut self, handle: i64, on: bool) {
        self.ensure();
        if let Some(it) = self.stash.as_mut().unwrap().get_any_mut(handle as u32) {
            it.pinned = on;
        }
    }

    /// Every placed item: handle, x, y, w, h, rotated, index, id, name, tier, condition, identified, pinned, sell, feed_cells.
    #[func]
    fn items(&mut self) -> VarArray {
        self.ensure();
        let t = self.table.clone().unwrap();
        let placed: Vec<(u32, inventory::Placed)> = self.stash.as_ref().unwrap().grid.placed().into_iter().map(|(h, p)| (h, p.clone())).collect();
        let mut out = VarArray::new();
        for (h, p) in placed {
            let mut e = self.item_dict(&t, h, &p.item);
            e.set("x", p.x as i64);
            e.set("y", p.y as i64);
            e.set("w", p.w as i64);
            e.set("h", p.h as i64);
            e.set("rotated", p.rotated);
            out.push(&e.to_variant());
        }
        out
    }

    #[func]
    fn count(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().count() as i64
    }

    #[func]
    fn free_cells(&mut self) -> i64 {
        self.ensure();
        self.stash.as_ref().unwrap().grid.free_cells() as i64
    }

    /// Take a Runner's rig (RoseRun.rig_export) into the Stash grid, best value first. Returns how many did not fit.
    #[func]
    fn import_rig(&mut self, bytes: PackedByteArray) -> i64 {
        self.ensure();
        let Ok(rig) = postcard::from_bytes::<inventory::Rig>(bytes.to_vec().as_slice()) else { return -1 };
        let t = self.table.clone().unwrap();
        let items = game::store::rig_items(&rig, &t);
        // what the grid cannot hold lies on the Heart's floor (game::store::take_haul)
        let (_, held) = game::store::take_haul(self.stash.as_mut().unwrap(), &t, items);
        held as i64
    }

    /// Take loose items (postcard Vec<ItemInstance>: a Mat haul, regrown gear) into the Stash; what the grid cannot
    /// hold lies on the Heart's floor (game::store::take_haul). Returns how many.
    #[func]
    fn import_items(&mut self, bytes: PackedByteArray) -> i64 {
        self.ensure();
        let Ok(items) = postcard::from_bytes::<Vec<ItemInstance>>(bytes.to_vec().as_slice()) else { return -1 };
        let t = self.table.clone().unwrap();
        let (_, held) = game::store::take_haul(self.stash.as_mut().unwrap(), &t, items);
        held as i64
    }

    /// Pack a Runner's rig from the Stash: the hand weapon by handle (-1 for none), up to four quick slots and any
    /// number of backpack items by handle. Packed items leave the Stash; a handle that is missing or does not fit is
    /// skipped. Returns the rig serialised (postcard) for `RoseRun.begin_run_rig`, or empty when nothing packed.
    #[func]
    fn pack_rig(&mut self, hand: i64, quick: PackedInt32Array, pack: PackedInt32Array, rows: i64, carry_kg: f64) -> PackedByteArray {
        self.ensure();
        let t = self.table.as_ref().unwrap().clone();
        let stash = self.stash.as_mut().unwrap();
        let mut rig = inventory::Rig::new(8, rows.clamp(3, 8) as u8, 4, carry_kg.max(1.0) as f32);
        let mut any = false;
        if hand >= 0 {
            if let Some(it) = stash.grid.remove(hand as u32) {
                rig.gear.hands[0] = Some(it);
                any = true;
            }
        }
        for (i, h) in quick.as_slice().iter().take(4).enumerate() {
            if *h < 0 {
                continue;
            }
            let Some(p) = stash.grid.get(*h as u32) else { continue };
            // quick slots take items of one or two cells
            if (p.w as u32) * (p.h as u32) > 2 {
                continue;
            }
            if let Some(it) = stash.grid.remove(*h as u32) {
                rig.quick[i] = Some(it);
                any = true;
            }
        }
        for h in pack.as_slice() {
            if *h < 0 {
                continue;
            }
            let Some(it) = stash.grid.remove(*h as u32) else { continue };
            if let Err(_) = rig.backpack.auto_place(t.def(it.def), it.clone()) {
                // no room: back into the Stash
                let _ = stash.grid.auto_place(t.def(it.def), it);
            } else {
                any = true;
            }
        }
        if !any {
            return PackedByteArray::new();
        }
        PackedByteArray::from(postcard::to_allocvec(&rig).unwrap_or_default().as_slice())
    }

    /// Serialise through the save crate and back; returns true when the round trip reproduces the Stash.
    #[func]
    fn save_round_trip(&mut self) -> bool {
        self.ensure();
        let mut file = save::SaveFile::new(1, 1);
        file.stash = self.stash.as_ref().unwrap().clone();
        file.ledger = self.ledger.clone();
        file.bench = self.bench.clone();
        let bytes = file.encode();
        match save::SaveFile::decode(&bytes) {
            Ok(back) => back.stash == file.stash && back.ledger == file.ledger && back.bench == file.bench,
            Err(_) => false,
        }
    }
}

// ---------------------------------------------------------------- the run

const CHUNKS_TOML: &str = include_str!("../../../data/chunks.toml");

/// Data files beside the project win in debug; the compiled copies ship.
fn data_text(name: &str, shipped: &str) -> String {
    let path = ProjectSettings::singleton().globalize_path(&format!("res://../data/{name}")).to_string();
    std::fs::read_to_string(&path).unwrap_or_else(|_| shipped.to_string())
}

fn chunk_lib() -> run::ChunkLib {
    run::ChunkLib::from_toml(&data_text("chunks.toml", CHUNKS_TOML)).unwrap_or_else(|_| run::ChunkLib::from_toml(CHUNKS_TOML).expect("built-in chunk library"))
}

fn loot_tables() -> run::LootTables {
    run::LootTables::from_toml(&data_text("loot.toml", run::loot::LOOT_TOML)).unwrap_or_else(|_| run::LootTables::shipped())
}

fn creature_defs() -> run::CreatureFile {
    run::CreatureFile::from_toml(&data_text("creatures.toml", run::creature::CREATURES_TOML)).unwrap_or_else(|_| run::CreatureFile::shipped())
}

fn source_state(s: run::SourceState) -> &'static str {
    match s {
        run::SourceState::Closed => "closed",
        run::SourceState::Open => "open",
        run::SourceState::Empty => "empty",
    }
}

/// An item as the Runner sees it in the run: unidentified items keep their name hidden behind the tier and category.
fn item_view(table: &ItemTable, k: usize, it: &ItemInstance) -> VarDictionary {
    let d = table.def(it.def);
    let mut out = VarDictionary::new();
    let tier = format!("{:?}", d.tier).to_lowercase();
    let cat = format!("{:?}", d.category).to_lowercase();
    out.set("k", k as i64);
    out.set("index", it.def.0 as i64);
    out.set("id", d.id.as_str());
    let label = if it.ident == items::Ident::Full { d.name.clone() } else { format!("unidentified {} {}", tier, cat) };
    out.set("label", label.as_str());
    out.set("tier", tier.as_str());
    out.set("category", cat.as_str());
    out.set("w", d.size[0] as i64);
    out.set("h", d.size[1] as i64);
    out.set("weight", d.weight as f64);
    out.set("condition", it.condition as i64);
    out.set("ident", it.ident == items::Ident::Full);
    out.set("found", it.found_in_run);
    out.set("value_blind", items::price(d, it, PriceContext::default()) as i64);
    out
}

fn tiles_array(w: usize, v: &[(u16, u16)]) -> PackedInt32Array {
    PackedInt32Array::from(v.iter().map(|&(x, y)| (y as usize * w + x as usize) as i32).collect::<Vec<i32>>().as_slice())
}

/// The run. `assemble` builds the route; `begin_run` starts the simulation on it; `step` advances it one tick
/// (20 Hz) from the input word; everything else reads state. Godot decides nothing.
#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseRun {
    base: Base<RefCounted>,
    lib: Option<run::ChunkLib>,
    map: Option<run::RouteMap>,
    sim: Option<run::RunSim>,
    mat_lume_gene: f32,
    memory: Option<Vec<u8>>,
    /// the shift the run is on: the colony's day, a Flood shift's roll and the next shift's (the card and the run
    /// read them)
    day: u32,
    flood: Option<u8>,
    next_flood: Option<u8>,
}

#[godot_api]
impl RoseRun {
    fn ensure_lib(&mut self) -> &run::ChunkLib {
        if self.lib.is_none() {
            self.lib = Some(chunk_lib());
        }
        self.lib.as_ref().unwrap()
    }

    /// Assemble a route. `chunks` 0 draws the count from the seed (8-14). Returns the chunk count, 0 on failure.
    #[func]
    fn assemble(&mut self, seed: i64, stratum: i64, chunks: i64) -> i64 {
        let lib = self.ensure_lib().clone();
        let lay = if chunks <= 0 { run::route(&lib, seed as u64, stratum as u8) } else { run::assemble(&lib, seed as u64, stratum as u8, chunks as usize) };
        match lay {
            Ok(lay) => {
                let m = run::RouteMap::compile(&lib, &lay);
                let n = m.chunks.len() as i64;
                self.map = Some(m);
                self.sim = None;
                n
            }
            Err(e) => {
                godot_error!("RoseRun.assemble: {e}");
                0
            }
        }
    }

    /// Lume gene of the Mat the route starts from (0..1); set before `begin_run`.
    #[func]
    fn set_mat_lume(&mut self, gene: f64) {
        self.mat_lume_gene = gene.clamp(0.0, 1.0) as f32;
    }

    /// The shift the route is run on: the colony's day, its Flood roll and the next shift's (`RoseSim.flood_roll`,
    /// `next_flood_roll`; -1 when that shift is not a Flood shift); before `card` and `begin_run`.
    #[func]
    fn set_shift(&mut self, day: i64, flood_roll: i64, next_flood_roll: i64) {
        self.day = day.max(0) as u32;
        self.flood = (flood_roll >= 0).then(|| flood_roll.min(255) as u8);
        self.next_flood = (next_flood_roll >= 0).then(|| next_flood_roll.min(255) as u8);
    }

    /// Start this run's fog from a previous run's memory of the route (bits from `memory_bits`); before `begin_run`.
    #[func]
    fn restore_memory(&mut self, bits: PackedByteArray) {
        self.memory = Some(bits.to_vec());
    }

    /// Start the run: creatures spawn, loot sources arm, the Runner stands on the Mat edge with the kit in the quick
    /// slots and the melee weapon in hand. `nonce` makes this run's contents, `route_runs` is how many runs this route
    /// has had before (0: first run), `day` goes into provenance; `kit` is up to four item ids for the quick slots.
    /// `followers`: [{id, name, body, damage}] up to two. `corpses`: [{x, y, chunk, day, rig: PackedByteArray,
    /// raided}] of our dead on this route (from the save); the raider roll runs here.
    #[func]
    fn begin_run(&mut self, nonce: i64, route_runs: i64, day: i64, kit: PackedStringArray, weapon: GString, runner: VarDictionary, followers: VarArray, corpses: VarArray) {
        let Some(m) = self.map.clone() else { return };
        let table = table();
        let setup = Self::runner_from(&runner);
        let mut rig = inventory::Rig::new(8, setup.rig_rows, 4, setup.carry_kg());
        for (i, id) in kit.as_slice().iter().take(4).enumerate() {
            if let Some((_, inst)) = inventory::instance(&table, &id.to_string()) {
                rig.quick[i] = Some(inst);
            }
        }
        if let Some((_, inst)) = inventory::instance(&table, &weapon.to_string()) {
            rig.gear.hands[0] = Some(inst);
        }
        let fs = Self::followers_from(&followers);
        let cs = Self::corpses_from(&corpses);
        self.start(m, table, nonce, route_runs, day, rig, setup, fs, cs);
    }

    /// Start the run with a rig packed on the Route Board (`RoseStash.pack_rig` bytes); otherwise as `begin_run`.
    /// `runner` is `RosePeople.runner_setup(id)` (or `plain_runner`): {id, name, body, nerve, hands, eye, veins,
    /// rig_rows, carry_kg, flags}.
    #[func]
    fn begin_run_rig(&mut self, nonce: i64, route_runs: i64, day: i64, rig: PackedByteArray, runner: VarDictionary, followers: VarArray, corpses: VarArray) {
        let Some(m) = self.map.clone() else { return };
        let table = table();
        let setup = Self::runner_from(&runner);
        let rig = postcard::from_bytes::<inventory::Rig>(rig.to_vec().as_slice()).unwrap_or_else(|_| inventory::Rig::new(8, setup.rig_rows, 4, setup.carry_kg()));
        let fs = Self::followers_from(&followers);
        let cs = Self::corpses_from(&corpses);
        self.start(m, table, nonce, route_runs, day, rig, setup, fs, cs);
    }

    fn runner_from(d: &VarDictionary) -> crate::people::RunnerSetup {
        let geti = |k: &str, def: i64| d.get(k).and_then(|v| v.try_to::<i64>().ok()).unwrap_or(def);
        let plain = crate::people::RunnerSetup::plain(geti("body", 5).clamp(1, 10) as u8);
        crate::people::RunnerSetup {
            id: geti("id", 0) as u32,
            name: d.get("name").map(|v| v.to_string()).unwrap_or(plain.name),
            body: plain.body,
            nerve: geti("nerve", plain.nerve as i64).clamp(1, 10) as u8,
            hands: geti("hands", plain.hands as i64).clamp(1, 10) as u8,
            eye: geti("eye", plain.eye as i64).clamp(1, 10) as u8,
            veins: geti("veins", plain.veins as i64).clamp(1, 5) as u8,
            rig_rows: geti("rig_rows", plain.rig_rows as i64).clamp(3, 8) as u8,
            // carry_kg is 18 + (body - 5) + the traits'; the dict carries the total
            carry_bonus_kg: d.get("carry_kg").and_then(|v| v.try_to::<f64>().ok()).map(|c| c as f32 - 18.0 - (plain.body as f32 - 5.0)).unwrap_or(0.0),
            flags: d.get("flags").and_then(|v| v.try_to::<PackedStringArray>().ok()).map(|a| a.as_slice().iter().map(|s| s.to_string()).collect()).unwrap_or_default(),
        }
    }

    fn followers_from(followers: &VarArray) -> Vec<run::FollowerSetup> {
        let mut fs = Vec::new();
        for v in followers.iter_shared() {
            let Ok(d) = v.try_to::<VarDictionary>() else { continue };
            fs.push(run::FollowerSetup {
                id: d.get("id").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u32,
                name: d.get("name").map(|v| v.to_string()).unwrap_or_default(),
                body: d.get("body").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(5) as u8,
                damage: d.get("damage").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(4) as i32,
                back_off_pct: d.get("back_off_pct").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(30).clamp(0, 100) as u8,
                bonded: d.get("bonded").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false),
            });
        }
        fs
    }

    fn corpses_from(corpses: &VarArray) -> Vec<run::Corpse> {
        let mut cs = Vec::new();
        for v in corpses.iter_shared() {
            let Ok(d) = v.try_to::<VarDictionary>() else { continue };
            let bytes = d.get("rig").and_then(|v| v.try_to::<PackedByteArray>().ok()).map(|b| b.to_vec()).unwrap_or_default();
            let Ok(rig) = postcard::from_bytes::<inventory::Rig>(&bytes) else { continue };
            cs.push(run::Corpse {
                x: d.get("x").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u16,
                y: d.get("y").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u16,
                chunk: d.get("chunk").map(|v| v.to_string()).unwrap_or_default(),
                day: d.get("day").and_then(|v| v.try_to::<i64>().ok()).unwrap_or(0) as u32,
                rig,
                raided: d.get("raided").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false),
            });
        }
        cs
    }

    #[allow(clippy::too_many_arguments)]
    fn start(&mut self, m: run::RouteMap, table: ItemTable, nonce: i64, route_runs: i64, day: i64, rig: inventory::Rig, runner: crate::people::RunnerSetup, fs: Vec<run::FollowerSetup>, cs: Vec<run::Corpse>) {
        let setup = run::RunSetup {
            mat_lume_gene: self.mat_lume_gene,
            memory: self.memory.as_deref(),
            run_nonce: nonce as u64,
            route_runs: route_runs.clamp(0, u16::MAX as i64) as u16,
            day: day.max(0) as u32,
            rig,
            runner,
            followers: fs,
            corpses: cs,
            flood: self.flood,
            next_flood: self.next_flood,
        };
        self.sim = Some(run::RunSim::new(m, table, loot_tables(), creature_defs(), setup));
    }

    /// The route card for the assembled route (before or during a run): {chunks, shift_s, danger, danger_band,
    /// burrows, patrol_chunks, known (0-1), known_lockers, known_crates, water (0-1), grates, flood_rise (-1 when
    /// not a Flood shift), flood_s (when the rise starts, in seconds of the shift)}. `memory` is the route's memory
    /// bits from the save (empty before any run); the shift is `set_shift`'s.
    #[func]
    fn card(&self, memory: PackedByteArray) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(m) = self.map_ref() else { return d };
        let bits = memory.to_vec();
        let c = run::card(m, &creature_defs(), if bits.is_empty() { None } else { Some(bits.as_slice()) }, self.day, self.flood, self.next_flood);
        d.set("chunks", c.chunks as i64);
        d.set("shift_s", c.shift_seconds() as i64);
        d.set("danger", c.danger as f64);
        d.set("density", c.density as f64);
        d.set("danger_band", c.danger_band());
        d.set("burrows", c.burrows as i64);
        d.set("patrol_chunks", c.patrol_chunks as i64);
        d.set("known", c.known as f64);
        d.set("known_lockers", c.known_lockers as i64);
        d.set("known_crates", c.known_crates as i64);
        d.set("water", c.water as f64);
        d.set("grates", c.grates as i64);
        d.set("flood_rise", c.flood.map_or(-1, |f| f.rise as i64));
        d.set("flood_s", c.flood.map_or(0, |f| f.start_s() as i64));
        d
    }

    /// The shift clock: {shift_ticks, tick, left_ticks, overtime_ticks, flood_rise (-1 when not a Flood shift),
    /// flood_s (when the rise starts, seconds into the shift), water (tiles risen so far)}.
    #[func]
    fn shift(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(s) = self.sim.as_ref() else { return d };
        d.set("shift_ticks", s.shift_ticks as i64);
        d.set("tick", s.tick as i64);
        d.set("left_ticks", s.shift_ticks.saturating_sub(s.tick) as i64);
        d.set("overtime_ticks", s.overtime_ticks() as i64);
        d.set("flood_rise", s.flood.map_or(-1, |f| f.rise as i64));
        d.set("flood_s", s.flood.map_or(0, |f| f.start_s() as i64));
        d.set("water", s.flood_level as i64);
        d
    }

    /// One byte per chunk in placement order: 1 where any tile has been seen or remembered.
    #[func]
    fn chunk_seen(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.chunk_seen().iter().map(|&b| b as u8).collect::<Vec<u8>>().as_slice()))
    }

    /// Container tiles of the route: [{x, y, kind: "locker"|"crate", tier, chunk}] (the Board draws the known ones).
    #[func]
    fn containers(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for c in &m.containers {
                let mut d = VarDictionary::new();
                d.set("x", c.x as i64);
                d.set("y", c.y as i64);
                d.set("kind", if c.kind == run::ContainerKind::Locker { "locker" } else { "crate" });
                d.set("tier", c.tier as i64);
                d.set("chunk", c.chunk as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// Followers: [{id, name, x, y, tx, ty, facing, hp, max_hp, order, life, carrying, windup}]
    #[func]
    fn followers(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        for f in &s.followers {
            let mut d = VarDictionary::new();
            d.set("id", f.id as i64);
            d.set("name", f.name.as_str());
            d.set("x", f.x as i64);
            d.set("y", f.y as i64);
            let (tx, ty) = f.tile();
            d.set("tx", tx as i64);
            d.set("ty", ty as i64);
            d.set("facing", f.facing as i64);
            d.set("hp", f.hp as i64);
            d.set("max_hp", f.max_hp as i64);
            d.set("order", f.order.name());
            d.set("life", match f.life {
                run::FollowerLife::Up => "up",
                run::FollowerLife::Down { .. } => "down",
                run::FollowerLife::Dead => "dead",
            });
            d.set("carrying", f.carrying.is_some());
            d.set("windup", f.windup_left > 0);
            out.push(&d.to_variant());
        }
        out
    }

    /// The Daily Seed of a date (YYYYMMDD): {date, stratum, seed, name, hand, kit} — one route, a stratum from S1-S3
    /// and its route from the date, the bot's fixed rig (Full Design v3.19). The Runner: `RosePeople.daily_runner`.
    #[func]
    fn daily(date: i64) -> VarDictionary {
        let dl = game::daily::daily(date.max(0) as u32);
        let mut d = VarDictionary::new();
        d.set("date", date);
        d.set("stratum", dl.stratum as i64);
        d.set("seed", dl.route_seed as i64);
        let region = logistics::StrataFile::shipped().stratum.iter().find(|s| s.n == dl.stratum).map(|s| s.region.clone()).unwrap_or_default();
        d.set("name", region.as_str());
        d.set("hand", game::city::KIT_HAND);
        let mut kit = PackedStringArray::new();
        for k in game::city::KIT_QUICK {
            kit.push(k);
        }
        d.set("kit", &kit);
        d
    }

    /// The run's outcome once it is over: {result, ticks, rig: bytes, corpse: {x, y, chunk, day, rig: bytes} or null,
    /// followers_extracted, followers_stranded, followers_dead, mat_haul: bytes (Vec<ItemInstance>), regrown: bytes,
    /// kills, chunks_seen, blind_value, score (the Daily's), route_memory: bytes}. Empty while running.
    #[func]
    fn outcome(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(o) = self.sim.as_ref().and_then(|s| s.outcome.as_ref()) else { return d };
        d.set("result", match o.result {
            run::RunResult::Extracted { .. } => "extracted",
            run::RunResult::Died { .. } => "died",
        });
        d.set("ticks", o.ticks as i64);
        d.set("rig", &PackedByteArray::from(o.rig.as_ref().map(|r| postcard::to_allocvec(r).unwrap_or_default()).unwrap_or_default().as_slice()));
        if let Some(c) = &o.corpse {
            let mut cd = VarDictionary::new();
            cd.set("x", c.x as i64);
            cd.set("y", c.y as i64);
            cd.set("chunk", c.chunk.as_str());
            cd.set("day", c.day as i64);
            cd.set("rig", &PackedByteArray::from(postcard::to_allocvec(&c.rig).unwrap_or_default().as_slice()));
            cd.set("raided", c.raided);
            d.set("corpse", &cd);
        }
        let ids = |v: &Vec<u32>| PackedInt32Array::from(v.iter().map(|&i| i as i32).collect::<Vec<i32>>().as_slice());
        d.set("followers_extracted", &ids(&o.followers_extracted));
        d.set("followers_stranded", &ids(&o.followers_stranded));
        d.set("followers_dead", &ids(&o.followers_dead));
        d.set("mat_haul", &PackedByteArray::from(postcard::to_allocvec(&o.mat_haul).unwrap_or_default().as_slice()));
        d.set("mat_haul_count", o.mat_haul.len() as i64);
        d.set("regrown", &PackedByteArray::from(postcard::to_allocvec(&o.regrown).unwrap_or_default().as_slice()));
        d.set("regrown_count", o.regrown.len() as i64);
        d.set("kills", o.kills as i64);
        d.set("oath_kills", o.oath_kills as i64);
        d.set("raider_kills", o.raider_kills as i64);
        let mut killed = PackedStringArray::new();
        for k in &o.killed {
            killed.push(k.as_str());
        }
        d.set("killed", &killed);
        d.set("chunks_seen", o.chunks_seen as i64);
        d.set("blind_value", o.blind_value as i64);
        // the Daily Seed's score: the base value of what was found and came home (game::daily)
        let score = match (&o.rig, self.sim.as_ref()) {
            (Some(rig), Some(s)) => game::daily::score(&s.table, rig.all_items()),
            _ => 0,
        };
        d.set("score", score as i64);
        d.set("route_memory", &PackedByteArray::from(o.route_memory.as_slice()));
        // what the roster reads (RosePeople.apply_run)
        d.set("items_taken", o.items_taken as i64);
        d.set("hits_survived", o.hits_survived as i64);
        d.set("follower_strikes", o.follower_strikes as i64);
        d.set("injuries", &PackedStringArray::from(o.injuries.iter().map(|s| GString::from(*s)).collect::<Vec<_>>().as_slice()));
        d.set("pulls", o.report.pulls as i64);
        // the cadence report (Full Design, The cadence budget and its log)
        let r = &o.report;
        let mut rep = VarDictionary::new();
        rep.set("landings", r.landings as i64);
        rep.set("pulls", r.pulls as i64);
        rep.set("notables", r.notables as i64);
        rep.set("losses", r.losses as i64);
        rep.set("damage", r.damage as i64);
        rep.set("decisions", r.decisions as i64);
        rep.set("pull_gap_median", r.pull_gap_median as f64);
        rep.set("landing_gap_median", r.landing_gap_median as f64);
        rep.set("loss_gap_max", r.loss_gap_max as f64);
        rep.set("value_per_min", r.value_per_min as f64);
        d.set("report", &rep);
        d
    }

    /// The run's cadence log as text (header, one line per landing / loss / decision, the report), for the
    /// save folder. Empty before the run ends.
    #[func]
    fn cadence_log(&self, header: GString) -> GString {
        self.sim.as_ref().and_then(|s| s.outcome.as_ref()).map_or_else(GString::new, |o| GString::from(o.log.text(&header.to_string(), &o.report).as_str()))
    }

    /// One tick from the input word (see run::sim::RunInput::pack). Returns false when the run is over.
    #[func]
    fn step(&mut self, buttons: i64, dx: i64, dy: i64) -> bool {
        let Some(sim) = self.sim.as_mut() else { return false };
        sim.step(run::RunInput::unpack(buttons as u16, dx as i16, dy as i16));
        sim.result.is_none()
    }

    /// Development only: put the Runner on a tile (tests and the screenshot script). Not an input; never replayed.
    #[func]
    fn debug_teleport(&mut self, x: i64, y: i64) -> bool {
        let Some(s) = self.sim.as_mut() else { return false };
        if x < 0 || y < 0 || x as usize >= s.map.w || y as usize >= s.map.h || !s.map.passable(x as usize, y as usize) {
            return false;
        }
        s.runner.x = x as i32 * run::creature::FP + run::creature::FP / 2;
        s.runner.y = y as i32 * run::creature::FP + run::creature::FP / 2;
        true
    }

    /// Development only (the screenshot script): a raider gang of `n` down at a tile now, and a Flood of `rise` tiles
    /// starting now over its usual time (0 leaves the water). Not inputs; never replayed.
    #[func]
    fn debug_raid(&mut self, x: i64, y: i64, n: i64, gang: i64) -> bool {
        let Some(s) = self.sim.as_mut() else { return false };
        if x < 0 || y < 0 || x as usize >= s.map.w || y as usize >= s.map.h || !s.map.passable(x as usize, y as usize) {
            return false;
        }
        let chunk = s.map.chunk_of[y as usize * s.map.w + x as usize];
        s.raids.push(run::Raid { at: s.tick + 1, tile: (x as i32, y as i32), chunk, n: n.clamp(1, 3) as u8, gang: gang.clamp(1, 2) as u8 });
        true
    }

    #[func]
    fn debug_flood(&mut self, rise: i64) {
        let Some(s) = self.sim.as_mut() else { return };
        let ticks = run::creature::ticks(s.defs.flood.rise_s).max(1) as u64;
        s.flood = Some(run::RunFlood { rise: rise.clamp(0, 3) as u8, start: s.tick + 1, ticks });
    }

    /// "running", "extracted" or "died".
    #[func]
    fn result(&self) -> GString {
        GString::from(match self.sim.as_ref().and_then(|s| s.result) {
            None => "running",
            Some(run::RunResult::Extracted { .. }) => "extracted",
            Some(run::RunResult::Died { .. }) => "died",
        })
    }

    #[func]
    fn tick(&self) -> i64 {
        self.sim.as_ref().map_or(0, |s| s.tick as i64)
    }

    #[func]
    fn library_size(&mut self) -> i64 {
        self.ensure_lib().chunks.len() as i64
    }

    fn map_ref(&self) -> Option<&run::RouteMap> {
        self.sim.as_ref().map(|s| &s.map).or(self.map.as_ref())
    }

    #[func]
    fn width(&self) -> i64 {
        self.map_ref().map_or(0, |m| m.w as i64)
    }

    #[func]
    fn height(&self) -> i64 {
        self.map_ref().map_or(0, |m| m.h as i64)
    }

    #[func]
    fn chunk_count(&self) -> i64 {
        self.map_ref().map_or(0, |m| m.chunks.len() as i64)
    }

    /// Substrate codes (mat::sim::sub) per tile, live: digging changes it.
    #[func]
    fn substrate(&self) -> PackedByteArray {
        self.map_ref().map_or_else(PackedByteArray::new, |m| PackedByteArray::from(m.substrate.as_slice()))
    }

    #[func]
    fn passable(&self) -> PackedByteArray {
        self.map_ref().map_or_else(PackedByteArray::new, |m| PackedByteArray::from(m.passable_mask().iter().map(|&p| p as u8).collect::<Vec<u8>>().as_slice()))
    }

    #[func]
    fn entry(&self) -> PackedInt32Array {
        self.map_ref().map_or_else(PackedInt32Array::new, |m| tiles_array(m.w, &m.entry))
    }

    /// The city's Mat in the route: where the Runner may extract.
    #[func]
    fn mat(&self) -> PackedInt32Array {
        self.map_ref().map_or_else(PackedInt32Array::new, |m| tiles_array(m.w, &m.mat()))
    }

    #[func]
    fn grates(&self) -> PackedInt32Array {
        self.map_ref().map_or_else(PackedInt32Array::new, |m| tiles_array(m.w, &m.grates))
    }

    /// [{x, y, kind: "sewerat"|"patrol"|"corpse", chunk}]
    #[func]
    fn spawns(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for s in &m.spawns {
                let mut d = VarDictionary::new();
                d.set("x", s.x as i64);
                d.set("y", s.y as i64);
                d.set("kind", match s.kind {
                    run::SpawnKind::Sewerat => "sewerat",
                    run::SpawnKind::Patrol => "patrol",
                    run::SpawnKind::Corpse => "corpse",
                });
                d.set("chunk", s.chunk as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// [{x, y, radius}]
    #[func]
    fn lights(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for l in &m.lights {
                let mut d = VarDictionary::new();
                d.set("x", l.x as i64);
                d.set("y", l.y as i64);
                d.set("radius", l.radius as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// [{x, y, kind: "gas"|"collapse"}]
    #[func]
    fn hazards(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for h in &m.hazards {
                let mut d = VarDictionary::new();
                d.set("x", h.x as i64);
                d.set("y", h.y as i64);
                d.set("kind", if h.kind == run::HazardKind::Gas { "gas" } else { "collapse" });
                out.push(&d.to_variant());
            }
        }
        out
    }

    /// [{id, kind, stratum, x0, y0}] in placement order; 0 is the entry chunk.
    #[func]
    fn chunks(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(m) = self.map_ref() {
            for c in &m.chunks {
                let mut d = VarDictionary::new();
                d.set("id", c.id.as_str());
                d.set("kind", c.kind.as_str());
                d.set("stratum", c.stratum as i64);
                d.set("x0", c.x0 as i64);
                d.set("y0", c.y0 as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    #[func]
    fn chunk_of(&self) -> PackedInt32Array {
        self.map_ref().map_or_else(PackedInt32Array::new, |m| {
            PackedInt32Array::from(m.chunk_of.iter().map(|&c| if c == run::map::NO_CHUNK { -1 } else { c as i32 }).collect::<Vec<i32>>().as_slice())
        })
    }

    #[func]
    fn connectivity(&self) -> f64 {
        self.map_ref().map_or(0.0, |m| m.connectivity() as f64)
    }

    #[func]
    fn digest(&self) -> GString {
        GString::from(self.map_ref().map_or_else(String::new, |m| m.digest()).as_str())
    }

    // ------------------------------------------------------------ the Runner

    /// {x, y (units of 1/256 tile), tx, ty, facing, sections [4], max_sections [4], stamina (0-100), life, down_left,
    ///  action, action_progress (0-1), statuses [names], lume, see_radius, exposure, exposure_band, notice_range,
    ///  noise_here, lantern, dimming, sneaking, sprinting, moved, ammo, overweight, weight, carry, rig_count,
    ///  rig_free, blind_value, on_extraction, source_here, open_source}
    #[func]
    fn runner(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(s) = self.sim.as_ref() else { return d };
        let r = &s.runner;
        d.set("x", r.x as i64);
        d.set("y", r.y as i64);
        let (tx, ty) = r.tile();
        d.set("tx", tx as i64);
        d.set("ty", ty as i64);
        d.set("facing", r.facing as i64);
        d.set("sections", &PackedInt32Array::from(r.sections.map(|v| v as i32).as_slice()));
        d.set("max_sections", &PackedInt32Array::from(r.max_sections.map(|v| v as i32).as_slice()));
        d.set("stamina", (r.stamina as f64) / 100.0);
        d.set("max_stamina", (r.max_stamina as f64) / 100.0);
        let (life, down_left) = match r.life {
            run::Life::Up => ("up", 0),
            run::Life::Down { left } => ("down", left),
            run::Life::Dead => ("dead", 0),
        };
        d.set("life", life);
        d.set("down_left", down_left as i64);
        let rules = &s.defs.runner;
        let (action, progress) = match r.action {
            run::Action::Idle => ("idle", 0.0),
            run::Action::Windup { left } => ("windup", 1.0 - left as f64 / run::creature::ticks(rules.melee_windup).max(1) as f64),
            run::Action::Swing { .. } => ("swing", 1.0),
            run::Action::Dodge { left, .. } => ("dodge", 1.0 - left as f64 / run::creature::ticks(rules.dodge_s).max(1) as f64),
            run::Action::Reload { left } => ("reload", 1.0 - left as f64 / run::creature::ticks(2.0) as f64),
            run::Action::Dig { progress, .. } => ("dig", progress as f64 / run::creature::ticks(rules.dig_s) as f64),
            run::Action::Extract { progress } => ("extract", progress as f64 / run::creature::ticks(rules.extract_s) as f64),
        };
        d.set("action", action);
        d.set("action_progress", progress.clamp(0.0, 1.0));
        let st = PackedStringArray::from(r.statuses.iter().map(|st| GString::from(st.kind.name())).collect::<Vec<_>>().as_slice());
        d.set("statuses", &st);
        d.set("lume", s.senses.lume.emit() as i64);
        d.set("see_radius", s.senses.lume.see_radius(s.senses.eye) as i64);
        d.set("exposure", s.senses.exposure as i64);
        d.set("exposure_band", run::LightField::band(s.senses.exposure));
        d.set("notice_range", s.senses.notice_range() as i64);
        d.set("noise_here", s.senses.noise.at(tx as usize, ty as usize) as i64);
        d.set("lantern", r.lantern_on);
        // under water: ticks so far and the ticks that are Down (a Flood shift's water)
        d.set("under", s.under as i64);
        d.set("drown", run::creature::ticks(s.defs.flood.drown_s) as i64);
        d.set("dimming", r.dimming);
        d.set("sneaking", r.sneaking);
        d.set("sprinting", r.sprinting);
        d.set("moved", r.moved);
        d.set("pose", r.pose.name());
        d.set("ammo", r.ammo as i64);
        d.set("overweight", r.rig.overweight(&s.table));
        d.set("weight", r.rig.weight(&s.table) as f64);
        d.set("carry", r.rig.carry_cap as f64);
        d.set("rig_count", r.rig.backpack.count() as i64);
        d.set("rig_free", r.rig.backpack.free_cells() as i64);
        let blind: i64 = r.rig.backpack.items().map(|it| items::price(s.table.def(it.def), it, PriceContext::default()) as i64).sum();
        d.set("blind_value", blind);
        d.set("on_extraction", s.on_extraction());
        d.set("source_here", s.source_here().map_or(-1, |i| i as i64));
        d.set("open_source", s.open_source.map_or(-1, |i| i as i64));
        d.set("melee", {
            let (dmg, _, reach) = r.melee(&s.table);
            format!("{dmg} dmg reach {reach}").as_str()
        });
        d.set("hand", r.rig.gear.hands[0].as_ref().map_or("", |h| s.table.def(h.def).id.as_str()));
        d.set("chunk", s.runner_chunk().map_or(-1, |c| c as i64));
        d
    }

    /// Live creatures: [{id, kind, sprite, x, y, tx, ty, facing, hp, max_hp, phase, awareness, windup, visible}]
    #[func]
    fn creatures(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        for c in &s.creatures {
            let mut d = VarDictionary::new();
            d.set("id", c.id as i64);
            d.set("kind", c.kind.key());
            d.set("sprite", s.defs.creature[c.kind.key()].sprite.as_str());
            d.set("x", c.x as i64);
            d.set("y", c.y as i64);
            let (tx, ty) = c.tile();
            d.set("tx", tx as i64);
            d.set("ty", ty as i64);
            d.set("facing", c.facing as i64);
            d.set("hp", c.hp as i64);
            d.set("max_hp", c.max_hp as i64);
            d.set("phase", c.phase.name());
            d.set("awareness", c.awareness as f64);
            d.set("gang", c.gang as i64);
            d.set("looting", c.loot_left > 0);
            d.set("windup", c.windup_left > 0 || c.fuse_left > 0);
            d.set("visible", s.senses.is_visible(tx as usize, ty as usize));
            d.set("heard", !s.senses.is_visible(tx as usize, ty as usize) && c.phase != run::Phase::Calm);
            out.push(&d.to_variant());
        }
        out
    }

    /// Events since the last call, drained: [{kind, ...fields}]
    #[func]
    fn events(&mut self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_mut() else { return out };
        for e in s.drain_events() {
            let mut d = VarDictionary::new();
            match e {
                run::Event::RunnerHit { section, damage, by } => {
                    d.set("kind", "runner_hit");
                    d.set("section", run::runner::SECTION_NAMES[section]);
                    d.set("damage", damage as i64);
                    d.set("by", by as i64);
                }
                run::Event::Bleed { damage } => {
                    d.set("kind", "bleed");
                    d.set("damage", damage as i64);
                }
                run::Event::CreatureHit { id, damage, behind } => {
                    d.set("kind", "creature_hit");
                    d.set("id", id as i64);
                    d.set("damage", damage as i64);
                    d.set("behind", behind);
                }
                run::Event::CreatureDied { id, kind, x, y, source } => {
                    d.set("kind", "creature_died");
                    d.set("id", id as i64);
                    d.set("creature", kind.key());
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("source", source.map_or(-1, |v| v as i64));
                }
                run::Event::CreatureLeft { id } => {
                    d.set("kind", "creature_left");
                    d.set("id", id as i64);
                }
                run::Event::PhaseChange { id, phase } => {
                    d.set("kind", "phase");
                    d.set("id", id as i64);
                    d.set("phase", phase.name());
                }
                run::Event::Swing { x, y, hit } => {
                    d.set("kind", "swing");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("hit", hit);
                }
                run::Event::Shot { x0, y0, x1, y1, by_runner, hit } => {
                    d.set("kind", "shot");
                    d.set("x0", x0 as i64);
                    d.set("y0", y0 as i64);
                    d.set("x1", x1 as i64);
                    d.set("y1", y1 as i64);
                    d.set("by_runner", by_runner);
                    d.set("hit", hit);
                }
                run::Event::Burst { x, y, radius } => {
                    d.set("kind", "burst");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("radius", radius as i64);
                }
                run::Event::Opened { source, revealed, notable, generous, forced, noise } => {
                    d.set("kind", "opened");
                    d.set("source", source as i64);
                    d.set("revealed", revealed as i64);
                    d.set("notable", notable as i64);
                    d.set("generous", generous);
                    d.set("forced", forced);
                    d.set("noise", noise as i64);
                }
                run::Event::Took { source, taken, left } => {
                    d.set("kind", "took");
                    d.set("source", source as i64);
                    d.set("taken", taken as i64);
                    d.set("left", left as i64);
                }
                run::Event::Dug { x, y, pile } => {
                    d.set("kind", "dug");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                    d.set("pile", pile.map_or(-1, |v| v as i64));
                }
                run::Event::Salted { x, y } => {
                    d.set("kind", "salted");
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                }
                run::Event::Used { slot, id } => {
                    d.set("kind", "used");
                    d.set("slot", slot as i64);
                    d.set("id", id.as_str());
                }
                run::Event::Down => d.set("kind", "down"),
                run::Event::Dead => d.set("kind", "dead"),
                run::Event::Extracted => d.set("kind", "extracted"),
                run::Event::Grabbed => d.set("kind", "grabbed"),
                run::Event::FollowerHit { id, damage } => {
                    d.set("kind", "follower_hit");
                    d.set("id", id as i64);
                    d.set("damage", damage as i64);
                }
                run::Event::FollowerDown { id } => {
                    d.set("kind", "follower_down");
                    d.set("id", id as i64);
                }
                run::Event::FollowerDead { id } => {
                    d.set("kind", "follower_dead");
                    d.set("id", id as i64);
                }
                run::Event::FollowerStrike { id, creature, damage } => {
                    d.set("kind", "follower_strike");
                    d.set("id", id as i64);
                    d.set("creature", creature as i64);
                    d.set("damage", damage as i64);
                }
                run::Event::Ordered { order } => {
                    d.set("kind", "ordered");
                    d.set("order", order.name());
                }
                run::Event::PickedUp { id } => {
                    d.set("kind", "picked_up");
                    d.set("id", id as i64);
                }
                run::Event::SetDown { id } => {
                    d.set("kind", "set_down");
                    d.set("id", id as i64);
                }
                run::Event::Dropped { source, id } => {
                    d.set("kind", "dropped");
                    d.set("source", source as i64);
                    d.set("id", id.as_str());
                }
                run::Event::Overtime { minute, packs, patrols } => {
                    d.set("kind", "overtime");
                    d.set("minute", minute as i64);
                    d.set("packs", packs as i64);
                    d.set("patrols", patrols as i64);
                }
                run::Event::Raiders { gang, n, x, y } => {
                    d.set("kind", "raiders");
                    d.set("gang", gang as i64);
                    d.set("n", n as i64);
                    d.set("x", x as i64);
                    d.set("y", y as i64);
                }
                run::Event::Raided { id, source, taken } => {
                    d.set("kind", "raided");
                    d.set("id", id as i64);
                    d.set("source", source as i64);
                    d.set("taken", taken as i64);
                }
                run::Event::WaterRose { level, tiles } => {
                    d.set("kind", "water_rose");
                    d.set("level", level as i64);
                    let w = s.map.w;
                    d.set("tiles", &PackedInt32Array::from(tiles.iter().map(|&(x, y)| (y as usize * w + x as usize) as i32).collect::<Vec<i32>>().as_slice()));
                }
            }
            out.push(&d.to_variant());
        }
        out
    }

    // ------------------------------------------------------------ senses

    #[func]
    fn memory_bits(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.fog.memory_bits().as_slice()))
    }

    #[func]
    fn light(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.light.level.as_slice()))
    }

    #[func]
    fn fog(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.fog.state.as_slice()))
    }

    #[func]
    fn noise(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.noise.level.as_slice()))
    }

    /// RGB8 per tile for the fog shader: R light*17, G fog (0/128/255), B noise*17.
    #[func]
    fn overlay_rgb(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.senses.overlay_rgb().as_slice()))
    }

    /// Salted tiles, 1 where salt lies.
    #[func]
    fn salt(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(s.salt.iter().map(|&b| b as u8).collect::<Vec<u8>>().as_slice()))
    }

    #[func]
    fn is_visible(&self, x: i64, y: i64) -> bool {
        self.sim.as_ref().is_some_and(|s| x >= 0 && y >= 0 && (x as usize) < s.map.w && (y as usize) < s.map.h && s.senses.is_visible(x as usize, y as usize))
    }

    /// [visible tiles, remembered+visible tiles, live noise tiles, tick]
    #[func]
    fn sense_stats(&self) -> PackedInt32Array {
        self.sim.as_ref().map_or_else(PackedInt32Array::new, |s| {
            PackedInt32Array::from(&[s.senses.fog.visible_count() as i32, s.senses.fog.seen_count() as i32, s.senses.noise.live_tiles() as i32, s.tick as i32][..])
        })
    }

    // ------------------------------------------------------------ loot and the rig

    /// [{index, x, y, kind, tier, locked, state, count}]
    #[func]
    fn sources(&self) -> VarArray {
        let mut out = VarArray::new();
        if let Some(s) = self.sim.as_ref() {
            for (i, src) in s.loot.sources.iter().enumerate() {
                let mut d = VarDictionary::new();
                d.set("index", i as i64);
                d.set("x", src.x as i64);
                d.set("y", src.y as i64);
                d.set("kind", src.kind.name());
                d.set("tier", src.tier as i64);
                d.set("locked", src.locked);
                d.set("state", source_state(src.state));
                d.set("count", src.items.len() as i64);
                out.push(&d.to_variant());
            }
        }
        out
    }

    #[func]
    fn source_at(&self, x: i64, y: i64) -> i64 {
        self.sim.as_ref().and_then(|s| s.loot.source_at(x.max(0) as usize, y.max(0) as usize)).map_or(-1, |i| i as i64)
    }

    /// The contents of a source as the Runner sees them (unidentified: tier, category, size, weight, no name).
    #[func]
    fn source_items(&self, index: i64) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        let Some(src) = s.loot.sources.get(index.max(0) as usize) else { return out };
        for (k, it) in src.items.iter().enumerate() {
            out.push(&item_view(&s.table, k, it).to_variant());
        }
        out
    }

    /// The backpack as placed items: [{k, index, id, label, tier, category, w, h, weight, condition, ident, x, y}]
    #[func]
    fn rig_items(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.sim.as_ref() else { return out };
        for (k, (_, p)) in s.runner.rig.backpack.placed().into_iter().enumerate() {
            let mut d = item_view(&s.table, k, &p.item);
            d.set("x", p.x as i64);
            d.set("y", p.y as i64);
            out.push(&d.to_variant());
        }
        out
    }

    /// Quick slots as item ids ("" when empty).
    #[func]
    fn quick_slots(&self) -> PackedStringArray {
        let Some(s) = self.sim.as_ref() else { return PackedStringArray::new() };
        PackedStringArray::from(s.runner.rig.quick.iter().map(|q| GString::from(q.as_ref().map_or("", |it| s.table.def(it.def).id.as_str()))).collect::<Vec<_>>().as_slice())
    }

    /// The rig serialised (postcard) for the Stash to import on extraction.
    #[func]
    fn rig_export(&self) -> PackedByteArray {
        self.sim.as_ref().map_or_else(PackedByteArray::new, |s| PackedByteArray::from(postcard::to_allocvec(&s.runner.rig).unwrap_or_default().as_slice()))
    }

    /// [sources, open, empty, opened containers, pity counter, longest dry, creatures alive]
    #[func]
    fn run_stats(&self) -> PackedInt32Array {
        self.sim.as_ref().map_or_else(PackedInt32Array::new, |s| {
            let open = s.loot.sources.iter().filter(|x| x.state == run::SourceState::Open).count() as i32;
            let empty = s.loot.sources.iter().filter(|x| x.state == run::SourceState::Empty).count() as i32;
            PackedInt32Array::from(&[s.loot.sources.len() as i32, open, empty, s.loot.opened as i32, s.loot.pity_n as i32, s.loot.longest_dry as i32, s.creatures.len() as i32][..])
        })
    }
}
