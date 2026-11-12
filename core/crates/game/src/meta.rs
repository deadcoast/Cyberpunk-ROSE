//! What outlives a run (Full Design v3.19, Meta and the Game shell): a New Run founded from its seed, strain code and
//! difficulty; New Game+'s carry from the profile; the strain code a run exports; what a run gives the profile.

use colony::Colony;
use save::{EndingRecord, EndlessRecord, Profile, StrainCode};
use story::campaign::{Campaign, Difficulty, Mode};

/// New Run's choices (the Game shell): seed, strain code (blank, the last run's or typed: New Game+), difficulty, and
/// whether it is the campaign or Endless Sump.
#[derive(Clone, Debug)]
pub struct NewRun {
    pub seed: u32,
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub code: Option<StrainCode>,
}

impl NewRun {
    /// The Heart's genome: the code's (New Game+) or the founding's (Pass 1's 0.5).
    pub fn genome(&self) -> mat::Genome {
        self.code.map_or_else(mat::Genome::default, |c| mat::Genome(c.genome))
    }

    pub fn campaign(&self) -> Campaign {
        let mut c = Campaign::new(self.mode, self.difficulty);
        c.seed = self.seed;
        c.founded_on = self.code.map(|k| k.encode());
        c
    }
}

/// New Game+ (a strain code given): every law the profile unlocked is signable without its tech (Authority still
/// paid), and the Scrapdex is kept (Pass 2); the Stash is the founding's.
pub fn carry(profile: &Profile, colony: &mut Colony) {
    colony.charter.unlocked = profile.laws.clone();
    for id in &profile.scrapdex {
        if !colony.progress.collections.seen(id) {
            colony.progress.collections.scrapdex.push((id.clone(), 0));
        }
    }
}

/// The classes held as the code's byte: bit k for the k-th class of `data/strain.toml`.
pub fn classes_byte(colony: &Colony) -> u8 {
    progress::StrainFile::shipped().class.iter().enumerate().filter(|(_, c)| colony.progress.strain.holds(&c.id)).fold(0u8, |b, (k, _)| b | (1 << k))
}

/// The strain code of the city as it stands.
pub fn strain_code(sim: &mat::MatSim, colony: &Colony, seed: u32) -> StrainCode {
    StrainCode { genome: sim.genome.0, classes: classes_byte(colony), seed }
}

/// The run's Strainbook and Scrapdex into the profile's: the new entries counted.
pub fn absorb(profile: &mut Profile, colony: &Colony) -> usize {
    let book = colony.progress.strain.book.iter().map(|(id, _)| id.clone());
    let dex = colony.progress.collections.scrapdex.iter().map(|(id, _)| id.clone());
    profile.absorb(book, dex)
}

/// A finished run into the profile: a campaign's ending with its warchest and code, and the laws it had signed; an
/// Endless city's rank for its seed. False while the run is not over.
pub fn record_end(profile: &mut Profile, c: &Campaign, sim: &mat::MatSim, colony: &Colony) -> bool {
    let Some(ending) = &c.ending else { return false };
    absorb(profile, colony);
    let shift = c.ended_at.unwrap_or(0);
    match c.mode {
        Mode::Campaign => {
            let code = strain_code(sim, colony, c.seed).encode();
            let laws = colony.charter.laws.iter().filter(|l| l.repeal_reach.is_none()).map(|l| l.id.clone());
            profile.ended(EndingRecord { ending: ending.clone(), warchest: c.warchest.unwrap_or(0), code, shift, difficulty: c.difficulty.name().into() }, laws);
        }
        Mode::Endless => {
            profile.endless_burned(EndlessRecord { seed: c.seed, empire_at: c.empire_at, burned_at: shift });
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_plus_founds_on_the_code_and_carries_the_profile() {
        let table = items::ItemTable::from_toml(items::ITEMS_TOML).unwrap();
        let code = StrainCode { genome: [0.8, 0.5, 0.5, 0.5, 0.9, 0.5, 0.5, 0.2, 0.5, 0.5], classes: 1, seed: 11 };
        let n = NewRun { seed: 11, mode: Mode::Campaign, difficulty: Difficulty::Undercroft, code: Some(code) };
        let mut profile = Profile { laws: vec!["dissolution_rite".into()], scrapdex: vec!["chr_lamp_core".into()], ..Profile::default() };
        let w = crate::World::new_run(&n, table, &profile);
        assert!((w.city.sim.genome.0[4] - 0.9).abs() < 0.01, "the Heart is born with the code's genome");
        assert!(w.colony.charter.unlocked.contains(&"dissolution_rite".to_string()));
        assert!(w.colony.progress.collections.seen("chr_lamp_core"), "the Scrapdex is kept");
        assert_eq!(w.campaign.founded_on.as_deref(), Some(code.encode().as_str()));
        // the difficulty's rules reach the threats
        assert!(w.threats.campaign.dawn_flood && w.threats.campaign.spored_first_only && w.threats.campaign.raid_corpses);
        assert!(w.threats.is_flood_shift(1) && !w.threats.is_flood_shift(2) && w.threats.is_flood_shift(3));
        // the run's code reads back to its genome and seed
        let back = StrainCode::parse(&strain_code(&w.city.sim, &w.colony, 11).encode()).unwrap();
        assert_eq!(back.seed, 11);
        assert!((back.genome[0] - w.city.sim.genome.0[0]).abs() < 0.01);
        // nothing to record until the run is over
        let mut c = w.campaign.clone();
        assert!(!record_end(&mut profile, &c, &w.city.sim, &w.colony));
        c.ending = Some("garden".into());
        c.warchest = Some(500);
        c.ended_at = Some(90);
        assert!(record_end(&mut profile, &c, &w.city.sim, &w.colony));
        assert_eq!(profile.endings.len(), 1);
        assert_eq!(profile.last_code.as_deref(), Some(profile.endings[0].code.as_str()));
    }

    #[test]
    fn an_endless_city_ends_when_its_heart_burns_and_ranks_by_its_empire() {
        let table = items::ItemTable::from_toml(items::ITEMS_TOML).unwrap();
        let n = NewRun { seed: 7, mode: Mode::Endless, difficulty: Difficulty::Sump, code: None };
        let mut w = crate::World::new_run(&n, table, &Profile::default());
        assert_eq!((w.campaign.chapter, w.campaign.card(1).as_str()), (0, "ENDLESS SUMP"));
        assert!(!w.threats.campaign.raid_corpses, "Sump: raiders never loot a Runner's corpse");
        assert!(!w.threats.is_flood_shift(1) && w.threats.is_flood_shift(3));
        w.colony.progress.collections.titles.push(2000);
        w.end_shift();
        let at = w.campaign.empire_at.expect("the Empire's shift");
        assert!(!w.campaign.over());
        w.city.sim.heart_lost = true;
        w.end_shift();
        assert_eq!(w.campaign.ending.as_deref(), Some("burned"));
        assert_eq!(w.campaign.card(w.day), "ENDLESS SUMP: THE HEART BURNED");
        let mut p = Profile::default();
        assert!(record_end(&mut p, &w.campaign, &w.city.sim, &w.colony));
        assert_eq!(p.endless, vec![EndlessRecord { seed: 7, empire_at: Some(at), burned_at: w.campaign.ended_at.unwrap() }]);
        assert!(p.endings.is_empty(), "an Endless city is a rank, not an ending");
    }
}
