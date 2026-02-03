use std::{
    fmt::{Debug, Display, Formatter},
    num::TryFromIntError,
};

use crate::ast::RollSpec;

/// Errors that can occur when parsing or evaluating dice expressions.
///
/// This enum covers all possible error conditions, including parse errors,
/// invalid roll specifications, and arithmetic overflow.
///
/// # Variants
///
/// - `Overflow`: Integer overflow during calculation
/// - `InvalidSpec`: Invalid roll specification (e.g., trying to keep more dice than rolled)
/// - `ParseError`: Error parsing the input string
/// - `SyntaxError`: Syntax error in the dice expression
/// - `TrailingInput`: Unexpected characters after the expression
///
/// # Examples
///
/// ```
/// use dice_parser::DiceExpr;
///
/// // Parse error - invalid syntax
/// let err = DiceExpr::parse("2d").unwrap_err();
///
/// // Syntax error - negative dice count
/// let err = DiceExpr::parse("-2d6").unwrap_err();
///
/// // Trailing input error
/// let err = DiceExpr::parse("2d6 extra").unwrap_err();
/// ```
#[derive(Clone)]
pub enum DiceError {
    /// Integer overflow occurred during calculation.
    Overflow(String),
    /// Invalid roll specification.
    ///
    /// This error occurs when a `RollSpec` is invalid, such as trying to
    /// keep more dice than were rolled.
    InvalidSpec(RollSpec, String),
    /// Parse error with details about what went wrong.
    ParseError {
        /// The kind of parse error.
        kind: ParseErrorKind,
        /// The input string that failed to parse.
        input: String,
        /// The byte position where the error started.
        start: usize,
        /// The byte position where the error ended (if applicable).
        stop: Option<usize>,
    },
    /// Syntax error in the dice expression.
    SyntaxError {
        /// The input string with the syntax error.
        input: String,
        /// The byte position where the error started.
        start: usize,
        /// The byte position where the error ended (if applicable).
        stop: Option<usize>,
        /// Description of what was expected.
        expected: Option<String>,
    },
    /// Unexpected characters found after the expression.
    TrailingInput {
        /// The input string with trailing characters.
        input: String,
        /// The byte position where trailing input begins.
        pos: usize,
    },
}

/// The specific kind of parse error that occurred.
#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    /// Expected a number but found something else.
    ExpectedNumber,
    /// Failed to parse an i32 value.
    InvalidI32,
    /// Failed to parse a u32 value.
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
