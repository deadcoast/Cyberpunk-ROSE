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
    let reached = senses.runner_sound(&m, run::Sound::SprintStep);
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
