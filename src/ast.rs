use std::str::FromStr;

use crate::{
    error::DiceError,
    parser::Parser,
    roller::{ExprResult, Roller},
};

#[derive(Debug, Clone)]
pub enum DiceExpr {
    Sum(Box<DiceExpr>, Box<DiceExpr>),
    Difference(Box<DiceExpr>, Box<DiceExpr>),
    Roll(RollSpec),
    Literal(i32),
}

impl FromStr for DiceExpr {
    type Err = DiceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(s);
        parser.parse()
    }
}

impl DiceExpr {
    pub fn roll(&self) -> Result<ExprResult, DiceError> {
        let mut roller = Roller::default();
        roller.roll_expr(self)
    }
    pub fn parse(input: &str) -> Result<DiceExpr, DiceError> {
        DiceExpr::from_str(input)
    }
}

#[derive(Debug, Clone)]
pub struct RollSpec {
    pub count: u32,
    pub sides: u32,
    pub keep: Option<Keep>,
}

impl RollSpec {
    pub fn new(count: u32, sides: u32, keep: Option<Keep>) -> Self {
        RollSpec { count, sides, keep }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keep {
    Highest(u32),
    Lowest(u32),
}
