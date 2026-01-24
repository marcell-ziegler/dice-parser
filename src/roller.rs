//! All rng and rolling logic.

use crate::ast::{Keep, RollSpec};
use rand::{Rng, rngs::ThreadRng};

/// An rng for rolling dice.
///
/// * `rng`: The rng base which is `rand::Rng`.
pub struct Roller<R: Rng> {
    rng: R,
}

/// Representation of the result of rolling 0 or more dice
///
/// * `total`: The sum of the rolls
/// * `detail`: A `RollDetail` containing the terms that made the `total`.
pub struct RollResult {
    pub total: u32,
    pub detail: RollDetail,
}

/// Represents the source of a `RollResult`.
///
/// * `Dice(Vec<u32>)`: The roll was based on dice, and the Vec<u32> are their individual rolls in
///   the order they were rolled.
/// * `Constant(i32)`: The roll was a constant value, and no rng was done. The i32 contains that
///   value.
#[derive(Debug, PartialEq, Eq)]
pub enum RollDetail {
    Dice(Vec<u32>),
    Constant(i32),
}

impl RollResult {
    pub fn new(total: u32, detail: RollDetail) -> Self {
        RollResult { total, detail }
    }
}

impl Default for Roller<ThreadRng> {
    fn default() -> Self {
        Roller { rng: rand::rng() }
    }
}

impl<R: Rng> Roller<R> {
    /// Instantiate a `Roller` from a custom `rand::Rng` object.
    ///
    /// * `rng`: The `rand::Rng` object to be used as the generator.
    pub fn from_rng(rng: R) -> Self {
        Roller { rng }
    }

    /// Roll a single die with the given number of sides.
    ///
    /// * `sides`: Number of sides on the dice.
    ///
    /// # Panics
    /// Panics in debug if `sides == 0`
    fn roll_die(&mut self, sides: u32) -> u32 {
        debug_assert!(sides > 0);
        self.rng.random_range(1..=sides)
    }

    /// Roll 1d100 correctly using 2d10, one for the ones and a percentile die for the tens.
    fn roll_d100(&mut self) -> u32 {
        let ones = self.rng.random_range(1..=10);
        let tens = self.rng.random_range(0..=9);
        if tens == 0 { ones } else { ones + tens * 10 }
    }

    /// Roll an arbitrary number of dice with the same number of sides.
    ///
    /// * `sides`: Number of sides.
    /// * `count`: Number of dice.
    ///
    /// # Panics
    /// Panics in debug if `sides == 0` or `count == 0`
    fn roll_dice(&mut self, sides: u32, count: u32) -> Vec<u32> {
        debug_assert!(sides > 0 && count > 0);
        (0..count)
            .map(|_| match sides {
                100 => self.roll_d100(),
                _ => self.roll_die(sides),
            })
            .collect()
    }

    /// Calculate the total roll value for a `RollSpec`.
    ///
    /// * `spec`: the `dice-parser::ast::RollSpec` to be rolled.
    ///
    /// # Returns
    /// Except the self-evident totals (where )
    /// * `Constant(0)`: if `spec.count == 0`
    /// * `Constant(spec.count)`: if `spec.sides == 0`
    /// * `Dice(...)`: if `spec.count > 0 && spec.sides > 0` and contains the rolled dice.
    ///
    /// # Examples
    pub fn roll_spec(&mut self, spec: &RollSpec) -> RollResult {
        if spec.count == 0 {
            return RollResult::new(0, RollDetail::Constant(0));
        }

        if spec.sides == 0 {
            return RollResult::new(spec.count, RollDetail::Constant(spec.count as i32));
        }

        let mut rolls = self.roll_dice(spec.sides, spec.count);
        let rolled = RollDetail::Dice(rolls.clone());

        if let Some(keep) = &spec.keep {
            rolls.sort_unstable();
            let total: u32 = match keep {
                Keep::Highest(n) => rolls[(rolls.len() - *n as usize)..].iter().sum(),
                Keep::Lowest(n) => rolls[..*n as usize].iter().sum(),
            };

            RollResult::new(total, rolled)
        } else {
            let total = rolls.iter().sum();
            RollResult::new(total, rolled)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{self, SeedableRng};

    #[test]
    fn test_roll() {
        let mut roller = Roller::from_rng(rand::rngs::StdRng::seed_from_u64(12));
        let spec = RollSpec::new(2, 20, None);

        let res = roller.roll_spec(&spec);

        // Make a reference generator to check rolling algorithm for correctness.
        let mut ref_rng = rand::rngs::StdRng::seed_from_u64(12);

        assert_eq!(res.total, 10 + 9);
        if let RollDetail::Dice(dice) = res.detail {
            assert_eq!(
                dice,
                vec![
                    ref_rng.random_range(1..=spec.sides),
                    ref_rng.random_range(1..=spec.sides)
                ]
            )
        }
    }

    #[test]
    fn test_0_sides() {
        let mut roller = Roller::from_rng(rand::rngs::StdRng::seed_from_u64(42));
        let spec = RollSpec::new(3, 0, None);
        let res = roller.roll_spec(&spec);
        assert_eq!(res.total, 3);
        assert_eq!(res.detail, RollDetail::Constant(3));
    }
}
