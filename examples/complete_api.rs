//! Comprehensive example demonstrating the complete public API.
//!
//! This example showcases all public types and methods available in the dice-parser crate.

use dice_parser::{DiceError, DiceExpr, ExprResult, Keep, RollSpec};
use rand::{SeedableRng, rngs::StdRng};

fn main() {
    // ========================================
    // 1. DiceExpr - The main expression type
    // ========================================

    // 1.1 Parsing from string
    println!("1. Parsing and expressions");
    let parsed = DiceExpr::parse("2d6+3").unwrap();
    println!("  1.1 Parsing e.g. \"2d6+3\" yields:");
    println!("    {:?}", parsed);

    // 1.2 DiceExpr variants - all publicly accessible
    let literal = DiceExpr::Literal(5);
    println!("  1.2 A literal expression \"5\": ");
    println!("    {:?}", literal);

    let roll = DiceExpr::Roll(RollSpec::new(2, 6, None));
    println!("  1.3 A single roll expression corresponding to 2d6:",);
    println!("    {:?}", roll);

    let sum = DiceExpr::Sum(Box::new(roll.clone()), Box::new(literal.clone()));
    println!("  1.4 A sum of two expression, e.g. \"2d6+5\":");
    println!("    {:?}", sum);

    let _diff = DiceExpr::Difference(Box::new(roll.clone()), Box::new(DiceExpr::Literal(1)));
    println!("  1.5 A difference of two expressions, e.g. \"2d6-1\":",);
    println!("    {:?}", _diff);

    // 1.3 Rolling methods
    println!("\n  1.6 DiceExpr::roll() - Rolling expressions");
    let result1 = parsed.roll().unwrap();
    println!("    An expression, 2d6+3: {}", result1.total);
    let result2 = literal.roll().unwrap();
    println!("    A literal 5: {}", result2.total);
    let result3 = roll.roll().unwrap();
    println!("    A single roll 2d6: {}", result3.total);
    let result4 = sum.roll().unwrap();
    println!("    A manual sum, 2d6+5: {}", result4.total);
    let result5 = _diff.roll().unwrap();
    println!("    A manual diff, 2d6-1: {}", result5.total);

    println!("\n   1.7 DiceExpr::roll_with_rng() - Roll with custom RNG");
    let rng = StdRng::seed_from_u64(42);
    let result6 = DiceExpr::parse("1d20").unwrap().roll_with_rng(rng).unwrap();
    println!("       Rolling 1d20: {}", result6.total);

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
    println!("   {:?} - Keep 3 highest dice", _keep_high);

    let _keep_low = Keep::Lowest(1);
    println!("   {:?} - Keep 1 lowest die", _keep_high);

    // Use in RollSpec
    let _advantage = RollSpec::new(2, 20, Some(Keep::Highest(1)));
    let _disadvantage = RollSpec::new(2, 20, Some(Keep::Lowest(1)));
    println!("\n  Example with Advantage:");
    println!(
        "    {:?}, result: {}",
        _advantage,
        DiceExpr::Roll(_advantage.clone()).roll().unwrap().total
    );
    println!("\n  Example with Disadvantage:");
    println!(
        "    {:?}, result: {}",
        _disadvantage,
        DiceExpr::Roll(_disadvantage.clone()).roll().unwrap().total
    );

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
    if let Err(DiceError::ParseError {
        kind,
        input,
        start,
        stop: _,
    }) = DiceExpr::parse("invalid")
    {
        println!("   DiceError::ParseError:");
        println!("   - kind: {:?}", kind);
        println!("   - input: {}", input);
        println!("   - start: {}", start);
    }

    // SyntaxError variant
    if let Err(DiceError::SyntaxError { expected, .. }) = DiceExpr::parse("-2d6") {
        println!("\n   DiceError::SyntaxError:");
        if let Some(exp) = expected {
            println!("   - expected: {}", exp);
        }
    }

    // TrailingInput variant
    if let Err(DiceError::TrailingInput { pos, .. }) = DiceExpr::parse("2d6 extra") {
        println!("\n   DiceError::TrailingInput:");
        println!("   - pos: {}", pos);
    }

    // InvalidSpec variant
    let bad_spec = RollSpec::new(2, 6, Some(Keep::Highest(5)));
    if let Err(DiceError::InvalidSpec(spec, msg)) = DiceExpr::Roll(bad_spec).roll() {
        println!("\n   DiceError::InvalidSpec:");
        println!("   - spec: {:?}", spec);
        println!("   - message: {}", msg);
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
            println!(
                "   Ability {}: {} (rolls: {:?})",
                i, result.total, result.rolls
            );
        }
    }
}
