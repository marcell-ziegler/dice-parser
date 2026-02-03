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

/// Representation of the result of rolling 0 or more dice. Note that a 0-sided dice is interpreted
/// as a constant. See also `dice-parser::ast::DiceExpr::roll()`.
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

/// The result of evaluating a dice expression.
///
/// This struct contains the final total, all individual die rolls (with their signs),
/// and the sum of all constant modifiers in the expression.
///
/// # Fields
///
/// - `total`: The final result after all rolls and modifiers are applied
/// - `rolls`: All individual die rolls. Rolls from subtracted expressions are negative.
/// - `modifier`: The sum of all constant (non-dice) terms in the expression
///
/// # Examples
///
/// ```
/// use dice_parser::DiceExpr;
///
/// let expr = DiceExpr::parse("2d6+3").unwrap();
/// let result = expr.roll().unwrap();
///
/// // Access the components of the result
/// println!("Total: {}", result.total);           // e.g., 10
/// println!("Dice rolls: {:?}", result.rolls);    // e.g., [3, 4]
/// println!("Modifier: {}", result.modifier);     // 3
///
/// // Total equals sum of rolls plus modifier
/// assert_eq!(result.total, result.rolls.iter().sum::<i32>() + result.modifier);
/// ```
///
/// ## Subtraction Example
///
/// ```
/// use dice_parser::DiceExpr;
///
/// let expr = DiceExpr::parse("10 - 2d6").unwrap();
/// let result = expr.roll().unwrap();
///
/// // Subtracted rolls are negative in the rolls vec
/// assert_eq!(result.modifier, 10);
/// assert_eq!(result.rolls.len(), 2);
/// // Both rolls should be negative
/// assert!(result.rolls[0] < 0 && result.rolls[1] < 0);
/// ```
#[derive(Debug, Clone)]
pub struct ExprResult {
    /// The total result of the expression.
    pub total: i32,
    /// All individual die rolls. Subtracted rolls are negative.
    pub rolls: Vec<i32>,
    /// The sum of all constant modifiers in the expression.
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
    /// Panics if `sides == 0`.
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
    /// Except the self-evident totals (where)
    /// * `Constant(0)`: if `spec.count == 0`
    /// * `Constant(spec.count)`: if `spec.sides == 0`
    /// * `Dice(...)`: if `spec.count > 0 && spec.sides > 0` and contains the rolled dice.
    ///
    /// # Examples
    pub fn roll_spec(&mut self, spec: &RollSpec) -> Result<RollResult, DiceError> {
        if spec.count == 0 {
            return Ok(RollResult::new(0, RollDetail::Constant(0)));
        }

        if spec.sides == 0 {
            return Ok(RollResult::new(
                spec.count,
                RollDetail::Constant(spec.count as i32),
            ));
        }

        let mut rolls = self.roll_dice(spec.sides, spec.count);
        let rolled = RollDetail::Dice(rolls.clone());

        if let Some(keep) = &spec.keep {
            rolls.sort_unstable();
            let total: u32 = match keep {
                Keep::Highest(n) => {
                    if *n as usize > rolls.len() {
                        return Err(DiceError::InvalidSpec(
                            spec.clone(),
                            String::from("tried to keep more than total amount of rolled dice"),
                        ));
                    }
                    rolls[(rolls.len() - *n as usize)..].iter().sum()
                }
                Keep::Lowest(n) => {
                    if *n as usize > rolls.len() {
                        return Err(DiceError::InvalidSpec(
                            spec.clone(),
                            String::from("tried to keep more than total amount of rolled dice"),
                        ));
                    }

                    rolls[..*n as usize].iter().sum()
                }
            };

            Ok(RollResult::new(total, rolled))
        } else {
            let total = rolls.iter().sum();
            Ok(RollResult::new(total, rolled))
        }
    }

    /// Evaluate a DiceExpr.
    ///
    /// * `expr`: The epression to be evaluated.
    ///
    /// # Returns
    /// * `Ok(ExprResult)` if successful.
    /// * `Err(DiceError)` if evaluation failed.
    pub fn roll_expr(&mut self, expr: &DiceExpr) -> Result<ExprResult, DiceError> {
        match expr {
            DiceExpr::Sum(lhs, rhs) => Ok(self.roll_expr(lhs)? + self.roll_expr(rhs)?),
            DiceExpr::Difference(lhs, rhs) => Ok(self.roll_expr(lhs)? - self.roll_expr(rhs)?),
            DiceExpr::Roll(spec) => self.roll_spec(spec)?.try_into(),
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

    // ==== Tests for rolling ====

    #[test]
    fn test_roll() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));
        let spec = RollSpec::new(2, 20, None);

        let res = roller.roll_spec(&spec).unwrap();

        assert_eq!(res.total, 10 + 9);
        if let RollDetail::Dice(dice) = res.detail {
            assert_eq!(dice, vec![9, 10])
        }
    }

    #[test]
    fn test_0_sides() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(3, 0, None);
        let res = roller.roll_spec(&spec).unwrap();
        assert_eq!(res.total, 3);
        assert_eq!(res.detail, RollDetail::Constant(3));
    }

    #[test]
    fn test_0_count() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(36));
        let spec = RollSpec::new(0, 12, None);

        let res = roller.roll_spec(&spec).unwrap();

        assert_eq!(res.total, 0);
        assert_eq!(res.detail, RollDetail::Constant(0))
    }

    #[test]
    fn test_keep_highest() {
        // Keep Highest 1
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));

        let spec = RollSpec::new(5, 20, Some(Keep::Highest(1)));
        let res = roller.roll_spec(&spec).unwrap();

        assert_eq!(res.total, 20);
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d, vec![9, 10, 14, 12, 20])
        } else {
            panic!()
        }

        // Keep Highest 2
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));
        let spec = RollSpec::new(5, 20, Some(Keep::Highest(2)));
        let res = roller.roll_spec(&spec).unwrap();

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
        let res = roller.roll_spec(&spec).unwrap();

        assert_eq!(res.total, 9);
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d, vec![9, 10, 14, 12, 20])
        } else {
            panic!()
        }

        // Keep Lowest 2
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(12));
        let spec = RollSpec::new(5, 20, Some(Keep::Lowest(2)));
        let res = roller.roll_spec(&spec).unwrap();

        assert_eq!(res.total, 19);
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d, vec![9, 10, 14, 12, 20])
        } else {
            panic!()
        }
    }

    #[test]
    fn test_keep_all_dice() {
        // Keep highest/lowest equal to count (should keep all)
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(4, 6, Some(Keep::Highest(4)));
        let res = roller.roll_spec(&spec).unwrap();
        if let RollDetail::Dice(d) = res.detail {
            assert_eq!(d.len(), 4);
        } else {
            panic!("Expected Dice variant")
        }
        // Total should equal sum of all rolls
    }

    #[test]
    fn test_keep_single_from_many() {
        // Keep 1 from 10d20
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(10, 20, Some(Keep::Highest(1)));
        let res = roller.roll_spec(&spec).unwrap();
        // Total should equal max roll
        if let RollDetail::Dice(rolls) = &res.detail {
            assert_eq!(rolls.len(), 10);
            let max = rolls.iter().max().unwrap();
            assert_eq!(res.total, *max);
        }
    }

    #[test]
    fn test_keep_too_many_high() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(4, 20, Some(Keep::Highest(5)));
        let res = roller.roll_spec(&spec);

        match res {
            Ok(_) => panic!("Expected Err variant"),
            Err(e) => {
                if !matches!(e, DiceError::InvalidSpec(_, _)) {
                    panic!("expected `DiceError::InvdalidSpec` variant")
                }
            }
        }
    }

    #[test]
    fn test_keep_too_many_low() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(4, 20, Some(Keep::Lowest(5)));
        let res = roller.roll_spec(&spec);

        match res {
            Ok(_) => panic!("Expected Err variant"),
            Err(e) => {
                if !matches!(e, DiceError::InvalidSpec(_, _)) {
                    panic!("expected `DiceError::InvdalidSpec` variant")
                }
            }
        }
    }

    #[test]
    fn test_keep_too_few_high() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(6, 8, Some(Keep::Highest(0)));
        let res = roller.roll_spec(&spec).unwrap();

        assert_eq!(res.total, 0);
        if let RollDetail::Dice(d) = &res.detail {
            assert_eq!(d.len(), 6);
        } else {
            panic!("expected Dice variant")
        }
    }

    #[test]
    fn test_keep_too_few_low() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let spec = RollSpec::new(4, 12, Some(Keep::Lowest(0)));
        let res = roller.roll_spec(&spec).unwrap();

        assert_eq!(res.total, 0);
        if let RollDetail::Dice(d) = &res.detail {
            assert_eq!(d.len(), 4);
        } else {
            panic!("expected Dice variant")
        }
    }
    #[test]
    fn test_d100_range_validation() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(500));
        let spec = RollSpec::new(1000000, 100, None);
        let res = roller.roll_spec(&spec).unwrap();

        if let RollDetail::Dice(rolls) = res.detail {
            for roll in rolls {
                assert!(
                    (1_u32..=100_u32).contains(&roll),
                    "D100 roll out of range: {}",
                    roll
                );
            }
        }
    }
    // ==== Tests for DiceExpr evaluation ====

    #[test]
    fn test_expr_literal() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(1));
        let expr = DiceExpr::Literal(7);
        let result = roller.roll_expr(&expr).unwrap();
        assert_eq!(result.total, 7);
        assert_eq!(result.rolls, vec![]);
        assert_eq!(result.modifier, 7);
    }

    #[test]
    fn test_expr_literal_negative() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(1));
        let expr = DiceExpr::Literal(-15);
        let result = roller.roll_expr(&expr).unwrap();
        assert_eq!(result.total, -15);
        assert_eq!(result.rolls, vec![]);
        assert_eq!(result.modifier, -15);
    }

    #[test]
    fn test_expr_literal_zero() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(1));
        let expr = DiceExpr::Literal(0);
        let result = roller.roll_expr(&expr).unwrap();
        assert_eq!(result.total, 0);
        assert_eq!(result.rolls, vec![]);
        assert_eq!(result.modifier, 0);
    }

    #[test]
    fn test_expr_roll_basic() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let expr = DiceExpr::Roll(RollSpec::new(2, 6, None));
        let result = roller.roll_expr(&expr).unwrap();
        assert_eq!(result.rolls.len(), 2);
        assert_eq!(result.total, result.rolls.iter().sum());
        assert_eq!(result.modifier, 0);
        // All rolls should be in range [1, 6]
        for roll in &result.rolls {
            assert!(*roll >= 1 && *roll <= 6);
        }
    }

    #[test]
    fn test_expr_roll_d20() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(999));
        let expr = DiceExpr::Roll(RollSpec::new(1, 20, None));
        let result = roller.roll_expr(&expr).unwrap();
        assert_eq!(result.rolls.len(), 1);
        assert!(result.rolls[0] >= 1 && result.rolls[0] <= 20);
        assert_eq!(result.total, result.rolls[0]);
        assert_eq!(result.modifier, 0);
    }

    #[test]
    fn test_expr_roll_multiple_d100() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(777));
        let expr = DiceExpr::Roll(RollSpec::new(3, 100, None));
        let result = roller.roll_expr(&expr).unwrap();
        assert_eq!(result.rolls.len(), 3);
        for roll in &result.rolls {
            assert!(*roll >= 1 && *roll <= 100);
        }
        assert_eq!(result.total, result.rolls.iter().sum());
    }

    #[test]
    fn test_expr_sum_literal_and_roll() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(100));
        let left = DiceExpr::Roll(RollSpec::new(1, 20, None));
        let right = DiceExpr::Literal(5);
        let sum_expr = DiceExpr::Sum(Box::new(left), Box::new(right));
        let result = roller.roll_expr(&sum_expr).unwrap();

        // Result should be die roll + 5
        assert_eq!(result.rolls.len(), 1);
        assert_eq!(result.modifier, 5);
        assert_eq!(result.total, result.rolls[0] + 5);
    }

    #[test]
    fn test_expr_sum_multiple_literals() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(1));
        let left = DiceExpr::Literal(10);
        let right = DiceExpr::Literal(20);
        let sum_expr = DiceExpr::Sum(Box::new(left), Box::new(right));
        let result = roller.roll_expr(&sum_expr).unwrap();

        assert_eq!(result.total, 30);
        assert_eq!(result.rolls, vec![]);
        assert_eq!(result.modifier, 30);
    }

    #[test]
    fn test_expr_sum_multiple_rolls() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(333));
        let left = DiceExpr::Roll(RollSpec::new(2, 6, None));
        let right = DiceExpr::Roll(RollSpec::new(1, 8, None));
        let sum_expr = DiceExpr::Sum(Box::new(left), Box::new(right));
        let result = roller.roll_expr(&sum_expr).unwrap();

        // Should have 3 total rolls (2 d6 + 1 d8)
        assert_eq!(result.rolls.len(), 3);
        assert_eq!(result.modifier, 0);
        assert_eq!(result.total, result.rolls.iter().sum());
    }

    #[test]
    fn test_expr_difference_roll_minus_literal() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(200));
        let left = DiceExpr::Roll(RollSpec::new(1, 20, None));
        let right = DiceExpr::Literal(5);
        let diff_expr = DiceExpr::Difference(Box::new(left), Box::new(right));
        let result = roller.roll_expr(&diff_expr).unwrap();

        // Result should be die roll - 5
        assert_eq!(result.rolls.len(), 1);
        assert_eq!(result.modifier, -5);
        assert_eq!(result.total, result.rolls[0] - 5);
    }

    #[test]
    fn test_expr_difference_literal_minus_roll() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(201));
        let left = DiceExpr::Literal(10);
        let right = DiceExpr::Roll(RollSpec::new(1, 6, None));
        let diff_expr = DiceExpr::Difference(Box::new(left), Box::new(right));
        let result = roller.roll_expr(&diff_expr).unwrap();

        // Result should be 10 - die roll
        assert_eq!(result.rolls.len(), 1);
        assert!(result.rolls[0].abs() >= 1 && result.rolls[0].abs() <= 6);
        assert_eq!(result.total, result.modifier + result.rolls[0]);
        assert_eq!(result.modifier, 10);
    }

    #[test]
    fn test_expr_difference_multiple_literals() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(1));
        let left = DiceExpr::Literal(50);
        let right = DiceExpr::Literal(20);
        let diff_expr = DiceExpr::Difference(Box::new(left), Box::new(right));
        let result = roller.roll_expr(&diff_expr).unwrap();

        assert_eq!(result.total, 30);
        assert_eq!(result.rolls, vec![]);
        assert_eq!(result.modifier, 30);
    }

    #[test]
    fn test_expr_nested_sum_difference() {
        // Evaluate (3 + 1d8) - 1d8
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(555));
        let left = DiceExpr::Roll(RollSpec::new(1, 8, None));
        let inner_sum = DiceExpr::Sum(Box::new(DiceExpr::Literal(3)), Box::new(left.clone()));
        let expr = DiceExpr::Difference(Box::new(inner_sum), Box::new(left));
        let result = roller.roll_expr(&expr).unwrap();

        // Get the values rolled
        let mut ref_rng = StdRng::seed_from_u64(555);
        let mut rolls: Vec<i32> = (1..=2).map(|_| ref_rng.random_range(1..=8)).collect();

        rolls[1] = -rolls[1];

        // The two d8 rolls cancel out, leaving 3
        assert_eq!(result.total, 3 + rolls.iter().sum::<i32>());
        assert_eq!(result.rolls, rolls);
        assert_eq!(result.modifier, 3);
        assert_eq!(result.rolls.len(), 2);
    }

    #[test]
    fn test_expr_complex_nested() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(666));
        // ((2d6 + 5) - 3) + 1d4
        let d2d6 = DiceExpr::Roll(RollSpec::new(2, 6, None));
        let sum1 = DiceExpr::Sum(Box::new(d2d6), Box::new(DiceExpr::Literal(5)));
        let diff = DiceExpr::Difference(Box::new(sum1), Box::new(DiceExpr::Literal(3)));
        let d1d4 = DiceExpr::Roll(RollSpec::new(1, 4, None));
        let final_expr = DiceExpr::Sum(Box::new(diff), Box::new(d1d4));

        let result = roller.roll_expr(&final_expr).unwrap();

        // Should have 3 rolls: 2 d6 + 1 d4
        assert_eq!(result.rolls.len(), 3);
        // Modifier should be 5 - 3 = 2
        assert_eq!(result.modifier, 2);
        // Total should be sum of all rolls + modifier
        assert_eq!(
            result.total,
            result.rolls.iter().sum::<i32>() + result.modifier
        );
    }

    #[test]
    fn test_expr_keep_highest() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(123));
        let expr = DiceExpr::Roll(RollSpec::new(4, 10, Some(Keep::Highest(2))));
        let result = roller.roll_expr(&expr).unwrap();

        // Should have all 4 rolls recorded
        assert_eq!(result.rolls.len(), 4);
        assert_eq!(result.modifier, 0);

        // Total should be the sum of the 2 highest rolls
        let mut rolls_sorted = result.rolls.clone();
        rolls_sorted.sort_unstable();
        let expected_total = rolls_sorted[2] + rolls_sorted[3];
        assert_eq!(result.total, expected_total);
    }

    #[test]
    fn test_expr_keep_lowest() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(124));
        let expr = DiceExpr::Roll(RollSpec::new(4, 10, Some(Keep::Lowest(2))));
        let result = roller.roll_expr(&expr).unwrap();

        // Should have all 4 rolls recorded
        assert_eq!(result.rolls.len(), 4);
        assert_eq!(result.modifier, 0);

        // Total should be the sum of the 2 lowest rolls
        let mut rolls_sorted = result.rolls.clone();
        rolls_sorted.sort_unstable();
        let expected_total = rolls_sorted[0] + rolls_sorted[1];
        assert_eq!(result.total, expected_total);
    }

    #[test]
    fn test_expr_keep_highest_with_sum() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(125));
        let keep_roll = DiceExpr::Roll(RollSpec::new(3, 6, Some(Keep::Highest(1))));
        let literal = DiceExpr::Literal(10);
        let expr = DiceExpr::Sum(Box::new(keep_roll), Box::new(literal));
        let result = roller.roll_expr(&expr).unwrap();

        // Should have 3 rolls (all rolls are recorded)
        assert_eq!(result.rolls.len(), 3);
        assert_eq!(result.modifier, 10);

        // The total should be the highest roll + 10
        let mut rolls_sorted = result.rolls.clone();
        rolls_sorted.sort_unstable();
        let expected_total = rolls_sorted[2] + 10;
        assert_eq!(result.total, expected_total);
    }

    #[test]
    fn test_exprresult_add_preserves_order() {
        let left = ExprResult::new(10, vec![3, 7], 0);
        let right = ExprResult::new(15, vec![5, 10], 0);
        let sum = left + right;

        assert_eq!(sum.rolls.len(), 4);
        assert_eq!(sum.total, 25);
        assert_eq!(sum.modifier, 0);
    }

    #[test]
    fn test_exprresult_negation_flips_signs() {
        let expr = ExprResult::new(10, vec![3, 7], 5);
        let neg = -expr;

        assert_eq!(neg.total, -10);
        assert_eq!(neg.rolls, vec![-3, -7]);
        assert_eq!(neg.modifier, -5);
    }

    #[test]
    fn test_exprresult_sub_uses_negation() {
        let left = ExprResult::new(20, vec![10, 10], 0);
        let right = ExprResult::new(5, vec![5], 0);
        let diff = left - right;

        assert_eq!(diff.total, 15);
        assert_eq!(diff.rolls.len(), 3);
        assert_eq!(diff.rolls, vec![10, 10, -5])
    }

    #[test]
    fn test_deeply_nested_expressions() {
        // Build a deeply nested expression: (((1d6 + 1) + 1) + 1) ...
        let mut expr = DiceExpr::Roll(RollSpec::new(1, 6, None));
        for _ in 0..10 {
            expr = DiceExpr::Sum(Box::new(expr), Box::new(DiceExpr::Literal(1)));
        }

        let mut roller = Roller::from_rng(StdRng::seed_from_u64(42));
        let result = roller.roll_expr(&expr).unwrap();

        assert_eq!(result.modifier, 10);
        assert_eq!(result.rolls.len(), 1);
    }

    // ==== Tests for RollResult to ExprResult conversion ====

    #[test]
    fn test_rollresult_to_exprresult_constant_positive() {
        let rr = RollResult::new(42, RollDetail::Constant(42));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 42);
        assert_eq!(expr_result.rolls, vec![]);
        assert_eq!(expr_result.modifier, 42);
    }

    #[test]
    fn test_rollresult_to_exprresult_constant_negative() {
        let rr = RollResult::new(5, RollDetail::Constant(-20));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 5);
        assert_eq!(expr_result.rolls, vec![]);
        assert_eq!(expr_result.modifier, -20);
    }

    #[test]
    fn test_rollresult_to_exprresult_constant_zero() {
        let rr = RollResult::new(0, RollDetail::Constant(0));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 0);
        assert_eq!(expr_result.rolls, vec![]);
        assert_eq!(expr_result.modifier, 0);
    }

    #[test]
    fn test_rollresult_to_exprresult_dice_simple() {
        let rr = RollResult::new(9, RollDetail::Dice(vec![4, 5]));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 9);
        assert_eq!(expr_result.rolls, vec![4, 5]);
        assert_eq!(expr_result.modifier, 0);
    }

    #[test]
    fn test_rollresult_to_exprresult_dice_multiple() {
        let rr = RollResult::new(21, RollDetail::Dice(vec![3, 7, 5, 6]));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 21);
        assert_eq!(expr_result.rolls, vec![3, 7, 5, 6]);
        assert_eq!(expr_result.modifier, 0);
    }

    #[test]
    fn test_rollresult_to_exprresult_dice_single() {
        let rr = RollResult::new(12, RollDetail::Dice(vec![12]));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 12);
        assert_eq!(expr_result.rolls, vec![12]);
        assert_eq!(expr_result.modifier, 0);
    }

    #[test]
    #[should_panic = "Overflow as expected"]
    fn test_rollresult_to_exprresult_overflow() {
        let rr = RollResult::new(u32::MAX, RollDetail::Dice(vec![u32::MAX]));
        let result = ExprResult::try_from(rr);

        assert!(result.is_err());
        match result.unwrap_err() {
            DiceError::Overflow(_) => panic!("Overflow as expected"),
            _ => panic!("Expected Overflow Variant"),
        }
    }

    #[test]
    #[should_panic = "Overflow as expected"]
    fn test_rollresult_to_exprresult_dice_with_overflow() {
        let rr = RollResult::new(100, RollDetail::Dice(vec![u32::MAX - 1, 2]));
        let result = ExprResult::try_from(rr);

        assert!(result.is_err());
        match result.unwrap_err() {
            DiceError::Overflow(_) => panic!("Overflow as expected"),
            _ => {
                panic!("Expected Overflow variant.")
            }
        }
    }

    #[test]
    fn test_rollresult_to_exprresult_empty_dice() {
        let rr = RollResult::new(0, RollDetail::Dice(vec![]));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 0);
        assert_eq!(expr_result.rolls, vec![]);
        assert_eq!(expr_result.modifier, 0);
    }

    #[test]
    fn test_rollresult_to_exprresult_large_valid_values() {
        let rr = RollResult::new(1000, RollDetail::Dice(vec![200, 300, 500]));
        let expr_result = ExprResult::try_from(rr).unwrap();

        assert_eq!(expr_result.total, 1000);
        assert_eq!(expr_result.rolls, vec![200, 300, 500]);
        assert_eq!(expr_result.modifier, 0);
    }

    #[test]
    fn test_rollresult_from_keep_highest_preserves_detail() {
        let mut roller = Roller::from_rng(StdRng::seed_from_u64(777));
        let spec = RollSpec::new(5, 12, Some(Keep::Highest(2)));
        let rr = roller.roll_spec(&spec).unwrap();

        // The detail should contain all 5 rolls
        if let RollDetail::Dice(ref rolls) = rr.detail {
            assert_eq!(rolls.len(), 5);
        } else {
            panic!("Expected Dice variant");
        }

        let expr_result = ExprResult::try_from(rr).unwrap();
        // ExprResult should have all 5 rolls, even though total only includes 2
        assert_eq!(expr_result.rolls.len(), 5);
    }
}
