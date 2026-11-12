//! The stack's strata as data (Full Design v3.19, Progression: The stack and the breach; `data/strata.toml`).

use serde::Deserialize;
use std::sync::OnceLock;

const STRATA_TOML: &str = include_str!("../../../../data/strata.toml");

#[derive(Clone, Debug, Deserialize)]
pub struct Ground {
    pub culvert_from: usize,
    pub culvert_every: usize,
    pub culvert_half: usize,
    pub pipe_above: usize,
    pub cave: f32,
    pub deposit: f32,
    pub water_rows: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StratumDef {
    pub n: u8,
    pub name: String,
    pub short: String,
    pub region: String,
    /// the grate key that opens this stratum's ceiling ("" none)
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub note: String,
    pub ground: Ground,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StrataFile {
    pub rows: usize,
    pub width: usize,
    pub stratum: Vec<StratumDef>,
}

impl StrataFile {
    pub fn shipped() -> &'static StrataFile {
        static F: OnceLock<StrataFile> = OnceLock::new();
        F.get_or_init(|| {
            let f: StrataFile = toml::from_str(STRATA_TOML).expect("data/strata.toml");
            assert!(f.stratum.iter().enumerate().all(|(k, s)| s.n as usize == k + 1), "strata in order from the Sump");
            f
        })
    }

    pub fn gens(&self) -> Vec<mat::StratumGen> {
        self.stratum
            .iter()
            .map(|s| {
                let g = &s.ground;
                mat::StratumGen { culvert_from: g.culvert_from, culvert_every: g.culvert_every, culvert_half: g.culvert_half, pipe_above: g.pipe_above, cave: g.cave, deposit: g.deposit, water_rows: g.water_rows }
            })
            .collect()
    }

    pub fn get(&self, n: u8) -> Option<&StratumDef> {
        self.stratum.get((n as usize).wrapping_sub(1))
    }

    /// The stratum whose ceiling a key opens.
    pub fn opened_by(&self, key: &str) -> Option<u8> {
        self.stratum.iter().find(|s| !s.key.is_empty() && s.key == key).map(|s| s.n)
    }
}

/// The Route Board's three cards: the strata from one below the city's to one above (Full Design v3.19).
pub fn board_strata(top: u8, count: u8) -> std::ops::RangeInclusive<u8> {
    let count = count.max(1);
    let lo = (top.max(1) as i32 - 1).clamp(1, (count as i32 - 2).max(1)) as u8;
    lo..=(lo + 2).min(count)
}

#[cfg(test)]
mod tests {
    #[test]
    fn six_strata_the_sump_first_and_the_board_one_below_to_one_above() {
        let f = super::StrataFile::shipped();
        assert_eq!(f.stratum.len(), 6);
        assert_eq!(f.opened_by("key_grate_sump"), Some(1));
        assert_eq!(super::board_strata(1, 6), 1..=3);
        assert_eq!(super::board_strata(2, 6), 1..=3);
        assert_eq!(super::board_strata(4, 6), 3..=5);
        assert_eq!(super::board_strata(6, 6), 4..=6);
        assert_eq!(super::board_strata(1, 1), 1..=1);
    }
}
