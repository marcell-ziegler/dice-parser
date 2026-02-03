//! Example demonstrating custom RNG usage for deterministic testing.

use dice_parser::DiceExpr;
use rand::{rngs::StdRng, SeedableRng};

fn main() {
    println!("=== Deterministic Rolling with Custom RNG ===\n");

    let expr = DiceExpr::parse("2d6+3").unwrap();

    println!("Rolling the same expression with the same seed produces identical results:\n");

    // First roll with seed 42
    let rng1 = StdRng::seed_from_u64(42);
    let result1 = expr.roll_with_rng(rng1).unwrap();
    println!("First roll (seed 42):");
    println!("  Total: {}", result1.total);
    println!("  Rolls: {:?}", result1.rolls);

    // Second roll with the same seed should give the same result
    let rng2 = StdRng::seed_from_u64(42);
    let result2 = expr.roll_with_rng(rng2).unwrap();
    println!("\nSecond roll (seed 42):");
    println!("  Total: {}", result2.total);
    println!("  Rolls: {:?}", result2.rolls);

    assert_eq!(result1.total, result2.total);
    assert_eq!(result1.rolls, result2.rolls);
    println!("\n✓ Results match as expected!");

    // Different seed produces different results
    let rng3 = StdRng::seed_from_u64(123);
    let result3 = expr.roll_with_rng(rng3).unwrap();
    println!("\nThird roll (seed 123):");
    println!("  Total: {}", result3.total);
    println!("  Rolls: {:?}", result3.rolls);
    println!("\n✓ Different seed produces different results (usually)");
}
