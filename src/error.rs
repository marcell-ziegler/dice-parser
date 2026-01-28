use std::{fmt::Display, num::TryFromIntError};

use crate::ast::RollSpec;

#[derive(Debug, Clone)]
pub enum DiceError {
    Overflow(String),
    InvalidSpec(RollSpec, String),
    ParseError {
        kind: ParseErrorKind,
        input: String,
        start: usize,
        stop: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    ExpectedNumber,
    InvalidNumber,
}

impl std::error::Error for DiceError {}

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
                        Some(i) => indicator.push_str(&("^".repeat(i - start))),
                        None => indicator.push('^'),
                    }
                    write!(f, "{}", indicator)
                }
                ParseErrorKind::InvalidNumber => {
                    writeln!(f, "invalid number in input, parse errored here:")?;
                    writeln!(f, "{}", input)?;
                    let mut indicator: String = " ".repeat(*start);
                    match stop {
                        Some(i) => indicator.push_str(&("^".repeat(i - start))),
                        None => indicator.push('^'),
                    }
                    write!(f, "{}", indicator)
                }
            },
        }
    }
}

impl From<TryFromIntError> for DiceError {
    fn from(value: TryFromIntError) -> Self {
        DiceError::Overflow(value.to_string())
    }
}
