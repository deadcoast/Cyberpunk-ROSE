use run::*;
fn main() {
    let l = ChunkLib::from_toml(CHUNKS_TOML).unwrap();
    for stratum in 1..=3u8 {
        let mut worst = (1.0f32, 0u64);
        let mut hist = std::collections::BTreeMap::new();
        for seed in 0..300u64 {
            let lay = route(&l, seed, stratum).unwrap();
            let m = RouteMap::compile(&l, &lay);
            let c = m.connectivity();
            if c < worst.0 { worst = (c, seed); }
            for pc in &m.chunks { *hist.entry(pc.id.clone()).or_insert(0) += 1; }
        }
        let lay = route(&l, worst.1, stratum).unwrap();
        let m = RouteMap::compile(&l, &lay);
        println!("s{stratum} worst {:.3} seed {} :: {}", worst.0, worst.1, m.digest());
        println!("  {:?}", m.chunks.iter().map(|c| c.id.as_str()).collect::<Vec<_>>());
        println!("  hist {:?}", hist);
    }
}
