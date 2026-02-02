//! Basic usage examples demonstrating the dice-parser API.

use dice_parser::{DiceExpr, Keep, RollSpec};

fn main() {
    println!("=== Basic Parsing and Rolling ===");
    let expr = DiceExpr::parse("2d6+3").unwrap();
    let result = expr.roll().unwrap();
    println!("Expression: 2d6+3");
    println!("Total: {}", result.total);
    println!("Rolls: {:?}", result.rolls);
    println!("Modifier: {}", result.modifier);
    println!();

    println!("=== Manual Construction ===");
    let roll_spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
    let expr = DiceExpr::Roll(roll_spec);
    let result = expr.roll().unwrap();
    println!("Expression: 4d6 keep highest 3");
    println!("Total: {}", result.total);
    println!("All rolls: {:?}", result.rolls);
    println!();

    println!("=== Complex Expression ===");
    let d2d6 = DiceExpr::Roll(RollSpec::new(2, 6, None));
    let d1d4 = DiceExpr::Roll(RollSpec::new(1, 4, None));
    let modifier = DiceExpr::Literal(2);
    let sum = DiceExpr::Sum(Box::new(d2d6), Box::new(d1d4));
    let expr = DiceExpr::Difference(Box::new(sum), Box::new(modifier));
    let result = expr.roll().unwrap();
    println!("Expression: (2d6 + 1d4) - 2");
    println!("Total: {}", result.total);
    println!();

    println!("=== D&D 5e Advantage ===");
    let advantage = DiceExpr::Roll(RollSpec::new(2, 20, Some(Keep::Highest(1))));
    let result = advantage.roll().unwrap();
    println!("Expression: 2d20 keep highest 1 (advantage)");
    println!("Rolls: {:?}", result.rolls);
    println!("Result: {}", result.total);
    println!();

    println!("=== D&D 5e Disadvantage ===");
    let disadvantage = DiceExpr::Roll(RollSpec::new(2, 20, Some(Keep::Lowest(1))));
    let result = disadvantage.roll().unwrap();
    println!("Expression: 2d20 keep lowest 1 (disadvantage)");
    println!("Rolls: {:?}", result.rolls);
    println!("Result: {}", result.total);
    println!();

    println!("=== Character Ability Scores ===");
    println!("Rolling 4d6 drop lowest for 6 abilities:");
    let ability_roll = DiceExpr::Roll(RollSpec::new(4, 6, Some(Keep::Highest(3))));
    for i in 0..6 {
        let result = ability_roll.roll().unwrap();
        println!("  Ability {}: {} (rolls: {:?})", i + 1, result.total, result.rolls);
    }
}
