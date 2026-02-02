//! # dice-parser
//!
//! A parser and roller for standard RPG dice notation.
//!
//! This crate provides a complete solution for parsing and evaluating dice expressions commonly
//! used in tabletop role-playing games. It supports:
//! - Basic dice notation (e.g., `2d6`, `1d20`)
//! - Arithmetic operations (addition and subtraction)
//! - Literal values
//! - Keep highest/lowest mechanics
//!
//! ## Quick Start
//!
//! Parse and roll a dice expression:
//!
//! ```
//! use dice_parser::{Parser, DiceExpr};
//!
//! // Parse a dice expression
//! let mut parser = Parser::new("2d6 + 3");
//! let expr = parser.parse().unwrap();
//!
//! // Roll the dice
//! let result = expr.roll().unwrap();
//!
//! // Access the results
//! println!("Total: {}", result.total);
//! println!("Rolls: {:?}", result.rolls);
//! println!("Modifier: {}", result.modifier);
//! ```
//!
//! ## Examples
//!
//! ### Simple dice roll
//!
//! ```
//! use dice_parser::Parser;
//!
//! let mut parser = Parser::new("1d20");
//! let expr = parser.parse().unwrap();
//! let result = expr.roll().unwrap();
//!
//! // Result will be between 1 and 20
//! assert!(result.total >= 1 && result.total <= 20);
//! assert_eq!(result.rolls.len(), 1);
//! ```
//!
//! ### Complex expressions
//!
//! ```
//! use dice_parser::Parser;
//!
//! let mut parser = Parser::new("2d6 + 1d8 - 3");
//! let expr = parser.parse().unwrap();
//! let result = expr.roll().unwrap();
//!
//! // Result contains all rolled dice
//! assert_eq!(result.rolls.len(), 3); // 2 d6 rolls + 1 d8 roll
//! assert_eq!(result.modifier, -3);
//! ```
//!
//! ### Error handling
//!
//! ```
//! use dice_parser::Parser;
//!
//! let mut parser = Parser::new("invalid dice");
//! let result = parser.parse();
//!
//! assert!(result.is_err());
//! ```

mod ast;
mod error;
mod parser;
mod roller;

// Re-export public API
pub use ast::{DiceExpr, Keep, RollSpec};
pub use error::{DiceError, ParseErrorKind};
pub use parser::Parser;
pub use roller::{ExprResult, RollDetail, RollResult, Roller};
