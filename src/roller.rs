//! All rng and rolling logic.

use std::{
    num::TryFromIntError,
    ops::{Add, Neg, Sub},
};

use crate::{
    ast::{DiceExpr, Keep, RollSpec},
    error::DiceError,
};
use rand::{Rng, rngs::ThreadRng};

/// An rng for rolling dice.
///
/// * `rng`: The rng base which is `rand::Rng`.
pub struct Roller<R: Rng> {
    rng: R,
}

impl Default for Roller<ThreadRng> {
    fn default() -> Self {
        Roller { rng: rand::rng() }
    }
}

/// Representation of the result of rolling 0 or more dice
///
/// * `total`: The sum of the rolls
/// * `detail`: A `RollDetail` containing the terms that made the `total`.
#[derive(Debug, Clone)]
pub struct RollResult {
    pub total: u32,
    pub detail: RollDetail,
}
impl RollResult {
    pub fn new(total: u32, detail: RollDetail) -> Self {
        RollResult { total, detail }
    }
}

/// Represents the dice or constant of a `RollResult`.
///
/// * `Dice(Vec<u32>)`: The roll was based on dice, and the Vec<u32> are their individual rolls in
///   the order they were rolled.
/// * `Constant(i32)`: The roll was a constant value, and no rng was done. The i32 contains that
///   value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollDetail {
    Dice(Vec<u32>),
    Constant(i32),
}

pub struct ExprResult {
    pub total: i32,
    pub rolls: Vec<i32>,
    pub modifier: i32,
}

impl TryFrom<RollResult> for ExprResult {
    type Error = DiceError;
    fn try_from(val: RollResult) -> Result<Self, DiceError> {
        let expr = ExprResult {
            // Checked try making total signed
            total: val.total.try_into()?,
            rolls: match &val.detail {
                RollDetail::Dice(d) => d
                    .iter()
                    .map(|&x| x.try_into())
                    .collect::<Result<Vec<i32>, TryFromIntError>>()?,
                // RollDetail::Dice(d) => {
                //     let mut v: Vec<i32> = Vec::new();
                //     for die in d.iter() {
                //         v.push((*die).try_into()?);
                //     }
                //     v
                // }
                RollDetail::Constant(_) => Vec::new(),
            },
            modifier: if let RollDetail::Constant(n) = &val.detail {
                *n
            } else {
                0
            },
        };
        Ok(expr)
    }
}

impl Add<ExprResult> for ExprResult {
    type Output = Self;
    fn add(mut self, other: ExprResult) -> Self {
        self.rolls.extend(other.rolls.iter());
        self.modifier += other.modifier;
        self.total += other.total;
        self
    }
}

impl Neg for ExprResult {
    type Output = Self;
    fn neg(mut self) -> Self {
        // Negate each roll
        self.rolls.iter_mut().for_each(|x| *x = -*x);
        self.modifier = -self.modifier;
        self.total = -self.total;
        self
    }
}

impl Sub<ExprResult> for ExprResult {
    type Output = Self;
    fn sub(self, other: ExprResult) -> Self {
        self + (-other)
    }
}

impl ExprResult {
    pub fn new(total: i32, rolls: Vec<i32>, modifier: i32) -> Self {
        ExprResult {
            total,
            rolls,
            modifier,
        }
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

    /// Evaluate a `RollSpec`.
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

    pub fn roll_expr(&mut self, expr: &DiceExpr) -> Result<ExprResult, DiceError> {
        match expr {
            DiceExpr::Sum(lhs, rhs) => Ok(self.roll_expr(lhs)? + self.roll_expr(rhs)?),
            DiceExpr::Difference(lhs, rhs) => Ok(self.roll_expr(lhs)? - self.roll_expr(rhs)?),
            DiceExpr::Roll(spec) => self.roll_spec(spec).try_into(),
            DiceExpr::Literal(lit) => Ok(ExprResult {
                total: *lit,
                rolls: Vec::new(),
                modifier: *lit,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{self, SeedableRng, rngs::StdRng};

    #[test]
    fn test_roll() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));
        let spec = RollSpec::new(2, 20, None);

        let res = roller.roll_spec(&spec);

        assert_eq!(res.total, 10 + 9);
        if let RollDetail::Dice(dice) = res.detail {
            assert_eq!(dice, vec![9, 10])
        }
    }

    #[test]
    fn test_0_sides() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(3, 0, None);
        let res = roller.roll_spec(&spec);
        assert_eq!(res.total, 3);
        assert_eq!(res.detail, RollDetail::Constant(3));
    }

    #[test]
    fn test_0_count() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(36));
        let spec = RollSpec::new(0, 12, None);

        let res = roller.roll_spec(&spec);

        assert_eq!(res.total, 0);
        assert_eq!(res.detail, RollDetail::Constant(0))
    }

    #[test]
    fn test_keep_highest() {
        // Keep Highest 1
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));

        let spec = RollSpec::new(5, 20, Some(Keep::Highest(1)));
        let res = roller.roll_spec(&spec);

        assert_eq!(res.total, 20);
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d, vec![9, 10, 14, 12, 20])
        } else {
            panic!()
        }

        // Keep Highest 2
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));
        let spec = RollSpec::new(5, 20, Some(Keep::Highest(2)));
        let res = roller.roll_spec(&spec);

        assert_eq!(res.total, 34);
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d, vec![9, 10, 14, 12, 20])
        } else {
            panic!()
        }
    }

    #[test]
    fn test_keep_lowest() {
        // Keep Lowest 1
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));

        let spec = RollSpec::new(5, 20, Some(Keep::Lowest(1)));
        let res = roller.roll_spec(&spec);

        assert_eq!(res.total, 9);
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d, vec![9, 10, 14, 12, 20])
        } else {
            panic!()
        }

        // Keep Lowest 2
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));
        let spec = RollSpec::new(5, 20, Some(Keep::Lowest(2)));
        let res = roller.roll_spec(&spec);

        assert_eq!(res.total, 19);
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d, vec![9, 10, 14, 12, 20])
        } else {
            panic!()
        }
    }
}
