//! Error types for dice parsing and rolling.
//!
//! This module defines all error types that can occur when parsing or evaluating dice expressions.

use std::{
    fmt::{Debug, Display, Formatter},
    num::TryFromIntError,
};

use crate::ast::RollSpec;

/// The error type for dice parsing and rolling operations.
///
/// This enum encompasses all errors that can occur during:
/// - Parsing dice notation strings
/// - Evaluating dice expressions
/// - Rolling dice
///
/// All variants implement `Display` with helpful error messages that include
/// context about where in the input the error occurred.
///
/// # Examples
///
/// ```
/// use dice_parser::Parser;
///
/// let mut parser = Parser::new("2d6 extra");
/// match parser.parse() {
///     Err(e) => {
///         // The error message shows where the problem is
///         let msg = format!("{}", e);
///         assert!(msg.contains("trailing input"));
///     }
///     Ok(_) => panic!("Expected an error"),
/// }
/// ```
#[derive(Clone)]
pub enum DiceError {
    /// An arithmetic overflow occurred during evaluation.
    Overflow(String),
    /// A roll specification is invalid (e.g., trying to keep more dice than were rolled).
    InvalidSpec(RollSpec, String),
    /// Failed to parse a number or other token from the input.
    ParseError {
        /// The kind of parse error that occurred.
        kind: ParseErrorKind,
        /// The full input string.
        input: String,
        /// The byte position where the error started.
        start: usize,
        /// The byte position where the error ended (if applicable).
        stop: Option<usize>,
    },
    /// The input contains invalid syntax.
    SyntaxError {
        /// The full input string.
        input: String,
        /// The byte position where the error started.
        start: usize,
        /// The byte position where the error ended (if applicable).
        stop: Option<usize>,
        /// What was expected at this position (if known).
        expected: Option<String>,
    },
    /// The input contains extra characters after a valid expression.
    TrailingInput {
        /// The full input string.
        input: String,
        /// The byte position where the trailing input starts.
        pos: usize,
    },
}

/// The specific kind of parse error that occurred.
///
/// This enum distinguishes between different types of parsing failures
/// to provide more specific error messages.
#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    /// Expected a number but found something else or nothing.
    ExpectedNumber,
    /// Found an invalid or out-of-range i32 literal.
    InvalidI32,
    /// Found an invalid or out-of-range u32 literal.
    InvalidU32,
}

impl std::error::Error for DiceError {}

impl Debug for DiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl Display for DiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiceError::Overflow(msg) => write!(f, "dice overflow error: {}", msg),
            DiceError::InvalidSpec(spec, msg) => {
                write!(f, "invalid RollSpec error: {}, {:?}", msg, spec)
            }
            DiceError::ParseError {
                kind,
                input,
                start,
                stop,
            } => match kind {
                ParseErrorKind::ExpectedNumber => {
                    writeln!(f, "expected number in input, found nothing here:")?;
                    writeln!(f, "{}", input)?;
                    let mut indicator: String = " ".repeat(*start);
                    match stop {
                        Some(i) => indicator.push_str(&("^".repeat((i - start).max(1)))),
                        None => indicator.push('^'),
                    }
                    write!(f, "{}", indicator)
                }
                ParseErrorKind::InvalidU32 => {
                    writeln!(f, "invalid u32 literal in input, parse errored here:")?;
                    writeln!(f, "{}", input)?;
                    let mut indicator: String = " ".repeat(*start);
                    match stop {
                        Some(i) => indicator.push_str(&("^".repeat((i - start).max(1)))),
                        None => indicator.push('^'),
                    }
                    write!(f, "{}", indicator)
                }
                ParseErrorKind::InvalidI32 => {
                    writeln!(f, "invalid i32 literal in input, parse errored here:")?;
                    writeln!(f, "{}", input)?;
                    let mut indicator: String = " ".repeat(*start);
                    match stop {
                        Some(i) => indicator.push_str(&("^".repeat((i - start).max(1)))),
                        None => indicator.push('^'),
                    }
                    write!(f, "{}", indicator)
                }
            },
            DiceError::SyntaxError {
                input,
                start,
                stop,
                expected,
            } => {
                writeln!(f, "syntax error in dice expression here:")?;
                writeln!(f, "{}", input)?;
                let mut indicator = " ".repeat(*start);
                match stop {
                    Some(i) => indicator.push_str(&("^".repeat((i - start).max(1)))),
                    None => indicator.push('^'),
                }
                if let Some(exp) = expected {
                    writeln!(f, "{}", indicator)?;
                    write!(f, "expected: {}", exp)
                } else {
                    write!(f, "{}", indicator)
                }
            }
            DiceError::TrailingInput { input, pos } => {
                writeln!(f, "trailing input encountered in expression:")?;
                writeln!(f, "{}", input)?;
                let mut indicator = " ".repeat(*pos);
                indicator.push_str(&("^".repeat(input.chars().count() - pos)));
                write!(f, "{}", indicator)
            }
        }
    }
}

impl From<TryFromIntError> for DiceError {
    fn from(value: TryFromIntError) -> Self {
        DiceError::Overflow(value.to_string())
    }
}
