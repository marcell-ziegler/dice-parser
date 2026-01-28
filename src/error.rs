use std::{
    fmt::{Debug, Display, Formatter},
    num::TryFromIntError,
};

use crate::ast::RollSpec;

#[derive(Clone)]
pub enum DiceError {
    Overflow(String),
    InvalidSpec(RollSpec, String),
    ParseError {
        kind: ParseErrorKind,
        input: String,
        start: usize,
        stop: Option<usize>,
    },
    SyntaxError {
        input: String,
        start: usize,
        stop: Option<usize>,
        expected: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    ExpectedNumber,
    InvalidI32,
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
        }
    }
}

impl From<TryFromIntError> for DiceError {
    fn from(value: TryFromIntError) -> Self {
        DiceError::Overflow(value.to_string())
    }
}
