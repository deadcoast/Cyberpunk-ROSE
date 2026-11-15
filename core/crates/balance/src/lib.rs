//! balance — the economy's knobs as data (`data/balance.toml`; Full Design v3.24, Balance). One process-wide copy:
//! `install` puts the copy the game or `rose --data` loaded in place before any reader asks; until then, and for the
//! tests, `get` is the compiled copy of the shipped file.

use serde::Deserialize;
use std::sync::OnceLock;

pub const BALANCE_TOML: &str = include_str!("../../../../data/balance.toml");

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recruitment {
    pub flush_size: (u32, u32),
    pub flush_every: (u32, u32),
    pub census_every: (u32, u32),
    pub flush_grates: [usize; 4],
    pub flush_over: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Food {
    pub larder_share: f32,
    pub meal_takes: f32,
    pub paste_eddies: u32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mat {
    pub translocate_share: f32,
    pub translocate_ends: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Economy {
    pub street_stock: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Balance {
    pub recruitment: Recruitment,
    pub food: Food,
    pub mat: Mat,
    pub economy: Economy,
}

impl Balance {
    pub fn from_toml(text: &str) -> Result<Balance, String> {
        let b: Balance = toml::from_str(text).map_err(|e| e.to_string())?;
        b.check()?;
        Ok(b)
    }

    /// The shipped file, compiled in.
    pub fn shipped() -> Balance {
        Balance::from_toml(BALANCE_TOML).expect("data/balance.toml parses")
    }

    /// Ranges a value must keep for the rules that read it to hold.
    fn check(&self) -> Result<(), String> {
        let r = &self.recruitment;
        let pair = |(lo, hi): (u32, u32), name: &str| if lo == 0 || lo > hi { Err(format!("recruitment.{name}: {lo} to {hi} is no range")) } else { Ok(()) };
        pair(r.flush_size, "flush_size")?;
        pair(r.flush_every, "flush_every")?;
        pair(r.census_every, "census_every")?;
        if !(0.0..=1.0).contains(&self.food.larder_share) || !(0.0..=1.0).contains(&self.mat.translocate_share) {
            return Err("a share is between 0 and 1".into());
        }
        if self.economy.street_stock < 0.0 {
            return Err("economy.street_stock is a share of a stall's day, 0 or more".into());
        }
        if self.food.meal_takes <= 0.0 {
            return Err("food.meal_takes must be above 0".into());
        }
        Ok(())
    }
}

static BALANCE: OnceLock<Balance> = OnceLock::new();

/// The knobs every rule reads: the installed copy, else the compiled one (which then stands for the process).
pub fn get() -> &'static Balance {
    BALANCE.get_or_init(Balance::shipped)
}

/// Put a loaded copy in place; false when a reader already took the compiled one (installed too late).
pub fn install(b: Balance) -> bool {
    BALANCE.set(b.clone()).is_ok() || BALANCE.get() == Some(&b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_balance_parses_and_a_bad_one_is_refused() {
        let b = Balance::shipped();
        assert_eq!(b.recruitment.flush_grates, [3, 5, 7, 8]);
        assert!(Balance::from_toml(&BALANCE_TOML.replace("flush_size = [1, 6]", "flush_size = [6, 1]")).is_err());
        assert!(Balance::from_toml(&BALANCE_TOML.replace("larder_share = 0.5", "larder_share = 1.5")).is_err());
        assert!(Balance::from_toml(&format!("{BALANCE_TOML}\n[extra]\nx = 1\n")).is_err());
    }
}
