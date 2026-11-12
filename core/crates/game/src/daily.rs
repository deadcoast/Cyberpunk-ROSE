//! The Daily Seed (Full Design v3.19, Narrative and Ivan; Pass 2): the date is the seed (YYYYMMDD) — one route, a
//! stratum from S1–S3 and its route, both from the seed; one Runner, a record rolled by Creation from the seed, D2 or
//! D3; the fixed rig (the bot's kit); no followers. One try a day, scored by the base value of what comes home.

use items::{ItemInstance, ItemTable};
use mat::rng::Pcg32;
use people::{Depth, NameFile, Record, Roster, TraitFile};

pub struct Daily {
    pub date: u32,
    pub stratum: u8,
    pub route_seed: u64,
    pub runner: Record,
}

/// The day's route and Runner.
pub fn daily(date: u32) -> Daily {
    let mut rng = Pcg32::new((date as u64) ^ 0xDA17_5EED);
    let stratum = 1 + rng.below(3) as u8;
    let route_seed = run::route_seed(date as u64, stratum);
    let (traits, names) = (TraitFile::shipped(), NameFile::shipped());
    let mut roster = Roster::new();
    // Creation rolls D2 or D3 seven times in ten; the first of them runs
    let runner = loop {
        let id = roster.create(&traits, &names, &mut rng, 1, false);
        let r = roster.get(id).expect("created");
        if matches!(r.depth, Depth::D2 | Depth::D3) {
            break r.clone();
        }
    };
    Daily { date, stratum, route_seed, runner }
}

/// The score: the base value of what was found on the run and came home (the fixed rig is not scored).
pub fn score<'a>(table: &ItemTable, home: impl Iterator<Item = &'a ItemInstance>) -> u32 {
    home.filter(|it| it.found_in_run).map(|it| table.def(it.def).value).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_date_fixes_the_route_and_the_runner() {
        let a = daily(20261111);
        let b = daily(20261111);
        assert_eq!((a.stratum, a.route_seed, a.runner.full_name()), (b.stratum, b.route_seed, b.runner.full_name()));
        assert!((1..=3).contains(&a.stratum) && matches!(a.runner.depth, Depth::D2 | Depth::D3));
        let strata: std::collections::BTreeSet<u8> = (20261101..20261131).map(|d| daily(d).stratum).collect();
        assert_eq!(strata.len(), 3, "a month draws every stratum");
        let table = ItemTable::from_toml(items::ITEMS_TOML).unwrap();
        let kit = ItemInstance::new(table.id_of("ger_pipe").unwrap());
        let found = ItemInstance::found(table.id_of("chr_lamp_core").unwrap(), 50, items::Provenance { stratum: 1, chunk: 0, day: 1 });
        assert_eq!(score(&table, [&kit, &found].into_iter()), table.get("chr_lamp_core").unwrap().value);
    }
}
