//! story — the campaign as data and its pure rules (Full Design v3.19, Narrative and Ivan): the seven chapters with
//! their panels and captions, the five endings and their scores, the campaign's state as a save keeps it, the string
//! table's tokens. What the chapters do to the city is the game crate's (`game::campaign`); this crate decides
//! nothing that needs the city.

pub mod barks;
pub mod campaign;
pub mod chapters;
pub mod onboarding;
pub mod strings;

pub use barks::{Barker, Ctx};
pub use campaign::{Accord, Campaign, Difficulty, Ivan, Mode, IVAN_ID};
pub use chapters::{ChapterDef, ChapterFile, EndingDef, Panel};
