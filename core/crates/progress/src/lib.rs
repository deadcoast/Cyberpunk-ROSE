//! progress — the city's progression (Full Design v3.18, Progression; research 21).
//!
//! * `tech`    the 36-node tree, blueprints and Research
//! * `strain`  the organ kinds, the classes, the Strainbook, the lineage
//! * `collect` the Scrapdex, the curio sets' passives, the trophies, the title cards
//!
//! The state is one `Progress` the colony carries (and the save with it); the organs themselves grow in the `mat`
//! crate, and the colony, the threats, the run and the economy read the flags.

pub mod collect;
pub mod strain;
pub mod tech;

pub use collect::{Collections, CollectionsFile};
pub use strain::{Strain, StrainFile, StrainShift};
pub use tech::{Done, Project, Tech, TechFile};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub tech: Tech,
    pub strain: Strain,
    pub collections: Collections,
}
