pub enum DiceExpr {
    Sum(Box<DiceExpr>, Box<DiceExpr>),
    Difference(Box<DiceExpr>, Box<DiceExpr>),
    Roll(RollSpec),
    Literal(i32),
}

pub struct RollSpec {
    pub count: u32,
    pub sides: u32,
    pub keep: Option<Keep>,
}

pub enum Keep {
    Highest(u32),
    Lowest(u32),
}
