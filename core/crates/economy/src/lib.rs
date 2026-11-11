//! economy — the surface's factions on the city's clock (Full Design v3.15, Economy; research 19): the street a
//! faction works above each grate, its buy list and the day's demand on the surface's clock, reputation in steps of
//! favour and the stock tiers it opens, the stalls' books the ledger prices by, buying and services, and contract
//! chains. A day is three shifts. The city's crates hand it what happened (sales by faction, deliveries, raider kills,
//! a Bloom standing, the Attention at each grate) and act on what it returns (eddies paid, goods and services bought,
//! hosts to shelter, unique items, notes for the log).

use inventory::ledger::Order;
use items::{Book, Category, ItemId, ItemInstance, ItemTable, Tier};
use mat::rng::Pcg32;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const FACTIONS_TOML: &str = include_str!("../../../../data/factions.toml");

/// The categories in `Category`'s order (a book's rows).
pub const CATEGORIES: [Category; 9] = [Category::Chrome, Category::Part, Category::Organic, Category::Tech, Category::Gear, Category::Contraband, Category::Key, Category::Curio, Category::Relic];
pub const TIERS: [Tier; 5] = [Tier::Common, Tier::Uncommon, Tier::Rare, Tier::Blackline, Tier::Relic];

pub fn category_name(c: Category) -> &'static str {
    match c {
        Category::Chrome => "chrome",
        Category::Part => "part",
        Category::Organic => "organic",
        Category::Tech => "tech",
        Category::Gear => "gear",
        Category::Contraband => "contraband",
        Category::Key => "key",
        Category::Curio => "curio",
        Category::Relic => "relic",
    }
}

fn category_of(s: &str) -> Option<Category> {
    CATEGORIES.iter().copied().find(|c| category_name(*c) == s)
}

fn tier_of(s: &str) -> Option<Tier> {
    match s {
        "common" => Some(Tier::Common),
        "uncommon" => Some(Tier::Uncommon),
        "rare" => Some(Tier::Rare),
        "blackline" => Some(Tier::Blackline),
        "relic" => Some(Tier::Relic),
        _ => None,
    }
}

/// splitmix64: neighbouring seeds draw unrelated first numbers.
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn rng(seed: u64, what: u64, a: u64, b: u64) -> Pcg32 {
    Pcg32::new(mix(seed ^ mix(what.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (a << 32) ^ b)))
}

// ---------------------------------------------------------------- the data

#[derive(Clone, Debug, Deserialize)]
pub struct Rules {
    pub revert: f32,
    pub step: f32,
    pub band: f32,
    pub range: [f32; 2],
    pub favour: i32,
    pub rep_price: f32,
    pub rep_cap: i32,
    pub link_favour: i32,
    pub rival_favour: i32,
    pub recover: i32,
    pub haggle: f32,
    pub markup: f32,
    pub tier_rep: [i32; 4],
    pub tier_volume: [u32; 4],
    pub offers: usize,
    pub offer_days: u32,
    pub chain: [u8; 2],
    pub growth: f32,
    pub unique_condition: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StratumWeights {
    pub weights: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Streets {
    pub first: String,
    pub every: Vec<String>,
    pub stratum: Vec<StratumWeights>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Push {
    pub faction: String,
    pub category: String,
    #[serde(default)]
    pub tier: String,
    pub offset: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventDef {
    pub id: String,
    pub name: String,
    pub chance: f32,
    pub days: u32,
    #[serde(default)]
    pub push: Vec<Push>,
    #[serde(default)]
    pub flush: bool,
    #[serde(default)]
    pub stock: BTreeMap<String, f32>,
    #[serde(default)]
    pub stock_item: BTreeMap<String, f32>,
    #[serde(default)]
    pub chapter: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StockDef {
    pub tier: u8,
    #[serde(default)]
    pub item: String,
    #[serde(default)]
    pub service: String,
    pub units: u16,
    #[serde(default)]
    pub fixed: bool,
    #[serde(default)]
    pub price: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ContractDef {
    pub kind: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub item: String,
    #[serde(default = "one")]
    pub n: u16,
    #[serde(default)]
    pub condition: u8,
    pub shifts: u32,
    pub eddies: u32,
}

fn one() -> u16 {
    1
}
fn unit() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize)]
pub struct FactionDef {
    pub id: String,
    pub name: String,
    pub past: String,
    pub rival: String,
    pub lever: String,
    #[serde(default)]
    pub lever_at: f32,
    pub unique: String,
    pub rotate: u8,
    #[serde(default)]
    pub chapter: u8,
    #[serde(default)]
    pub fence: u8,
    #[serde(default = "unit")]
    pub fir: f32,
    #[serde(default)]
    pub demand: BTreeMap<String, f32>,
    #[serde(default)]
    pub items: BTreeMap<String, f32>,
    pub stock: Vec<StockDef>,
    pub contracts: Vec<ContractDef>,
}

impl FactionDef {
    /// The buy list's value for a category (1.0 where it names none).
    pub fn base(&self, c: Category) -> f32 {
        self.demand.get(category_name(c)).copied().unwrap_or(1.0)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServiceRules {
    pub hire: u32,
    pub convoy_fee: f32,
    pub intel_shifts: u32,
    pub timings_shifts: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FactionFile {
    pub rules: Rules,
    pub streets: Streets,
    #[serde(rename = "event")]
    pub events: Vec<EventDef>,
    #[serde(rename = "faction")]
    pub factions: Vec<FactionDef>,
    pub services: ServiceRules,
}

impl FactionFile {
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let f: FactionFile = toml::from_str(text).map_err(|e| e.to_string())?;
        for d in &f.factions {
            for k in d.demand.keys() {
                category_of(k).ok_or(format!("{}: no category {k}", d.id))?;
            }
            if !d.rival.is_empty() && !f.factions.iter().any(|o| o.id == d.rival) {
                return Err(format!("{}: no rival {}", d.id, d.rival));
            }
            for c in &d.contracts {
                if !c.category.is_empty() && category_of(&c.category).is_none() {
                    return Err(format!("{}: contract category {}", d.id, c.category));
                }
            }
        }
        Ok(f)
    }
    pub fn shipped() -> Self {
        Self::from_toml(FACTIONS_TOML).expect("data/factions.toml")
    }
    pub fn index(&self, id: &str) -> Option<u8> {
        self.factions.iter().position(|f| f.id == id).map(|i| i as u8)
    }
}

// ---------------------------------------------------------------- the state

/// The street above a grate: the grate's column and number on the stratum, and the faction that works it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Street {
    pub grate: u16,
    pub number: u8,
    pub faction: u8,
}

/// The city's standing with a faction: reputation −3..+3, favour toward the next step, eddies traded both ways, and
/// the tier a finished chain opened outright.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Standing {
    pub rep: i32,
    pub favour: i32,
    pub traded: u32,
    pub opened: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Deliver,
    ClearRaiders,
    ClearBloom,
    Recover,
    Host,
}

impl Kind {
    fn of(s: &str) -> Option<Kind> {
        Some(match s {
            "deliver" => Kind::Deliver,
            "clear_raiders" => Kind::ClearRaiders,
            "clear_bloom" => Kind::ClearBloom,
            "recover" => Kind::Recover,
            "host" => Kind::Host,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Kind::Deliver => "deliver",
            Kind::ClearRaiders => "clear raiders",
            Kind::ClearBloom => "clear the Bloom",
            Kind::Recover => "recover",
            Kind::Host => "host",
        }
    }
}

/// What a link asks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ask {
    pub category: Option<Category>,
    pub tier: Option<Tier>,
    pub item: Option<ItemId>,
    pub n: u16,
    pub condition: u8,
}

/// A contract offered: a chain's first link, standing until `until` (a day).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub id: u32,
    pub faction: u8,
    pub template: u16,
    pub links: u8,
    pub until: u32,
    /// the named newcomer, corpse or item (host and recover)
    pub name: String,
}

/// A link in hand.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub id: u32,
    pub faction: u8,
    pub template: u16,
    pub kind: Kind,
    pub link: u8,
    pub links: u8,
    pub ask: Ask,
    pub eddies: u32,
    /// the shift it was taken on and the shift it is due by
    pub from: u32,
    pub due: u32,
    pub got: u16,
    pub name: String,
    /// host: the record sheltered; recover: the route (stratum, seed) the body lies on
    pub host: Option<u32>,
    pub route: Option<(u8, u64)>,
}

impl Contract {
    pub fn line(&self, file: &FactionFile, table: &ItemTable) -> String {
        let f = &file.factions[self.faction as usize].name;
        let what = match self.kind {
            Kind::Deliver => {
                let thing = match (self.ask.item, self.ask.category) {
                    (Some(i), _) => table.def(i).name.clone(),
                    (None, Some(c)) => format!("{}{}", self.ask.tier.map(|t| format!("{t:?} ")).unwrap_or_default(), category_name(c)),
                    _ => "goods".into(),
                };
                let cond = if self.ask.condition > 0 { format!(" at {}+", self.ask.condition) } else { String::new() };
                format!("deliver {} {thing}{cond} ({}/{})", self.ask.n, self.got, self.ask.n)
            }
            Kind::ClearRaiders => format!("kill {} raiders on runs ({}/{})", self.ask.n, self.got, self.ask.n),
            Kind::ClearBloom => "clear the Bloom".into(),
            Kind::Recover => format!("recover {}", self.name),
            Kind::Host => format!("shelter {} five shifts", self.name),
        };
        format!("{f} {}/{}: {what}, {} e by shift {}", self.link, self.links, self.eddies, self.due)
    }
}

/// What the economy says happened, for the city's log: Gain lands, Loss is a loss, Event is the surface's clock.
#[derive(Clone, Debug, PartialEq)]
pub enum Note {
    Gain(String),
    Loss(String),
    Event(String),
}

/// Something bought at a stall and paid at the shift's end: goods to send down from the market, or a service.
#[derive(Clone, Debug, PartialEq)]
pub enum Bought {
    Goods { market: u8, item: ItemId, units: u16 },
    Hire { faction: u8 },
    Service { faction: u8, name: String },
}

/// What a shift's end hands back to the city.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ended {
    pub notes: Vec<Note>,
    pub paid: u32,
    /// the unique items of finished chains, into the Stash
    pub uniques: Vec<ItemInstance>,
    /// chains finished at their last link this shift (Authority's deed: Full Design v3.18)
    pub chains_done: u32,
}

/// What the city tells the shift's end.
#[derive(Clone, Debug, Default)]
pub struct ShiftFacts {
    /// (faction, units, eddies) sold this shift, from the ledger's report
    pub sold: Vec<(u8, u32, u32)>,
    /// a Bloom standing in the city now
    pub bloom: bool,
    /// the Attention at each grate (column, value) now
    pub attention: Vec<(u16, f32)>,
    /// records in the city, and those with the newcomers still waiting at a grate (a host is kept by the first and
    /// lost when it is in neither)
    pub hosts_here: Vec<u32>,
    pub hosts_alive: Vec<u32>,
    /// the grates the Mat reaches (columns)
    pub reached: Vec<u16>,
    /// the Road set holds: +2 favour a unit sold at a Nomad, Badge or Gang stall (Full Design v3.18, Collections)
    pub road_favour: bool,
    /// raiders can walk the Board's routes (a stratum with raiders is open)
    pub raiders: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EconomySave {
    pub seed: u64,
    pub streets: Vec<Street>,
    pub standing: Vec<Standing>,
    pub day: u32,
    pub walk: Vec<[f32; 9]>,
    pub events: Vec<(u16, u32)>,
    /// today's stock by faction: (entry, units left)
    pub stock: Vec<Vec<(u16, u16)>>,
    pub offers: Vec<Offer>,
    pub contracts: Vec<Contract>,
    pub next_id: u32,
    /// shifts of intel and timings left; Patrols to hold off; markets with a convoy this shift
    pub intel: u32,
    pub timings: u32,
    pub protection: u32,
    pub convoy: Vec<u8>,
    /// the levers' services bought this shift, by faction
    pub bought_service: Vec<u8>,
    /// a clinic day: this shift's end brings the flush
    pub flush: bool,
}

pub struct Economy {
    pub file: FactionFile,
    pub s: EconomySave,
    /// each faction's item multipliers, resolved against the table
    items: Vec<Vec<(ItemId, f32)>>,
    /// each faction's stock entries' items (None: a service)
    stock_items: Vec<Vec<Option<ItemId>>>,
    uniques: Vec<Option<ItemId>>,
    templates_items: Vec<Vec<Option<ItemId>>>,
}

impl Economy {
    fn resolve(file: FactionFile, table: &ItemTable, s: EconomySave) -> Economy {
        let items = file.factions.iter().map(|f| f.items.iter().filter_map(|(k, v)| table.id_of(k).map(|i| (i, *v))).collect()).collect();
        let stock_items = file.factions.iter().map(|f| f.stock.iter().map(|e| if e.item.is_empty() { None } else { table.id_of(&e.item) }).collect()).collect();
        let uniques = file.factions.iter().map(|f| table.id_of(&f.unique)).collect();
        let templates_items = file.factions.iter().map(|f| f.contracts.iter().map(|c| if c.item.is_empty() { None } else { table.id_of(&c.item) }).collect()).collect();
        Economy { file, s, items, stock_items, uniques, templates_items }
    }

    /// The founding (Full Design v3.15, Economy: The streets above the grates): the first grate's street is the
    /// `first` faction's, each of `every` takes one more of the stratum's grates, the rest roll by the stratum's
    /// weights. `grates` are the stratum's top-row grate columns west to east.
    pub fn found(file: FactionFile, table: &ItemTable, seed: u64, grates: &[u16], first: u16, stratum: u8) -> Economy {
        let n = file.factions.len();
        let mut r = rng(seed, 1, stratum as u64, 0);
        let first_f = file.index(&file.streets.first).unwrap_or(0);
        let mut faction: Vec<Option<u8>> = grates.iter().map(|&g| (g == first).then_some(first_f)).collect();
        let mut free: Vec<usize> = (0..grates.len()).filter(|&k| faction[k].is_none()).collect();
        for id in &file.streets.every {
            if free.is_empty() {
                break;
            }
            let k = free.remove(r.below(free.len() as u32) as usize);
            faction[k] = file.index(id);
        }
        let weights = &file.streets.stratum[(stratum.max(1) as usize - 1).min(file.streets.stratum.len() - 1)].weights;
        let pool: Vec<(u8, u32)> = weights.iter().filter_map(|(k, w)| file.index(k).map(|i| (i, *w))).collect();
        let total: u32 = pool.iter().map(|p| p.1).sum();
        for k in free {
            let mut x = r.below(total.max(1));
            for &(i, w) in &pool {
                if x < w {
                    faction[k] = Some(i);
                    break;
                }
                x -= w;
            }
        }
        let streets = grates.iter().enumerate().map(|(k, &g)| Street { grate: g, number: (k + 1) as u8, faction: faction[k].unwrap_or(first_f) }).collect();
        let walk = file.factions.iter().map(|f| std::array::from_fn(|c| f.base(CATEGORIES[c]))).collect();
        let s = EconomySave { seed, streets, standing: vec![Standing::default(); n], walk, stock: vec![Vec::new(); n], bought_service: vec![0; n], next_id: 1, ..EconomySave::default() };
        let mut e = Economy::resolve(file, table, s);
        e.begin_day(1, 1);
        e
    }

    pub fn save(&self) -> EconomySave {
        self.s.clone()
    }

    pub fn restore(file: FactionFile, table: &ItemTable, s: EconomySave) -> Economy {
        Economy::resolve(file, table, s)
    }

    pub fn day_of(shift: u32) -> u32 {
        shift.max(1).div_ceil(3)
    }

    pub fn faction_at(&self, grate: usize) -> Option<u8> {
        self.s.streets.iter().find(|s| s.grate as usize == grate).map(|s| s.faction)
    }

    pub fn name(&self, f: u8) -> &str {
        &self.file.factions[f as usize].name
    }

    fn trades(&self, f: usize, chapter: u8) -> bool {
        let d = &self.file.factions[f];
        d.chapter == 0 || chapter >= d.chapter
    }

    // ------------------------------------------------------------ the surface's clock

    /// A new day: each demand walks a quarter of the way back to its buy-list value plus a seeded step; the surface's
    /// events roll and age; the stock rotates; old offers lapse.
    pub fn begin_day(&mut self, day: u32, chapter: u8) -> Vec<Note> {
        let mut notes = Vec::new();
        self.s.day = day;
        let rules = &self.file.rules;
        for (f, def) in self.file.factions.iter().enumerate() {
            for (c, cat) in CATEGORIES.iter().enumerate() {
                let mut r = rng(self.s.seed, 2, day as u64, (f * 16 + c) as u64);
                let base = def.base(*cat);
                let w = &mut self.s.walk[f][c];
                *w += rules.revert * (base - *w) + rules.step * (2.0 * r.f32() - 1.0);
            }
        }
        // the events: those running age a day; each rolls to start
        for e in &mut self.s.events {
            e.1 = e.1.saturating_sub(1);
        }
        self.s.events.retain(|e| e.1 > 0);
        self.s.flush = false;
        for (k, ev) in self.file.events.iter().enumerate() {
            if ev.chance <= 0.0 || ev.days == 0 || (ev.chapter > 0 && chapter < ev.chapter) || self.s.events.iter().any(|e| e.0 as usize == k) {
                continue;
            }
            let mut r = rng(self.s.seed, 3, day as u64, k as u64);
            if r.f32() < ev.chance {
                self.s.events.push((k as u16, ev.days));
                if ev.flush {
                    self.s.flush = true;
                }
                notes.push(Note::Event(format!("{} (day {day}, {} day{})", ev.name, ev.days, if ev.days == 1 { "" } else { "s" })));
            }
        }
        // the stock: fixed entries and `rotate` rotating entries of each tier
        for (f, def) in self.file.factions.iter().enumerate() {
            let mut today = Vec::new();
            let mut r = rng(self.s.seed, 4, day as u64, f as u64);
            for tier in 1..=4u8 {
                let mut rot: Vec<usize> = (0..def.stock.len()).filter(|&i| def.stock[i].tier == tier && !def.stock[i].fixed).collect();
                for i in (0..def.stock.len()).filter(|&i| def.stock[i].tier == tier && def.stock[i].fixed) {
                    today.push(i);
                }
                for _ in 0..def.rotate.min(rot.len() as u8) {
                    today.push(rot.remove(r.below(rot.len() as u32) as usize));
                }
            }
            today.sort();
            let mult = self.stock_mult(f, chapter);
            self.s.stock[f] = today
                .into_iter()
                .map(|i| {
                    let e = &def.stock[i];
                    let m = mult * self.item_stock_mult(&e.item, chapter);
                    (i as u16, ((e.units as f32 * m).round() as u16).max(1))
                })
                .collect();
        }
        let open: Vec<bool> = (0..self.file.factions.len() as u8).map(|f| self.tier_open(f) > 0).collect();
        self.s.offers.retain(|o| o.until > day && open[o.faction as usize]);
        notes
    }

    fn stock_mult(&self, f: usize, chapter: u8) -> f32 {
        let id = &self.file.factions[f].id;
        self.s.events.iter().map(|&(k, _)| &self.file.events[k as usize]).filter(|ev| ev.chapter == 0 || chapter >= ev.chapter).filter_map(|ev| ev.stock.get(id)).product()
    }

    fn item_stock_mult(&self, item: &str, chapter: u8) -> f32 {
        self.file.events.iter().filter(|ev| ev.chapter > 0 && chapter >= ev.chapter).filter_map(|ev| ev.stock_item.get(item)).product()
    }

    /// The walk's distance from the buy-list value (the proof's band).
    pub fn drift(&self, f: u8, c: Category) -> f32 {
        self.s.walk[f as usize][c as usize] - self.file.factions[f as usize].base(c)
    }

    /// The day's demand: the walk and the surface's events, clamped to the Items formula's range.
    pub fn demand(&self, f: u8, c: Category, t: Tier) -> f32 {
        let def = &self.file.factions[f as usize];
        let mut d = self.s.walk[f as usize][c as usize];
        for &(k, _) in &self.s.events {
            for p in &self.file.events[k as usize].push {
                let who = p.faction == "all" || p.faction == def.id;
                let what = p.category == "all" || category_of(&p.category) == Some(c);
                let tier = p.tier.is_empty() || tier_of(&p.tier) == Some(t);
                if who && what && tier {
                    d += p.offset;
                }
            }
        }
        d.clamp(self.file.rules.range[0], self.file.rules.range[1])
    }

    pub fn events_now(&self) -> Vec<&str> {
        self.s.events.iter().map(|&(k, _)| self.file.events[k as usize].name.as_str()).collect()
    }

    // ------------------------------------------------------------ standing

    /// The stock tier open with a faction: 0 when its stalls are closed (−3).
    pub fn tier_open(&self, f: u8) -> u8 {
        let st = &self.s.standing[f as usize];
        let r = &self.file.rules;
        if st.rep <= -3 {
            return 0;
        }
        let by_rule = (1..=4u8).filter(|&t| st.rep >= r.tier_rep[t as usize - 1] && st.traded >= r.tier_volume[t as usize - 1]).max().unwrap_or(0);
        by_rule.max(st.opened.min(4))
    }

    /// Reputation at a stall: +1 for a staff slimo of the faction's Past, +1 under Open Grates.
    pub fn rep_at(&self, f: u8, past_staff: bool, open_grates: bool) -> i32 {
        (self.s.standing[f as usize].rep + past_staff as i32 + open_grates as i32).clamp(-3, 3)
    }

    /// Favour moves reputation in steps (+2 the cap until Parley, −3 the floor). Returns the step taken (−1, 0, 1).
    pub fn favour(&mut self, f: u8, by: i32) -> i32 {
        let (step, cap) = (self.file.rules.favour, self.file.rules.rep_cap);
        let st = &mut self.s.standing[f as usize];
        st.favour += by;
        if st.favour >= step {
            if st.rep < cap {
                st.rep += 1;
                st.favour = 0;
                return 1;
            }
            st.favour = step - 1;
        } else if st.favour <= -step {
            st.rep = (st.rep - 1).max(-3);
            st.favour = 0;
            return -1;
        }
        0
    }

    fn rep_step(&mut self, f: u8, by: i32) {
        let cap = self.file.rules.rep_cap;
        let st = &mut self.s.standing[f as usize];
        st.rep = (st.rep + by).clamp(-3, cap);
        st.favour = 0;
    }

    // ------------------------------------------------------------ the books

    /// The book of faction `f` at a stall whose best Trade is `trade`, with a staff slimo of its Past or not.
    pub fn book(&self, f: u8, trade: u8, past_staff: bool, open_grates: bool) -> Book {
        let def = &self.file.factions[f as usize];
        let r = &self.file.rules;
        let rep = self.rep_at(f, past_staff, open_grates);
        let demand = std::array::from_fn(|c| std::array::from_fn(|t| self.demand(f, CATEGORIES[c], TIERS[t])));
        let open_tier = self.tier_open(f);
        Book {
            faction: f,
            demand,
            items: self.items[f as usize].clone(),
            rep: 1.0 + r.rep_price * rep as f32,
            haggle: 1.0 + r.haggle * trade as f32,
            fence: def.fence > 0 && open_tier >= def.fence,
            fir: def.fir,
            open: open_tier > 0,
        }
    }

    /// Each market's books: the faction of its grate's street, or under Open Grates (or a convoy at that market this
    /// shift) every faction that trades. `markets` are the market sites' grate columns; `staff[m]` is the best Trade at
    /// that market's Seep Market and the Pasts of its staff.
    pub fn books(&self, markets: &[usize], staff: &[(u8, Vec<String>)], open_grates: bool, chapter: u8) -> Vec<Vec<Book>> {
        markets
            .iter()
            .enumerate()
            .map(|(m, &g)| {
                let (trade, pasts) = staff.get(m).cloned().unwrap_or_default();
                let all = open_grates || self.s.convoy.contains(&(m as u8));
                let fs: Vec<u8> = if all { (0..self.file.factions.len()).filter(|&f| self.trades(f, chapter)).map(|f| f as u8).collect() } else { self.faction_at(g).into_iter().collect() };
                fs.into_iter()
                    .map(|f| {
                        let past = pasts.iter().any(|p| *p == self.file.factions[f as usize].past);
                        let mut b = self.book(f, trade, past, open_grates);
                        if self.s.convoy.contains(&(m as u8)) && !open_grates && Some(f) != self.faction_at(g) {
                            // the convoy's cut (Pass 1: the Nomads move goods laterally for a fee)
                            b.rep *= 1.0 - self.file.services.convoy_fee;
                        }
                        b
                    })
                    .collect()
            })
            .collect()
    }

    // ------------------------------------------------------------ the stock

    /// Today's stock of a faction that the city may buy: (entry, units left, price a unit) at a stall with `trade`.
    pub fn stock(&self, f: u8, table: &ItemTable, trade: u8, past_staff: bool, open_grates: bool) -> Vec<(u16, u16, u32)> {
        let open = self.tier_open(f);
        self.s.stock[f as usize]
            .iter()
            .filter(|&&(e, _)| self.file.factions[f as usize].stock[e as usize].tier <= open)
            .filter_map(|&(e, units)| self.price_of(f, e, table, trade, past_staff, open_grates).map(|p| (e, units, p)))
            .collect()
    }

    /// A stock entry's price a unit: the item's base × markup (or the entry's price) × the day's demand ×
    /// (1 − 0.1 × rep) × (1 − 0.02 × Trade). Services: their price, or the hire's.
    pub fn price_of(&self, f: u8, entry: u16, table: &ItemTable, trade: u8, past_staff: bool, open_grates: bool) -> Option<u32> {
        let def = &self.file.factions[f as usize];
        let e = def.stock.get(entry as usize)?;
        let r = &self.file.rules;
        let rep = self.rep_at(f, past_staff, open_grates) as f32;
        let off = (1.0 - r.rep_price * rep) * (1.0 - r.haggle * trade as f32);
        let base = match self.stock_items[f as usize][entry as usize] {
            Some(i) => {
                let d = table.def(i);
                let list = if e.price > 0 { e.price as f32 } else { d.value as f32 * r.markup };
                list * self.demand(f, d.category, d.tier)
            }
            None => match e.service.as_str() {
                "hire" => self.file.services.hire as f32,
                "convoy" => return Some(0),
                _ => e.price as f32,
            },
        };
        Some(((base * off).round() as u32).max(1))
    }

    /// An order at a stall: the units come off today's stock now; the price is fixed now and paid at the shift's
    /// end. Returns the price a unit, or why not.
    #[allow(clippy::too_many_arguments)]
    pub fn order(&mut self, f: u8, entry: u16, units: u16, table: &ItemTable, trade: u8, past_staff: bool, open_grates: bool) -> Result<u32, String> {
        let def = &self.file.factions[f as usize];
        let e = def.stock.get(entry as usize).ok_or("no such stock")?;
        if e.tier > self.tier_open(f) {
            return Err(format!("{}'s tier {} is not open", def.name, e.tier));
        }
        let each = self.price_of(f, entry, table, trade, past_staff, open_grates).ok_or("no price")?;
        let left = self.s.stock[f as usize].iter_mut().find(|s| s.0 == entry).ok_or("not in today's stock")?;
        if left.1 < units {
            return Err(format!("{} left today", left.1));
        }
        left.1 -= units;
        Ok(each)
    }

    /// The shift's end at the stalls: each order is paid while the eddies last (the rest wait a shift) and becomes
    /// goods to send down or a service in force.
    pub fn settle(&mut self, orders: &mut Vec<Order>, eddies: &mut u32) -> (Vec<Bought>, u32) {
        let mut out = Vec::new();
        let mut paid = 0;
        let mut keep = Vec::new();
        for o in std::mem::take(orders) {
            let total = o.each * o.units as u32;
            if *eddies < total {
                keep.push(o);
                continue;
            }
            *eddies -= total;
            paid += total;
            self.s.standing[o.faction as usize].traded += total;
            let e = &self.file.factions[o.faction as usize].stock[o.entry as usize];
            match self.stock_items[o.faction as usize][o.entry as usize] {
                Some(item) => out.push(Bought::Goods { market: o.market, item, units: o.units }),
                None => {
                    let name = e.service.clone();
                    match name.as_str() {
                        "hire" => out.push(Bought::Hire { faction: o.faction }),
                        "intel" => self.s.intel = self.file.services.intel_shifts,
                        "timings" => self.s.timings = self.file.services.timings_shifts,
                        "protection" => self.s.protection += o.units as u32,
                        "convoy" if !self.s.convoy.contains(&o.market) => self.s.convoy.push(o.market),
                        _ => {}
                    }
                    if self.s.bought_service.len() > o.faction as usize {
                        self.s.bought_service[o.faction as usize] += o.units.min(255) as u8;
                    }
                    if name != "hire" {
                        out.push(Bought::Service { faction: o.faction, name });
                    }
                }
            }
        }
        *orders = keep;
        (out, paid)
    }

    /// A Patrol due is held off a shift if protection was bought (Pass 1: the Gangs delay one Patrol).
    pub fn take_protection(&mut self) -> bool {
        if self.s.protection > 0 {
            self.s.protection -= 1;
            true
        } else {
            false
        }
    }

    // ------------------------------------------------------------ contracts

    fn template_open(&self, c: &ContractDef, facts: &ShiftFacts) -> bool {
        match Kind::of(&c.kind) {
            Some(Kind::ClearBloom) => facts.bloom,
            Some(Kind::ClearRaiders) => facts.raiders,
            Some(_) => true,
            None => false,
        }
    }

    /// Each faction whose grate the Mat reaches keeps `offers` offers; new ones are first links of chains of three
    /// to five, standing `offer_days`. `namer` names a newcomer, corpse or item from a seed.
    pub fn refresh_offers(&mut self, facts: &ShiftFacts, chapter: u8, namer: &mut dyn FnMut(u64) -> String) {
        let day = self.s.day;
        let mut fs: Vec<u8> = facts.reached.iter().filter_map(|&g| self.faction_at(g as usize)).collect();
        fs.sort();
        fs.dedup();
        for f in fs {
            if !self.trades(f as usize, chapter) || self.tier_open(f) == 0 {
                continue;
            }
            let have = self.s.offers.iter().filter(|o| o.faction == f).count();
            let def = &self.file.factions[f as usize];
            let open: Vec<usize> = (0..def.contracts.len()).filter(|&k| self.template_open(&def.contracts[k], facts)).collect();
            if open.is_empty() {
                continue;
            }
            for slot in have..self.file.rules.offers {
                let id = self.s.next_id;
                self.s.next_id += 1;
                let mut r = rng(self.s.seed, 5, id as u64, f as u64);
                let template = open[r.below(open.len() as u32) as usize] as u16;
                let (lo, hi) = (self.file.rules.chain[0] as i32, self.file.rules.chain[1] as i32);
                let links = r.range(lo, hi) as u8;
                let name = namer(mix(self.s.seed ^ (id as u64) << 8 ^ slot as u64));
                self.s.offers.push(Offer { id, faction: f, template, links, until: day + self.file.rules.offer_days, name });
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn make_link(&self, f: u8, template: u16, link: u8, links: u8, id: u32, shift: u32, name: String) -> Contract {
        let def = &self.file.factions[f as usize];
        let t = &def.contracts[template as usize];
        let g = self.file.rules.growth.powi(link as i32 - 1);
        let kind = Kind::of(&t.kind).unwrap_or(Kind::Deliver);
        let n = match kind {
            Kind::Deliver | Kind::ClearRaiders => ((t.n as f32 * g).ceil() as u16).max(1),
            _ => 1,
        };
        let ask = Ask { category: category_of(&t.category), tier: tier_of(&t.tier), item: self.templates_items[f as usize][template as usize], n, condition: t.condition };
        Contract { id, faction: f, template, kind, link, links, ask, eddies: (t.eddies as f32 * g).round() as u32, from: shift, due: shift + t.shifts, got: 0, name, host: None, route: None }
    }

    /// What an offer's first link asks and pays, as it would stand if taken on `shift`.
    pub fn first_link(&self, o: &Offer, shift: u32) -> Contract {
        self.make_link(o.faction, o.template, 1, o.links, o.id, shift, o.name.clone())
    }

    /// The stock entry of a faction that is a service by name.
    pub fn service_entry(&self, f: u8, name: &str) -> Option<u16> {
        self.file.factions[f as usize].stock.iter().position(|e| e.service == name).map(|e| e as u16)
    }

    /// Take an offer: its first link is in hand from this shift. The city shelters the host or plants the body.
    pub fn accept(&mut self, offer: u32, shift: u32) -> Option<Contract> {
        let k = self.s.offers.iter().position(|o| o.id == offer)?;
        if self.tier_open(self.s.offers[k].faction) == 0 {
            return None;
        }
        let o = self.s.offers.remove(k);
        let c = self.make_link(o.faction, o.template, 1, o.links, o.id, shift, o.name);
        self.s.contracts.push(c.clone());
        Some(c)
    }

    pub fn contract_mut(&mut self, id: u32) -> Option<&mut Contract> {
        self.s.contracts.iter_mut().find(|c| c.id == id)
    }

    /// Whether an item meets a deliver or recover link.
    pub fn fits(&self, c: &Contract, item: &ItemInstance, table: &ItemTable) -> bool {
        let d = table.def(item.def);
        match c.kind {
            Kind::Deliver => {
                c.ask.item.is_none_or(|i| i == item.def) && c.ask.category.is_none_or(|k| k == d.category) && c.ask.tier.is_none_or(|t| d.tier >= t) && item.condition >= c.ask.condition
            }
            Kind::Recover => item.bound == Some(c.id),
            _ => false,
        }
    }

    /// An item delivered to a contract landed at its market: it counts if it fits. Returns whether it counted.
    pub fn delivered(&mut self, contract: u32, item: &ItemInstance, table: &ItemTable) -> bool {
        let Some(c) = self.s.contracts.iter().find(|c| c.id == contract).cloned() else { return false };
        if !self.fits(&c, item, table) || c.got >= c.ask.n {
            return false;
        }
        if let Some(c) = self.contract_mut(contract) {
            c.got += 1;
        }
        true
    }

    /// Raiders a run killed count toward every clear-raiders link in hand.
    pub fn raider_kills(&mut self, n: u32) {
        for c in self.s.contracts.iter_mut().filter(|c| c.kind == Kind::ClearRaiders) {
            c.got = (c.got as u32 + n).min(c.ask.n as u32) as u16;
        }
    }

    // ------------------------------------------------------------ the shift's end

    /// Contracts finish or lapse, the levers move favour, the services run down, sales count as volume.
    pub fn end_shift(&mut self, shift: u32, facts: &ShiftFacts, eddies: &mut u32, table: &ItemTable) -> Ended {
        let mut out = Ended::default();
        let rules = self.file.rules.clone();
        // volume: sold to them counts as traded
        for &(f, _, e) in &facts.sold {
            if let Some(st) = self.s.standing.get_mut(f as usize) {
                st.traded += e;
            }
        }
        if facts.road_favour {
            for &(f, u, _) in &facts.sold {
                if matches!(self.file.factions.get(f as usize).map(|d| d.id.as_str()), Some("nomads" | "badges" | "gangs")) {
                    self.favour(f, 2 * u as i32);
                }
            }
        }
        // contracts
        let mut keep = Vec::new();
        for c in std::mem::take(&mut self.s.contracts) {
            let done = match c.kind {
                Kind::Deliver | Kind::ClearRaiders | Kind::Recover => c.got >= c.ask.n,
                Kind::ClearBloom => !facts.bloom,
                Kind::Host => shift >= c.due && c.host.is_some_and(|h| facts.hosts_here.contains(&h)),
            };
            let gone = c.kind == Kind::Host && c.host.is_some_and(|h| !facts.hosts_alive.contains(&h));
            let fname = self.file.factions[c.faction as usize].name.clone();
            if done {
                *eddies += c.eddies;
                out.paid += c.eddies;
                let step = self.favour(c.faction, rules.link_favour);
                let rival = self.file.factions[c.faction as usize].rival.clone();
                if let Some(rv) = self.file.index(&rival) {
                    self.favour(rv, -rules.rival_favour);
                }
                if step > 0 {
                    out.notes.push(Note::Gain(format!("the {fname} think better of the city: reputation {:+}", self.s.standing[c.faction as usize].rep)));
                }
                if c.link >= c.links {
                    out.chains_done += 1;
                    let next_tier = (self.tier_open(c.faction) + 1).min(4);
                    self.rep_step(c.faction, 1);
                    self.s.standing[c.faction as usize].opened = self.s.standing[c.faction as usize].opened.max(next_tier);
                    let mut unique = String::new();
                    if let Some(i) = self.uniques[c.faction as usize] {
                        let mut it = ItemInstance::new(i);
                        it.condition = rules.unique_condition;
                        unique = format!(" and a {}", table.def(i).name);
                        out.uniques.push(it);
                    }
                    out.notes.push(Note::Gain(format!("the {fname}' chain done, {} links: {} e, reputation {:+}, tier {next_tier} open{unique}", c.links, c.eddies, self.s.standing[c.faction as usize].rep)));
                } else {
                    out.notes.push(Note::Gain(format!("the {fname}' link {}/{} done: {} e", c.link, c.links, c.eddies)));
                    keep.push(self.next_link(&c, shift, facts));
                }
                continue;
            }
            if shift >= c.due || gone {
                self.rep_step(c.faction, -1);
                out.notes.push(Note::Loss(format!("the {fname}' contract missed ({}): the chain breaks, reputation {:+}", c.kind.name(), self.s.standing[c.faction as usize].rep)));
                continue;
            }
            keep.push(c);
        }
        self.s.contracts = keep;
        // the levers
        for f in 0..self.file.factions.len() {
            let (lever, lever_at, name) = { let d = &self.file.factions[f]; (d.lever.clone(), d.lever_at, d.name.clone()) };
            let reached: Vec<u16> = self.s.streets.iter().filter(|s| s.faction as usize == f && facts.reached.contains(&s.grate)).map(|s| s.grate).collect();
            if reached.is_empty() {
                continue;
            }
            let by = match lever.as_str() {
                "volume" => facts.sold.iter().find(|s| s.0 as usize == f).is_some_and(|s| s.1 as f32 >= lever_at) as i32,
                "quiet" => reached.iter().all(|g| facts.attention.iter().find(|a| a.0 == *g).is_none_or(|a| a.1 < lever_at)) as i32,
                "protection" | "intel" => 2 * self.s.bought_service.get(f).copied().unwrap_or(0).min(1) as i32,
                _ => 0,
            };
            if by != 0 {
                let step = self.favour(f as u8, by);
                if step > 0 {
                    out.notes.push(Note::Gain(format!("the {name} think better of the city: reputation {:+}", self.s.standing[f].rep)));
                }
            }
        }
        // below 0 a faction comes round, a favour a shift (research 19: RimWorld's goodwill drifts back toward its range)
        for f in 0..self.file.factions.len() {
            if self.s.standing[f].rep < 0 && self.favour(f as u8, rules.recover) > 0 {
                let name = self.file.factions[f].name.clone();
                out.notes.push(Note::Gain(format!("the {name} come round: reputation {:+}", self.s.standing[f].rep)));
            }
        }
        for b in &mut self.s.bought_service {
            *b = 0;
        }
        out
    }

    /// A shift's services run down before the stalls settle its orders, so what is bought this shift runs from the
    /// next: intel and timings a shift less, the convoy gone.
    pub fn services_tick(&mut self) {
        self.s.intel = self.s.intel.saturating_sub(1);
        self.s.timings = self.s.timings.saturating_sub(1);
        self.s.convoy.clear();
    }

    fn next_link(&mut self, c: &Contract, shift: u32, facts: &ShiftFacts) -> Contract {
        let def = &self.file.factions[c.faction as usize];
        let open: Vec<usize> = (0..def.contracts.len()).filter(|&k| self.template_open(&def.contracts[k], facts)).collect();
        let mut r = rng(self.s.seed, 6, c.id as u64, c.link as u64);
        let template = if open.is_empty() { c.template } else { open[r.below(open.len() as u32) as usize] as u16 };
        self.make_link(c.faction, template, c.link + 1, c.links, c.id, shift, c.name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn econ(seed: u64) -> (Economy, ItemTable) {
        let table = ItemTable::from_toml(items::ITEMS_TOML).unwrap();
        let grates = [10u16, 33, 56, 79, 102, 125, 148, 171];
        (Economy::found(FactionFile::shipped(), &table, seed, &grates, 102, 1), table)
    }

    #[test]
    fn the_first_grate_is_the_nomads_and_every_trading_faction_works_one() {
        for seed in 1..40 {
            let (e, _) = econ(seed);
            assert_eq!(e.faction_at(102), e.file.index("nomads"));
            for id in ["gangs", "badges", "corpo"] {
                let f = e.file.index(id);
                assert!(e.s.streets.iter().any(|s| Some(s.faction) == f), "seed {seed}: no {id}");
            }
        }
    }

    #[test]
    fn demand_walks_inside_its_band_and_events_push_on_top() {
        let (mut e, _) = econ(11);
        let mut worst: f32 = 0.0;
        for day in 2..200 {
            e.begin_day(day, 1);
            for f in 0..4u8 {
                for c in CATEGORIES {
                    worst = worst.max(e.drift(f, c).abs());
                    let d = e.demand(f, c, Tier::Common);
                    assert!((0.6..=1.6).contains(&d));
                }
            }
        }
        assert!(worst < e.file.rules.band, "the walk reached {worst}");
        assert!(worst > 0.05, "the walk barely moves: {worst}");
    }

    #[test]
    fn reputation_opens_tiers_and_a_closed_faction_buys_nothing() {
        let (mut e, table) = econ(11);
        let g = e.file.index("gangs").unwrap();
        assert_eq!(e.tier_open(g), 1);
        e.s.standing[g as usize].traded = 150;
        assert_eq!(e.tier_open(g), 2);
        for _ in 0..e.file.rules.favour {
            e.favour(g, 1);
        }
        e.s.standing[g as usize].traded = 600;
        assert_eq!((e.s.standing[g as usize].rep, e.tier_open(g)), (1, 3));
        let pistol = table.id_of("ger_scrap_pistol").unwrap();
        let mut it = ItemInstance::new(pistol);
        it.condition = 100;
        let at_one = items::sell_at(table.def(pistol), &it, &e.book(g, 0, false, false), 0).unwrap();
        e.rep_step(g, -4);
        assert_eq!(e.tier_open(g), 0);
        assert!(items::sell_at(table.def(pistol), &it, &e.book(g, 0, false, false), 0).is_none());
        e.rep_step(g, 3);
        assert!(items::sell_at(table.def(pistol), &it, &e.book(g, 0, false, false), 0).unwrap() < at_one);
        // only the Corpo remnants fence Blackline, and only at their tier 3
        let lamp = table.id_of("chr_lamp_core").unwrap();
        let core = ItemInstance::new(lamp);
        let c = e.file.index("corpo").unwrap();
        assert!(items::sell_at(table.def(lamp), &core, &e.book(g, 0, false, false), 0).is_none());
        assert!(items::sell_at(table.def(lamp), &core, &e.book(c, 0, false, false), 0).is_none());
        e.s.standing[c as usize].opened = 3;
        assert!(items::sell_at(table.def(lamp), &core, &e.book(c, 0, false, false), 0).is_some());
    }

    #[test]
    fn a_chain_rises_link_by_link_and_a_miss_breaks_it() {
        let (mut e, table) = econ(11);
        let n = e.file.index("nomads").unwrap();
        let facts = ShiftFacts { reached: vec![102], ..ShiftFacts::default() };
        e.refresh_offers(&facts, 1, &mut |s| format!("N{s}"));
        assert_eq!(e.s.offers.iter().filter(|o| o.faction == n).count(), 3);
        // a deliver chain of chrome: link 1 then link 2 asks 1.55x
        let k = e.file.factions[n as usize].contracts.iter().position(|c| c.kind == "deliver" && c.category == "chrome").unwrap() as u16;
        e.s.offers[0].template = k;
        e.s.offers[0].links = 3;
        let id = e.s.offers[0].id;
        let c = e.accept(id, 1).unwrap();
        assert_eq!((c.ask.n, c.eddies, c.due), (8, 48, 4));
        let chrome = table.id_of("chr_ear_coil").unwrap();
        let mut it = ItemInstance::new(chrome);
        it.condition = 60;
        for _ in 0..8 {
            assert!(e.delivered(id, &it, &table));
        }
        let mut eddies = 0;
        let ended = e.end_shift(2, &facts, &mut eddies, &table);
        assert_eq!(eddies, 48, "{:?}", ended.notes);
        let next = e.s.contracts.iter().find(|c| c.id == id).unwrap().clone();
        assert_eq!(next.link, 2);
        assert_eq!(next.eddies, (e.file.factions[n as usize].contracts[next.template as usize].eddies as f32 * 1.55).round() as u32);
        // the rival lost favour; the Nomads gained it
        assert_eq!(e.s.standing[n as usize].favour, 4);
        assert_eq!(e.s.standing[e.file.index("corpo").unwrap() as usize].favour, -2);
        // a miss: reputation down a step and the chain gone
        let ended = e.end_shift(next.due, &facts, &mut eddies, &table);
        assert!(e.s.contracts.iter().all(|c| c.id != id), "{:?}", ended.notes);
        assert_eq!(e.s.standing[n as usize].rep, -1);
    }
}
