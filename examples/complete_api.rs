//! Comprehensive example demonstrating the complete public API.
//!
//! This example showcases all public types and methods available in the dice-parser crate.

use dice_parser::{DiceError, DiceExpr, ExprResult, Keep, RollSpec};
use rand::{rngs::StdRng, SeedableRng};

fn main() {
    println!("=== COMPREHENSIVE PUBLIC API DEMONSTRATION ===\n");

    // ========================================
    // 1. DiceExpr - The main expression type
    // ========================================
    println!("1. DiceExpr Usage:");
    println!("   All public variants and methods\n");

    // 1.1 Parsing from string
    println!("   1.1 DiceExpr::parse() - Parse from string");
    let parsed = DiceExpr::parse("2d6+3").unwrap();
    println!("       Parsed: 2d6+3");

    // 1.2 DiceExpr variants - all publicly accessible
    println!("\n   1.2 DiceExpr variants - Manual construction");
    
    let literal = DiceExpr::Literal(5);
    println!("       DiceExpr::Literal(5)");

    let roll = DiceExpr::Roll(RollSpec::new(2, 6, None));
    println!("       DiceExpr::Roll(RollSpec::new(2, 6, None))");

    let sum = DiceExpr::Sum(Box::new(roll.clone()), Box::new(literal));
    println!("       DiceExpr::Sum(roll, literal)");

    let _diff = DiceExpr::Difference(Box::new(sum), Box::new(DiceExpr::Literal(1)));
    println!("       DiceExpr::Difference(sum, literal)");

    // 1.3 Rolling methods
    println!("\n   1.3 DiceExpr::roll() - Roll with default RNG");
    let result1 = parsed.roll().unwrap();
    println!("       Result: {}", result1.total);

    println!("\n   1.4 DiceExpr::roll_with_rng() - Roll with custom RNG");
    let rng = StdRng::seed_from_u64(42);
    let result2 = DiceExpr::parse("1d20").unwrap().roll_with_rng(rng).unwrap();
    println!("       Result: {}", result2.total);

    // ========================================
    // 2. RollSpec - Dice roll specification
    // ========================================
    println!("\n2. RollSpec Usage:");
    println!("   Struct with public fields and constructor\n");

    let spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
    println!("   RollSpec::new(4, 6, Some(Keep::Highest(3)))");
    println!("   - count: {}", spec.count);
    println!("   - sides: {}", spec.sides);
    println!("   - keep: {:?}", spec.keep);

    // ========================================
    // 3. Keep - Keep modifier enum
    // ========================================
    println!("\n3. Keep Usage:");
    println!("   Enum with public variants\n");

    let _keep_high = Keep::Highest(3);
    println!("   Keep::Highest(3) - Keep 3 highest dice");

    let _keep_low = Keep::Lowest(1);
    println!("   Keep::Lowest(1) - Keep 1 lowest die");

    // Use in RollSpec
    let _advantage = RollSpec::new(2, 20, Some(Keep::Highest(1)));
    let _disadvantage = RollSpec::new(2, 20, Some(Keep::Lowest(1)));
    println!("   Used in RollSpec for advantage/disadvantage");

    // ========================================
    // 4. ExprResult - Roll result
    // ========================================
    println!("\n4. ExprResult Usage:");
    println!("   Struct with public fields\n");

    let expr = DiceExpr::parse("2d6+3").unwrap();
    let result: ExprResult = expr.roll().unwrap();
    
    println!("   ExprResult from rolling 2d6+3:");
    println!("   - total: {} (final result)", result.total);
    println!("   - rolls: {:?} (individual dice)", result.rolls);
    println!("   - modifier: {} (constant modifiers)", result.modifier);

    // ========================================
    // 5. DiceError - Error handling
    // ========================================
    println!("\n5. DiceError Usage:");
    println!("   Enum with public variants for error handling\n");

    // ParseError variant
    match DiceExpr::parse("invalid") {
        Err(DiceError::ParseError { kind, input, start, stop: _ }) => {
            println!("   DiceError::ParseError:");
            println!("   - kind: {:?}", kind);
            println!("   - input: {}", input);
            println!("   - start: {}", start);
        }
        _ => {}
    }

    // SyntaxError variant
    match DiceExpr::parse("-2d6") {
        Err(DiceError::SyntaxError { expected, .. }) => {
            println!("\n   DiceError::SyntaxError:");
            if let Some(exp) = expected {
                println!("   - expected: {}", exp);
            }
        }
        _ => {}
    }

    // TrailingInput variant
    match DiceExpr::parse("2d6 extra") {
        Err(DiceError::TrailingInput { pos, .. }) => {
            println!("\n   DiceError::TrailingInput:");
            println!("   - pos: {}", pos);
        }
        _ => {}
    }

    // InvalidSpec variant
    let bad_spec = RollSpec::new(2, 6, Some(Keep::Highest(5)));
    match DiceExpr::Roll(bad_spec).roll() {
        Err(DiceError::InvalidSpec(spec, msg)) => {
            println!("\n   DiceError::InvalidSpec:");
            println!("   - spec: {:?}", spec);
            println!("   - message: {}", msg);
        }
        _ => {}
    }

    // ========================================
    // 6. Complete Example
    // ========================================
    println!("\n6. Complete Example:");
    println!("   Building a complex expression with all features\n");

    // Character creation: 6 abilities, each 4d6 keep highest 3
    println!("   Rolling D&D ability scores (4d6 drop lowest):");
    let ability_roll = DiceExpr::Roll(RollSpec::new(4, 6, Some(Keep::Highest(3))));
    
    for i in 1..=6 {
        if let Ok(result) = ability_roll.roll() {
            println!("   Ability {}: {} (rolls: {:?})", i, result.total, result.rolls);
        }
    }

    println!("\n=== ALL PUBLIC API FEATURES DEMONSTRATED ===");
    println!("\nPublic Types:");
    println!("  ✓ DiceExpr (enum with Sum, Difference, Roll, Literal)");
    println!("  ✓ RollSpec (struct with count, sides, keep)");
    println!("  ✓ Keep (enum with Highest, Lowest)");
    println!("  ✓ ExprResult (struct with total, rolls, modifier)");
    println!("  ✓ DiceError (enum with 5 error variants)");
    println!("\nPublic Methods:");
    println!("  ✓ DiceExpr::parse()");
    println!("  ✓ DiceExpr::roll()");
    println!("  ✓ DiceExpr::roll_with_rng()");
    println!("  ✓ RollSpec::new()");
}
