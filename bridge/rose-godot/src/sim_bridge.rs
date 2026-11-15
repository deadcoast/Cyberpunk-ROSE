//! `RoseSim`: the game's city as Godot reads and drives it. It owns the one `game::World` the headless proofs play
//! (Phase G, PAC-101) and the bare cities of the scale proof and the tests: the founding, the tick, the shift's end,
//! the Mat's arrays for the views, and the colony's verbs. The rules are the World's.

use super::*;

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
pub struct RoseSim {
    base: Base<RefCounted>,
    /// The game's city: the one `game::World` `rose` plays headless (Phase G, PAC-101). Every rule runs inside it:
    /// the bridge founds it, steps it, ends its shifts and reads it for the screens, and holds no rule of its own.
    pub(crate) world: Option<game::World>,
    /// A city with no colony and no people: the scale proof (`start_scale`) and the tests' empty maps
    /// (`start_empty`). The screens that draw the Mat read it as they read the World's city.
    bare: Option<City>,
    last_step_us: i64,
    /// New Run's choices waiting for `found` (mode, difficulty, strain code), and the profile New Game+ carries
    new_run: Option<(story::Mode, story::Difficulty, Option<save::StrainCode>)>,
    carry: Option<save::Profile>,
}

impl RoseSim {
    /// The city the Mat views draw: the World's, else the bare one.
    pub(crate) fn city(&self) -> Option<&City> {
        self.world.as_ref().map(|w| &w.city).or(self.bare.as_ref())
    }

    pub(crate) fn city_mut(&mut self) -> Option<&mut City> {
        match self.world.as_mut() {
            Some(w) => Some(&mut w.city),
            None => self.bare.as_mut(),
        }
    }
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
        let Some(w) = self.world.as_ref() else { return GString::from("no city") };
        let c = &w.city;
        GString::from(game::meta::strain_code(&c.sim, &w.colony, w.campaign.seed).encode().as_str())
    }

    /// Whether a shift (the colony's day counter) is a Flood shift: the day's third, its first too in Undercroft.
    #[func]
    fn flood_shift(&self, day: i64) -> bool {
        let Some(w) = self.world.as_ref() else { return false };
        Some(&w.threats).is_some_and(|t| t.is_flood_shift(day.max(1) as u32))
    }

    /// The scale proof (Phase A): a grown stratum with `agents` synthetic slimos and synthetic market hauls. A load
    /// test of the arrays, not the game's city (Full Design v3.11, The city's clock).
    #[func]
    fn start_scale(&mut self, seed: i64, agents: i64, grow_ticks: i64) {
        self.world = None;
        self.bare = Some(logistics::scale_scenario(seed as u64, agents.max(0) as usize, grow_ticks.max(0) as u64));
    }

    /// The game's city, founded as `rose` founds it (`game::World::new_run`): the Mat steered to the first grate,
    /// the Heart, a Route Board, a Bench, a Pit on every Relay, the founders and their Bunkhouse, the starting haul,
    /// the surface's factions, the campaign on New Run's choices (`configure_run`; New Game+ carries the profile's
    /// laws and Scrapdex). Once: a city already founded stays.
    #[func]
    fn found(&mut self, seed: i64) {
        if self.world.is_some() {
            return;
        }
        let (mode, difficulty, code) = self.new_run.take().unwrap_or((story::Mode::Campaign, story::Difficulty::Culvert, None));
        let n = game::meta::NewRun { seed: seed as u32, mode, difficulty, code };
        let profile = self.carry.take().unwrap_or_default();
        self.bare = None;
        self.world = Some(game::World::new_run(&n, data().items.clone(), &profile));
    }

    /// The city's stratum: the highest the Mat has reached (Full Design v3.19, The stack and the breach).
    #[func]
    fn city_stratum(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        Some(&w.threats).map_or(1, |t| t.stratum as i64)
    }

    /// The Route Board's three strata: one below the city's to one above (Full Design v3.19).
    #[func]
    fn board_strata(&self) -> PackedInt32Array {
        let Some(w) = self.world.as_ref() else { return PackedInt32Array::new() };
        let n = Some(&w.city).map_or(6, |c| c.sim.strata);
        PackedInt32Array::from(logistics::board_strata(self.city_stratum() as u8, n).map(|s| s as i32).collect::<Vec<_>>().as_slice())
    }

    /// The stack's strata for the views: [{n, name, short, region, ceiling (row), rows, reached}] from the Sump up.
    #[func]
    fn strata(&self) -> Array<VarDictionary> {
        let mut out = Array::new();
        let Some(c) = self.city() else { return out };
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
        let Some(w) = self.world.as_ref() else { return Array::new() };
        let mut out = Array::new();
        let c = &w.city;
        let held: Vec<String> = w.stash.handles().iter().map(|(_, _, it)| w.table.def(it.def).id.clone()).collect();
        let table = Some(&w.table);
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
        let Some(w) = self.world.as_mut() else { return "no city".into() };
        match game::breach::open_with_key(&mut w.city, Some(&mut w.threats), &mut w.stash, &w.table, tile.max(0) as usize, &mut w.log) {
            Ok(_) => "".into(),
            Err(e) => e.as_str().into(),
        }
    }

    /// Fresh stratum with a Heart at (x, y) and no city; feed it to make it grow.
    #[func]
    fn start_empty(&mut self, seed: i64, w: i64, h: i64, heart_x: i64, heart_y: i64) {
        let mut sim = MatSim::new(w.max(16) as usize, h.max(8) as usize, seed as u64, Genome::default());
        sim.seed_heart(heart_x as usize, heart_y as usize);
        self.world = None;
        self.bare = Some(City::game(sim, seed as u64));
    }

    /// One city tick (`game::World::tick`): slimos walk, cargo rides and lands, the work clock, the threats, Ivan,
    /// the Mat on its own clock. A bare city only steps its Mat and its movers.
    #[func]
    fn step(&mut self) -> i64 {
        let t0 = std::time::Instant::now();
        let changed = match (self.world.as_mut(), self.bare.as_mut()) {
            (Some(w), _) => {
                w.tick();
                w.city.sim.changed.len() as i64
            }
            (None, Some(c)) => {
                c.step();
                c.sim.changed.len() as i64
            }
            _ => 0,
        };
        self.last_step_us = t0.elapsed().as_micros() as i64;
        changed
    }

    /// Where the game's data came from: "<file> disk" or "<file> compiled" for each of the eleven files `game::data`
    /// loads, then "ERROR data/<file>: <why>" for each file of `data/` that did not parse (Phase G, PAC-101).
    #[func]
    fn data_report(&self) -> PackedStringArray {
        let lines: Vec<GString> = DATA_REPORT.get().map(|r| r.iter().map(|l| GString::from(l.as_str())).collect()).unwrap_or_default();
        PackedStringArray::from(lines.as_slice())
    }

    /// The city's digest split by part ("whole", "mat", "structures", "stash", "roster", "log", "surface"), each as
    /// "<part> <16 hex digits>": the same lines `rose idle` prints for the headless World (Phase G, PAC-101). Empty
    /// until the Stash and the roster are bound.
    #[func]
    fn digest_split(&self) -> PackedStringArray {
        let Some(w) = self.world.as_ref() else { return PackedStringArray::new() };
        let lines: Vec<GString> = game::city::digest_split(&w.city, &w.colony, &w.stash, &w.ledger, &w.roster, &w.log, Some(&w.economy)).iter().map(|(n, d)| GString::from(format!("{n} {d:016x}").as_str())).collect();
        PackedStringArray::from(lines.as_slice())
    }

    // ------------------------------------------------------------ pressure (Full Design v3.14)

    /// The bodies on the map: role, x, y, hp, max_hp, lamp (tiles of white light), oathkeeper.
    #[func]
    fn threat_bodies(&self) -> VarArray {
        let Some(w) = self.world.as_ref() else { return VarArray::new() };
        let mut out = VarArray::new();
        let Some(t) = Some(&w.threats) else { return out };
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
        let Some(w) = self.world.as_ref() else { return VarArray::new() };
        let mut out = VarArray::new();
        let (Some(t), Some(c)) = (Some(&w.threats), Some(&w.city)) else { return out };
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
            let e = Some(&w.economy);
            d.set("faction", e.and_then(|e| e.faction_at(z.grate).map(|f| e.name(f).to_string())).unwrap_or_default().as_str());
            d.set("exact", e.is_some_and(|e| e.s.intel > 0));
            out.push(&d.to_variant());
        }
        out
    }

    /// Tiles the Flood has under water now.
    #[func]
    fn flooded_cells(&self) -> PackedInt32Array {
        let Some(w) = self.world.as_ref() else { return PackedInt32Array::new() };
        Some(&w.threats).map_or(PackedInt32Array::new(), |t| PackedInt32Array::from(t.flooded().map(|i| i as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// Tiles with bleach on them.
    #[func]
    fn stain_cells(&self) -> PackedInt32Array {
        let Some(w) = self.world.as_ref() else { return PackedInt32Array::new() };
        Some(&w.threats).map_or(PackedInt32Array::new(), |t| PackedInt32Array::from((0..t.stain.len()).filter(|&i| t.stain[i] > 0).map(|i| i as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// A Bloom's tissue.
    #[func]
    fn bloom_cells(&self) -> PackedInt32Array {
        let Some(w) = self.world.as_ref() else { return PackedInt32Array::new() };
        Some(&w.city).map_or(PackedInt32Array::new(), |c| PackedInt32Array::from((0..c.sim.hy.len()).filter(|&i| c.sim.is_bloom(i)).map(|i| i as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// Oathkeepers the shift's run killed: their Attention counts in the Heart's zone at the shift's end; raiders it
    /// killed count toward the clear-raiders contracts in hand (Economy).
    #[func]
    fn add_run_kills(&mut self, oathkeepers: i64, raiders: i64) {
        let Some(w) = self.world.as_mut() else { return };
        w.run_kills += oathkeepers.max(0) as u32;
        if let Some(e) = Some(&mut w.economy) {
            e.raider_kills(raiders.max(0) as u32);
        }
    }

    /// What the run killed, by creature id: the first of each is a trophy (Full Design v3.18, Collections).
    #[func]
    fn add_run_trophies(&mut self, killed: PackedStringArray, day: i64) -> PackedStringArray {
        let Some(w) = self.world.as_mut() else { return PackedStringArray::new() };
        let mut won = PackedStringArray::new();
        for k in killed.as_slice() {
            if let Some(name) = w.colony.progress.collections.trophy("kill", &k.to_string(), day.max(1) as u32) {
                w.log.push(w.city.tick, game::Class::Landing, format!("Trophy: {name}"));
                won.push(name.as_str());
            }
        }
        won
    }

    /// The run's flags from the city (Full Design v3.18, Scav yield): the curio sets' and Conscription's cost.
    #[func]
    fn run_flags(&self) -> PackedStringArray {
        let Some(w) = self.world.as_ref() else { return PackedStringArray::new() };
        let city = &w.city;
        let mut a = PackedStringArray::new();
        for f in game::grow::run_flags(&w.colony, Some(&w.threats), &city.sim) {
            a.push(f.as_str());
        }
        a
    }

    /// Development only (the shots and the play script): set a gene, as a run of feeds would leave it.
    #[func]
    fn debug_gene(&mut self, gene: i64, value: f64) {
        if let Some(c) = self.city_mut() {
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
        let Some(c) = self.city_mut() else { return };
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
        let Some(w) = self.world.as_mut() else { return };
        // the HP nodes raise what stands by their share, as researching them does
        let defs = &w.defs;
        let before: Vec<i32> = w.colony.structures.iter().map(|s| w.colony.hp_max(&defs, s)).collect();
        w.colony.progress.tech = progress::Tech::all();
        let after: Vec<i32> = w.colony.structures.iter().map(|s| w.colony.hp_max(&defs, s)).collect();
        for (s, (b, a)) in w.colony.structures.iter_mut().zip(before.into_iter().zip(after)) {
            s.hp = s.hp * a / b.max(1);
        }
    }

    /// Development only (the play script): the Mat's proposal round a jam on the first strand tile that has ground
    /// round it, as a jam standing a minute on a bred strain makes it (game::handoff).
    #[func]
    fn debug_proposal(&mut self) -> i64 {
        let Some(w) = self.world.as_mut() else { return 0 };
        let c = &mut w.city;
        let (mw, mh) = (c.sim.w, c.sim.h);
        let saved = c.jam.clone();
        let mut path = None;
        for i in mw..mw * (mh - 1) {
            let hy_at = |j: usize| matches!(c.sim.hy[j], mat::sim::hy::CORD | mat::sim::hy::NODE);
            if !hy_at(i) || c.sim.sig[i] <= 0.0 || i % mw == 0 || i % mw + 1 == mw || !(hy_at(i - 1) && hy_at(i + 1)) {
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
            w.colony.proposal = p;
            w.colony.proposal_confirmed = false;
        }
        n
    }

    /// Development only (the play script): set a built structure's HP, as a flood or a flamer would leave it.
    #[func]
    fn debug_hp(&mut self, structure: i64, hp: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let Some(s) = w.colony.get_mut(structure as u32) else { return false };
        s.hp = hp.clamp(1, i32::MAX as i64) as i32;
        true
    }

    /// Development only (the screenshot script): send a stage ("patrol", "sweep", "cleansing") down a zone's grate now.
    #[func]
    fn debug_send(&mut self, stage: GString, zone: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let defs = &w.defs;
        let (Some(t), Some(c)) = (Some(&mut w.threats), Some(&w.city)) else { return false };
        let Some(k) = t.rules.stage.iter().position(|st| st.id == stage.to_string()) else { return false };
        if zone < 0 || zone as usize >= t.zones.len() {
            return false;
        }
        let notes = t.send(c, &w.colony, &defs, k, zone as usize);
        game::pressure::note(&mut w.log, c.tick, notes);
        true
    }

    /// This shift's Flood roll for a run on it (0-3 tiles before the Lungs), -1 when it is not a Flood shift.
    #[func]
    fn flood_roll(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        Some(&w.threats).and_then(|t| t.flood_roll()).map_or(-1, |r| r as i64)
    }

    /// The next shift's Flood roll when it is a Flood shift (a run's overtime runs into it), else -1.
    #[func]
    fn next_flood_roll(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        let Some(t) = Some(&w.threats) else { return -1 };
        let next = t.shift + 1;
        if t.is_flood_shift(next) { t.roll_for(next) as i64 } else { -1 }
    }

    #[func]
    fn width(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        Some(&w.city).map_or(0, |c| c.sim.w as i64)
    }

    #[func]
    fn height(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        Some(&w.city).map_or(0, |c| c.sim.h as i64)
    }

    #[func]
    fn tick(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        Some(&w.city).map_or(0, |c| c.tick as i64)
    }

    /// Substrate per cell (see matsim::sim::sub), row-major.
    #[func]
    fn substrate(&self) -> PackedByteArray {
        self.city().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.sim.sub.as_slice()))
    }

    /// Hyphal state per cell (0 empty, 1 tip, 2 cord, 3 node, 4 necrotic).
    #[func]
    fn hyphae(&self) -> PackedByteArray {
        self.city().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.sim.hy.as_slice()))
    }

    /// Signal 0..1 per cell.
    #[func]
    fn signal(&self) -> PackedFloat32Array {
        self.city().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.sig.as_slice()))
    }

    /// Nutrient 0..1 per cell.
    #[func]
    fn nutrient(&self) -> PackedFloat32Array {
        self.city().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.n.as_slice()))
    }

    /// Cell indices whose hyphal state changed on the last step.
    #[func]
    fn changed_cells(&self) -> PackedInt32Array {
        self.city().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.sim.changed.iter().map(|&i| i as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Slimo positions as cell indices.
    #[func]
    fn agent_positions(&self) -> PackedInt32Array {
        self.city().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.pos.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Which slimo each mover is (roster ids; the scale proof's are indices).
    #[func]
    fn agent_ids(&self) -> PackedInt32Array {
        self.city().map_or_else(PackedInt32Array::new, |c| PackedInt32Array::from(c.ids.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// Each mover's depth in `agent_ids` order, 1 to 5 for D1 to D5 (2 where the roster holds no record: the scale
    /// proof's crowd), so the crowd draws its sprites from one packed array instead of a record a slimo a tick
    /// (Phase G, PAC-101; CURRENT-STATE SC-05).
    #[func]
    fn agent_depths(&self) -> PackedByteArray {
        let Some(c) = self.city() else { return PackedByteArray::new() };
        let roster = self.world.as_ref().map(|w| &w.roster);
        let depth = |id: u32| -> u8 { roster.and_then(|r| r.get(id)).map_or(2, |r| r.depth.index() as u8 + 1) };
        PackedByteArray::from(c.ids.iter().map(|&id| depth(id)).collect::<Vec<u8>>().as_slice())
    }

    /// Slimo states: the roster's 0 at home, 3 going home, 4 going to work, 5 at work (the scale proof's 1 to market,
    /// 2 carrying to a Pit).
    #[func]
    fn agent_states(&self) -> PackedByteArray {
        self.city().map_or_else(PackedByteArray::new, |c| PackedByteArray::from(c.state.as_slice()))
    }

    /// Cargo packet positions as cell indices.
    #[func]
    fn packet_positions(&self) -> PackedInt32Array {
        self.city().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.pk_pos.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    /// Which shipment each packet carries, in packet order: the id the view follows a packet by between ticks.
    #[func]
    fn packet_ids(&self) -> PackedInt32Array {
        self.city().map_or_else(PackedInt32Array::new, |c| PackedInt32Array::from(c.pk_cargo.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice()))
    }

    /// The tier rank (0 Common … 4 Relic) of the item each packet carries, in packet order; the Street view's pulse.
    #[func]
    fn packet_tiers(&self) -> PackedByteArray {
        let Some(c) = self.city() else { return PackedByteArray::new() };
        // the scale proof's synthetic hauls carry no item: Common
        let Some(w) = self.world.as_ref() else { return PackedByteArray::from(vec![0u8; c.pk_pos.len()].as_slice()) };
        let t = &w.table;
        let v: Vec<u8> = c
            .pk_cargo
            .iter()
            .map(|id| w.ledger.in_flight.iter().find(|s| s.id == *id).map_or(0, |s| game::cargo::TIERS.iter().position(|x| *x == t.def(s.item.def).tier).unwrap_or(0) as u8))
            .collect();
        PackedByteArray::from(v.as_slice())
    }

    /// The Stack view, one pixel a tile, RGB (render::stack_view: the Mat dim, traffic as light, a jam — a full tile
    /// with cargo waiting to enter it — Chalk white, consumers Acid, markets Hot Pink).
    #[func]
    fn stack_image(&self) -> PackedByteArray {
        self.city().map_or_else(PackedByteArray::new, |c| {
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
        self.city().map_or_else(PackedInt32Array::new, |c| {
            PackedInt32Array::from(c.traffic().iter().map(|&t| t as i32).collect::<Vec<i32>>().as_slice())
        })
    }

    #[func]
    fn sites(&self) -> VarDictionary {
        let Some(w) = self.world.as_ref() else { return VarDictionary::new() };
        let mut d = VarDictionary::new();
        if let Some(c) = Some(&w.city) {
            let to_arr = |v: &Vec<usize>| PackedInt32Array::from(v.iter().map(|&p| p as i32).collect::<Vec<i32>>().as_slice());
            // the Pits: the Heart first, then every built Pit (the scale proof: the Heart and the Relays)
            let pits: Vec<usize> = if c.synthetic() {
                c.pits.clone()
            } else {
                let defs = &w.defs;
                w.colony.pits(&defs).into_iter().filter_map(|sid| w.colony.cargo_cell(&defs, &c.sim, sid)).collect()
            };
            d.set("markets", &to_arr(&c.markets));
            d.set("markets_reached", &PackedByteArray::from(c.market_cells.iter().map(|m| m.is_some() as u8).collect::<Vec<u8>>().as_slice()));
            // each market by its grate's number and the faction whose street it is (Economy)
            let names: Vec<GString> = (0..c.markets.len())
                .map(|k| {
                    let who = Some(&w.economy).and_then(|e| e.faction_at(c.markets[k]).map(|f| e.name(f).split(' ').next().unwrap_or("").to_uppercase())).unwrap_or_default();
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
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        if !Self::inside(&w.city, x, y) {
            return GString::from("off the map");
        }
        let Some(n) = game::store::take_bait(&mut w.stash, &w.table) else { return GString::from("bait takes a Common Chrome from the Stash: none") };
        w.city.sim.bait(x as usize, y as usize, n / 9.0);
        decide(w, format!("bait at {x},{y}"));
        GString::new()
    }

    /// The colony strain's gene value (0..1) by index (mat::genome constants); the Run reads Lume (4).
    #[func]
    fn gene(&self, index: i64) -> f64 {
        let Some(w) = self.world.as_ref() else { return 0.0 };
        Some(&w.city).map_or(0.5, |c| c.sim.genome.0.get(index as usize).copied().unwrap_or(0.5) as f64)
    }

    /// All ten genes, 0..1.
    #[func]
    fn genes(&self) -> PackedFloat32Array {
        self.city().map_or_else(PackedFloat32Array::new, |c| PackedFloat32Array::from(c.sim.genome.0.as_slice()))
    }

    /// Stock sitting in a Pit (nutrient units), by index among the city's Pits (0 is the Heart).
    #[func]
    fn pit_stock(&mut self, pit: i64) -> f64 {
        let cells = self.sites().get("pits").and_then(|v| v.try_to::<PackedInt32Array>().ok()).unwrap_or_default();
        let Some(c) = self.city() else { return 0.0 };
        let Some(&i) = cells.as_slice().get(pit.max(0) as usize) else { return 0.0 };
        let (i, w) = (i as usize, c.sim.w);
        c.sim.pits.get(&(i % w, i / w)).copied().unwrap_or(0.0) as f64
    }

    /// Cells of growth one unit of nutrient buys at the current genome (the Stash's "feed now").
    #[func]
    fn cells_per_nutrient(&self) -> f64 {
        let Some(w) = self.world.as_ref() else { return 0.0 };
        Some(&w.city).map_or(4.0, |c| c.sim.cells_per_nutrient() as f64)
    }

    /// Tiles of Mat a feed of `nutrient` grows as it lands (translocation; the rest is the Pit's stock).
    #[func]
    fn feed_growth(&self, nutrient: f64) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        Some(&w.city).map_or(0, |c| c.sim.feed_growth(nutrient as f32) as i64)
    }

    #[func]
    fn stats(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        if let Some(c) = self.city() {
            let s = c.sim.stats();
            let cs = c.stats();
            d.set("tick", s.tick as i64);
            // the city's own clock (2 Hz): the HUD's time
            d.set("city_tick", c.tick as i64);
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

    /// A decision for the city's log: see `decide`.
    pub(crate) fn decide(&mut self, what: String) {
        if let Some(w) = self.world.as_mut() {
            decide(w, what);
        }
    }

    fn inside(c: &City, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < c.sim.w && (y as usize) < c.sim.h
    }

    // ------------------------------------------------------------ the colony (Full Design v3.9, Colony mode)

    /// The 24 structure defs for the Build menu.
    #[func]
    fn structure_defs(&mut self) -> VarArray {
        let Some(w) = self.world.as_mut() else { return VarArray::new() };
        let defs = &w.defs;
        let mut a = colony_bridge::defs_array(&defs);
        // what the tree and the organs ask of each (Full Design v3.18): the node it waits on, the organ it stands on
        let f = progress::StrainFile::shipped();
        for (k, d) in defs.structure.iter().enumerate() {
            let mut e: VarDictionary = a.at(k).to();
            e.set("locked", w.colony.progress.tech.structure_locked(&d.id).map_or("", |n| n.name.as_str()));
            e.set("organ", f.organ.iter().find(|o| o.id == d.organ).map_or("", |o| o.name.as_str()));
            e.set("free_on", f.organ.iter().find(|o| o.id == d.free_on).map_or("", |o| o.name.as_str()));
            a.set(k, &e.to_variant());
        }
        a
    }

    /// Every placed structure, ghosts included.
    #[func]
    fn structures(&mut self) -> VarArray {
        let Some(w) = self.world.as_mut() else { return VarArray::new() };
        let defs = &w.defs;
        colony_bridge::structures_array(&w.colony, &defs)
    }

    /// "" when the def may stand here, else why not.
    #[func]
    fn can_place(&mut self, def_id: GString, x: i64, y: i64) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let defs = &w.defs;
        let c = &w.city;
        if x < 0 || y < 0 {
            return GString::from("off the map");
        }
        match w.colony.can_place(&defs, &c.sim, &def_id.to_string(), x as u16, y as u16) {
            Ok(()) => GString::new(),
            Err(e) => GString::from(e.to_string().as_str()),
        }
    }

    /// Place a ghost; -1 when it cannot stand.
    #[func]
    fn place(&mut self, def_id: GString, x: i64, y: i64) -> i64 {
        let Some(w) = self.world.as_mut() else { return 0 };
        let defs = &w.defs;
        let c = &w.city;
        if x < 0 || y < 0 {
            return -1;
        }
        let id = w.colony.place(&defs, &c.sim, &def_id.to_string(), x as u16, y as u16).map(|id| id as i64).unwrap_or(-1);
        if id >= 0 {
            crate::decide(w, format!("{} placed at {x},{y}", defs.get(&def_id.to_string()).map_or("?", |d| d.name.as_str())));
        }
        id
    }

    /// Cancel an unpaid ghost (a paid one is cut and welded already).
    #[func]
    fn cancel_ghost(&mut self, id: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let defs = &w.defs;
        if w.colony.get(id as u32).is_some_and(|s| s.paid) {
            return false;
        }
        w.colony.cancel(&defs, id as u32).is_none() && w.colony.get(id as u32).is_none()
    }

    /// Start an upgrade in place; "" or why not. `population` sets the city tier.
    #[func]
    fn upgrade(&mut self, id: i64, population: i64) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let defs = &w.defs;
        match w.colony.upgrade(&defs, id as u32, colony::Colony::city_tier(population.max(0) as usize)) {
            Ok(_) => {
                let name = w.colony.get(id as u32).map_or("?".into(), |s| defs.structure[s.def].name.clone());
                crate::decide(w, format!("{name} to be upgraded"));
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
        let Some(w) = self.world.as_mut() else { return PackedInt32Array::new() };
        let defs = &w.defs;
        let (a, b) = w.colony.bed_count(&defs);
        PackedInt32Array::from([a, b].as_slice())
    }

    /// slimo id -> structure id of its bed.
    #[func]
    fn homes(&self) -> VarDictionary {
        let Some(w) = self.world.as_ref() else { return VarDictionary::new() };
        let mut d = VarDictionary::new();
        for (slimo, home) in &w.colony.homes {
            d.set(*slimo as i64, *home as i64);
        }
        d
    }

    #[func]
    fn zones(&self) -> VarArray {
        let Some(w) = self.world.as_ref() else { return VarArray::new() };
        colony_bridge::zones_array(&w.colony)
    }

    /// Paint a zone: kind "stockpile" | "hab" | "quarantine" | "kill", cells as indices; -1 when no cell took.
    #[func]
    fn paint_zone(&mut self, kind: GString, cells: PackedInt32Array, categories: PackedStringArray, min_tier: i64, min_condition: i64, anchor: i64) -> i64 {
        let Some(w) = self.world.as_mut() else { return 0 };
        let Some(k) = colony_bridge::parse_zone(&kind.to_string()) else { return -1 };
        let c = &w.city;
        let cells: Vec<u32> = cells.as_slice().iter().filter(|x| **x >= 0).map(|x| *x as u32).collect();
        let id = w.colony.paint_zone(&c.sim, k, &cells, colony_bridge::filter_from(&categories, min_tier, min_condition), if anchor >= 0 { Some(anchor as u32) } else { None }).map(|id| id as i64).unwrap_or(-1);
        if id >= 0 {
            crate::decide(w, format!("{} zone {id} painted", kind));
        }
        id
    }

    #[func]
    fn erase_zone(&mut self, id: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let ok = w.colony.erase_zone(id as u32);
        if ok {
            crate::decide(w, format!("zone {id} erased"));
            w.publish();
        }
        ok
    }

    /// Whether a cell may hold a zone.
    #[func]
    fn zone_cell_ok(&self, cell: i64) -> bool {
        self.city().is_some_and(|c| cell >= 0 && colony::Colony::zone_cell_ok(&c.sim, cell as usize))
    }

    #[func]
    fn districts(&self) -> VarArray {
        let Some(w) = self.world.as_ref() else { return VarArray::new() };
        colony_bridge::districts_array(&w.colony, Some(&w.roster), Some(&w.defs), Some(&w.city.sim))
    }

    // ------------------------------------------------------------ the Charter (Full Design v3.10)

    /// The eight laws and their state on the Charter.
    #[func]
    fn laws(&mut self) -> VarArray {
        let Some(w) = self.world.as_mut() else { return VarArray::new() };
        let laws = &w.laws;
        if w.colony.heart_dist.is_empty() {
            if let Some(c) = Some(&w.city) {
                w.colony.refresh_heart_dist(&c.sim);
            }
        }
        match Some(&w.city) {
            Some(c) => colony_bridge::laws_array(&w.colony, &laws, &c.sim),
            None => VarArray::new(),
        }
    }

    #[func]
    fn authority(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        w.colony.charter.authority as i64
    }

    /// Ivan's Authority state 1-4 (from the Authority earned over the run).
    #[func]
    fn ivan_state(&self) -> i64 {
        let Some(w) = self.world.as_ref() else { return 0 };
        w.colony.charter.ivan_state() as i64
    }

    /// Cords the wave walks a shift (16 + 32 × Conductance).
    #[func]
    fn wave_speed(&self) -> i64 {
        self.city().map(|c| colony::charter::wave_speed(&c.sim) as i64).unwrap_or(0)
    }

    /// The board at the Heart: every signing, repeal and refusal with its shift.
    #[func]
    fn charter_history(&self) -> PackedStringArray {
        let Some(w) = self.world.as_ref() else { return PackedStringArray::new() };
        PackedStringArray::from(w.colony.charter.history.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice())
    }

    /// Sign a law; "" or why not.
    #[func]
    fn sign_law(&mut self, id: GString) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let defs = &w.defs;
        let laws = &w.laws;
        let Some(def) = laws.get(&id.to_string()).cloned() else { return GString::from("no such law") };
        let c = &w.city;
        match w.colony.sign(&defs, &c.sim, &def) {
            Ok(()) => {
                crate::decide(w, format!("{} signed", def.name));
                w.log.bark("law", &[("name", &def.name)]);
                GString::new()
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// Repeal a law; "" or why not.
    #[func]
    fn repeal_law(&mut self, id: GString) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let c = &w.city;
        match w.colony.repeal(&c.sim, &id.to_string()) {
            Ok(()) => {
                crate::decide(w, format!("{id} repealed"));
                GString::new()
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// Refuse a district's petition: "refused", "dissolved" or why not.
    #[func]
    fn refuse_petition(&mut self, district: i64) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        match w.colony.refuse_petition(district as u32) {
            Ok(true) => {
                crate::decide(w, format!("district {district}'s petition refused; it dissolved"));
                GString::from("dissolved")
            }
            Ok(false) => {
                crate::decide(w, format!("district {district}'s petition refused"));
                GString::from("refused")
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// Dissolve a D5 into the Heart's Pit; the line for the report, or why not.
    #[func]
    fn dissolve(&mut self, slimo: i64) -> GString {
        let Some(w) = self.world.as_mut() else { return GString::from("no city") };
        let name = w.roster.get(slimo as u32).map(|r| r.full_name()).unwrap_or_default();
        match w.colony.dissolve(&mut w.roster, &mut w.city.sim, slimo as u32) {
            Ok(line) => {
                w.log.bark("d5_fed", &[("name", &name)]);
                GString::from(line.as_str())
            }
            Err(e) => GString::from(e.as_str()),
        }
    }

    /// A district's job priority (the foreman's, over its members' where higher).
    #[func]
    fn set_district_priority(&mut self, district: i64, job: i64, value: i64) {
        let Some(w) = self.world.as_mut() else { return };
        if let Some(d) = w.colony.districts.iter_mut().find(|d| d.id == district as u32) {
            if (0..9).contains(&job) {
                d.priorities[job as usize] = value.clamp(0, 4) as u8;
            }
        }
    }

    /// The last shift's assignments: [{slimo, job, structure}].
    #[func]
    fn assignments(&mut self) -> VarArray {
        let Some(w) = self.world.as_mut() else { return VarArray::new() };
        let defs = &w.defs;
        let mut out = VarArray::new();
        for (slimo, job, sid) in &w.last.as_ref().map(|o| o.assignments.clone()).unwrap_or_default() {
            let mut e = VarDictionary::new();
            e.set("slimo", *slimo as i64);
            e.set("job", job.name());
            e.set("structure", *sid as i64);
            e.set("structure_name", w.colony.get(*sid).map(|s| defs.structure[s.def].name.as_str()).unwrap_or(""));
            out.push(&e.to_variant());
        }
        out
    }

    /// The shift, whole (`game::World::end_shift`): the route loops, the colony works and pays, the Stash's side,
    /// the cargo, the stockpiles, the roster's records, the breach, the threats and the stalls, the campaign, the
    /// next shift's water; then the next shift's start. `day` is the shift that ends (the colony's day counter). The
    /// report is what landed since the last shift end and what just left; "laid" the recover links' bodies.
    #[func]
    fn colony_end_shift(&mut self, day: i64) -> VarDictionary {
        let Some(w) = self.world.as_mut() else { return VarDictionary::new() };
        w.day = day.max(1) as u32;
        let shift = w.end_shift();
        shift_dict(w, &shift)
    }

    /// A player's shift (`game::World::play_shift`, Full Design v3.21): the city's clock for a shift — the ghosts rise,
    /// the researchers put their points in, the cargo rides — then its end, reported as `colony_end_shift` reports it.
    /// The shift a player spends at home; the play script's "a shift passes".
    #[func]
    fn colony_play_shift(&mut self, day: i64) -> VarDictionary {
        let Some(w) = self.world.as_mut() else { return VarDictionary::new() };
        w.day = day.max(1) as u32;
        let shift = w.play_shift();
        shift_dict(w, &shift)
    }

    /// The movers follow the roster again (a Runner left the map, or came back): the shift's plan as it stands and
    /// the posts it sets (`game::World::movers`). The shift's start does this itself.
    #[func]
    fn refresh_movers(&mut self) {
        if let Some(w) = self.world.as_mut() {
            w.movers();
        }
    }

    // ------------------------------------------------------------ quotas (Full Design v3.11, Cargo: Quotas)

    /// The Pit or Still whose footprint holds this tile, or -1.
    #[func]
    fn consumer_at(&mut self, x: i64, y: i64) -> i64 {
        let Some(w) = self.world.as_mut() else { return 0 };
        let defs = &w.defs;
        for s in &w.colony.structures {
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
        let Some(w) = self.world.as_ref() else { return VarDictionary::new() };
        cargo_bridge::quota_view(&w.colony, &w.defs, &w.city, &w.ledger, &w.table, structure as u32).unwrap_or_default()
    }

    /// Set a Pit's or Still's quota (a Pit's target in nutrient, a Still's in Parts; ceiling 0 Common … 4 Relic).
    #[func]
    fn set_quota(&mut self, structure: i64, target: f64, ceiling: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let defs = &w.defs;
        let ok = w.colony.set_quota(&defs, structure as u32, Some(colony::QuotaSet { target: target.max(0.0) as f32, ceiling: ceiling.clamp(0, 4) as u8 }));
        w.publish();
        if ok {
            crate::decide(w, format!("quota at structure {structure}: keep {target:.1}"));
        }
        ok
    }

    /// Back to the default: off, the foreman's, or a Still's slots.
    #[func]
    fn clear_quota(&mut self, structure: i64) -> bool {
        let Some(w) = self.world.as_mut() else { return false };
        let defs = &w.defs;
        let ok = w.colony.set_quota(&defs, structure as u32, None);
        w.publish();
        ok
    }

}

/// A shift's end as the Colony reads it: the outcome and the people's lines, the Stash's report, the bodies laid.
fn shift_dict(w: &game::World, shift: &game::Shift) -> VarDictionary {
    let mut report = closed_dict(w, &shift.closed);
    report.set("stash_lines", &PackedStringArray::from(shift.stash_lines.iter().map(|s| GString::from(s.as_str())).collect::<Vec<_>>().as_slice()));
    let mut out = colony_bridge::outcome_dict(&shift.outcome, &shift.people_lines, report, &w.defs, &w.colony);
    out.set("laid", &market_bridge::laid_array(&w.laid));
    out
}
