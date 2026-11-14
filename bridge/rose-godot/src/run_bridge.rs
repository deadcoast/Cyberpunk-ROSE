//! `RoseRun`: one run on a Board route (Full Design v3.1, the run as one Rust simulation) as the Run screen plays it:
//! the route assembled from the chunk library, the creatures, the fog, light and noise, the loot, the outcome.

use super::*;
use crate::stash_bridge::table;
use items::{ItemInstance, ItemTable, PriceContext};

fn loot_tables() -> run::LootTables {
    data().loot.clone()
}

fn creature_defs() -> run::CreatureFile {
    data().creatures.clone()
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
    /// The process's chunk library (`game::data`).
    fn ensure_lib(&self) -> &'static run::ChunkLib {
        &data().chunks
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
