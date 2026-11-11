//! The collections (Full Design v3.18, Progression: Collections): the Scrapdex, the curio sets' passives, the
//! trophies. The Memorial is the roster's; the Strainbook the strain's.

use items::{Category, ItemInstance, ItemTable};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const COLLECTIONS_TOML: &str = include_str!("../../../../data/collections.toml");

#[derive(Clone, Debug, Deserialize)]
pub struct SetDef {
    pub id: String,
    pub name: String,
    pub flag: String,
    pub text: String,
    #[serde(default)]
    pub design: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrophyDef {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub key: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionsFile {
    pub scrapdex_skip: Vec<String>,
    pub set: Vec<SetDef>,
    pub trophy: Vec<TrophyDef>,
}

impl CollectionsFile {
    pub fn from_toml(text: &str) -> Result<CollectionsFile, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    pub fn shipped() -> &'static CollectionsFile {
        static F: OnceLock<CollectionsFile> = OnceLock::new();
        F.get_or_init(|| CollectionsFile::from_toml(COLLECTIONS_TOML).expect("data/collections.toml"))
    }

    /// Whether an item's kind has a Scrapdex entry: every category but the skipped.
    pub fn in_scrapdex(&self, c: Category) -> bool {
        let key = format!("{c:?}").to_lowercase();
        !self.scrapdex_skip.contains(&key)
    }

    /// The Scrapdex's entries in the table's order.
    pub fn scrapdex<'a>(&self, table: &'a ItemTable) -> Vec<&'a str> {
        table.defs().filter(|d| self.in_scrapdex(d.category)).map(|d| d.id.as_str()).collect()
    }

    /// The passives the Stash's curio sets hold now: three of a set (Full Design: Curios).
    pub fn set_flags<'a>(&self, table: &ItemTable, items: impl Iterator<Item = &'a ItemInstance>) -> Vec<String> {
        items::set_bonuses(table, items).into_iter().filter(|b| b.complete).filter_map(|b| self.set.iter().find(|s| s.id == b.set)).map(|s| s.flag.clone()).collect()
    }
}

/// The collections as the city stands (in the save with the colony).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Collections {
    /// Scrapdex entries seen, by item id, with the shift
    pub scrapdex: Vec<(String, u32)>,
    /// trophies won, by id, with the shift
    pub trophies: Vec<(String, u32)>,
    /// the curio sets' passives as of the last shift's end
    #[serde(default)]
    pub sets: Vec<String>,
    /// the city-tier title cards shown: 20, 100, 500, 2000
    #[serde(default)]
    pub titles: Vec<u32>,
}

impl Collections {
    pub fn seen(&self, id: &str) -> bool {
        self.scrapdex.iter().any(|(s, _)| s == id)
    }

    /// An item identified: its Scrapdex entry is seen. True when it is new.
    pub fn see(&mut self, table: &ItemTable, it: &ItemInstance, day: u32) -> bool {
        let d = table.def(it.def);
        if it.ident != items::Ident::Full || !CollectionsFile::shipped().in_scrapdex(d.category) || self.seen(&d.id) {
            return false;
        }
        self.scrapdex.push((d.id.clone(), day));
        true
    }

    pub fn has_trophy(&self, id: &str) -> bool {
        self.trophies.iter().any(|(t, _)| t == id)
    }

    /// A body down (by the run's creature id or the city's threat role) or a hazard lived through: the trophy's name
    /// when it is new.
    pub fn trophy(&mut self, kind: &str, key: &str, day: u32) -> Option<String> {
        let f = CollectionsFile::shipped();
        let t = f.trophy.iter().find(|t| t.kind == kind && t.key == key)?;
        if self.has_trophy(&t.id) {
            return None;
        }
        self.trophies.push((t.id.clone(), day));
        Some(t.name.clone())
    }

    pub fn set_flag(&self, flag: &str) -> bool {
        self.sets.iter().any(|s| s == flag)
    }

    /// A population count reached for the first time: its title card.
    pub fn title(&mut self, population: usize) -> Option<(u32, &'static str)> {
        const TITLES: [(u32, &str); 4] = [(20, "Hamlet"), (100, "Quarter"), (500, "City"), (2000, "Empire")];
        for (n, name) in TITLES {
            if population as u32 >= n && !self.titles.contains(&n) {
                self.titles.push(n);
                return Some((n, name));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scrapdex_is_150_and_sets_and_trophies_load() {
        let f = CollectionsFile::from_toml(COLLECTIONS_TOML).unwrap();
        let table = ItemTable::from_toml(items::ITEMS_TOML).unwrap();
        assert_eq!(f.scrapdex(&table).len(), 150);
        assert_eq!(f.set.len(), 7);
        assert_eq!(f.trophy.len(), 12);
        // the sets name the curios' sets
        for s in &f.set {
            assert!(table.defs().any(|d| d.set.as_deref() == Some(s.id.as_str())), "{}", s.id);
        }
        let mut c = Collections::default();
        assert_eq!(c.trophy("kill", "flamer", 3), Some("Flamer".into()));
        assert_eq!(c.trophy("kill", "flamer", 4), None);
        assert_eq!(c.title(25), Some((20, "Hamlet")));
        assert_eq!(c.title(25), None);
    }
}
