# dice-parser

A parser and roller for standard RPG dice notation.

## Overview

`dice-parser` is a Rust library for parsing and evaluating dice expressions commonly used in tabletop role-playing games. It provides a simple API for parsing dice notation strings and rolling the dice.

## Features

- **Parse dice notation**: Convert strings like `"2d6 + 3"` into an AST
- **Roll dice**: Evaluate expressions to produce random results
- **Flexible expressions**: Support for addition, subtraction, and literal values
- **Keep mechanics**: Keep highest or lowest dice (e.g., roll 4d6 keep highest 3)
- **Detailed results**: Access individual roll values, modifiers, and totals
- **Custom RNG**: Use your own random number generator or the default

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dice-parser = "0.1.0"
```

## Quick Start

```rust
use dice_parser::Parser;

fn main() {
    // Parse a dice expression
    let mut parser = Parser::new("2d6 + 3");
    let expr = parser.parse().unwrap();
    
    // Roll the dice
    let result = expr.roll().unwrap();
    
    // Print the results
    println!("Total: {}", result.total);
    println!("Rolls: {:?}", result.rolls);
    println!("Modifier: {}", result.modifier);
}
```

## Examples

### Basic Dice Rolling

```rust
use dice_parser::Parser;

let mut parser = Parser::new("1d20");
let expr = parser.parse().unwrap();
let result = expr.roll().unwrap();

println!("You rolled: {}", result.total);
```

### Complex Expressions

```rust
use dice_parser::Parser;

// Parse a damage roll: 2d6 + 1d4 + 2
let mut parser = Parser::new("2d6 + 1d4 + 2");
let expr = parser.parse().unwrap();
let result = expr.roll().unwrap();

println!("Damage: {}", result.total);
println!("Individual rolls: {:?}", result.rolls);
println!("Modifier: {}", result.modifier);
```

### Keep Highest/Lowest

```rust
use dice_parser::{DiceExpr, RollSpec, Keep};

// Roll 4d6, keep highest 3 (common in D&D character creation)
let spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
let expr = DiceExpr::Roll(spec);
let result = expr.roll().unwrap();

println!("Ability score: {}", result.total);
println!("All rolls: {:?}", result.rolls); // Shows all 4 rolls
```

### Custom RNG

```rust
use dice_parser::{Parser, Roller};
use rand::{SeedableRng, rngs::StdRng};

// Use a seeded RNG for reproducible results
let mut parser = Parser::new("2d6");
let expr = parser.parse().unwrap();

let mut roller = Roller::from_rng(StdRng::seed_from_u64(12345));
let result = roller.roll_expr(&expr).unwrap();
```

### Error Handling

```rust
use dice_parser::Parser;

let mut parser = Parser::new("invalid dice notation");
match parser.parse() {
    Ok(expr) => println!("Parsed successfully"),
    Err(e) => eprintln!("Parse error: {}", e),
}
```

## Syntax

The parser supports the following syntax:

- **Dice rolls**: `NdS` (e.g., `2d6`, `1d20`, `3d10`)
- **Literals**: Integer constants (e.g., `5`, `-3`, `0`)
- **Addition**: `expr + expr`
- **Subtraction**: `expr - expr`
- **Whitespace**: Ignored

## API Documentation

For detailed API documentation, run:

```bash
cargo doc --open
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

See the [LICENSE](LICENSE) file for details.

