//! Manifest loading: every TOML under game/assets/manifest, plus the `expand = "items"` slot expanded
//! into one icon slot per entry of data/items.toml.

use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct Slot {
    pub id: String,
    pub kind: String,
    pub size: [u32; 2],
    pub frames: u32,
    pub states: Vec<String>,
    pub ramp: String,
    pub silhouette: String,
    pub tag: String,
    #[serde(default)]
    pub anchor: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub negative: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub expand: Option<String>,
    /// manifest file this came from (set by the loader)
    #[serde(skip)]
    pub source: String,
    /// for expanded item icons: the item id
    #[serde(skip)]
    pub expanded_from: Option<String>,
}

fn default_status() -> String {
    "placeholder".into()
}

impl Slot {
    pub fn file_name(&self, state: &str) -> String {
        if state.is_empty() { format!("{}.png", self.id) } else { format!("{}.{}.png", self.id, state) }
    }
}

#[derive(Debug, Deserialize)]
struct File {
    slot: Vec<Slot>,
}

#[derive(Debug, Deserialize)]
struct ItemRow {
    id: String,
    name: String,
    tier: String,
    size: [u32; 2],
    #[serde(default)]
    codex: String,
}

#[derive(Debug, Deserialize)]
struct ItemsFile {
    item: Vec<ItemRow>,
}

pub struct Manifest {
    pub slots: Vec<Slot>,
    pub files: usize,
    pub expanded_items: usize,
}

impl Manifest {
    pub fn load(dir: &Path, items_path: &Path) -> Result<Self, String> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect();
        entries.sort();
        let mut slots = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut expanded_items = 0;
        for path in &entries {
            let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            let f: File = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            let source = path.file_name().unwrap().to_string_lossy().to_string();
            for mut s in f.slot {
                if s.size[0] == 0 || s.size[1] == 0 || s.frames == 0 || s.states.is_empty() || s.tag.chars().count() != 3 {
                    return Err(format!("{}: slot {} has a bad size, frames, states or tag", source, s.id));
                }
                if !seen.insert(s.id.clone()) {
                    return Err(format!("duplicate slot id {}", s.id));
                }
                s.source = source.clone();
                if s.expand.as_deref() == Some("items") {
                    let items_text = std::fs::read_to_string(items_path).map_err(|e| format!("{}: {e}", items_path.display()))?;
                    let items: ItemsFile = toml::from_str(&items_text).map_err(|e| e.to_string())?;
                    let template = s.clone();
                    slots.push(s);
                    for it in items.item {
                        let mut child = template.clone();
                        child.id = format!("icon.item.{}", it.id);
                        child.expand = None;
                        child.states = vec![String::new()];
                        child.frames = 1;
                        child.tag = tag_for(&it.id, &it.tier);
                        child.prompt = format!("{} — {}. {}", it.name, it.codex, template.prompt);
                        child.silhouette = if it.size[0] >= 3 || it.size[1] >= 3 { "strip".into() } else { "rect".into() };
                        child.expanded_from = Some(it.id.clone());
                        child.source = source.clone();
                        slots.push(child);
                        expanded_items += 1;
                    }
                } else {
                    slots.push(s);
                }
            }
        }
        Ok(Manifest { slots, files: entries.len(), expanded_items })
    }

    pub fn slot_for_file(&self, file_name: &str) -> Option<(&Slot, String)> {
        let stem = file_name.strip_suffix(".png")?;
        for s in &self.slots {
            if s.expand.is_some() {
                continue;
            }
            if stem == s.id {
                return Some((s, String::new()));
            }
            if let Some(rest) = stem.strip_prefix(&format!("{}.", s.id)) {
                if s.states.iter().any(|st| st == rest) {
                    return Some((s, rest.to_string()));
                }
            }
        }
        None
    }
}

/// Tier letter + first two letters of the item's own name part (after the category prefix).
fn tag_for(id: &str, tier: &str) -> String {
    let t = match tier {
        "common" => 'C',
        "uncommon" => 'U',
        "rare" => 'R',
        "blackline" => 'B',
        "relic" => 'X',
        _ => '?',
    };
    let body: String = id.split('_').nth(1).unwrap_or(id).chars().filter(|c| c.is_ascii_alphanumeric()).take(2).collect::<String>().to_uppercase();
    format!("{t}{body:0>2}")
}
