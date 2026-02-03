//! Example demonstrating error handling.

use dice_parser::{DiceExpr, DiceError};

fn main() {
    println!("=== Error Handling Examples ===\n");

    // Parse error - invalid syntax
    println!("1. Parse error (missing sides):");
    match DiceExpr::parse("2d") {
        Ok(_) => println!("  Unexpectedly succeeded!"),
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Syntax error - negative dice count
    println!("2. Syntax error (negative dice count):");
    match DiceExpr::parse("-2d6") {
        Ok(_) => println!("  Unexpectedly succeeded!"),
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Trailing input error
    println!("3. Trailing input error:");
    match DiceExpr::parse("2d6 extra") {
        Ok(_) => println!("  Unexpectedly succeeded!"),
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Invalid spec error (keep more than rolled)
    println!("4. Invalid roll specification:");
    use dice_parser::{RollSpec, Keep};
    let spec = RollSpec::new(2, 6, Some(Keep::Highest(5))); // Try to keep 5 dice when only rolling 2
    let expr = DiceExpr::Roll(spec);
    match expr.roll() {
        Ok(_) => println!("  Unexpectedly succeeded!"),
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Pattern matching on error types
    println!("5. Pattern matching on DiceError:");
    let result = DiceExpr::parse("abc");
    match result {
        Ok(_) => println!("  Unexpectedly succeeded!"),
        Err(DiceError::ParseError { kind, input, start, stop: _ }) => {
            println!("  Parse error at position {}: {:?}", start, kind);
            println!("  Input: {}", input);
        }
        Err(e) => println!("  Other error: {}", e),
    }
}
