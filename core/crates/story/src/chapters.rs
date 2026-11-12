//! The chapters and endings as data (`data/chapters.toml`; Full Design v3.19, Narrative and Ivan).

use serde::Deserialize;
use std::sync::OnceLock;

const CHAPTERS_TOML: &str = include_str!("../../../../data/chapters.toml");

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Panel {
    pub slot: String,
    pub caption: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChapterDef {
    pub n: u8,
    pub id: String,
    pub name: String,
    pub opens: String,
    pub goal: String,
    /// the goal as the HUD's chapter card says it
    pub card: String,
    pub beat: String,
    pub open: Vec<Panel>,
    pub close: Vec<Panel>,
    /// chapter 3's closing captions when the envoy was refused
    #[serde(default)]
    pub close_refused: Vec<Panel>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EndingDef {
    pub id: String,
    pub name: String,
    /// the two genes whose mean scores it (none: Scattered)
    pub genes: Vec<String>,
    /// added to the score for each step of the ally's reputation
    pub ally: f32,
    pub panels: Vec<Panel>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChapterFile {
    pub chapter: Vec<ChapterDef>,
    pub ending: Vec<EndingDef>,
}

/// The stratum the city must have reached for a chapter to open (Full Design v3.19, the chapters' table): the
/// Culverts for chapter 3, the Storm Drains for 4, the Service Tunnels for 5, the Vault for 6.
pub fn opens_at(n: u8) -> u8 {
    match n {
        3 => 2,
        4 => 3,
        5 => 4,
        6 => 5,
        _ => 1,
    }
}

impl ChapterFile {
    pub fn shipped() -> &'static ChapterFile {
        static F: OnceLock<ChapterFile> = OnceLock::new();
        F.get_or_init(|| {
            let f: ChapterFile = toml::from_str(CHAPTERS_TOML).expect("data/chapters.toml");
            assert!(f.chapter.iter().enumerate().all(|(k, c)| c.n as usize == k + 1), "chapters in order");
            f
        })
    }

    pub fn get(&self, n: u8) -> Option<&ChapterDef> {
        self.chapter.get((n as usize).wrapping_sub(1))
    }

    pub fn ending(&self, id: &str) -> Option<&EndingDef> {
        self.ending.iter().find(|e| e.id == id)
    }

    /// Every panel slot the campaign shows.
    pub fn slots(&self) -> Vec<&str> {
        let mut v: Vec<&str> = Vec::new();
        for c in &self.chapter {
            v.extend(c.open.iter().chain(c.close.iter()).chain(c.close_refused.iter()).map(|p| p.slot.as_str()));
        }
        v.extend(self.ending.iter().flat_map(|e| e.panels.iter().map(|p| p.slot.as_str())));
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Each strain ending's score at the Great Cleansing's end (Full Design v3.19, Endings): its two genes' mean, plus
    /// its `ally` share for each step of the ally's reputation. Scattered has none.
    pub fn scores(&self, genome: &[f32; 10], ally_rep: i32) -> Vec<(&str, f32)> {
        self.ending
            .iter()
            .filter(|e| e.genes.len() == 2)
            .map(|e| {
                let g: Vec<f32> = e.genes.iter().map(|k| progress::strain::gene_index(k).map_or(0.0, |i| genome[i])).collect();
                (e.id.as_str(), (g[0] + g[1]) / 2.0 + e.ally * ally_rep as f32)
            })
            .collect()
    }

    /// The ending (Full Design v3.19): Scattered when the Heart burned or Ivan fell; otherwise the strain ending with
    /// the highest score, ties in the table's order.
    pub fn ending_of(&self, genome: &[f32; 10], ally_rep: i32, heart_lost: bool, ivan_fell: bool) -> &EndingDef {
        if heart_lost || ivan_fell {
            return self.ending("scattered").expect("the Scattered ending");
        }
        let scores = self.scores(genome, ally_rep);
        let mut best = scores[0];
        for &s in &scores[1..] {
            if s.1 > best.1 + 1e-6 {
                best = s;
            }
        }
        self.ending(best.0).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_chapters_of_four_and_four_and_five_endings_of_four() {
        let f = ChapterFile::shipped();
        assert_eq!(f.chapter.len(), 7);
        assert!(f.chapter.iter().all(|c| c.open.len() == 4 && c.close.len() == 4));
        assert_eq!(f.get(3).unwrap().close_refused.len(), 4);
        assert_eq!(f.ending.len(), 5);
        assert!(f.ending.iter().all(|e| e.panels.len() == 4));
        assert_eq!(f.slots().len(), 76, "56 chapter panels and 20 ending panels");
        let sentences = |c: &str| c.matches(". ").count() + c.matches("? ").count() + c.matches("! ").count() + 1;
        let all: Vec<&Panel> = f.chapter.iter().flat_map(|c| c.open.iter().chain(c.close.iter()).chain(c.close_refused.iter())).chain(f.ending.iter().flat_map(|e| e.panels.iter())).collect();
        assert!(all.iter().all(|p| sentences(&p.caption) <= 2), "two sentences at most");
    }

    #[test]
    fn the_genome_reads_its_ending_and_the_alliance_tips_garden_against_world_eater() {
        let f = ChapterFile::shipped();
        let mut g = [0.5f32; 10];
        let ix = |k: &str| progress::strain::gene_index(k).unwrap();
        g[ix("lume")] = 0.8;
        g[ix("thrift")] = 0.7;
        assert_eq!(f.ending_of(&g, 1, false, false).id, "garden");
        let mut h = [0.5f32; 10];
        h[ix("chrome")] = 0.8;
        h[ix("conductance")] = 0.75;
        assert_eq!(f.ending_of(&h, 2, false, false).id, "hive");
        let mut w = [0.5f32; 10];
        w[ix("appetite")] = 0.8;
        w[ix("feral")] = 0.8;
        assert_eq!(f.ending_of(&w, 0, false, false).id, "world_eater");
        let mut a = [0.5f32; 10];
        a[ix("thrift")] = 0.8;
        a[ix("toxicity")] = 0.85;
        assert_eq!(f.ending_of(&a, 0, false, false).id, "accord");
        assert_eq!(f.ending_of(&a, 0, true, false).id, "scattered");
        // a Garden and a World-Eater level on their genes: the ally decides
        let mut t = [0.5f32; 10];
        t[ix("lume")] = 0.7;
        t[ix("thrift")] = 0.7;
        t[ix("appetite")] = 0.7;
        t[ix("feral")] = 0.7;
        assert_eq!(f.ending_of(&t, 2, false, false).id, "garden");
        assert_eq!(f.ending_of(&t, -1, false, false).id, "world_eater");
    }
}
