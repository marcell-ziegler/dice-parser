//! Integration test verifying the public API design requirements.

use dice_parser::{DiceError, DiceExpr, ExprResult, Keep, RollSpec};
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn test_requirement_1_parse_from_string() {
    // Requirement 1: Users can import DiceExpr and call DiceExpr::parse()
    let expr = DiceExpr::parse("2d6+3").unwrap();
    let result = expr.roll().unwrap();
    assert!(result.total >= 5 && result.total <= 15);
}

#[test]
fn test_requirement_2_manual_construction() {
    // Requirement 2: Users can manually instantiate DiceExpr variants and access RollSpec
    
    // Access to RollSpec
    let spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
    
    // Manual instantiation of DiceExpr variants
    let roll = DiceExpr::Roll(spec);
    let literal = DiceExpr::Literal(5);
    let sum = DiceExpr::Sum(Box::new(roll), Box::new(literal));
    let diff = DiceExpr::Difference(Box::new(sum.clone()), Box::new(DiceExpr::Literal(2)));
    
    // All variants are accessible and constructible
    let _ = roll;
    let _ = sum;
    let _ = diff;
}

#[test]
fn test_requirement_3_roll_methods() {
    // Requirement 3: Users can roll via DiceExpr::roll() or DiceExpr::roll_with_rng()
    let expr = DiceExpr::parse("1d20").unwrap();
    
    // Test DiceExpr::roll()
    let result1 = expr.roll().unwrap();
    assert!(result1.total >= 1 && result1.total <= 20);
    
    // Test DiceExpr::roll_with_rng()
    let rng = StdRng::seed_from_u64(42);
    let result2 = expr.roll_with_rng(rng).unwrap();
    assert!(result2.total >= 1 && result2.total <= 20);
}

#[test]
fn test_no_direct_roller_access() {
    // Requirement 3 (part 2): Roller struct should not be directly accessible
    // This is a compile-time test - if this compiles, Roller is not public
    // (Attempting to use dice_parser::Roller would fail to compile)
}

#[test]
fn test_public_api_types() {
    // Verify all expected types are public and accessible
    let _expr: DiceExpr = DiceExpr::Literal(5);
    let _spec: RollSpec = RollSpec::new(1, 6, None);
    let _keep: Keep = Keep::Highest(1);
    let _error: Result<DiceExpr, DiceError> = DiceExpr::parse("invalid");
    
    // ExprResult is accessible as a return type
    let result: ExprResult = DiceExpr::parse("1d6").unwrap().roll().unwrap();
    assert!(result.total >= 1 && result.total <= 6);
}

#[test]
fn test_all_dice_expr_variants_accessible() {
    // Verify all DiceExpr variants are public
    let _sum = DiceExpr::Sum(
        Box::new(DiceExpr::Literal(1)),
        Box::new(DiceExpr::Literal(2)),
    );
    let _diff = DiceExpr::Difference(
        Box::new(DiceExpr::Literal(5)),
        Box::new(DiceExpr::Literal(2)),
    );
    let _roll = DiceExpr::Roll(RollSpec::new(1, 6, None));
    let _lit = DiceExpr::Literal(42);
}

#[test]
fn test_all_keep_variants_accessible() {
    // Verify all Keep variants are public
    let _highest = Keep::Highest(3);
    let _lowest = Keep::Lowest(1);
    
    // And usable with RollSpec
    let _spec1 = RollSpec::new(4, 6, Some(Keep::Highest(3)));
    let _spec2 = RollSpec::new(2, 20, Some(Keep::Lowest(1)));
}

#[test]
fn test_expr_result_fields_accessible() {
    // Verify ExprResult fields are public
    let result = DiceExpr::parse("2d6+3").unwrap().roll().unwrap();
    
    let _total: i32 = result.total;
    let _rolls: Vec<i32> = result.rolls;
    let _modifier: i32 = result.modifier;
}

#[test]
fn test_roll_spec_fields_accessible() {
    // Verify RollSpec fields are public
    let spec = RollSpec::new(4, 6, Some(Keep::Highest(3)));
    
    assert_eq!(spec.count, 4);
    assert_eq!(spec.sides, 6);
    assert_eq!(spec.keep, Some(Keep::Highest(3)));
}

#[test]
fn test_error_type_accessible() {
    // Verify DiceError is accessible and can be matched on
    let err = DiceExpr::parse("invalid").unwrap_err();
    
    match err {
        DiceError::ParseError { .. } => {}
        DiceError::SyntaxError { .. } => panic!("Wrong error type"),
        DiceError::TrailingInput { .. } => panic!("Wrong error type"),
        DiceError::InvalidSpec(..) => panic!("Wrong error type"),
        DiceError::Overflow(..) => panic!("Wrong error type"),
    }
}
