//! save — the save file: a header and the World's snapshot beside the shell's view state (Full Design: Saves).
//! Encoded with postcard (compact binary), versioned, checksummed. The strain code and the profile: `meta`.

use serde::{Deserialize, Serialize};

pub mod meta;
pub use meta::{EndingRecord, EndlessRecord, Profile, StrainCode};

pub const SAVE_VERSION: u16 = 18;
const MAGIC: &[u8; 4] = b"ROSE";

/// Every player action that changes simulation state, in sim ticks. Replaying these on the same seed
/// reproduces the run exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Input {
    Feed { x: u16, y: u16, item: u16, units: u16 },
    Bait { x: u16, y: u16, amount_milli: u16 },
    Sever { x: u16, y: u16 },
    Salt { x: u16, y: u16, on: bool },
    Relay { x: u16, y: u16 },
    Build { structure: u16, x: u16, y: u16 },
    Demolish { x: u16, y: u16 },
    Quota { site: u16, item_tier: u8, target: u16 },
    Priority { slimo: u32, job: u8, level: u8 },
    Law { law: u8, sign: bool },
    Order { vendor: u8, item: u16, units: u16, sell: bool },
    RunBegin { route: u16, runner: u32, followers: [u32; 2] },
    RunInput { frame: u32, buttons: u16, aim_x: i16, aim_y: i16 },
    RunEnd { extracted: bool },
    StashMove { handle: u32, to_x: u8, to_y: u8, rotated: bool },
    Pin { handle: u32, on: bool },
    Speed { hz: u8 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimedInput {
    pub tick: u64,
    pub input: Input,
}

/// A save (Full Design: Saves): the World's snapshot at the moment it was written (`game::World::snapshot`, every
/// part of the city with its caches, so the loaded city steps on exactly as the saved one would have) and the shell's
/// view state beside it, opaque here. The header says what it is without decoding the World: the seed, the strain
/// code, the shift and a line for the Continue button and the slots. (v18: until v17 a save was meant to be the seed,
/// an input log and a snapshot every 10 shifts; no build ever wrote one, and replaying nine shifts of a late city
/// would take about 30 s.)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u16,
    pub seed: u64,
    pub strain_code: Option<String>,
    pub difficulty: u8,
    /// when the run was founded and when this was written, seconds since 1970 (the shell's clock)
    pub created_unix: u64,
    pub saved_unix: u64,
    /// the shift it was written in and the city's clock
    pub day: u32,
    pub tick: u64,
    /// what the Continue button and the slots say of it: "shift 12 · Quarter · 104 slimos"
    pub label: String,
    /// the World's snapshot
    pub world: Vec<u8>,
    /// the shell's view state (the camera, the bookmarks, the last run's outcome card): the shell's to read
    pub ui: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SaveError {
    BadMagic,
    BadChecksum,
    Version(u16),
    Decode(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::BadMagic => write!(f, "not a ROSE save"),
            SaveError::BadChecksum => write!(f, "save is corrupt (checksum)"),
            SaveError::Version(v) => write!(f, "save version {v} does not match this build ({SAVE_VERSION})"),
            SaveError::Decode(e) => write!(f, "save decode: {e}"),
        }
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

impl SaveFile {
    pub fn new(seed: u64, difficulty: u8) -> Self {
        SaveFile { version: SAVE_VERSION, seed, strain_code: None, difficulty, created_unix: 0, saved_unix: 0, day: 1, tick: 0, label: String::new(), world: Vec::new(), ui: Vec::new() }
    }

    /// MAGIC(4) | VERSION(2 LE) | checksum(8 LE) | postcard body
    pub fn encode(&self) -> Vec<u8> {
        let body = postcard::to_allocvec(self).expect("save encodes");
        let mut out = Vec::with_capacity(body.len() + 14);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.version.to_le_bytes());
        out.extend_from_slice(&fnv1a(&body).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SaveError> {
        if bytes.len() < 14 || &bytes[0..4] != MAGIC {
            return Err(SaveError::BadMagic);
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version > SAVE_VERSION {
            return Err(SaveError::Version(version));
        }
        let sum = u64::from_le_bytes(bytes[6..14].try_into().unwrap());
        let body = &bytes[14..];
        if fnv1a(body) != sum {
            return Err(SaveError::BadChecksum);
        }
        // migrations begin with the first shipped save format; development formats before it do not load
        if version < SAVE_VERSION {
            return Err(SaveError::Version(version));
        }
        postcard::from_bytes(body).map_err(|e| SaveError::Decode(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_save() -> SaveFile {
        let mut save = SaveFile::new(11, 1);
        save.strain_code = Some("ROSE-ABCD".into());
        save.created_unix = 1_700_000_000;
        save.saved_unix = 1_700_003_600;
        save.day = 12;
        save.tick = 13_200;
        save.label = "shift 12 · Quarter · 104 slimos".into();
        save.world = (0..5000u32).map(|i| (i * 7 % 251) as u8).collect();
        save.ui = vec![1, 2, 3];
        save
    }

    #[test]
    fn a_save_round_trips_and_older_formats_are_refused() {
        let save = a_save();
        let back = SaveFile::decode(&save.encode()).unwrap();
        assert_eq!(back, save);
        // an older development format is refused by version, not misread
        let body = postcard::to_allocvec(&save).unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&17u16.to_le_bytes());
        bytes.extend_from_slice(&fnv1a(&body).to_le_bytes());
        bytes.extend_from_slice(&body);
        assert!(matches!(SaveFile::decode(&bytes), Err(SaveError::Version(17))));
    }

    #[test]
    fn corrupt_and_foreign_bytes_are_rejected() {
        let mut bytes = a_save().encode();
        assert!(SaveFile::decode(&bytes[..3]).is_err());
        let k = bytes.len() / 2;
        bytes[k] ^= 0xFF;
        assert_eq!(SaveFile::decode(&bytes), Err(SaveError::BadChecksum));
        let mut other = b"NOPE".to_vec();
        other.extend_from_slice(&[0; 20]);
        assert_eq!(SaveFile::decode(&other), Err(SaveError::BadMagic));
    }

    #[test]
    fn a_newer_format_is_refused_by_version() {
        let mut bytes = a_save().encode();
        bytes[4..6].copy_from_slice(&(SAVE_VERSION + 1).to_le_bytes());
        assert_eq!(SaveFile::decode(&bytes), Err(SaveError::Version(SAVE_VERSION + 1)));
    }
}
