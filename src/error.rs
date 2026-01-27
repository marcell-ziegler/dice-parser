use std::{fmt::Display, num::TryFromIntError};

use crate::ast::RollSpec;

#[derive(Debug, Clone)]
pub enum DiceError {
    Overflow(String),
    InvalidSpec(RollSpec, String),
}

impl std::error::Error for DiceError {}

impl Display for DiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiceError::Overflow(msg) => write!(f, "dice overflow error: {}", msg),
            DiceError::InvalidSpec(spec, msg) => {
                write!(f, "invalid RollSpec error: {}, {:?}", msg, spec)
            }
        }
    }
}

impl From<TryFromIntError> for DiceError {
    fn from(value: TryFromIntError) -> Self {
        DiceError::Overflow(value.to_string())
    }
}
