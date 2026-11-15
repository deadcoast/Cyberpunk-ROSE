//! The data files the rules read, loaded once a process (Phase G, PAC-101; CURRENT-STATE OVR-005): the item table,
//! the structures, the laws, the recipes, the traits, the names, the run's chunks, loot tables and creatures, the
//! factions, and the economy's knobs (`data/balance.toml`, PAC-104), which it installs into the `balance` crate.
//! Before this the live game read `data/*.toml` from disk while the proofs read the copies compiled into the binary,
//! and a file that failed to parse fell back to the compiled copy without a word.
//!
//! A process installs its data once, before it founds a World: the game from `res://../data` (the bridge, at the
//! extension's load), `rose` from `--data DIR`. A file the directory holds wins; a file it does not hold is the
//! compiled copy; a file it holds that does not parse is an error the caller reports, and that file falls back to
//! the compiled copy so the rest still loads. Without an install every reader gets the compiled data (the tests).
//! One data set a process: the files the rules read cannot differ between two Worlds of one process.

use std::path::Path;
use std::sync::OnceLock;

use bench::RecipeFile;
use colony::{LawFile, StructureFile};
use economy::FactionFile;
use items::ItemTable;
use people::{NameFile, TraitFile};
use run::{ChunkLib, CreatureFile, LootTables};

/// Where a file came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// the data directory's copy
    Disk,
    /// the copy compiled into the binary
    Compiled,
}

/// A data file the directory holds that does not parse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataError {
    pub file: &'static str,
    pub msg: String,
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "data/{}: {}", self.file, self.msg)
    }
}

#[derive(Clone, Debug)]
pub struct Data {
    pub items: ItemTable,
    pub structures: StructureFile,
    pub laws: LawFile,
    pub recipes: RecipeFile,
    pub traits: TraitFile,
    pub names: NameFile,
    pub chunks: ChunkLib,
    pub loot: LootTables,
    pub creatures: CreatureFile,
    pub factions: FactionFile,
    /// the economy's knobs (data/balance.toml, PAC-104): installed into the `balance` crate with the rest
    pub balance: balance::Balance,
    /// each file and where it came from, in the order above
    pub sources: Vec<(&'static str, Source)>,
}

/// One file: the directory's copy parsed when it holds one, else the compiled copy. A copy that does not parse is
/// reported and the compiled copy stands in.
fn one<T>(dir: Option<&Path>, file: &'static str, compiled: &str, parse: impl Fn(&str) -> Result<T, String>, sources: &mut Vec<(&'static str, Source)>, errors: &mut Vec<DataError>) -> T {
    if let Some(text) = dir.and_then(|d| std::fs::read_to_string(d.join(file)).ok()) {
        match parse(&text) {
            Ok(v) => {
                sources.push((file, Source::Disk));
                return v;
            }
            Err(msg) => errors.push(DataError { file, msg }),
        }
    }
    sources.push((file, Source::Compiled));
    parse(compiled).unwrap_or_else(|e| panic!("the compiled data/{file} does not parse: {e}"))
}

impl Data {
    /// Every file from the copies compiled into the binary.
    pub fn compiled() -> Data {
        Data::load(None).0
    }

    /// Every file from `dir` where it holds one, else compiled; the files that did not parse.
    pub fn load(dir: Option<&Path>) -> (Data, Vec<DataError>) {
        let (mut s, mut e) = (Vec::new(), Vec::new());
        let d = Data {
            items: one(dir, "items.toml", items::ITEMS_TOML, |t| ItemTable::from_toml(t).map_err(|e| e.to_string()), &mut s, &mut e),
            structures: one(dir, "structures.toml", colony::STRUCTURES_TOML, StructureFile::from_toml, &mut s, &mut e),
            laws: one(dir, "laws.toml", colony::charter::LAWS_TOML, LawFile::from_toml, &mut s, &mut e),
            recipes: one(dir, "recipes.toml", bench::RECIPES_TOML, RecipeFile::from_toml, &mut s, &mut e),
            traits: one(dir, "traits.toml", people::TRAITS_TOML, TraitFile::from_toml, &mut s, &mut e),
            names: one(dir, "names.toml", people::NAMES_TOML, NameFile::from_toml, &mut s, &mut e),
            chunks: one(dir, "chunks.toml", run::CHUNKS_TOML, ChunkLib::from_toml, &mut s, &mut e),
            loot: one(dir, "loot.toml", run::loot::LOOT_TOML, LootTables::from_toml, &mut s, &mut e),
            creatures: one(dir, "creatures.toml", run::creature::CREATURES_TOML, CreatureFile::from_toml, &mut s, &mut e),
            factions: one(dir, "factions.toml", economy::FACTIONS_TOML, FactionFile::from_toml, &mut s, &mut e),
            balance: one(dir, "balance.toml", balance::BALANCE_TOML, balance::Balance::from_toml, &mut s, &mut e),
            sources: Vec::new(),
        };
        (Data { sources: s, ..d }, e)
    }
}

static DATA: OnceLock<Data> = OnceLock::new();

/// Install the process's data. Once, before anything reads it: false when the data was already installed or
/// already read (the reader got the compiled data, and this install is refused so two sets never mix).
pub fn install(d: Data) -> bool {
    let knobs = balance::install(d.balance.clone());
    DATA.set(d).is_ok() && knobs
}

/// The process's data: what was installed, else the compiled copies.
pub fn get() -> &'static Data {
    DATA.get_or_init(Data::compiled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_wins_file_by_file_and_a_bad_file_is_reported() {
        let dir = std::env::temp_dir().join(format!("rose-data-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // laws.toml from the directory with one law renamed; recipes.toml that does not parse
        let laws = colony::charter::LAWS_TOML.replacen("name = \"", "name = \"Test ", 1);
        std::fs::write(dir.join("laws.toml"), &laws).unwrap();
        std::fs::write(dir.join("recipes.toml"), "this is [not toml").unwrap();
        let (d, errors) = Data::load(Some(&dir));
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].file, "recipes.toml");
        assert!(d.laws.law.iter().any(|l| l.name.starts_with("Test ")), "the directory's laws.toml was not read");
        assert_eq!(d.recipes.recipe.len(), RecipeFile::shipped().recipe.len(), "the bad recipes.toml did not fall back");
        let src = |f: &str| d.sources.iter().find(|(n, _)| *n == f).map(|p| p.1);
        assert_eq!(src("laws.toml"), Some(Source::Disk));
        assert_eq!(src("recipes.toml"), Some(Source::Compiled));
        assert_eq!(src("items.toml"), Some(Source::Compiled));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_compiled_data_is_the_shipped_files() {
        let d = Data::compiled();
        assert_eq!(d.structures.structure.len(), StructureFile::shipped().structure.len());
        assert_eq!(d.laws.law.len(), LawFile::shipped().law.len());
        assert!(d.sources.iter().all(|(_, s)| *s == Source::Compiled));
        assert_eq!(d.sources.len(), 11);
        assert_eq!(d.balance, balance::Balance::shipped());
    }
}
