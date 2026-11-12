//! rose — headless runner for the Cyberpunk: ROSE core.
//!
//!   matsim grow      one stratum, one Heart, fed then cut; four snapshots
//!   matsim evidence  four runs on one seed: default, feed cut, Runner, Bramble
//!   matsim scale     grow a stratum with relays, then run thousands of slimos and cargo on it
//!   matsim bench     per-stage timings at several agent counts

use clap::{Parser, Subcommand};
use mat::genome::{Genome, BRANCHING, CONDUCTANCE, VIGOR};
use render::{self, Canvas};
use logistics::City;
use mat::MatSim;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "rose", version, about = "Cyberpunk: ROSE core — headless runner")]
struct Cli {
    /// output folder for PNGs and stats
    #[arg(long, default_value = "out")]
    out: PathBuf,
    /// no ANSI colour
    #[arg(long)]
    plain: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Grow {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        #[arg(long, default_value_t = 600)]
        ticks: u64,
        #[arg(long, default_value_t = 350)]
        cutoff: u64,
    },
    Evidence {
        #[arg(long, default_value_t = 11)]
        seed: u64,
    },
    Scale {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        #[arg(long, default_value_t = 3000)]
        agents: usize,
        #[arg(long, default_value_t = 500)]
        grow: u64,
        #[arg(long, default_value_t = 300)]
        ticks: u64,
    },
    Bench {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        #[arg(long, default_value_t = 300)]
        ticks: u64,
    },
    /// Assemble a run route from the authored chunks and render its map.
    Route {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        #[arg(long, default_value_t = 1)]
        stratum: u8,
        /// chunk count; 0 = from the seed (8-14)
        #[arg(long, default_value_t = 0)]
        chunks: usize,
        /// also sweep this many seeds and report the route statistics
        #[arg(long, default_value_t = 0)]
        sweep: u64,
        /// open every container on this many seeds per stratum and report the loot distribution
        #[arg(long, default_value_t = 0)]
        loot: u64,
    },
    /// Play seeded runs with the cautious bot and check the reward cadence against the design's budget.
    Cadence {
        /// runs per stratum
        #[arg(long, default_value_t = 20)]
        runs: u64,
        /// strata to play, e.g. 1,2,3
        #[arg(long, default_value = "1,2,3")]
        strata: String,
        /// first route seed; runs use seed, seed+1, ...
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// write the aggregate and every run's report here (markdown)
        #[arg(long, default_value = "docs/telemetry/cadence.md")]
        report: PathBuf,
        /// also write each run's full log into this folder
        #[arg(long)]
        logs: Option<PathBuf>,
    },
    /// Play the whole game headless: a bot run each shift, the steward's standing orders, the city's shift in core;
    /// assert the Phase C proof and judge the merged run-and-city landing cadence.
    City {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        #[arg(long, default_value_t = 20)]
        shifts: u32,
        /// the report (markdown)
        #[arg(long, default_value = "docs/telemetry/city.md")]
        report: PathBuf,
        /// the merged log, every event of every shift on one clock
        #[arg(long, default_value = "out/city.log")]
        log: PathBuf,
    },
    /// Phase D's proof: a Cleansing down the first grate of a city its steward has played, twice from the same state,
    /// as it stands and with two Gates and a manned Turret on the way; the survival spread.
    Cleansing {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        /// shifts the steward plays before the Cleansing
        #[arg(long, default_value_t = 10)]
        shifts: u32,
        #[arg(long, default_value = "docs/telemetry/cleansing.md")]
        report: PathBuf,
    },
    /// Phase D's proof of the economy: the steward trades its surplus for the shifts, every demand's walk inside its
    /// band every day; then the seed again with the steward feeding its Chrome to the ground, and when the nearer grate
    /// is reached each way.
    Economy {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        #[arg(long, default_value_t = 30)]
        shifts: u32,
        #[arg(long, default_value = "docs/telemetry/economy.md")]
        report: PathBuf,
    },
    /// Phase E's proof: the campaign headless at the bot's pace, chapter 1 to an ending, for each aim's script.
    Campaign {
        #[arg(long, default_value_t = 11)]
        seed: u64,
        /// garden, hive, world_eater, accord, scattered, or all
        #[arg(long, default_value = "all")]
        aim: String,
        #[arg(long, default_value_t = 200)]
        shifts: u32,
        #[arg(long, default_value = "docs/telemetry/campaign.md")]
        report: PathBuf,
    },
}

// ---------------------------------------------------------------- terminal
struct Term {
    colour: bool,
    tty: bool,
}

impl Term {
    fn dim(&self, s: &str) -> String {
        if self.colour { format!("\x1b[2m{s}\x1b[0m") } else { s.to_string() }
    }
    fn hi(&self, s: &str) -> String {
        if self.colour { format!("\x1b[1;32m{s}\x1b[0m") } else { s.to_string() }
    }
    fn head(&self, title: &str, sub: &str) {
        println!();
        println!("  {}  {}", self.hi("MATSIM"), self.dim("Cyberpunk: ROSE core"));
        println!("  {}  {}", title, self.dim(sub));
        println!("  {}", self.dim(&"-".repeat(72)));
    }
    fn bar(&self, label: &str, done: u64, total: u64, tail: &str) {
        if !self.tty && done != total {
            return; // piped output: only the final state of each bar
        }
        let width = 28usize;
        let filled = ((done as f64 / total.max(1) as f64) * width as f64) as usize;
        let bar: String = "#".repeat(filled) + &".".repeat(width - filled);
        print!("\r  {:<9} [{}] {:>5}/{:<5} {}", label, bar, done, total, self.dim(tail));
        let _ = std::io::stdout().flush();
    }
    fn bar_done(&self) {
        println!();
    }
    fn row(&self, cols: &[(&str, usize)]) {
        let mut line = String::from("  ");
        for (c, w) in cols {
            line.push_str(&format!("{:>w$}", c, w = *w));
            line.push_str("  ");
        }
        println!("{line}");
    }
    fn hrow(&self, cols: &[(&str, usize)]) {
        let mut line = String::from("  ");
        for (c, w) in cols {
            line.push_str(&format!("{:>w$}", c, w = *w));
            line.push_str("  ");
        }
        println!("{}", self.dim(&line));
    }
    fn wrote(&self, p: &Path, w: usize, h: usize) {
        println!("  {} {}  {}", self.dim("wrote"), p.display(), self.dim(&format!("{w}x{h}")));
    }
}

fn stack_frames(frames: &[Canvas]) -> Canvas {
    let w = frames.iter().map(|f| f.w).max().unwrap_or(0);
    let gap = 6;
    let h: usize = frames.iter().map(|f| f.h).sum::<usize>() + gap * frames.len().saturating_sub(1);
    let mut out = Canvas::new(w, h, render::pal::SUMP);
    let mut y = 0;
    for f in frames {
        for yy in 0..f.h {
            let src = &f.px[yy * f.w * 3..(yy + 1) * f.w * 3];
            let dst = ((y + yy) * out.w) * 3;
            out.px[dst..dst + f.w * 3].copy_from_slice(src);
        }
        y += f.h + gap;
    }
    out
}

fn fed(sim: &mut MatSim, cutoff: u64, pit2: bool) {
    if sim.tick < cutoff {
        sim.feed(96, 25, 0.60);
        if pit2 && sim.tick >= 120 {
            sim.feed(150, 12, 0.30);
        }
    }
}

// ---------------------------------------------------------------- commands
fn cmd_grow(t: &Term, out: &Path, seed: u64, ticks: u64, cutoff: u64) {
    t.head("grow", &format!("seed {seed}  stratum 192x48  feed 0.6/tick at the Heart to t={cutoff}, second Pit from t=120"));
    let mut sim = MatSim::new(192, 48, seed, Genome::default());
    sim.seed_heart(96, 25);
    let want = [40u64, 150, cutoff, ticks];
    let mut frames = Vec::new();
    let mut rows: Vec<(u64, mat::Stats)> = Vec::new();
    for tick in 1..=ticks {
        fed(&mut sim, cutoff, true);
        let s = sim.step();
        if tick % 10 == 0 || tick == ticks {
            t.bar("growing", tick, ticks, &format!("cords {:>4}  nodes {:>3}  reach {:>3}", s.cords, s.nodes, s.reach));
        }
        if want.contains(&tick) {
            let pits: Vec<(usize, usize)> = if tick <= cutoff { vec![(96, 25), (150, 12)] } else { vec![] };
            frames.push(render::mat_view(&sim, &pits, 4));
            rows.push((tick, s.clone()));
        }
    }
    t.bar_done();
    t.hrow(&[("t", 5), ("tips", 5), ("cords", 6), ("nodes", 6), ("necrotic", 9), ("reach", 6), ("feed", 10)]);
    for (tick, s) in &rows {
        t.row(&[
            (&tick.to_string(), 5),
            (&s.tips.to_string(), 5),
            (&s.cords.to_string(), 6),
            (&s.nodes.to_string(), 6),
            (&s.necro.to_string(), 9),
            (&s.reach.to_string(), 6),
            (if *tick <= cutoff { "feeding" } else { "cut" }, 10),
        ]);
    }
    let sheet = stack_frames(&frames);
    let p = out.join(format!("grow_seed{seed}.png"));
    sheet.save_png(&p).expect("png");
    t.wrote(&p, sheet.w, sheet.h);
}

fn cmd_evidence(t: &Term, out: &Path, seed: u64) {
    t.head("evidence", &format!("seed {seed}  four runs, same substrate, different genomes and feed schedules"));
    let runs: Vec<(&str, Genome, u64, u64)> = vec![
        ("default, fed to 350", Genome::default(), 350, 350),
        ("default, cut at 350, shown at 600", Genome::default(), 600, 350),
        ("Runner: Vigor .9 Conductance .9", Genome::default().with(VIGOR, 0.9).with(CONDUCTANCE, 0.9), 350, 350),
        ("Bramble: Branching .9", Genome::default().with(BRANCHING, 0.9), 350, 350),
    ];
    let mut frames = Vec::new();
    t.hrow(&[("run", 36), ("cords", 6), ("nodes", 6), ("necrotic", 9), ("reach", 6), ("ms/tick", 8)]);
    for (label, g, ticks, cutoff) in runs {
        let mut sim = MatSim::new(192, 48, seed, g);
        sim.seed_heart(96, 25);
        let t0 = Instant::now();
        let mut s = sim.stats();
        for _ in 1..=ticks {
            fed(&mut sim, cutoff, true);
            s = sim.step();
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
        let pits: Vec<(usize, usize)> = if ticks <= cutoff { vec![(96, 25), (150, 12)] } else { vec![] };
        frames.push(render::mat_view(&sim, &pits, 4));
        t.row(&[
            (label, 36),
            (&s.cords.to_string(), 6),
            (&s.nodes.to_string(), 6),
            (&s.necro.to_string(), 9),
            (&s.reach.to_string(), 6),
            (&format!("{ms:.2}"), 8),
        ]);
    }
    let sheet = stack_frames(&frames);
    let p = out.join(format!("evidence_seed{seed}.png"));
    sheet.save_png(&p).expect("png");
    t.wrote(&p, sheet.w, sheet.h);
}

fn cmd_scale(t: &Term, out: &Path, seed: u64, agents: usize, grow: u64, ticks: u64) {
    t.head("scale", &format!("seed {seed}  {agents} slimos  grow {grow} ticks, then {ticks} ticks of city"));
    let t0 = Instant::now();
    let mut city: City = logistics::scale_scenario(seed, agents, grow);
    let grow_ms = t0.elapsed().as_secs_f64() * 1000.0 / grow.max(1) as f64;
    let cells = city.sim.hyphal_count();
    t.bar("grown", grow, grow, &format!("cells {cells}  nodes {}  markets {}  pits {}", city.sim.stats().nodes, city.markets.len(), city.pits.len()));
    t.bar_done();
    let t1 = Instant::now();
    let mut peak_jam = 0u32;
    let mut last = city.stats();
    for tick in 1..=ticks {
        last = city.step();
        peak_jam = peak_jam.max(last.jammed_tiles);
        if tick % 10 == 0 || tick == ticks {
            t.bar("city", tick, ticks, &format!("carrying {:>5}  cargo {:>4}  delivered {:>6}  jam {:>3}", last.carrying, last.packets, last.deliveries, last.jammed_tiles));
        }
    }
    t.bar_done();
    let city_ms = t1.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
    t.hrow(&[("measure", 26), ("value", 12)]);
    for (k, v) in [
        ("Mat cells", cells.to_string()),
        ("Nodes", city.sim.stats().nodes.to_string()),
        ("Slimos", agents.to_string()),
        ("Deliveries", last.deliveries.to_string()),
        ("Hauls", last.hauls.to_string()),
        ("Peak jammed tiles", peak_jam.to_string()),
        ("Grow ms/tick (CA only)", format!("{grow_ms:.2}")),
        ("City ms/tick (all)", format!("{city_ms:.2}")),
    ] {
        t.row(&[(k, 26), (&v, 12)]);
    }
    let stack = render::stack_view(&city, 4);
    let street = render::street_view(&city, 76, 19, 48, 14, 16);
    let p1 = out.join(format!("scale_stack_seed{seed}.png"));
    let p2 = out.join(format!("scale_street_seed{seed}.png"));
    stack.save_png(&p1).expect("png");
    street.save_png(&p2).expect("png");
    t.wrote(&p1, stack.w, stack.h);
    t.wrote(&p2, street.w, street.h);
}

fn cmd_bench(t: &Term, seed: u64, ticks: u64) {
    t.head("bench", &format!("seed {seed}  {ticks} city ticks per row  release build, one thread"));
    t.hrow(&[("slimos", 8), ("cells", 7), ("ms/tick", 8), ("us/slimo", 9), ("deliveries", 11)]);
    for agents in [1000usize, 3000, 5000, 10000, 20000] {
        let mut city = logistics::scale_scenario(seed, agents, 500);
        let cells = city.sim.hyphal_count();
        let t0 = Instant::now();
        let mut last = city.stats();
        for _ in 0..ticks {
            last = city.step();
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
        t.row(&[
            (&agents.to_string(), 8),
            (&cells.to_string(), 7),
            (&format!("{ms:.2}"), 8),
            (&format!("{:.2}", ms * 1000.0 / agents as f64), 9),
            (&last.deliveries.to_string(), 11),
        ]);
    }
}

fn cmd_route(t: &Term, out: &Path, seed: u64, stratum: u8, chunks: usize, sweep: u64, loot: u64) {
    let lib = run::ChunkLib::from_toml(run::CHUNKS_TOML).expect("chunks.toml");
    t.head("route", &format!("seed {seed}  stratum {stratum}  library {} chunks over strata {:?}", lib.chunks.len(), lib.strata()));
    let lay = if chunks == 0 { run::route(&lib, seed, stratum) } else { run::assemble(&lib, seed, stratum, chunks) }.expect("assemble");
    let m = run::RouteMap::compile(&lib, &lay);
    t.hrow(&[("#", 3), ("chunk", 18), ("kind", 9), ("cell", 8), ("joined", 8)]);
    for (i, p) in lay.placed.iter().enumerate() {
        let c = lay.chunk(&lib, i);
        let joined: String = run::Dir::ALL.iter().filter(|d| p.joined & d.bit() != 0).map(|d| d.glyph()).collect();
        t.row(&[(&i.to_string(), 3), (&c.id, 18), (&c.kind, 9), (&format!("{},{}", p.cx, p.cy), 8), (&joined, 8)]);
    }
    t.row(&[(&m.digest(), 100)]);
    let img = render::route_view(&m, 4);
    let p = out.join(format!("route_s{stratum}_seed{seed}.png"));
    img.save_png(&p).expect("png");
    t.wrote(&p, img.w, img.h);
    // the senses: how dark the route is, what a lantern does, what a sprint step sounds like
    let mut senses = run::Senses::new(&m, 0.5, None);
    let pass: Vec<usize> = (0..m.w * m.h).filter(|&i| run::chunk::substrate_passable(m.substrate[i])).collect();
    let band = |field: &[u8], lo: u8, hi: u8| pass.iter().filter(|&&i| field[i] >= lo && field[i] <= hi).count() * 100 / pass.len().max(1);
    t.hrow(&[("light", 8), ("dark", 7), ("dim", 7), ("lit", 7), ("note", 48)]);
    let st = &senses.light.static_;
    t.row(&[("authored", 8), (&format!("{}%", band(st, 0, 2)), 7), (&format!("{}%", band(st, 3, 6)), 7), (&format!("{}%", band(st, 7, 15)), 7), ("fixtures, grates and the Mat edge only", 48)]);
    senses.runner = (m.entry[1].0 as usize + 8, m.entry[1].1 as usize);
    senses.lume = run::Lume { veins: 2, gear: 6, mat_bonus: 0, dimmed: false, soaked: false };
    senses.tick(&m);
    let lit_seen = senses.visible.iter().filter(|&&v| v).count();
    let lit_exp = (senses.exposure, senses.notice_range());
    let reached = senses.runner_sound(&m, run::Sound::SprintStep, 0);
    let light_img = render::route_light_view(&m, &senses.light, Some(&senses.noise), 4);
    senses.lume = run::Lume { veins: 2, gear: 0, mat_bonus: 0, dimmed: true, soaked: false };
    senses.tick(&m);
    let dark_seen = senses.visible.iter().filter(|&&v| v).count();
    let dark_exp = (senses.exposure, senses.notice_range());
    t.hrow(&[("runner", 22), ("sees", 7), ("exposure", 9), ("noticed", 8)]);
    t.row(&[("lantern on, 8 tiles in", 22), (&lit_seen.to_string(), 7), (&format!("{} {}", lit_exp.0, run::LightField::band(lit_exp.0)), 9), (&format!("{} tiles", lit_exp.1), 8)]);
    t.row(&[("lantern off, veins dimmed", 22), (&dark_seen.to_string(), 7), (&format!("{} {}", dark_exp.0, run::LightField::band(dark_exp.0)), 9), (&format!("{} tiles", dark_exp.1), 8)]);
    t.row(&[(&format!("a sprint step there reaches {reached} tiles; sewerats hear it at level >= {}, scouts at >= {}", run::noise::HEAR_SEWERAT, run::noise::HEAR_SCOUT), 100)]);
    let p = out.join(format!("route_light_s{stratum}_seed{seed}.png"));
    light_img.save_png(&p).expect("png");
    t.wrote(&p, light_img.w, light_img.h);
    if loot > 0 {
        loot_sweep(t, &lib, loot);
    }
    if sweep > 0 {
        t.hrow(&[("stratum", 8), ("routes", 7), ("chunks", 10), ("containers", 11), ("spawns", 8), ("reach min", 10), ("ms/route", 9)]);
        for s in lib.strata() {
            let t0 = Instant::now();
            let (mut chunks_sum, mut cont_sum, mut spawn_sum, mut reach_min) = (0usize, 0usize, 0usize, 1.0f32);
            for sd in 0..sweep {
                let lay = run::route(&lib, sd, s).expect("assemble");
                let m = run::RouteMap::compile(&lib, &lay);
                chunks_sum += m.chunks.len();
                cont_sum += m.containers.len();
                spawn_sum += m.spawns.len();
                reach_min = reach_min.min(m.connectivity());
            }
            let ms = t0.elapsed().as_secs_f64() * 1000.0 / sweep as f64;
            t.row(&[
                (&format!("S{s}"), 8),
                (&sweep.to_string(), 7),
                (&format!("{:.1} avg", chunks_sum as f64 / sweep as f64), 10),
                (&format!("{:.1} avg", cont_sum as f64 / sweep as f64), 11),
                (&format!("{:.1} avg", spawn_sum as f64 / sweep as f64), 8),
                (&format!("{:.1}%", reach_min * 100.0), 10),
                (&format!("{ms:.2}"), 9),
            ]);
        }
    }
}

/// Open every locker, crate and corpse on `n` seeds per stratum with a pry bar (not a first run), and report what
/// the tables pay: value per container, tier mix, notable rate, stratum-to-stratum ratio, longest dry streak.
fn loot_sweep(t: &Term, lib: &run::ChunkLib, n: u64) {
    let tables = run::LootTables::shipped();
    let table = items::ItemTable::from_toml(include_str!("../../../../data/items.toml")).expect("items");
    t.hrow(&[("loot", 6), ("opens", 6), ("mean", 7), ("p50", 6), ("p90", 6), ("x prev", 7), ("C/U/R/B/X %", 20), ("notable", 8), ("dry max", 8)]);
    let mut prev_mean = 0.0f32;
    for stratum in lib.strata() {
        let mut values: Vec<u32> = Vec::new();
        let mut tiers = [0u32; 5];
        let mut notable = 0usize;
        let mut picks = 0usize;
        let mut dry = 0u32;
        for seed in 0..n {
            let lay = run::route(lib, seed, stratum).expect("assemble");
            let m = run::RouteMap::compile(lib, &lay);
            let mut st = run::LootState::new(&m, &tables, seed ^ 0xA5, 1, 1);
            for i in 0..st.sources.len() {
                let Ok(r) = st.open(&tables, &table, i, run::OpenMethod::PryBar) else { continue };
                notable += r.notable;
                let mut v = 0u32;
                for it in &st.sources[i].items {
                    let d = table.def(it.def);
                    tiers[d.tier.index()] += 1;
                    v += run::loot::value_of(d, it);
                    picks += 1;
                }
                values.push(v);
            }
            dry = dry.max(st.longest_dry);
        }
        values.sort_unstable();
        let mean = values.iter().sum::<u32>() as f32 / values.len().max(1) as f32;
        let p = |q: f32| values.get(((values.len() as f32 - 1.0) * q) as usize).copied().unwrap_or(0);
        let total = tiers.iter().sum::<u32>().max(1) as f32;
        let mix: Vec<String> = tiers.iter().map(|&c| format!("{:.0}", c as f32 * 100.0 / total)).collect();
        t.row(&[
            (&format!("S{stratum}"), 6),
            (&values.len().to_string(), 6),
            (&format!("{mean:.1}"), 7),
            (&p(0.5).to_string(), 6),
            (&p(0.9).to_string(), 6),
            (&if prev_mean > 0.0 { format!("{:.2}", mean / prev_mean) } else { "-".into() }, 7),
            (&mix.join("/"), 20),
            (&format!("{:.1}%", notable as f32 * 100.0 / picks.max(1) as f32), 8),
            (&dry.to_string(), 8),
        ]);
        prev_mean = mean;
    }
}

fn main() {
    let cli = Cli::parse();
    std::fs::create_dir_all(&cli.out).expect("out dir");
    use std::io::IsTerminal;
    let tty = std::io::stdout().is_terminal();
    let t = Term { colour: !cli.plain && tty, tty };
    match cli.cmd {
        Cmd::Grow { seed, ticks, cutoff } => cmd_grow(&t, &cli.out, seed, ticks, cutoff),
        Cmd::Evidence { seed } => cmd_evidence(&t, &cli.out, seed),
        Cmd::Scale { seed, agents, grow, ticks } => cmd_scale(&t, &cli.out, seed, agents, grow, ticks),
        Cmd::Bench { seed, ticks } => cmd_bench(&t, seed, ticks),
        Cmd::Route { seed, stratum, chunks, sweep, loot } => cmd_route(&t, &cli.out, seed, stratum, chunks, sweep, loot),
        Cmd::Cadence { runs, strata, seed, report, logs } => cmd_cadence(&t, runs, &strata, seed, &report, logs.as_deref()),
        Cmd::City { seed, shifts, report, log } => cmd_city(&t, seed, shifts, &report, &log),
        Cmd::Cleansing { seed, shifts, report } => cmd_cleansing(&t, seed, shifts, &report),
        Cmd::Economy { seed, shifts, report } => cmd_economy(&t, seed, shifts, &report),
        Cmd::Campaign { seed, aim, shifts, report } => cmd_campaign(&t, seed, &aim, shifts, &report),
    }
    println!();
}

// ---------------------------------------------------------------- cadence
fn median(v: &mut [f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

fn in_band(v: f32, lo: f32, hi: f32) -> &'static str {
    if v >= lo && v <= hi { "ok" } else if v < lo { "fast" } else { "slow" }
}

fn cmd_cadence(t: &Term, runs: u64, strata: &str, seed0: u64, report_path: &Path, logs: Option<&Path>) {
    use run::telemetry::{bot_bands, BOT_FACTOR, DEATH_RATE, LANDING_GAP_S, LOSS_GAP_MAX_S, PULL_GAP_S};
    let (bp, bl, bq) = bot_bands();
    let lib = run::ChunkLib::from_toml(run::CHUNKS_TOML).expect("chunks.toml");
    let table = items::ItemTable::from_toml(items::ITEMS_TOML).expect("items.toml");
    let tables = run::LootTables::shipped();
    let defs = run::CreatureFile::shipped();
    let strata: Vec<u8> = strata.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    t.head("cadence", &format!("{runs} bot runs per stratum {strata:?} from seed {seed0}  player budget: pull {}-{} s, landing {}-{} s, loss under {} s, deaths {:.0}-{:.0} %  |  bot bands (/{}): pull {:.0}-{:.0} s, landing {:.0}-{:.0} s, loss under {:.0} s", PULL_GAP_S.0, PULL_GAP_S.1, LANDING_GAP_S.0, LANDING_GAP_S.1, LOSS_GAP_MAX_S, DEATH_RATE.0 * 100.0, DEATH_RATE.1 * 100.0, BOT_FACTOR, bp.0, bp.1, bl.0, bl.1, bq));
    if let Some(d) = logs {
        std::fs::create_dir_all(d).expect("logs dir");
    }
    let mut md = String::new();
    md.push_str("# Reward cadence — bot runs against the budget\n\n");
    md.push_str(&format!("Written by `rose cadence --runs {runs} --strata {} --seed {seed0}` on {}. The bot (`run::bot`) opens what it sees, takes what fits, fights adjacent threats over 40 % HP, digs deposits it stands beside, extracts when the rig is nearly full, the shift is under 90 s or the body is under 30 %; it walks the lit route and never waits or reads. Player budget (Full Design, The cadence budget): pull gap {}–{} s median, landing gap {}–{} s median, longest quiet between losses under {} s, death rate {:.0}–{:.0} %. The bot is taken as {}× faster than a cautious player (an assumption until a recorded human run calibrates it), so its bands are pull {:.0}–{:.0} s, landing {:.0}–{:.0} s, quiet under {:.0} s; the death band is the player's. The run is verdicted on pulls, pressure and deaths; the landing budget is the whole game's (run and colony together) and is judged on the combined log from Phase C, so the landing gap is reported and not verdicted.\n\n",
        strata.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(","), chrono_free_date(), PULL_GAP_S.0, PULL_GAP_S.1, LANDING_GAP_S.0, LANDING_GAP_S.1, LOSS_GAP_MAX_S, DEATH_RATE.0 * 100.0, DEATH_RATE.1 * 100.0, BOT_FACTOR, bp.0, bp.1, bl.0, bl.1, bq));
    md.push_str("## Aggregate\n\n| stratum | runs | deaths | run length (median) | pull gap median | landing gap median | pulls / run | notables / run | kills / run | damage / run | longest quiet (median) | blind e / min | verdict |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    let mut per_run_md = String::from("## Every run\n\n| stratum | seed | result | length | pulls | landings | notables | kills | damage | pull gap med | landing gap med | quiet max | blind e |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    t.hrow(&[("S", 2), ("runs", 5), ("deaths", 7), ("length", 7), ("pull gap", 9), ("land gap", 9), ("pulls", 6), ("notab", 6), ("kills", 6), ("dmg", 5), ("quiet", 6), ("e/min", 6), ("verdict", 34)]);
    for &stratum in &strata {
        let mut reports = Vec::new();
        for k in 0..runs {
            let seed = seed0 + k;
            let Ok(lay) = run::route(&lib, seed, stratum) else { continue };
            let m = run::RouteMap::compile(&lib, &lay);
            let o = run::play(m, table.clone(), tables.clone(), defs.clone(), 7 + k, 1, 1, None);
            t.bar(&format!("S{stratum}"), k + 1, runs, &format!("seed {seed}  {}  {} pulls", o.report.result, o.report.pulls));
            if let Some(d) = logs {
                let header = format!("route seed {seed}  stratum {stratum}  nonce {}  day 1  bot", 7 + k);
                let _ = std::fs::write(d.join(format!("run_s{stratum}_seed{seed}.log")), o.log.text(&header, &o.report));
            }
            let r = &o.report;
            per_run_md.push_str(&format!("| S{stratum} | {seed} | {} | {} | {} | {} | {} | {} | {} | {:.0} s | {:.0} s | {:.0} s | {} |\n", r.result, run::telemetry::clock(r.ticks), r.pulls, r.landings, r.notables, r.kills, r.damage, r.pull_gap_median, r.landing_gap_median, r.loss_gap_max, r.blind_value));
            reports.push(o.report);
        }
        t.bar_done();
        if reports.is_empty() {
            continue;
        }
        let n = reports.len() as f32;
        let deaths = reports.iter().filter(|r| r.result == "died").count();
        let death_rate = deaths as f32 / n;
        let mut lens: Vec<f32> = reports.iter().map(|r| r.ticks as f32 / 20.0).collect();
        let mut pull_gaps: Vec<f32> = reports.iter().filter(|r| r.pulls >= 2).map(|r| r.pull_gap_median).collect();
        let mut land_gaps: Vec<f32> = reports.iter().filter(|r| r.landings >= 2).map(|r| r.landing_gap_median).collect();
        let mut quiet: Vec<f32> = reports.iter().map(|r| r.loss_gap_max).collect();
        let pulls = reports.iter().map(|r| r.pulls).sum::<usize>() as f32 / n;
        let notables = reports.iter().map(|r| r.notables).sum::<usize>() as f32 / n;
        let kills = reports.iter().map(|r| r.kills).sum::<usize>() as f32 / n;
        let damage = reports.iter().map(|r| r.damage).sum::<i32>() as f32 / n;
        let epm = reports.iter().map(|r| r.value_per_min).sum::<f32>() / n;
        let (len_m, pull_m, land_m, quiet_m) = (median(&mut lens), median(&mut pull_gaps), median(&mut land_gaps), median(&mut quiet));
        // the run is judged on pulls, pressure and deaths; the landing budget (10-30 s) is the whole game's and is
        // judged on the combined run + colony log in Phase C, so landings are reported here and not verdicted
        let v_pull = in_band(pull_m, bp.0, bp.1);
        let v_quiet = if quiet_m <= bq { "ok" } else { "lapses" };
        let v_death = in_band(death_rate, DEATH_RATE.0, DEATH_RATE.1);
        let v_death = match v_death { "fast" => "corridor", "slow" => "wall", v => v };
        let verdict = format!("pull {v_pull} · loss {v_quiet} · death {v_death}");
        let _ = bl;
        t.row(&[(&format!("{stratum}"), 2), (&format!("{}", reports.len()), 5), (&format!("{deaths} ({:.0}%)", death_rate * 100.0), 7), (&format!("{:.0}:{:02.0}", (len_m / 60.0).floor(), len_m % 60.0), 7), (&format!("{pull_m:.0} s"), 9), (&format!("{land_m:.0} s"), 9), (&format!("{pulls:.1}"), 6), (&format!("{notables:.2}"), 6), (&format!("{kills:.1}"), 6), (&format!("{damage:.0}"), 5), (&format!("{quiet_m:.0} s"), 6), (&format!("{epm:.1}"), 6), (&verdict, 34)]);
        md.push_str(&format!("| S{stratum} | {} | {} ({:.0} %) | {:.0}:{:02.0} | {:.0} s | {:.0} s | {:.1} | {:.2} | {:.1} | {:.0} | {:.0} s | {:.1} | {} |\n", reports.len(), deaths, death_rate * 100.0, (len_m / 60.0).floor(), len_m % 60.0, pull_m, land_m, pulls, notables, kills, damage, quiet_m, epm, verdict));
    }
    md.push('\n');
    md.push_str(&per_run_md);
    if let Some(parent) = report_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(report_path, md).expect("report");
    println!("  {} {}", t.dim("wrote"), report_path.display());
}

// ---------------------------------------------------------------- city
fn cmd_city(t: &Term, seed: u64, shifts: u32, report_path: &Path, log_path: &Path) {
    use run::telemetry::{bot_bands, BOT_FACTOR, LANDING_GAP_S};
    let (_, bl, _) = bot_bands();
    t.head("city", &format!("{shifts} shifts from seed {seed}: a bot run each shift on S1's first route card, the steward's standing orders, the shift in core  |  landing gap: player {}-{} s, bot band (/{}) {:.1}-{:.1} s", LANDING_GAP_S.0, LANDING_GAP_S.1, BOT_FACTOR, bl.0, bl.1));
    t.hrow(&[("shift", 5), ("runner", 22), ("run", 10), ("length", 6), ("run land", 8), ("city land", 9), ("run gap", 7), ("merged", 7), ("pop", 3), ("stash", 5), ("rooms", 5), ("e", 5), ("late", 4), ("grate", 5), ("built", 26)]);
    let started = Instant::now();
    // the cord to the first grate at the shift's end: held, lost with a Relay going out, or lost
    let grate = |r: &game::city::ShiftRecord| if r.reached { "held" } else if r.steering { "steer" } else { "lost" };
    let row = |k: usize, r: &game::city::ShiftRecord| {
        let len = format!("{}:{:02}", (r.seconds / 60.0) as u32, r.seconds as u32 % 60);
        t.row(&[(&format!("{}", k + 1), 5), (&r.runner.chars().take(22).collect::<String>(), 22), (&r.result, 10), (&len, 6), (&format!("{}", r.run_landings), 8), (&format!("{}", r.city_landings), 9), (&format!("{:.1} s", r.run_gap), 7), (&format!("{:.1} s", r.merged_gap), 7), (&format!("{}", r.population), 3), (&format!("{}", r.stash), 5), (&format!("{}", r.in_rooms), 5), (&format!("{}", r.eddies), 5), (&format!("{}", r.late), 4), (grate(r), 5), (&r.built.join(", ").chars().take(26).collect::<String>(), 26)]);
    };
    let (run_, replay) = match game::city::replays(seed, shifts, &mut |k, r| row(k, r)) {
        Ok(c) => (c, "the city replays bit-exact".to_string()),
        Err(e) => (game::city::play(seed, shifts, &mut |_, _| {}), e),
    };
    let secs = started.elapsed().as_secs_f32();
    let mut run_gaps: Vec<f32> = run_.shifts.iter().filter(|r| r.run_landings >= 2).map(|r| r.run_gap).collect();
    let mut merged: Vec<f32> = run_.shifts.iter().filter(|r| r.run_landings + r.city_landings >= 2).map(|r| r.merged_gap).collect();
    let (rg, mg) = (median(&mut run_gaps), median(&mut merged));
    let verdict = in_band(mg, bl.0, bl.1);
    let deaths = run_.shifts.iter().filter(|r| r.result == "died").count();
    let mut failures = run_.failures.clone();
    if !replay.contains("bit-exact") {
        failures.insert(0, replay.clone());
    }
    println!();
    println!("  landing gap median: run alone {rg:.1} s, run and city merged {mg:.1} s against the bot band {:.1}-{:.1} s: {verdict}  (bot factor {BOT_FACTOR}, an assumption until Phase F)", bl.0, bl.1);
    println!("  build plan {}/{}   deaths {deaths}/{}   {replay}   {:.0} s", run_.plan_built, game::city::BUILD_PLAN.len(), run_.shifts.len(), secs);
    for f in &failures {
        println!("  {} {f}", t.hi("FAIL"));
    }
    if failures.is_empty() {
        println!("  {}", t.hi("PASS"));
    }
    // the report
    let mut md = String::new();
    md.push_str("# The city, twenty shifts headless\n\n");
    md.push_str(&format!("Written by `rose city --seed {seed} --shifts {shifts}` on {} (Full Design v3.13, The city's log; The founding; The Mat's metabolism; Build plan, Phase C). The city is founded with the Mat steered to the first grate (Pass 2's first session to 5:30). Each shift the steward gives its standing orders — the Heart's Pit kept at {:.0} nutrient with Uncommon its ceiling, every Curio and Tech sold at the first grate while the Mat reaches it, a Relay at a time toward the first grate whenever the cord to it is lost, the build plan ({}) placed one ghost at a time at the nearest anchor to the Heart (after the first Pod it waits while the cord is being won back), with eight stockpile cells painted at the Rack, every petition answered by placing what it asks — then the eligible slimo of best Scav runs the first route card of S1 as the bot, alone, with the bot's kit packed from the Stash where the Stash has it; the city ticks through the run at the bot's pace ({BOT_FACTOR} city ticks for every ten of the run's, so a shift does to the city what a player's shift does), the haul comes home, and the shift ends in `game`. The steward is this design's stand-in for a player, not a strategy the game recommends; the bot does not go back for its dead.\n\n", chrono_free_date(), game::city::HEART_KEEP.target, game::city::BUILD_PLAN.join(", ")));
    md.push_str(&format!("**Assertions**: the city replays bit-exact (two plays of the seed, compared shift by shift); every shift's cargo lands within the next shift; the build plan stands by the end; every shift ends with the cord to the first grate standing or a Relay able to go out to win it back; nobody starves (Fed under 10); the Heart's core keeps signal 0.3. **Result**: {}.\n\n", if failures.is_empty() { "all hold".to_string() } else { failures.join("; ") }));
    md.push_str(&format!("**The landing cadence** (research 02's whole-game budget of something landing every {}–{} s, the bot's band {:.1}–{:.1} s at the bot factor {BOT_FACTOR}, an assumption until the Phase F playtest): the run alone lands every {rg:.1} s (median of the shifts' medians), the run and the city merged every {mg:.1} s — **{verdict}**. Landings a second apart count as one moment, as the run's report counts them. Deaths: {deaths} of {} runs.\n\n", LANDING_GAP_S.0, LANDING_GAP_S.1, bl.0, bl.1, run_.shifts.len()));
    md.push_str("| shift | day | runner | run | length | run landings | city landings | run gap | merged gap | population | Stash | in stockpiles | on the Heart's floor | e (+earned −spent) | paste meals | Mat cells | Mat nutrient | late cargo | first grate | built |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for (k, r) in run_.shifts.iter().enumerate() {
        md.push_str(&format!("| {} | {} | {} | {} | {}:{:02} | {} | {} | {:.1} s | {:.1} s | {} | {} | {} | {} | {} (+{} −{}) | {} | {} | {:.1} | {} | {} | {} |\n", k + 1, r.day, if r.runner.is_empty() { "-" } else { &r.runner }, r.result, (r.seconds / 60.0) as u32, r.seconds as u32 % 60, r.run_landings, r.city_landings, r.run_gap, r.merged_gap, r.population, r.stash, r.in_rooms, r.floor, r.eddies, r.earned, r.spent, r.paste_meals, r.mat_cells, r.mat_nutrient, r.late, grate(r), if r.built.is_empty() { "-".to_string() } else { r.built.join(", ") }));
    }
    md.push_str("\n**Alerts standing at the last shift's end**: ");
    let last_alerts = run_.shifts.last().map(|r| r.alerts.clone()).unwrap_or_default();
    md.push_str(&if last_alerts.is_empty() { "none".to_string() } else { last_alerts.join("; ") });
    md.push_str(&format!("\n\nThe merged log, every run and city event on one clock, is `{}` (not committed).\n", log_path.display()));
    if let Some(parent) = report_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(report_path, md).expect("report");
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(log_path, &run_.log).expect("log");
    println!("  {} {}  {}", t.dim("wrote"), report_path.display(), log_path.display());
    if !failures.is_empty() {
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------- cleansing
fn cmd_cleansing(t: &Term, seed: u64, shifts: u32, report_path: &Path) {
    t.head("cleansing", &format!("seed {seed}: the steward plays {shifts} shifts, then a Cleansing comes down the first grate  |  as it stands, then with two Gates and a manned Turret on the way"));
    t.hrow(&[("city", 10), ("Heart", 9), ("Mat cells", 13), ("structures", 11), ("slimos", 8), ("dead", 5), ("bleeding", 8), ("fell", 7), ("burned", 7), ("ruined", 7), ("seconds", 8)]);
    let started = Instant::now();
    let mut runs = Vec::new();
    for defended in [false, true] {
        let r = game::siege::cleansing(seed, shifts, defended);
        let heart = if r.heart_lost { "burned".to_string() } else { format!("{} HP", r.after.heart_hp) };
        t.row(&[(if defended { "defended" } else { "as it is" }, 10), (&heart, 9), (&format!("{} > {}", r.before.cells, r.after.cells), 13), (&format!("{} > {}", r.before.structures, r.after.structures), 11), (&format!("{} > {}", r.before.slimos, r.after.slimos), 8), (&r.dead.to_string(), 5), (&r.bleeding.to_string(), 8), (&format!("{}/{}", r.fallen, r.sent), 7), (&r.burned.to_string(), 7), (&r.ruined.to_string(), 7), (&format!("{:.0}", r.seconds), 8)]);
        runs.push(r);
    }
    let (a, b) = (&runs[0], &runs[1]);
    // the designed spread (Full Design, Pressure: The proof): as it stands the Heart falls; defended it stands with most of its Mat
    let spread = a.heart_lost && !b.heart_lost && b.after.cells * 2 >= b.before.cells;
    println!();
    for line in &b.defence {
        println!("  {} {line}", t.dim("defence"));
    }
    println!("  the Gates close the way: {}", if b.closed { "yes" } else { "no, the squads can walk round" });
    println!("  {}   {:.0} s", if spread { t.hi("PASS the designed spread") } else { t.hi("FAIL the designed spread") }, started.elapsed().as_secs_f32());
    let mut md = String::new();
    md.push_str("# A Cleansing, with and without Gates\n\n");
    md.push_str(&format!("Written by `rose cleansing --seed {seed} --shifts {shifts}` on {} (Full Design v3.14, Colony mode: Pressure — The proof; the Build plan's Phase D). The steward of `rose city` plays {shifts} shifts; then a Cleansing comes down the first grate — the sluice's bleach flood, four lamp carriers advancing a tile a Mat step, four flamers for the Pits and the Heart once the carriers are halfway — twice from the same state: as the city stands, and with two Sphincter Gates and a Scrap Turret on the squads' way from the grate, a Guard ordered to each. The designed spread: as it stands the Heart falls; defended, the Heart stands and most of the Mat with it. **Result: {}.**\n\n", chrono_free_date(), if spread { "the spread holds" } else { "the spread does not hold" }));
    md.push_str("| city | the Heart | Mat cells | structures | slimos | dead | bleeding | Oathkeepers fell | Mat tiles burned | structures ruined | seconds | Blackline |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for r in &runs {
        md.push_str(&format!("| {} | {} | {} → {} | {} → {} | {} → {} | {} | {} | {} of {} | {} | {} | {:.0} | {} |\n", if r.defended { "defended" } else { "as it stands" }, if r.heart_lost { "burned".to_string() } else { format!("{} HP", r.after.heart_hp) }, r.before.cells, r.after.cells, r.before.structures, r.after.structures, r.before.slimos, r.after.slimos, r.dead, r.bleeding, r.fallen, r.sent, r.burned, r.ruined, r.seconds, r.blackline));
    }
    md.push_str(&format!("\n**The defence**: {}. The Gates {} the way from the grate to the Heart.\n\n", b.defence.join("; "), if b.closed { "close" } else { "do not close" }));
    // the aftermath: HP the standing structures are short, shift by shift while the builders mend (v3.17)
    for r in [a, b] {
        let line = if r.heart_lost { "the Heart burned: nothing left to mend it".to_string() } else { r.short.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" → ") };
        md.push_str(&format!("**After, {}** (HP short across the standing structures at the Cleansing's shift end and the next three, the builders mending from the shift after): {line}.\n\n", if r.defended { "defended" } else { "as it stands" }));
        println!("  {} {}: {}{line}", t.dim("after"), if r.defended { "defended" } else { "as it is" }, if r.heart_lost { "" } else { "HP short " });
    }
    for r in &runs {
        md.push_str(&format!("**The Cleansing, {}**\n\n```\n{}```\n\n", if r.defended { "defended" } else { "as it stands" }, r.log));
    }
    if let Some(parent) = report_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(report_path, md).expect("report");
    println!("  {} {}", t.dim("wrote"), report_path.display());
    if !spread {
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------- economy
fn cmd_economy(t: &Term, seed: u64, shifts: u32, report_path: &Path) {
    use game::city::{play_with, Haul, ShiftRecord};
    t.head("economy", &format!("seed {seed}, {shifts} shifts: the steward trades its surplus, Chrome too, at the best stall in reach  |  then the same seed feeding its Chrome to the Heart"));
    t.hrow(&[("shift", 5), ("day", 3), ("grates", 6), ("sold", 4), ("earned", 6), ("e", 5), ("drift", 5), ("contracts and services", 48)]);
    let started = Instant::now();
    let row = |k: usize, r: &ShiftRecord| {
        let earned: u32 = r.sales.iter().map(|s| s.eddies).sum();
        t.row(&[(&format!("{}", k + 1), 5), (&format!("{}", game::trade::day_of(r.day)), 3), (&format!("{}", r.markets), 6), (&format!("{}", r.sales.len()), 4), (&format!("{earned}"), 6), (&format!("{}", r.eddies), 5), (&format!("{:.2}", r.drift), 5), (&r.decisions.iter().filter(|d| d.starts_with("took") || d.contains("protection")).cloned().collect::<Vec<_>>().join("; ").chars().take(48).collect::<String>(), 48)]);
    };
    let (trade, w) = play_with(seed, shifts, Haul::Trade, &mut |k, r| row(k, r));
    let (grow, g) = play_with(seed, shifts, Haul::Feed, &mut |_, _| {});
    let secs = started.elapsed().as_secs_f32();
    let table = &w.table;
    let e = &w.economy;
    let band = e.file.rules.band;
    // the proof: every faction's walk inside its band every day
    let worst = trade.shifts.iter().map(|r| r.drift).fold(0.0, f32::max).max(grow.shifts.iter().map(|r| r.drift).fold(0.0, f32::max));
    let mut failures: Vec<String> = Vec::new();
    for (name, run_) in [("trading", &trade), ("feeding", &grow)] {
        for (k, r) in run_.shifts.iter().enumerate() {
            if r.drift >= band {
                failures.push(format!("{name}, shift {}: a demand walked {:.2} from its buy-list value", k + 1, r.drift));
            }
        }
    }
    // realised prices against base (base x condition), by faction and category
    let mut by: std::collections::BTreeMap<(String, String), (u32, f32, u32, u32)> = Default::default();
    for s in trade.shifts.iter().flat_map(|r| r.sales.iter()) {
        let d = table.def(s.item);
        let base = d.value as f32 * s.condition as f32 / 100.0 * if s.blind { 0.4 } else { 1.0 };
        let k = (e.name(s.faction).to_string(), economy::category_name(d.category).to_string());
        let v = by.entry(k).or_insert((0, 0.0, 0, 0));
        v.0 += 1;
        v.1 += if base > 0.0 { s.eddies as f32 / base } else { 0.0 };
        v.2 += s.eddies;
        v.3 += s.blind as u32;
    }
    let first_two = |r: &game::city::CityRun| r.shifts.iter().position(|s| s.markets >= 2).map(|k| k + 1);
    let (tr, gr) = (first_two(&trade), first_two(&grow));
    let at = |o: Option<usize>| o.map_or(format!("not in {shifts} shifts"), |k| format!("shift {k}"));
    println!();
    println!("  the walk: widest {:.2} from a buy-list value over {} days, against the band {band:.2}", worst, game::trade::day_of(shifts));
    for ((f, c), (n, ratio, eddies, blind)) in &by {
        println!("  {:<16} {:<10} {:>3} sold ({blind:>2} blind)  {:>5} e  {:>4.0} % of base", f, c, n, eddies, 100.0 * ratio / *n as f32);
    }
    for (k, st) in e.s.standing.iter().enumerate() {
        println!("  {:<16} reputation {:+}  favour {:>3}  traded {:>5} e  tier {}", e.name(k as u8), st.rep, st.favour, st.traded, e.tier_open(k as u8));
    }
    let held: Vec<String> = e.s.contracts.iter().map(|c| c.line(&e.file, table)).collect();
    println!("  contracts in hand: {}", if held.is_empty() { "none".to_string() } else { held.join("; ") });
    // the services the steward bought, by name, and what they did
    let bought = |w: &game::World| -> String {
        let n = |svc: &str| w.log.events.iter().filter(|ev| ev.what.starts_with(&format!("{svc} from the "))).count();
        let did = |pat: &str| w.log.events.iter().filter(|ev| ev.what.contains(pat)).count();
        format!("protection {} (held off {} Patrols), intel {}, timings {}, convoy {}, hire {} ({} hired)", n("protection"), did("held off the Patrol"), n("intel"), n("timings"), n("convoy"), n("hire"), did("hired through the"))
    };
    println!("  services: {}", bought(&w));
    let chain_log: Vec<String> = w.log.events.iter().filter(|ev| ev.what.contains("chain") || ev.what.contains("link") || ev.what.starts_with("took ") || ev.what.contains("missed")).map(|ev| ev.what.clone()).collect();
    println!("  the nearer grate reached: trading {}, feeding {}   shifts with no market: trading {}, feeding {}", at(tr), at(gr), trade.shifts.iter().filter(|s| s.markets == 0).count(), grow.shifts.iter().filter(|s| s.markets == 0).count());
    println!("  eddies at the end: trading {}, feeding {}", w.ledger.eddies, g.ledger.eddies);
    let pass = failures.is_empty();
    for f in &failures {
        println!("  {} {f}", t.hi("FAIL"));
    }
    println!("  {}   {:.0} s", if pass { t.hi("PASS every demand's walk inside its band") } else { t.hi("FAIL the band") }, secs);
    // the report
    let mut md = String::new();
    md.push_str("# The economy, thirty shifts\n\n");
    md.push_str(&format!("Written by `rose economy --seed {seed} --shifts {shifts}` on {} (Full Design v3.15, Economy — The proof; the Build plan's Phase D). The steward of `rose city` plays the seed trading — the surplus, its Chrome too, to whichever stall in reach pays most, five of a category a stall a shift, the three-link deliver and host contracts it can fill taken, protection bought when a Patrol is due at a Gang grate — then plays it again feeding, as `rose city` does: the same, but the Heart keeping 2.0 nutrient on the haul's Common and Uncommon Chrome. (A district's foreman keeps the Heart fed on the members' meals either way.) **Result: {}** (widest walk {:.2}, band {band:.2}).\n\n", chrono_free_date(), if pass { "every demand's walk stayed inside its band every day" } else { "a walk left its band" }, worst));
    md.push_str("## The streets\n\n| grate | column | faction |\n|---|---|---|\n");
    for st in &e.s.streets {
        md.push_str(&format!("| {} | {} | {} |\n", st.number, st.grate, e.name(st.faction)));
    }
    md.push_str("\n## Shift by shift, trading\n\n| shift | day | grates | sold | earned | eddies | widest walk |\n|---|---|---|---|---|---|---|\n");
    for (k, r) in trade.shifts.iter().enumerate() {
        md.push_str(&format!("| {} | {} | {} | {} | {} | {} | {:.2} |\n", k + 1, game::trade::day_of(r.day), r.markets, r.sales.len(), r.sales.iter().map(|s| s.eddies).sum::<u32>(), r.eddies, r.drift));
    }
    md.push_str("\n## What sold, against its base\n\nRealised price over base × condition (× 0.4 for an item sold blind), by the faction that bought and the category: what demand, reputation, haggling and glut made of it (over 100 % where a cheap item's few eddies round up, a stall paying at least 1 e).\n\n| faction | category | sold | blind | eddies | of base |\n|---|---|---|---|---|---|\n");
    for ((f, c), (n, ratio, eddies, blind)) in &by {
        md.push_str(&format!("| {f} | {c} | {n} | {blind} | {eddies} | {:.0} % |\n", 100.0 * ratio / *n as f32));
    }
    md.push_str("\n## Standing at the end\n\n| faction | reputation | favour | traded | tier open |\n|---|---|---|---|---|\n");
    for (k, st) in e.s.standing.iter().enumerate() {
        md.push_str(&format!("| {} | {:+} | {} | {} e | {} |\n", e.name(k as u8), st.rep, st.favour, st.traded, e.tier_open(k as u8)));
    }
    md.push_str(&format!("\n## Contracts\n\n{}\n\nIn hand at the end: {}.\n", if chain_log.is_empty() { "None taken.".to_string() } else { chain_log.iter().map(|l| format!("- {l}")).collect::<Vec<_>>().join("\n") }, if held.is_empty() { "none".to_string() } else { held.join("; ") }));
    let fed = |w: &game::World| w.log.events.iter().filter(|ev| ev.what.ends_with("fed to the Heart")).count();
    let lost = |r: &game::city::CityRun| r.shifts.iter().filter(|s| s.markets == 0).count();
    md.push_str(&format!("\n## Trading against feeding\n\n| | the nearer grate reached | shifts with no market | eddies at the end | sold | Chrome fed to the Heart |\n|---|---|---|---|---|---|\n| trading | {} | {} | {} | {} | {} |\n| feeding | {} | {} | {} | {} | {} |\n", at(tr), lost(&trade), w.ledger.eddies, trade.shifts.iter().map(|r| r.sales.len()).sum::<usize>(), fed(&w), at(gr), lost(&grow), g.ledger.eddies, grow.shifts.iter().map(|r| r.sales.len()).sum::<usize>(), fed(&g)));
    md.push_str(&format!("\n## Services\n\nTrading: {}.\n\nFeeding: {}.\n", bought(&w), bought(&g)));
    let events: Vec<String> = w.log.events.iter().filter(|ev| ev.what.starts_with("topside")).map(|ev| ev.what.clone()).collect();
    md.push_str(&format!("\n## Topside\n\n{}\n", if events.is_empty() { "No events.".to_string() } else { events.iter().map(|l| format!("- {l}")).collect::<Vec<_>>().join("\n") }));
    if let Some(parent) = report_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(report_path, md).expect("report");
    println!("  {} {}", t.dim("wrote"), report_path.display());
    if !pass {
        std::process::exit(1);
    }
}

/// Today as YYYY-MM-DD from the system clock, without a date crate.
fn chrono_free_date() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86_400;
    // civil-from-days (Howard Hinnant)
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

// ---------------------------------------------------------------- campaign
fn cmd_campaign(t: &Term, seed: u64, aim: &str, shifts: u32, report: &Path) {
    let aims: Vec<game::proof::Aim> = game::proof::aims().into_iter().filter(|a| aim == "all" || a.ending == aim).collect();
    t.head("campaign", &format!("seed {seed}: {} script{} up to {shifts} shifts", aims.len(), if aims.len() == 1 { "" } else { "s" }));
    let started = Instant::now();
    let mut runs: Vec<(game::proof::Aim, game::proof::CampaignRun)> = Vec::new();
    for a in aims {
        t.hrow(&[("shift", 5), ("ch", 3), ("S", 2), ("run", 4), ("result", 10), ("pop", 4), ("beds", 4), ("cells", 6), ("top", 4), ("genes (aim)", 12), ("tiers", 5), ("def", 3), ("bloom", 12), ("P/C/e", 10), ("att", 9), ("research", 18), ("story", 50)]);
        let (run_, _) = game::proof::play(seed, a, shifts, &mut |c, _| {
            let story = c.story.join("; ");
            t.row(&[(&c.day.to_string(), 5), (&c.chapter.to_string(), 3), (&c.stratum.to_string(), 2), (&format!("S{}", c.run_on), 4), (&c.result, 10), (&c.population.to_string(), 4), (&c.beds.to_string(), 4), (&c.cells.to_string(), 6), (&c.top_row.to_string(), 4), (&format!("{:.2} {:.2}", c.genes[a.genes[0]], c.genes[a.genes[1]]), 12), (&format!("{}{}{}", c.open[0], c.open[1], c.open[2]), 5), (&c.defences.to_string(), 3), (&format!("{}/{}/{}", c.bloom.0, c.bloom.1, c.bloom.2), 12), (&format!("{}/{}/{}", c.stock.0, c.stock.1, c.stock.2), 10), (&format!("{:.0}+{:.0}", c.attention.0, c.attention.1), 9), (&c.research.chars().take(18).collect::<String>(), 18), (&story.chars().take(50).collect::<String>(), 50)]);
        });
        println!("  {}: ending {:?} warchest {:?} in {} shifts, {:.0} s", a.ending, run_.ending, run_.warchest, run_.shifts.len(), run_.seconds);
        let log = PathBuf::from(format!("out/campaign_{}_{seed}.log", a.ending));
        let _ = std::fs::create_dir_all("out");
        std::fs::write(&log, &run_.log).expect("log");
        runs.push((a, run_));
    }
    // the proof (Full Design v3.20, Narrative and Ivan: The proof): each script ends the campaign in its own ending
    let pass = runs.iter().all(|(a, r)| r.ending.as_deref() == Some(a.ending));
    println!("  {}   {:.0} s", if pass { t.hi("PASS every script reached its ending") } else { t.hi("FAIL a script missed its ending") }, started.elapsed().as_secs_f32());
    let gene = |k: usize| mat::genome::GENE_NAMES[k];
    // the first shift a column reached each value
    let first = |r: &game::proof::CampaignRun, v: u8, f: &dyn Fn(&game::proof::CampaignShift) -> u8| r.shifts.iter().find(|c| f(c) >= v).map_or("—".to_string(), |c| c.day.to_string());
    let mut md = String::new();
    md.push_str("# The campaign, chapter 1 to an ending\n\n");
    md.push_str(&format!("Written by `rose campaign --seed {seed} --shifts {shifts}` on {} (Full Design v3.20, Narrative and Ivan: The proof; the Build plan's Phase E). Five scripts play the campaign headless at the bot's pace, one for each ending: the steward of `rose city` running the shifts and the campaign's own player on top — the research the chapters ask for, the climb up the stack, the defences once the Oathkeepers come, the gene pair it steers with Ivan's feeds and the Stash's Chrome, the accord refused once a Gate and a Turret stand (taken before), an alliance at chapter 6 and the Great Cleansing met at chapter 7; Scattered's script raises no defence and refuses the accord, and the chapter's Cleansing burns the Heart. **Result: {}.**\n\n", chrono_free_date(), if pass { "every script reached its ending" } else { "a script missed its ending" }));
    md.push_str(&format!("| script | genes steered | the accord | ending | warchest | shifts | chapters 2–7 opened at shift | strata 2–6 reached at shift | the genome at the end, {} to {} | slimos at the end | seconds |\n|---|---|---|---|---|---|---|---|---|---|---|\n", gene(0), gene(9)));
    for (a, r) in &runs {
        // a chapter opened when the city first stood in it or past it short of the ending (which closes them all)
        let chapters: Vec<String> = (2..=7).map(|v| r.shifts.iter().find(|c| c.chapter >= v && c.chapter <= 7).map_or("—".to_string(), |c| c.day.to_string())).collect();
        let strata: Vec<String> = (2..=6).map(|v| first(r, v, &|c| c.stratum)).collect();
        // the ten genes, Vigor to Plasticity, as the Great Cleansing read them
        let genome = r.shifts.last().map_or(String::new(), |c| c.genes.iter().map(|g| format!("{:.2}", g)).collect::<Vec<_>>().join(" "));
        md.push_str(&format!("| {} | {} and {} | {} | {} | {} | {} | {} | {} | {genome} | {} | {:.0} |\n", a.ending, gene(a.genes[0]), gene(a.genes[1]), if r.log.contains("the envoy answered: accepted") { "accepted" } else { "refused" }, r.ending.clone().unwrap_or_else(|| "none".into()), r.warchest.map_or("—".into(), |v| v.to_string()), r.shifts.len(), chapters.join(" · "), strata.join(" · "), r.shifts.last().map_or(0, |c| c.population), r.seconds));
    }
    for (a, r) in &runs {
        // the steered pair, and Feral Drive (the bloom line) where the pair is not the World-Eater's own
        let mut shown = a.genes.to_vec();
        if !shown.contains(&mat::genome::FERAL) {
            shown.push(mat::genome::FERAL);
        }
        let heads: Vec<&str> = shown.iter().map(|&k| gene(k)).collect();
        md.push_str(&format!("\n## {}, every tenth shift\n\n| shift | chapter | stratum | slimos | Mat cells joined | top row | {} | Bloom tiles | Parts / Chrome / eddies | the loudest grate |\n|---|---|---|---|---|---|{}---|---|---|\n", a.ending, heads.join(" | "), "---|".repeat(shown.len())));
        for c in r.shifts.iter().filter(|c| c.day % 10 == 0 || Some(c.day) == r.shifts.last().map(|l| l.day)) {
            let gs: Vec<String> = shown.iter().map(|&k| format!("{:.2}", c.genes[k])).collect();
            md.push_str(&format!("| {} | {} | S{} | {} | {} | {} | {} | {} | {} / {} / {} | {:.0} |\n", c.day, c.chapter, c.stratum, c.population, c.cells, c.top_row, gs.join(" | "), c.bloom.0, c.stock.0, c.stock.1, c.stock.2, c.attention.0));
        }
    }
    md.push_str(&format!("\nThe logs, every decision and event of each run: `out/campaign_<ending>_{seed}.log`.\n"));
    if let Some(parent) = report.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(report, md).expect("report");
    println!("  {} {}", t.dim("wrote"), report.display());
    if !pass {
        std::process::exit(1);
    }
}
