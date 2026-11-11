//! The strain's progression (Full Design v3.18, Progression: Organs, Strain classes, The Strainbook): the organ kinds
//! and the classes from `data/strain.toml`, the classes the genome holds, the Strainbook's 38 cards and the lineage.

use mat::genome::GENE_NAMES;
use mat::organ::{Kinds, OrganDef, Site};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const STRAIN_TOML: &str = include_str!("../../../../data/strain.toml");

/// The genes by their key in the data files.
pub const GENE_KEYS: [&str; 10] = ["vigor", "branching", "thrift", "appetite", "lume", "toxicity", "chrome", "feral", "conductance", "plasticity"];

pub fn gene_index(key: &str) -> Option<usize> {
    GENE_KEYS.iter().position(|k| *k == key)
}

#[derive(Clone, Debug, Deserialize)]
pub struct OrganData {
    pub id: String,
    pub name: String,
    pub gene: String,
    pub line: f32,
    pub site: String,
    #[serde(default)]
    pub housing: Option<String>,
    pub text: String,
    pub art: String,
    #[serde(default)]
    pub pass1: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClassDef {
    pub id: String,
    pub name: String,
    pub genes: [String; 2],
    pub line: f32,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CardDef {
    pub id: String,
    pub name: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StrainFile {
    pub organ_spacing: i32,
    pub organ_per_nodes: usize,
    pub relic_state_shifts: u32,
    pub organ: Vec<OrganData>,
    pub class: Vec<ClassDef>,
    pub card: Vec<CardDef>,
}

impl StrainFile {
    pub fn from_toml(text: &str) -> Result<StrainFile, String> {
        let f: StrainFile = toml::from_str(text).map_err(|e| e.to_string())?;
        for o in &f.organ {
            gene_index(&o.gene).ok_or(format!("organ {}: gene {}", o.id, o.gene))?;
            Site::from_key(&o.site).ok_or(format!("organ {}: site {}", o.id, o.site))?;
        }
        for c in &f.class {
            for g in &c.genes {
                gene_index(g).ok_or(format!("class {}: gene {g}", c.id))?;
            }
        }
        Ok(f)
    }

    pub fn shipped() -> &'static StrainFile {
        static F: OnceLock<StrainFile> = OnceLock::new();
        F.get_or_init(|| StrainFile::from_toml(STRAIN_TOML).expect("data/strain.toml"))
    }

    /// The organ kinds as the Mat grows them.
    pub fn organ_defs(&self) -> Vec<OrganDef> {
        self.organ.iter().map(|o| OrganDef { gene: gene_index(&o.gene).unwrap(), line: o.line, site: Site::from_key(&o.site).unwrap() }).collect()
    }

    pub fn organ_index(&self, id: &str) -> Option<u8> {
        self.organ.iter().position(|o| o.id == id).map(|k| k as u8)
    }

    /// The kinds the Mat's step reads.
    pub fn kinds(&self) -> Kinds {
        Kinds { spur: self.organ_index("spur"), cyst: self.organ_index("cyst"), ganglion: self.organ_index("ganglion"), graft: self.organ_index("graft") }
    }

    /// Every card of the Strainbook, in the book's order: organs, classes, the rest.
    pub fn cards(&self) -> Vec<(&str, &str, &str)> {
        let mut v: Vec<(&str, &str, &str)> = Vec::new();
        v.extend(self.organ.iter().map(|o| (o.id.as_str(), o.name.as_str(), o.text.as_str())));
        v.extend(self.class.iter().map(|c| (c.id.as_str(), c.name.as_str(), c.text.as_str())));
        v.extend(self.card.iter().map(|c| (c.id.as_str(), c.name.as_str(), c.text.as_str())));
        v
    }

    /// Whether the genome stands over a class's line.
    pub fn crosses(&self, c: &ClassDef, g: &[f32; 10]) -> bool {
        c.genes.iter().all(|k| g[gene_index(k).unwrap()] > c.line)
    }
}

/// What the shift did to the strain, for the Strainbook (Full Design v3.18).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StrainShift {
    /// organ kinds grown this shift
    pub grown: Vec<u8>,
    /// genes that reflected: (gene, off the ceiling)
    pub bounced: Vec<(usize, bool)>,
    pub relic_fed: bool,
    /// feral tiles taken back into the city's strain
    pub reclaimed: u32,
    /// feral tissue of a blooming strain stands
    pub bloom: bool,
    pub hyphal: usize,
}

/// The strain as the city stands: the classes held, the Strainbook, the lineage.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Strain {
    /// classes held: (id, the shift it was first held this time)
    pub held: Vec<(String, u32)>,
    /// the last shift a Relic was fed
    pub relic_fed_at: Option<u32>,
    /// the Strainbook: (card id, the shift it was earned)
    pub book: Vec<(String, u32)>,
    /// the genome at every shift's end
    pub lineage: Vec<(u32, [f32; 10])>,
}

impl Strain {
    pub fn has_card(&self, id: &str) -> bool {
        self.book.iter().any(|(c, _)| c == id)
    }

    pub fn holds(&self, class: &str) -> bool {
        self.held.iter().any(|(c, _)| c == class)
    }

    fn earn(&mut self, f: &StrainFile, id: &str, day: u32, lines: &mut Vec<String>) {
        if !self.has_card(id) {
            self.book.push((id.to_string(), day));
            let name = f.cards().into_iter().find(|c| c.0 == id).map_or(id.to_string(), |c| c.1.to_string());
            lines.push(format!("Strainbook: {name}"));
        }
    }

    /// The shift's end for the strain: the classes the genome holds now, the cards earned, the lineage. Returns the
    /// lines for the log.
    pub fn end_shift(&mut self, f: &StrainFile, genome: &[f32; 10], day: u32, s: &StrainShift) -> Vec<String> {
        let mut lines = Vec::new();
        if s.relic_fed {
            self.relic_fed_at = Some(day);
            self.earn(f, "relic_fed", day, &mut lines);
        }
        for &k in &s.grown {
            if let Some(o) = f.organ.get(k as usize) {
                let id = o.id.clone();
                self.earn(f, &id, day, &mut lines);
            }
        }
        for &(g, ceiling) in &s.bounced {
            if ceiling {
                self.earn(f, &format!("reflect_{}", GENE_KEYS[g]), day, &mut lines);
            } else {
                self.earn(f, "floor", day, &mut lines);
            }
        }
        if s.reclaimed > 0 {
            self.earn(f, "merge_shock", day, &mut lines);
        }
        if s.bloom {
            self.earn(f, "first_bloom", day, &mut lines);
        }
        if s.hyphal >= 2000 {
            self.earn(f, "wide", day, &mut lines);
        }
        if genome.iter().any(|&v| v >= 0.9) {
            self.earn(f, "monoculture", day, &mut lines);
        }
        if let Some((_, prev)) = self.lineage.last() {
            if (0..10).any(|g| (genome[g] - prev[g]).abs() >= 0.1) {
                self.earn(f, "swing", day, &mut lines);
            }
        }
        // the classes: lost under the line, held over it, two at most, three only near a Relic fed (Pass 2)
        let lost: Vec<String> = self.held.iter().filter(|(c, _)| f.class.iter().find(|d| d.id == *c).is_none_or(|d| !f.crosses(d, genome))).map(|(c, _)| c.clone()).collect();
        for c in &lost {
            self.held.retain(|(h, _)| h != c);
            let name = f.class.iter().find(|d| d.id == *c).map_or(c.as_str(), |d| d.name.as_str());
            lines.push(format!("the strain lost {name}"));
            self.earn(f, "class_lost", day, &mut lines);
        }
        let relic = self.relic_fed_at.is_some_and(|r| day.saturating_sub(r) <= f.relic_state_shifts);
        let cap = if relic { 3 } else { 2 };
        while self.held.len() > cap {
            // the class held longest goes first
            let k = (0..self.held.len()).min_by_key(|&k| self.held[k].1).unwrap();
            let gone = self.held.remove(k);
            let name = f.class.iter().find(|d| d.id == gone.0).map_or(gone.0.clone(), |d| d.name.clone());
            lines.push(format!("the strain let {name} go: three classes are a Relic's"));
        }
        for d in &f.class {
            if self.held.len() >= cap {
                break;
            }
            if f.crosses(d, genome) && !self.holds(&d.id) {
                self.held.push((d.id.clone(), day));
                lines.push(format!("the strain is {} now", d.name));
                let id = d.id.clone();
                self.earn(f, &id, day, &mut lines);
            }
        }
        if self.held.len() >= 2 {
            self.earn(f, "two_classes", day, &mut lines);
        }
        if self.held.len() >= 3 {
            self.earn(f, "relic_state", day, &mut lines);
        }
        self.lineage.push((day, *genome));
        lines
    }

    /// The genome's dominant gene by name (the bark pools and the endings read it).
    pub fn dominant(genome: &[f32; 10]) -> &'static str {
        let g = (0..10).max_by(|&a, &b| genome[a].partial_cmp(&genome[b]).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0);
        GENE_NAMES[g]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_book_is_thirty_eight_and_the_organs_twelve() {
        let f = StrainFile::from_toml(STRAIN_TOML).unwrap();
        assert_eq!(f.organ.len(), 12);
        assert_eq!(f.class.len(), 5);
        assert_eq!(f.cards().len(), 38);
        assert_eq!(f.organ.iter().filter(|o| o.pass1).count(), 4 + 1, "Pass 1's five");
        // every gene grows something
        for g in GENE_KEYS {
            assert!(f.organ.iter().any(|o| o.gene == g), "{g}");
        }
    }

    #[test]
    fn classes_are_held_over_their_lines_two_at_most_three_by_a_relic() {
        let f = StrainFile::shipped();
        let mut s = Strain::default();
        let mut g = [0.5f32; 10];
        g[4] = 0.7; // lume
        g[2] = 0.7; // thrift
        s.end_shift(f, &g, 1, &StrainShift::default());
        assert!(s.holds("garden"));
        g[6] = 0.7; // chrome
        g[8] = 0.7; // conductance
        g[0] = 0.7; // vigor: hive and runner both cross
        let lines = s.end_shift(f, &g, 2, &StrainShift::default());
        assert_eq!(s.held.len(), 2, "{lines:?}");
        assert!(s.holds("garden") && s.holds("hive"), "data order picks Hive before Runner");
        assert!(s.has_card("two_classes"));
        // a Relic fed: the third holds
        s.end_shift(f, &g, 3, &StrainShift { relic_fed: true, ..Default::default() });
        assert_eq!(s.held.len(), 3);
        assert!(s.has_card("relic_state") && s.has_card("relic_fed"));
        // ten shifts on, the class held longest goes
        s.end_shift(f, &g, 14, &StrainShift::default());
        assert_eq!(s.held.len(), 2);
        assert!(!s.holds("garden"), "Garden was held longest");
        // a gene under the line loses its class
        g[8] = 0.6;
        s.end_shift(f, &g, 15, &StrainShift::default());
        assert!(!s.holds("hive") && !s.holds("runner"));
        assert!(s.has_card("class_lost"));
        assert_eq!(s.lineage.len(), 5);
    }
}
