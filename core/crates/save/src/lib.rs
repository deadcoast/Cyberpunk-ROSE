//! save — seed + input log + periodic snapshots. A save is small; a replay rebuilds the run from inputs.
//! Encoded with postcard (compact binary), versioned, checksummed.

use inventory::Stash;
use serde::{Deserialize, Serialize};

pub const SAVE_VERSION: u16 = 7;
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

/// A periodic full state capture so loading does not replay from tick 0. Opaque bytes per module.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub mat: Vec<u8>,
    pub city: Vec<u8>,
    pub stash: Stash,
}

/// What a route's fog remembers between runs: one bit per tile of the assembled route (run::Fog::memory_bits).
/// Keyed by the route's (seed, stratum); a size mismatch on load is ignored by the fog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteMemory {
    pub seed: u64,
    pub stratum: u8,
    pub w: u16,
    pub h: u16,
    pub bits: Vec<u8>,
}

/// A dead Runner's body on a route (Full Design, Death and recovery): where, when, and the rig it holds. The next run
/// of that route places it as a loot source after the raider roll.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorpseRecord {
    pub seed: u64,
    pub stratum: u8,
    pub x: u16,
    pub y: u16,
    pub chunk: String,
    pub day: u32,
    pub runner: String,
    /// the Rig, postcard-encoded (inventory::Rig), Spored gear already removed
    pub rig: Vec<u8>,
    pub raided: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u16,
    pub seed: u64,
    pub strain_code: Option<String>,
    pub difficulty: u8,
    pub created_unix: u64,
    pub inputs: Vec<TimedInput>,
    pub snapshots: Vec<Snapshot>,
    pub stash: Stash,
    pub play_ticks: u64,
    pub route_memory: Vec<RouteMemory>,
    pub corpses: Vec<CorpseRecord>,
    /// the Stash's queues, eddies and glut (v4)
    pub ledger: inventory::Ledger,
    /// the Bench queue and the tier open (v5)
    pub bench: bench::BenchQueue,
    /// the slimos: records, the memorial (v6)
    pub roster: people::Roster,
    /// the city: structures, zones, districts, homes (v7)
    pub colony: colony::Colony,
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
        SaveFile {
            version: SAVE_VERSION,
            seed,
            strain_code: None,
            difficulty,
            created_unix: 0,
            inputs: Vec::new(),
            snapshots: Vec::new(),
            stash: Stash::new(),
            play_ticks: 0,
            route_memory: Vec::new(),
            corpses: Vec::new(),
            ledger: inventory::Ledger::new(),
            bench: bench::BenchQueue::new(),
            roster: people::Roster::new(),
            colony: colony::Colony::new(),
        }
    }

    pub fn corpses_on(&self, seed: u64, stratum: u8) -> Vec<&CorpseRecord> {
        self.corpses.iter().filter(|c| c.seed == seed && c.stratum == stratum).collect()
    }

    /// Remove a recovered (or stripped and looted) corpse.
    pub fn forget_corpse(&mut self, seed: u64, stratum: u8, x: u16, y: u16) -> bool {
        let n = self.corpses.len();
        self.corpses.retain(|c| !(c.seed == seed && c.stratum == stratum && c.x == x && c.y == y));
        self.corpses.len() != n
    }

    /// Store a route's fog memory, replacing any earlier record of the same route.
    pub fn remember_route(&mut self, seed: u64, stratum: u8, w: u16, h: u16, bits: Vec<u8>) {
        self.route_memory.retain(|r| !(r.seed == seed && r.stratum == stratum));
        self.route_memory.push(RouteMemory { seed, stratum, w, h, bits });
    }

    pub fn route_memory(&self, seed: u64, stratum: u8) -> Option<&RouteMemory> {
        self.route_memory.iter().find(|r| r.seed == seed && r.stratum == stratum)
    }

    pub fn record(&mut self, tick: u64, input: Input) {
        self.inputs.push(TimedInput { tick, input });
        self.play_ticks = self.play_ticks.max(tick);
    }

    /// Inputs at or after `from_tick`, in order: what a replay feeds after loading the nearest snapshot.
    pub fn inputs_from(&self, from_tick: u64) -> impl Iterator<Item = &TimedInput> {
        self.inputs.iter().filter(move |i| i.tick >= from_tick)
    }

    pub fn latest_snapshot(&self) -> Option<&Snapshot> {
        self.snapshots.iter().max_by_key(|s| s.tick)
    }

    /// Keep at most `keep` snapshots (the newest), so saves stay small.
    pub fn trim_snapshots(&mut self, keep: usize) {
        self.snapshots.sort_by_key(|s| s.tick);
        while self.snapshots.len() > keep {
            self.snapshots.remove(0);
        }
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

    /// Strain code: genome as 10 two-hex-digit values plus a class letter and a checksum nibble.
    pub fn encode_strain(genome: &[f32; 10], class: char) -> String {
        let mut s = String::with_capacity(24);
        let mut sum = 0u32;
        for g in genome {
            let v = (g.clamp(0.0, 1.0) * 255.0).round() as u32;
            sum += v;
            s.push_str(&format!("{v:02X}"));
        }
        s.push(class);
        s.push_str(&format!("{:X}", sum % 16));
        s
    }

    pub fn decode_strain(code: &str) -> Option<([f32; 10], char)> {
        if code.len() != 22 {
            return None;
        }
        let mut g = [0.0f32; 10];
        let mut sum = 0u32;
        for i in 0..10 {
            let v = u32::from_str_radix(&code[i * 2..i * 2 + 2], 16).ok()?;
            sum += v;
            g[i] = v as f32 / 255.0;
        }
        let class = code.chars().nth(20)?;
        let check = u32::from_str_radix(&code[21..22], 16).ok()?;
        if check != sum % 16 {
            return None;
        }
        Some((g, class))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory::Grid;
    use items::{ItemInstance, ItemTable, Provenance};

    const DATA: &str = include_str!("../../../../data/items.toml");

    #[test]
    fn items_round_trip_through_a_save() {
        let t = ItemTable::from_toml(DATA).unwrap();
        let mut save = SaveFile::new(11, 1);
        let core = t.get("chr_lamp_core").unwrap();
        let found = ItemInstance::found(t.id_of("chr_lamp_core").unwrap(), 73, Provenance { stratum: 4, chunk: 9, day: 3 });
        let h = save.stash.grid.auto_place(core, found.clone()).unwrap();
        save.stash.grid.get_mut(h).unwrap().item.pinned = true;
        let mut vault = inventory::Container { name: "Vault".into(), grid: Grid::new(4, 4), accepts: vec![], vault: true };
        vault.grid.auto_place(t.get("rel_hive_seed").unwrap(), ItemInstance::new(t.id_of("rel_hive_seed").unwrap())).unwrap();
        save.stash.containers.push(vault);
        save.record(12, Input::Feed { x: 96, y: 25, item: 3, units: 5 });
        save.record(40, Input::Law { law: 2, sign: true });
        save.snapshots.push(Snapshot { tick: 40, mat: vec![1, 2, 3], city: vec![], stash: save.stash.clone() });

        let bytes = save.encode();
        let back = SaveFile::decode(&bytes).unwrap();
        assert_eq!(back, save);
        let item = &back.stash.grid.get(h).unwrap().item;
        assert_eq!(item.condition, 73);
        assert!(item.pinned && item.found_in_run);
        assert_eq!(item.provenance.unwrap().chunk, 9);
        assert_eq!(back.stash.exposed_contraband(&t), 1);
        assert_eq!(back.inputs_from(20).count(), 1);
    }

    #[test]
    fn route_memory_and_corpses_round_trip_and_older_formats_are_refused() {
        let mut save = SaveFile::new(7, 1);
        save.remember_route(11, 1, 160, 72, vec![0xFF; 160 * 72 / 8]);
        save.remember_route(11, 1, 160, 72, vec![0x0F; 160 * 72 / 8]);
        save.remember_route(12, 2, 192, 48, vec![1; 192 * 48 / 8]);
        assert_eq!(save.route_memory.len(), 2);
        save.corpses.push(CorpseRecord { seed: 11, stratum: 1, x: 40, y: 13, chunk: "sump_culvert".into(), day: 3, runner: "Pell".into(), rig: vec![1, 2, 3], raided: false });
        // the roster rides along (v6): six founders with their records
        save.roster.found(&people::TraitFile::shipped(), &people::NameFile::shipped(), 7, 1);
        save.colony.zones.push(colony::Zone { id: 9, kind: colony::ZoneKind::Stockpile, cells: vec![1, 2, 3], filter: colony::Filter { categories: vec!["chrome".into()], min_tier: 1, min_condition: 20 }, anchor: None });
        let back = SaveFile::decode(&save.encode()).unwrap();
        assert_eq!(back.roster, save.roster);
        assert_eq!(back.roster.records.len(), 6);
        assert_eq!(back.colony, save.colony);
        assert_eq!(back.route_memory(11, 1).unwrap().bits[0], 0x0F);
        assert!(back.route_memory(99, 1).is_none());
        assert_eq!(back.corpses_on(11, 1).len(), 1);
        let mut back = back;
        assert!(back.forget_corpse(11, 1, 40, 13));
        assert!(back.corpses_on(11, 1).is_empty());
        // an older development format is refused by version, not misread
        let body = postcard::to_allocvec(&save).unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&fnv1a(&body).to_le_bytes());
        bytes.extend_from_slice(&body);
        assert!(matches!(SaveFile::decode(&bytes), Err(SaveError::Version(2))));
    }

    #[test]
    fn corrupt_and_foreign_bytes_are_rejected() {
        let save = SaveFile::new(1, 0);
        let mut bytes = save.encode();
        assert!(SaveFile::decode(&bytes[..3]).is_err());
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        assert_eq!(SaveFile::decode(&bytes), Err(SaveError::BadChecksum));
        let mut other = b"NOPE".to_vec();
        other.extend_from_slice(&[0; 20]);
        assert_eq!(SaveFile::decode(&other), Err(SaveError::BadMagic));
    }

    #[test]
    fn strain_codes_round_trip() {
        let g = [0.5, 0.9, 0.1, 0.0, 1.0, 0.33, 0.66, 0.25, 0.75, 0.5];
        let code = SaveFile::encode_strain(&g, 'H');
        assert_eq!(code.len(), 22);
        let (back, class) = SaveFile::decode_strain(&code).unwrap();
        assert_eq!(class, 'H');
        for i in 0..10 {
            assert!((back[i] - g[i]).abs() < 1.0 / 255.0 + 1e-6);
        }
        let mut bad = code.clone();
        bad.replace_range(0..2, "FF");
        assert!(SaveFile::decode_strain(&bad).is_none());
    }

    #[test]
    fn snapshots_trim_to_newest() {
        let mut save = SaveFile::new(1, 0);
        for t in [10u64, 30, 20, 40, 50] {
            save.snapshots.push(Snapshot { tick: t, mat: vec![], city: vec![], stash: Stash::new() });
        }
        save.trim_snapshots(2);
        assert_eq!(save.snapshots.iter().map(|s| s.tick).collect::<Vec<_>>(), vec![40, 50]);
        assert_eq!(save.latest_snapshot().unwrap().tick, 50);
    }
}
