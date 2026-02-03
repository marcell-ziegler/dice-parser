# dice-parser

A parser and roller for standard RPG dice notation in Rust.

[![Crates.io](https://img.shields.io/crates/v/dice-parser.svg)](https://crates.io/crates/dice-parser)
[![Documentation](https://docs.rs/dice-parser/badge.svg)](https://docs.rs/dice-parser)

This crate provides a simple way to parse and evaluate dice expressions commonly used in tabletop role-playing games. It supports basic arithmetic operations (addition and subtraction) and dice rolling with various modifiers.

## Features

- Parse standard dice notation (e.g., "2d6", "1d20+5", "3d8-2")
- Support for keeping n highest or lowest rolls (currently via manual construction only; parsing support planned for a future release)
- Detailed roll results including individual die rolls and modifiers
- Custom RNG support for deterministic testing
- Comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dice-parser = "0.1"
```

## Quick Start

### Parsing and Rolling

```rust
use dice_parser::DiceExpr;

// Parse a dice expression from a string
let expr = DiceExpr::parse("2d6+3").unwrap();

// Roll the dice
let result = expr.roll().unwrap();
println!("Total: {}", result.total);
println!("Rolls: {:?}", result.rolls);
println!("Modifier: {}", result.modifier);
```

### Manual Construction

You can also manually construct dice expressions for more control:

```rust
use dice_parser::{DiceExpr, RollSpec, Keep};

// Create a "4d6 keep highest 3" roll (common for D&D ability scores)
let roll_spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
let expr = DiceExpr::Roll(roll_spec);

let result = expr.roll().unwrap();
println!("Total: {}", result.total);
```

### Complex Expressions

Build complex expressions by combining multiple operations:

```rust
use dice_parser::{DiceExpr, RollSpec};

// Create "2d6 + 1d4 - 2"
let d2d6 = DiceExpr::Roll(RollSpec::new(2, 6, None));
let d1d4 = DiceExpr::Roll(RollSpec::new(1, 4, None));
let modifier = DiceExpr::Literal(2);

let sum = DiceExpr::Sum(Box::new(d2d6), Box::new(d1d4));
let expr = DiceExpr::Difference(Box::new(sum), Box::new(modifier));

let result = expr.roll().unwrap();
```

### Using Custom RNG

For deterministic testing or custom randomness:

```rust
use dice_parser::DiceExpr;
use rand::{SeedableRng, rngs::StdRng};

let expr = DiceExpr::parse("1d20").unwrap();

// Use a seeded RNG for reproducible results
let rng = StdRng::seed_from_u64(42);
let result = expr.roll_with_rng(rng).unwrap();
```

## Supported Syntax

When parsing from strings, the following syntax is supported:

- **Dice rolls**: `NdS` where N is the number of dice and S is the number of sides
  - Example: `2d6`, `1d20`, `3d8`
  - Note: Both the number of dice, and the number of sides needs to be strictly non-negative.
- **Literals**: Any integer (positive or negative)
  - Example: `5`, `-3`, `100`
- **Addition**: `expr + expr`
  - Example: `2d6 + 3`, `1d20 + 1d4`
- **Subtraction**: `expr - expr`
  - Example: `1d20 - 2`, `10 - 2d6`
- **Whitespace**: Ignored throughout the expression
  - Example: `2d6+3` and `2d6 + 3` are equivalent

**Note:** Keep mechanics (`Keep::Highest` and `Keep::Lowest`) are currently only available through manual construction. Parsing support for keep syntax (e.g., "2d20kh" for keep highest, "6d6kl3" for keep lowest 3) is planned for a future release.

## Examples

### D&D 5e Advantage/Disadvantage

```rust
use dice_parser::{DiceExpr, RollSpec, Keep};

// Advantage: roll 2d20, keep highest
let advantage = DiceExpr::Roll(RollSpec::new(2, 20, Some(Keep::Highest(1))));
let result = advantage.roll().unwrap();

// Disadvantage: roll 2d20, keep lowest
let disadvantage = DiceExpr::Roll(RollSpec::new(2, 20, Some(Keep::Lowest(1))));
let result = disadvantage.roll().unwrap();
```

### Character Ability Scores (4d6 drop lowest)

```rust
use dice_parser::{DiceExpr, RollSpec, Keep};

let ability_roll = DiceExpr::Roll(RollSpec::new(4, 6, Some(Keep::Highest(3))));

// Roll 6 times for all abilities
for i in 0..6 {
    let result = ability_roll.roll().unwrap();
    println!("Ability {}: {}", i + 1, result.total);
}
```

### Error Handling

```rust
use dice_parser::DiceExpr;

// Parse error - invalid syntax
match DiceExpr::parse("2d") {
    Ok(expr) => println!("Parsed successfully"),
    Err(e) => eprintln!("Parse error: {}", e),
}

// Syntax error - negative dice count
match DiceExpr::parse("-2d6") {
    Ok(expr) => println!("Parsed successfully"),
    Err(e) => eprintln!("Syntax error: {}", e),
}
```

## API Cheat-sheet

The public API consists of the following main types:

- **`DiceExpr`**: The main enum representing a dice expression
  - `DiceExpr::parse(input)`: Parse from a string
  - `DiceExpr::roll()`: Roll using default RNG
  - `DiceExpr::roll_with_rng(rng)`: Roll using custom RNG
  - Variants: `Sum`, `Difference`, `Roll`, `Literal`

- **`RollSpec`**: Specification for a dice roll
  - Fields: `count`, `sides`, `keep`
  - `RollSpec::new(count, sides, keep)`: Create a new specification

- **`Keep`**: Keep modifier for roll specifications
  - `Keep::Highest(n)`: Keep N highest dice
  - `Keep::Lowest(n)`: Keep N lowest dice

- **`ExprResult`**: Result of evaluating an expression
  - Fields: `total`, `rolls`, `modifier`

- **`DiceError`**: Error type for all operations
  - Variants: `Overflow`, `InvalidSpec`, `ParseError`, `SyntaxError`, `TrailingInput`

## License

This library is licensed under GNU-GPL v3.

## Contributing

This began as a personal project, but if it was of any use for you and you have suggestions on how to improve it: feel free to reach out on GitHub or submit a pull request!
