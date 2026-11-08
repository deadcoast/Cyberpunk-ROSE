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

fn cmd_route(t: &Term, out: &Path, seed: u64, stratum: u8, chunks: usize, sweep: u64) {
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
        Cmd::Route { seed, stratum, chunks, sweep } => cmd_route(&t, &cli.out, seed, stratum, chunks, sweep),
    }
    println!();
}
