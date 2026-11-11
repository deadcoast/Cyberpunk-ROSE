//! The tech tree (Full Design v3.18, Progression: Research; Pass 2's Pillar 6; research 21): 36 nodes in
//! `data/tech.toml`, three branches by four tiers. Tier 1 is the founding's; a blueprint studied opens a branch's next
//! tier; a node of an open tier is researched by points from slimos on Research at a Tweaker Bench.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const TECH_TOML: &str = include_str!("../../../../data/tech.toml");

pub const BRANCHES: [&str; 3] = ["chrome", "flesh", "mycelium"];
pub const BRANCH_NAMES: [&str; 3] = ["Chrome", "Flesh", "Mycelium"];

#[derive(Clone, Debug, Deserialize)]
pub struct NodeDef {
    pub id: String,
    pub branch: String,
    pub tier: u8,
    pub name: String,
    #[serde(default)]
    pub structures: Vec<String>,
    #[serde(default)]
    pub laws: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    pub text: String,
    #[serde(default)]
    pub pass2: bool,
}

impl NodeDef {
    pub fn branch_index(&self) -> usize {
        BRANCHES.iter().position(|b| *b == self.branch).unwrap_or(0)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Blueprints {
    pub chrome: Vec<String>,
    pub flesh: Vec<String>,
    pub mycelium: Vec<String>,
}

impl Blueprints {
    fn of(&self, b: usize) -> &[String] {
        match b {
            0 => &self.chrome,
            1 => &self.flesh,
            _ => &self.mycelium,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TechFile {
    pub points: [u32; 4],
    pub parts: [u32; 4],
    pub chrome: [u32; 4],
    pub study: [u32; 3],
    pub research_gate: u8,
    pub blueprints: Blueprints,
    pub node: Vec<NodeDef>,
}

impl TechFile {
    pub fn from_toml(text: &str) -> Result<TechFile, String> {
        let f: TechFile = toml::from_str(text).map_err(|e| e.to_string())?;
        if f.node.len() != 36 {
            return Err(format!("the tree is 36 nodes, not {}", f.node.len()));
        }
        for (b, name) in BRANCHES.iter().enumerate() {
            for t in 1..=4u8 {
                let n = f.node.iter().filter(|n| n.branch == *name && n.tier == t).count();
                if n != 3 {
                    return Err(format!("{name} tier {t} has {n} nodes, not 3"));
                }
            }
            if f.blueprints.of(b).len() != 3 {
                return Err(format!("{name} needs three blueprints"));
            }
        }
        Ok(f)
    }

    /// The shipped tree, parsed once.
    pub fn shipped() -> &'static TechFile {
        static F: OnceLock<TechFile> = OnceLock::new();
        F.get_or_init(|| TechFile::from_toml(TECH_TOML).expect("data/tech.toml"))
    }

    pub fn get(&self, id: &str) -> Option<&NodeDef> {
        self.node.iter().find(|n| n.id == id)
    }

    /// The node that names a structure, if any does.
    pub fn structure_node(&self, def_id: &str) -> Option<&NodeDef> {
        self.node.iter().find(|n| n.structures.iter().any(|s| s == def_id))
    }

    /// The node that names a law, if any does.
    pub fn law_node(&self, law: &str) -> Option<&NodeDef> {
        self.node.iter().find(|n| n.laws.iter().any(|s| s == law))
    }

    /// A blueprint item's branch and the tier it opens (I opens 2, II 3, III 4).
    pub fn blueprint(&self, item_id: &str) -> Option<(usize, u8)> {
        for b in 0..3 {
            if let Some(k) = self.blueprints.of(b).iter().position(|x| x == item_id) {
                return Some((b, k as u8 + 2));
            }
        }
        None
    }

    /// Points a project asks.
    pub fn points_of(&self, p: &Project) -> u32 {
        match p {
            Project::Node(id) => self.get(id).map_or(0, |n| self.points[(n.tier as usize).clamp(1, 4) - 1]),
            Project::Study(item) => self.blueprint(item).map_or(0, |(_, t)| self.study[(t as usize - 2).min(2)]),
        }
    }

    /// Parts and Chrome a project costs (a study costs its blueprint).
    pub fn cost_of(&self, p: &Project) -> (u32, u32) {
        match p {
            Project::Node(id) => self.get(id).map_or((0, 0), |n| {
                let t = (n.tier as usize).clamp(1, 4) - 1;
                (self.parts[t], self.chrome[t])
            }),
            Project::Study(_) => (0, 0),
        }
    }
}

/// What the Research slots work on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Project {
    /// a node of an open tier, by id
    Node(String),
    /// a blueprint from the Stash, by item id: studying it opens its branch's next tier
    Study(String),
}

/// What a finished project did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Done {
    Node(String),
    /// branch, tier opened
    Opened(usize, u8),
}

/// The tree as the city stands (in the save with the colony).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tech {
    /// node ids researched, tier 1's first
    pub researched: Vec<String>,
    /// the highest tier open in each branch (1 at the founding)
    pub open: [u8; 3],
    pub project: Option<Project>,
    /// points got on the project standing
    pub got: u32,
    /// points kept on projects dropped before they finished
    pub kept: Vec<(Project, u32)>,
    /// (shift, what) for each project finished: the Bench's history
    pub history: Vec<(u32, String)>,
}

impl Default for Tech {
    fn default() -> Self {
        Tech::founding()
    }
}

impl Tech {
    /// The founding: every branch's tier 1 stands researched (Full Design v3.18: the founding's kit).
    pub fn founding() -> Tech {
        let f = TechFile::shipped();
        Tech { researched: f.node.iter().filter(|n| n.tier == 1).map(|n| n.id.clone()).collect(), open: [1, 1, 1], project: None, got: 0, kept: Vec::new(), history: Vec::new() }
    }

    /// Every node researched and every tier open: the scenarios that test a system the tree gates (the Cleansing's
    /// Gates, a Still) and the tests.
    pub fn all() -> Tech {
        let f = TechFile::shipped();
        Tech { researched: f.node.iter().map(|n| n.id.clone()).collect(), open: [4, 4, 4], project: None, got: 0, kept: Vec::new(), history: Vec::new() }
    }

    /// Every node that names a structure or a law researched, none of the tree's numbers: the Cleansing proof's
    /// city, which stands Gates and Turrets on the design's own numbers, and the tests.
    pub fn every_structure() -> Tech {
        let f = TechFile::shipped();
        let mut t = Tech::all();
        t.researched.retain(|id| f.get(id).is_some_and(|n| n.flags.is_empty()));
        t
    }

    pub fn has(&self, id: &str) -> bool {
        self.researched.iter().any(|r| r == id)
    }

    /// Whether a researched node carries a flag (the numbers the crates read).
    pub fn flag(&self, flag: &str) -> bool {
        let f = TechFile::shipped();
        self.researched.iter().filter_map(|id| f.get(id)).any(|n| n.flags.iter().any(|x| x == flag))
    }

    /// A structure the tree has not let be placed yet: the node it waits on.
    pub fn structure_locked(&self, def_id: &str) -> Option<&'static NodeDef> {
        TechFile::shipped().structure_node(def_id).filter(|n| !self.has(&n.id))
    }

    /// A law the tree has not let be signed yet: the node it waits on.
    pub fn law_locked(&self, law: &str) -> Option<&'static NodeDef> {
        TechFile::shipped().law_node(law).filter(|n| !self.has(&n.id))
    }

    /// Recipe tiers the Bench has open (Full Design v3.18): Chrome's open tier, capped by the best Bench standing —
    /// tier 1 to recipe tier 1, tier 2 to 2, tier 3 to 4.
    pub fn recipe_tier(&self, bench_tier: u8) -> u8 {
        let cap = match bench_tier {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 4,
        };
        self.open[0].min(cap)
    }

    /// Why a project cannot be picked, if it cannot (the Stash's side, the blueprint held and the cost, is the
    /// caller's).
    pub fn cannot_pick(&self, p: &Project) -> Option<String> {
        let f = TechFile::shipped();
        match p {
            Project::Node(id) => {
                let n = f.get(id)?;
                if self.has(id) {
                    return Some("researched".into());
                }
                let b = n.branch_index();
                if n.tier > self.open[b] {
                    return Some(format!("needs Blueprint {}", ["I", "II", "III"][(n.tier as usize - 2).min(2)]));
                }
                None
            }
            Project::Study(item) => {
                let Some((b, t)) = f.blueprint(item) else { return Some("not a blueprint".into()) };
                if self.open[b] >= t {
                    return Some(format!("{} tier {t} is open", BRANCH_NAMES[b]));
                }
                if self.open[b] + 1 < t {
                    return Some(format!("open {} tier {} first", BRANCH_NAMES[b], t - 1));
                }
                None
            }
        }
    }

    /// Pick a project (its cost already paid by the caller). A project standing is dropped and keeps its points.
    pub fn pick(&mut self, p: Project) -> Result<(), String> {
        if let Some(why) = self.cannot_pick(&p) {
            return Err(why);
        }
        self.drop_project();
        self.got = self.kept.iter().position(|(k, _)| *k == p).map(|i| self.kept.remove(i).1).unwrap_or(0);
        self.project = Some(p);
        Ok(())
    }

    /// Drop the project standing: its points are kept, its cost is not refunded.
    pub fn drop_project(&mut self) {
        if let Some(p) = self.project.take() {
            if self.got > 0 {
                self.kept.push((p, self.got));
            }
        }
        self.got = 0;
    }

    /// Whether a project was paid for before and dropped (picking it again costs nothing).
    pub fn was_paid(&self, p: &Project) -> bool {
        self.kept.iter().any(|(k, _)| k == p)
    }

    /// A shift's Research points. Returns what finished.
    pub fn work(&mut self, points: u32, day: u32) -> Option<Done> {
        let f = TechFile::shipped();
        let p = self.project.clone()?;
        self.got += points;
        if self.got < f.points_of(&p) {
            return None;
        }
        self.project = None;
        self.got = 0;
        match p {
            Project::Node(id) => {
                let name = f.get(&id).map_or(id.clone(), |n| n.name.clone());
                self.researched.push(id.clone());
                self.history.push((day, name));
                Some(Done::Node(id))
            }
            Project::Study(item) => {
                let (b, t) = f.blueprint(&item)?;
                self.open[b] = self.open[b].max(t);
                self.history.push((day, format!("{} tier {t} opened", BRANCH_NAMES[b])));
                Some(Done::Opened(b, t))
            }
        }
    }

    /// The project's name for the HUD and the log.
    pub fn project_name(&self) -> Option<String> {
        let f = TechFile::shipped();
        self.project.as_ref().map(|p| match p {
            Project::Node(id) => f.get(id).map_or(id.clone(), |n| n.name.clone()),
            Project::Study(item) => f.blueprint(item).map_or(item.clone(), |(b, t)| format!("study {} tier {t}", BRANCH_NAMES[b])),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tree_is_thirty_six_and_the_founding_holds_tier_one() {
        let f = TechFile::from_toml(TECH_TOML).unwrap();
        assert_eq!(f.node.len(), 36);
        assert_eq!(f.node.iter().filter(|n| !n.pass2).count(), 6, "six nodes are this design's");
        let t = Tech::founding();
        assert_eq!(t.researched.len(), 9);
        assert!(t.structure_locked("pod").is_none() && t.structure_locked("forge").is_none());
        assert_eq!(t.structure_locked("gate").map(|n| n.name.as_str()), Some("Sphincter housing"));
        assert_eq!(t.law_locked("dissolution_rite").map(|n| n.id.as_str()), Some("f4_rite"));
        assert_eq!(t.recipe_tier(3), 1, "Chrome's tier 1 caps the recipes");
    }

    #[test]
    fn a_blueprint_studied_opens_its_tier_and_points_finish_a_node() {
        let mut t = Tech::founding();
        let node = Project::Node("c2_gate".into());
        assert_eq!(t.cannot_pick(&node), Some("needs Blueprint I".into()));
        assert_eq!(t.cannot_pick(&Project::Study("tec_blueprint_chrome2".into())), Some("open Chrome tier 2 first".into()));
        t.pick(Project::Study("tec_blueprint_chrome1".into())).unwrap();
        assert_eq!(t.work(4, 3), None);
        assert_eq!(t.work(4, 4), Some(Done::Opened(0, 2)), "Blueprint I is 8 points");
        assert_eq!(t.open[0], 2);
        assert_eq!(t.recipe_tier(2), 2);
        assert_eq!(t.recipe_tier(1), 1, "a tier-1 Bench caps it");
        t.pick(node.clone()).unwrap();
        t.work(10, 5);
        // dropped, it keeps its 10 points; picked again it carries on from them
        t.pick(Project::Node("c2_vault".into())).unwrap();
        assert_eq!(t.got, 0);
        assert!(t.was_paid(&node));
        t.pick(node).unwrap();
        assert_eq!(t.got, 10);
        assert_eq!(t.work(14, 6), Some(Done::Node("c2_gate".into())), "a tier-2 node is 24");
        assert!(t.structure_locked("gate").is_none());
        assert_eq!(t.history.last().unwrap(), &(6, "Sphincter housing".to_string()));
    }
}
