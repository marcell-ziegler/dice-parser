use std::{fmt::Display, num::TryFromIntError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiceError {
    Overflow(String),
}

impl std::error::Error for DiceError {}

impl Display for DiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiceError::Overflow(msg) => write!(f, "dice overflow error: {}", msg),
        }
    }
}

impl From<TryFromIntError> for DiceError {
    fn from(value: TryFromIntError) -> Self {
        DiceError::Overflow(value.to_string())
    }
}
