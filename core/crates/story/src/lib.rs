//! story — the campaign as data and its pure rules (Full Design v3.19, Narrative and Ivan): the seven chapters with
//! their panels and captions, the five endings and their scores, the campaign's state as a save keeps it, the string
//! table's tokens. What the chapters do to the city is the game crate's (`game::campaign`); this crate decides
//! nothing that needs the city.

pub mod campaign;
pub mod chapters;
pub mod strings;

pub use campaign::{Accord, Campaign, Difficulty, Mode};
pub use chapters::{ChapterDef, ChapterFile, EndingDef, Panel};
