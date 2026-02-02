//! # dice-parser
//!
//! A parser and roller for standard RPG dice notation.
//!
//! This crate provides a simple and flexible way to parse and evaluate dice expressions
//! commonly used in tabletop role-playing games. It supports basic arithmetic operations
//! (addition and subtraction) and dice rolling with various modifiers.
//!
//! ## Features
//!
//! - Parse standard dice notation (e.g., "2d6", "1d20+5", "3d8-2")
//! - Support for keep highest/lowest mechanics (currently via manual construction only; parsing support planned for future release)
//! - Detailed roll results including individual die rolls and modifiers
//! - Custom RNG support for deterministic testing
//! - Comprehensive error handling
//!
//! ## Quick Start
//!
//! ### Parsing and Rolling
//!
//! ```
//! use dice_parser::DiceExpr;
//!
//! // Parse a dice expression from a string
//! let expr = DiceExpr::parse("2d6+3").unwrap();
//!
//! // Roll the dice
//! let result = expr.roll().unwrap();
//! println!("Total: {}", result.total);
//! println!("Rolls: {:?}", result.rolls);
//! println!("Modifier: {}", result.modifier);
//! ```
//!
//! ### Manual Construction
//!
//! ```
//! use dice_parser::{DiceExpr, RollSpec, Keep};
//!
//! // Manually construct a dice expression: 4d6 keep highest 3
//! let roll_spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
//! let expr = DiceExpr::Roll(roll_spec);
//!
//! let result = expr.roll().unwrap();
//! println!("Total: {}", result.total);
//! ```
//!
//! ### Using Custom RNG
//!
//! ```
//! use dice_parser::DiceExpr;
//! use rand::{SeedableRng, rngs::StdRng};
//!
//! let expr = DiceExpr::parse("1d20").unwrap();
//! let rng = StdRng::seed_from_u64(42);
//! let result = expr.roll_with_rng(rng).unwrap();
//! ```

mod ast;
mod error;
mod parser;
mod roller;

pub use crate::ast::{DiceExpr, Keep, RollSpec};
pub use crate::error::DiceError;
pub use crate::roller::ExprResult;
