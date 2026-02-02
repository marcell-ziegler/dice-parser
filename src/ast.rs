//! Abstract Syntax Tree (AST) definitions for dice expressions.
//!
//! This module contains the core data structures that represent parsed dice expressions.

use crate::{
    error::DiceError,
    roller::{ExprResult, Roller},
};

/// A dice expression that can be evaluated to produce a random result.
///
/// `DiceExpr` represents the AST of a parsed dice notation string. It supports:
/// - Rolling dice with a specified number of sides
/// - Literal integer values (constants)
/// - Addition of two expressions
/// - Subtraction of two expressions
///
/// # Examples
///
/// ```
/// use dice_parser::{DiceExpr, RollSpec};
///
/// // A simple 2d6 roll
/// let expr = DiceExpr::Roll(RollSpec::new(2, 6, None));
/// let result = expr.roll().unwrap();
/// assert_eq!(result.rolls.len(), 2);
/// ```
///
/// ```
/// use dice_parser::DiceExpr;
///
/// // A literal value
/// let expr = DiceExpr::Literal(5);
/// let result = expr.roll().unwrap();
/// assert_eq!(result.total, 5);
/// assert_eq!(result.modifier, 5);
/// ```
///
/// ```
/// use dice_parser::{DiceExpr, RollSpec};
///
/// // An addition: 1d20 + 5
/// let roll = DiceExpr::Roll(RollSpec::new(1, 20, None));
/// let modifier = DiceExpr::Literal(5);
/// let expr = DiceExpr::Sum(Box::new(roll), Box::new(modifier));
/// let result = expr.roll().unwrap();
/// assert_eq!(result.rolls.len(), 1);
/// assert_eq!(result.modifier, 5);
/// ```
#[derive(Debug, Clone)]
pub enum DiceExpr {
    /// The sum of two dice expressions.
    Sum(Box<DiceExpr>, Box<DiceExpr>),
    /// The difference of two dice expressions (left - right).
    Difference(Box<DiceExpr>, Box<DiceExpr>),
    /// A dice roll specification.
    Roll(RollSpec),
    /// A literal integer value.
    Literal(i32),
}

impl DiceExpr {
    /// Roll this dice expression and return the result.
    ///
    /// This is a convenience method that creates a default `Roller` and evaluates the expression.
    /// For more control over the random number generator, use [`Roller::roll_expr`] directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use dice_parser::{DiceExpr, RollSpec};
    ///
    /// let expr = DiceExpr::Roll(RollSpec::new(1, 6, None));
    /// let result = expr.roll().unwrap();
    /// assert!(result.total >= 1 && result.total <= 6);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`DiceError`](crate::DiceError) if:
    /// - The expression would cause integer overflow
    /// - A roll specification is invalid
    pub fn roll(&self) -> Result<ExprResult, DiceError> {
        let mut roller = Roller::default();
        roller.roll_expr(self)
    }
}

/// A specification for rolling one or more dice.
///
/// `RollSpec` describes a dice roll in the format `NdS` where:
/// - `N` is the number of dice to roll (`count`)
/// - `S` is the number of sides on each die (`sides`)
/// - Optionally, which dice to keep (highest or lowest)
///
/// # Examples
///
/// ```
/// use dice_parser::RollSpec;
///
/// // 2d6 - roll two six-sided dice
/// let spec = RollSpec::new(2, 6, None);
/// assert_eq!(spec.count, 2);
/// assert_eq!(spec.sides, 6);
/// ```
///
/// # Special Cases
///
/// - If `count` is 0, the roll evaluates to 0
/// - If `sides` is 0, the roll evaluates to `count` (treated as a constant)
#[derive(Debug, Clone)]
pub struct RollSpec {
    /// The number of dice to roll.
    pub count: u32,
    /// The number of sides on each die.
    pub sides: u32,
    /// Optional keep specification for keeping highest or lowest dice.
    pub keep: Option<Keep>,
}

impl RollSpec {
    /// Create a new `RollSpec`.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of dice to roll
    /// * `sides` - The number of sides on each die
    /// * `keep` - Optional specification for keeping highest or lowest dice
    ///
    /// # Examples
    ///
    /// ```
    /// use dice_parser::RollSpec;
    ///
    /// // Roll 3 six-sided dice
    /// let spec = RollSpec::new(3, 6, None);
    /// ```
    pub fn new(count: u32, sides: u32, keep: Option<Keep>) -> Self {
        RollSpec { count, sides, keep }
    }
}

/// Specifies which dice to keep from a roll.
///
/// When rolling multiple dice, you can optionally keep only the highest or lowest results.
/// This is commonly used in systems like D&D for advantage/disadvantage mechanics.
///
/// # Examples
///
/// ```
/// use dice_parser::{Keep, RollSpec, DiceExpr};
///
/// // Roll 4d6, keep the highest 3 (common in D&D character creation)
/// let spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
/// let expr = DiceExpr::Roll(spec);
/// let result = expr.roll().unwrap();
///
/// // All 4 dice were rolled and recorded
/// assert_eq!(result.rolls.len(), 4);
/// // But only the 3 highest were counted in the total
/// ```
#[derive(Debug, Clone)]
pub enum Keep {
    /// Keep the N highest dice rolls.
    Highest(u32),
    /// Keep the N lowest dice rolls.
    Lowest(u32),
}
