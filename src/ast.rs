use crate::{
    error::DiceError,
    roller::{ExprResult, Roller},
};

#[derive(Debug, Clone)]
pub enum DiceExpr {
    Sum(Box<DiceExpr>, Box<DiceExpr>),
    Difference(Box<DiceExpr>, Box<DiceExpr>),
    Roll(RollSpec),
    Literal(i32),
}

impl DiceExpr {
    pub fn roll(&self) -> Result<ExprResult, DiceError> {
        let mut roller = Roller::default();
        roller.roll_expr(self)
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

#[derive(Debug, Clone)]
pub enum Keep {
    Highest(u32),
    Lowest(u32),
}
