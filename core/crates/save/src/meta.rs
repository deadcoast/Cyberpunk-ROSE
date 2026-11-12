//! What outlives a run (Full Design v3.19, Meta): the strain code a finished campaign exports and New Game+ takes,
//! and the profile — the Strainbook, the Scrapdex, the endings with their warchests and codes, the laws unlocked, the
//! Daily bests and the Endless ranks — kept in `user://profile` by the game and read by the Codex.

use serde::{Deserialize, Serialize};

use crate::fnv1a;

/// A strain code: `ROSE-` then 32 hex digits in eight groups of four — the genome as ten bytes (0–1 across 00–FF),
/// the classes held as one (bit k for the k-th class of `data/strain.toml`), the seed as four and a checksum byte
/// (this design's: a code a person can read out, and a typo is caught before a run is founded on it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrainCode {
    pub genome: [f32; 10],
    pub classes: u8,
    pub seed: u32,
}

impl StrainCode {
    fn bytes(&self) -> [u8; 15] {
        let mut b = [0u8; 15];
        for (i, g) in self.genome.iter().enumerate() {
            b[i] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        b[10] = self.classes;
        b[11..15].copy_from_slice(&self.seed.to_be_bytes());
        b
    }

    /// "ROSE-8040-1AFF-…" — eight groups of four.
    pub fn encode(&self) -> String {
        let b = self.bytes();
        let mut hex: String = b.iter().map(|x| format!("{x:02X}")).collect();
        hex.push_str(&format!("{:02X}", (fnv1a(&b) & 0xFF) as u8));
        let groups: Vec<&str> = (0..8).map(|k| &hex[k * 4..k * 4 + 4]).collect();
        format!("ROSE-{}", groups.join("-"))
    }

    /// A typed code: case, spaces and dashes are forgiven, the `ROSE` is optional; a wrong checksum is no code.
    pub fn parse(text: &str) -> Option<StrainCode> {
        let up = text.trim().to_ascii_uppercase();
        let body = up.strip_prefix("ROSE").unwrap_or(&up);
        let hex: String = body.chars().filter(|c| !matches!(c, '-' | ' ')).collect();
        if hex.len() != 32 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let all: Vec<u8> = (0..16).map(|k| u8::from_str_radix(&hex[k * 2..k * 2 + 2], 16).unwrap()).collect();
        if (fnv1a(&all[..15]) & 0xFF) as u8 != all[15] {
            return None;
        }
        let mut genome = [0.0f32; 10];
        for i in 0..10 {
            genome[i] = all[i] as f32 / 255.0;
        }
        Some(StrainCode { genome, classes: all[10], seed: u32::from_be_bytes([all[11], all[12], all[13], all[14]]) })
    }
}

/// An ending reached, as the Codex's Lore shows it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EndingRecord {
    pub ending: String,
    pub warchest: u32,
    pub code: String,
    pub shift: u32,
    pub difficulty: String,
}

/// An Endless Sump city that burned: the shift the Empire's card landed on (none if it never did) and the shift the
/// Heart burned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndlessRecord {
    pub seed: u32,
    pub empire_at: Option<u32>,
    pub burned_at: u32,
}

impl EndlessRecord {
    /// Lower ranks first: the earlier Empire, then the longer city.
    fn key(&self) -> (u32, std::cmp::Reverse<u32>) {
        (self.empire_at.unwrap_or(u32::MAX), std::cmp::Reverse(self.burned_at))
    }
}

pub const PROFILE_VERSION: u16 = 1;
const PROFILE_MAGIC: &[u8; 4] = b"ROSP";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// Strainbook cards ever earned
    pub strainbook: Vec<String>,
    /// Scrapdex entries ever seen
    pub scrapdex: Vec<String>,
    pub endings: Vec<EndingRecord>,
    /// every law a finished run had signed: New Game+ unlocks them
    pub laws: Vec<String>,
    /// (YYYYMMDD, score): one try a day
    pub daily: Vec<(u32, u32)>,
    /// the best Endless city per seed
    pub endless: Vec<EndlessRecord>,
    /// the last finished run's strain code (New Run offers it)
    pub last_code: Option<String>,
}

fn merge(into: &mut Vec<String>, from: impl IntoIterator<Item = String>) -> usize {
    let mut n = 0;
    for s in from {
        if !into.contains(&s) {
            into.push(s);
            n += 1;
        }
    }
    n
}

impl Profile {
    /// The run's collections into the profile's (each shift's end and at an ending): new entries counted.
    pub fn absorb(&mut self, strainbook: impl IntoIterator<Item = String>, scrapdex: impl IntoIterator<Item = String>) -> usize {
        merge(&mut self.strainbook, strainbook) + merge(&mut self.scrapdex, scrapdex)
    }

    /// A campaign's ending: its record, its code offered to the next New Run, the laws it signed unlocked.
    pub fn ended(&mut self, record: EndingRecord, laws: impl IntoIterator<Item = String>) {
        self.last_code = Some(record.code.clone());
        self.endings.push(record);
        merge(&mut self.laws, laws);
    }

    pub fn daily_tried(&self, date: u32) -> bool {
        self.daily.iter().any(|(d, _)| *d == date)
    }

    /// The day's try; false when the day was already tried.
    pub fn daily_score(&mut self, date: u32, score: u32) -> bool {
        if self.daily_tried(date) {
            return false;
        }
        self.daily.push((date, score));
        true
    }

    pub fn daily_best(&self) -> Option<(u32, u32)> {
        self.daily.iter().copied().max_by_key(|&(d, s)| (s, d))
    }

    /// An Endless city burned: kept when it ranks above the seed's best. Returns whether it did.
    pub fn endless_burned(&mut self, r: EndlessRecord) -> bool {
        match self.endless.iter_mut().find(|e| e.seed == r.seed) {
            Some(e) if r.key() < e.key() => {
                *e = r;
                true
            }
            Some(_) => false,
            None => {
                self.endless.push(r);
                true
            }
        }
    }

    /// MAGIC(4) | VERSION(2 LE) | checksum(8 LE) | postcard body — the save's framing.
    pub fn encode(&self) -> Vec<u8> {
        let body = postcard::to_allocvec(self).expect("profile encodes");
        let mut out = Vec::with_capacity(body.len() + 14);
        out.extend_from_slice(PROFILE_MAGIC);
        out.extend_from_slice(&PROFILE_VERSION.to_le_bytes());
        out.extend_from_slice(&fnv1a(&body).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    /// A profile from its file; a missing, foreign or corrupt one is None (the game starts a fresh profile and keeps
    /// the bad file aside).
    pub fn decode(bytes: &[u8]) -> Option<Profile> {
        if bytes.len() < 14 || &bytes[0..4] != PROFILE_MAGIC || u16::from_le_bytes([bytes[4], bytes[5]]) != PROFILE_VERSION {
            return None;
        }
        let sum = u64::from_le_bytes(bytes[6..14].try_into().ok()?);
        if fnv1a(&bytes[14..]) != sum {
            return None;
        }
        postcard::from_bytes(&bytes[14..]).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_strain_code_reads_back_and_a_typo_is_caught() {
        let c = StrainCode { genome: [0.5, 0.9, 0.1, 0.0, 1.0, 0.33, 0.66, 0.25, 0.75, 0.5], classes: 0b10010, seed: 20261111 };
        let s = c.encode();
        assert!(s.starts_with("ROSE-") && s.len() == 5 + 8 * 4 + 7, "{s}");
        let back = StrainCode::parse(&s.to_lowercase().replace('-', " ")).unwrap();
        assert_eq!((back.classes, back.seed), (c.classes, c.seed));
        for i in 0..10 {
            assert!((back.genome[i] - c.genome[i]).abs() <= 0.5 / 255.0 + 1e-6);
        }
        let mut typo = s.clone().into_bytes();
        typo[6] = if typo[6] == b'0' { b'1' } else { b'0' };
        assert!(StrainCode::parse(std::str::from_utf8(&typo).unwrap()).is_none());
        assert!(StrainCode::parse("ROSE-1234").is_none());
    }

    #[test]
    fn the_profile_keeps_what_outlives_a_run() {
        let mut p = Profile::default();
        assert_eq!(p.absorb(["organ_lume".to_string()], ["chr_lamp_core".to_string(), "chr_lamp_core".to_string()]), 2);
        assert_eq!(p.absorb(["organ_lume".to_string()], []), 0);
        p.ended(EndingRecord { ending: "garden".into(), warchest: 900, code: "ROSE-X".into(), shift: 101, difficulty: "Culvert".into() }, ["rationing".to_string()]);
        assert_eq!(p.last_code.as_deref(), Some("ROSE-X"));
        assert!(p.daily_score(20261111, 340) && !p.daily_score(20261111, 999));
        p.daily_score(20261112, 120);
        assert_eq!(p.daily_best(), Some((20261111, 340)));
        assert!(p.endless_burned(EndlessRecord { seed: 7, empire_at: None, burned_at: 60 }));
        assert!(p.endless_burned(EndlessRecord { seed: 7, empire_at: Some(88), burned_at: 90 }));
        assert!(!p.endless_burned(EndlessRecord { seed: 7, empire_at: Some(95), burned_at: 200 }));
        let bytes = p.encode();
        assert_eq!(Profile::decode(&bytes), Some(p));
        let mut bad = bytes.clone();
        let last = bad.len() - 1;
        bad[last] ^= 1;
        assert_eq!(Profile::decode(&bad), None);
    }
}
