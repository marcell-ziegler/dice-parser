use crate::{
    ast::{DiceExpr, RollSpec, Keep},
    error::{DiceError, ParseErrorKind},
};

pub struct Parser<'a> {
    input: &'a str,
    byte_pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, byte_pos: 0 }
    }

    /// Peek at the `char` at `byte_pos` with an immutable borrow. `None` if no char under cursor.
    fn peek(&self) -> Option<char> {
        self.input[self.byte_pos..].chars().next()
    }

    /// Consume the current `char`, moving the cursor forward one `char`, and return the consumed
    /// `char`. `None` if no next `char`.
    fn consume(&mut self) -> Option<char> {
        let ch = self.peek()?;
        // Move forward one character in the string, even if that character is more than one byte.
        self.byte_pos += ch.len_utf8();
        Some(ch)
    }

    /// Consume any whitespace until a non-whitespace character is reached.
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.consume();
        }
    }

    fn parse_u32(&mut self) -> Result<u32, DiceError> {
        self.skip_ws();
        let start = self.byte_pos;

        // Bump cursor while input contains valid digits
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.consume();
        }

        // If there was no consumption
        if start == self.byte_pos {
            return Err(DiceError::ParseError {
                kind: ParseErrorKind::ExpectedNumber,
                input: self.input.to_string(),
                start,
                stop: None,
            });
        }

        // Return parsed slice
        self.input[start..self.byte_pos]
            .parse()
            .map_err(|_| DiceError::ParseError {
                kind: ParseErrorKind::InvalidU32,
                input: self.input.to_string(),
                start,
                stop: Some(self.byte_pos),
            })
    }

    fn parse_i32(&mut self) -> Result<i32, DiceError> {
        self.skip_ws();
        let start = self.byte_pos;

        // Check if unary negation is used
        let mut is_negative = false;
        if matches!(self.peek(), Some('-')) {
            is_negative = true;
            self.consume();
        }

        // Consume number
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.consume();
        }

        // If no unary minus, and no consumption
        if !is_negative && (start == self.byte_pos) {
            return Err(DiceError::ParseError {
                kind: ParseErrorKind::ExpectedNumber,
                input: self.input.to_string(),
                start,
                stop: None,
            });
        // If unary minus but no number
        } else if is_negative && (start == self.byte_pos + '-'.len_utf8()) {
            return Err(DiceError::ParseError {
                kind: ParseErrorKind::ExpectedNumber,
                input: self.input.to_string(),
                start,
                stop: Some(self.byte_pos),
            });
        }

        // Return parsed slice
        self.input[start..self.byte_pos]
            .parse()
            .map_err(|_| DiceError::ParseError {
                kind: ParseErrorKind::InvalidI32,
                input: self.input.to_string(),
                start,
                stop: Some(self.byte_pos),
            })
    }

    fn parse_term(&mut self) -> Result<DiceExpr, DiceError> {
        let start = self.byte_pos;
        let count = self.parse_i32()?;
        self.skip_ws();

        if matches!(self.peek(), Some('d')) {
            if count < 0 {
                return Err(DiceError::SyntaxError {
                    input: self.input.to_string(),
                    start,
                    stop: Some(self.byte_pos),
                    expected: Some(String::from("non-negative value for dice count")),
                });
            }

            // Consume the 'd'
            self.consume();

            let sides = self.parse_u32()?;

            self.skip_ws();


            let keep = 
                if matches!(self.peek(), Some('k')) {
                    self.consume();
                    if matches!(self.peek(), Some('h')) {
                        self.skip_ws();
                        self.consume();
                        self.skip_ws();
                        if matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                            Some(Keep::Highest(self.parse_u32()?))
                        }
                        else {
                            Some(Keep::Highest(1))
                        }
                    }
                    else if matches!(self.peek(), Some('l')) {
                        self.skip_ws();
                        self.consume();
                        self.skip_ws();
                        if matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                            Some(Keep::Lowest(self.parse_u32()?))
                        }
                        else {
                            Some(Keep::Lowest(1))
                        }
                    }
                    else {
                        return Err(DiceError::SyntaxError {
                            input: self.input.to_string(),
                            start,
                            stop: Some(self.byte_pos),
                            expected: Some(String::from("'kh' for keep highest or 'kl' for keep lowest")),
                        });
                    }
                }
                else {
                        None
                };

            return Ok(DiceExpr::Roll(RollSpec::new(
                // count is always >= 0 and <= u32::MAX
                count.try_into().unwrap(),
                sides,
                keep,
            )));
        }

        // If no 'd' then return the literal
        Ok(DiceExpr::Literal(count))
    }

    fn parse_expr(&mut self) -> Result<DiceExpr, DiceError> {
        let mut node = self.parse_term()?;

        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.consume();
                    let rhs = self.parse_term()?;
                    node = DiceExpr::Sum(Box::new(node), Box::new(rhs));
                }
                Some('-') => {
                    self.consume();
                    let rhs = self.parse_term()?;
                    node = DiceExpr::Difference(Box::new(node), Box::new(rhs));
                }
                _ => break,
            }
        }

        Ok(node)
    }

    /// Try to parse the input into a `DiceExpr`
    pub fn parse(&mut self) -> Result<DiceExpr, DiceError> {
        let expr = self.parse_expr()?;
        self.skip_ws();

        if self.peek().is_some() {
            return Err(DiceError::TrailingInput {
                input: self.input.to_string(),
                pos: self.byte_pos,
            });
        }
        Ok(expr)
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::DiceExpr, parser::Parser, roller::Roller, ast::Keep};

    #[test]
    fn test_new_parser() {
        let input = "2d6";
        let p = Parser::new(input);
        assert_eq!(p.byte_pos, 0);
        assert_eq!(p.input, input)
    }

    #[test]
    fn test_peek() {
        let p = Parser::new("abc");
        assert_eq!(p.peek().unwrap(), 'a')
    }

    #[test]
    fn test_consume() {
        let mut p = Parser::new("åab");
        assert_eq!(p.peek().unwrap(), 'å');
        assert_eq!(p.consume().unwrap(), 'å');
        assert_eq!(p.peek().unwrap(), 'a');
    }

    #[test]
    fn test_skip_ws() {
        let mut p = Parser::new("a        c");
        assert_eq!(p.peek().unwrap(), 'a');
        p.consume();
        p.skip_ws();
        assert_eq!(p.peek().unwrap(), 'c');
    }

    #[test]
    fn test_parse_u32() {
        let mut p = Parser::new("123");
        let num = p.parse_u32().unwrap();
        assert_eq!(num, 123);
        let mut p = Parser::new("0");
        let num = p.parse_u32().unwrap();
        assert_eq!(num, 0);
        let mut p = Parser::new("4154875165");
        let num = p.parse_u32().unwrap();
        assert_eq!(num, 4154875165);
    }

    #[test]
    fn test_parse_u32_fail() {
        let mut p = Parser::new("abc");
        if p.parse_u32().is_ok() {
            panic!("expected errr")
        };
    }

    #[test]
    fn test_parse_i32() {
        let mut p = Parser::new("123");
        let num = p.parse_i32().unwrap();
        assert_eq!(num, 123);
        let mut p = Parser::new("0");
        let num = p.parse_i32().unwrap();
        assert_eq!(num, 0);
        let mut p = Parser::new("415487516");
        let num = p.parse_i32().unwrap();
        assert_eq!(num, 415487516);

        let mut p = Parser::new("-123");
        let num = p.parse_i32().unwrap();
        assert_eq!(num, -123);
        let mut p = Parser::new("-0");
        let num = p.parse_i32().unwrap();
        assert_eq!(num, 0);
        let mut p = Parser::new("-415487516");
        let num = p.parse_i32().unwrap();
        assert_eq!(num, -415487516);
    }

    #[test]
    fn test_parse_i32_fail() {
        let mut p = Parser::new("abc");
        assert!(p.parse_i32().is_err());

        let mut p = Parser::new("   -");
        assert!(p.parse_i32().is_err());

        let mut p = Parser::new("-abc");
        assert!(p.parse_i32().is_err());
    }

    #[test]
    fn test_parse_term_literal() {
        let mut parser = Parser::new("42");
        let result = parser.parse_term().unwrap();
        if let DiceExpr::Literal(value) = result {
            assert_eq!(value, 42);
        } else {
            panic!("Expected a DiceExpr::Literal, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_term_roll() {
        let mut parser = Parser::new("2d6");
        let result = parser.parse_term().unwrap();
        if let DiceExpr::Roll(roll_spec) = result {
            assert_eq!(roll_spec.count, 2);
            assert_eq!(roll_spec.sides, 6);
            assert_eq!(roll_spec.keep, None);
        } else {
            panic!("Expected a DiceExpr::Roll, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_term_roll_keep_highest() {
        let mut parser = Parser::new("4d6kh2");
        let result = parser.parse_term().unwrap();
        if let DiceExpr::Roll(roll_spec) = result {
            assert_eq!(roll_spec.count, 4);
            assert_eq!(roll_spec.sides, 6);
            assert_eq!(roll_spec.keep, Some(Keep::Highest(2)));
        } else {
            panic!("Expected a DiceExpr::Roll, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_term_roll_keep_highest_default() {
        let mut parser = Parser::new("4d6kh");
        let result = parser.parse_term().unwrap();
        if let DiceExpr::Roll(roll_spec) = result {
            assert_eq!(roll_spec.count, 4);
            assert_eq!(roll_spec.sides, 6);
            assert_eq!(roll_spec.keep, Some(Keep::Highest(1)));
        } else {
            panic!("Expected a DiceExpr::Roll, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_term_roll_keep_lowest() {
        let mut parser = Parser::new("4d6 kl 2");
        let result = parser.parse_term().unwrap();
        if let DiceExpr::Roll(roll_spec) = result {
            assert_eq!(roll_spec.count, 4);
            assert_eq!(roll_spec.sides, 6);
            assert_eq!(roll_spec.keep, Some(Keep::Lowest(2)));
        } else {
            panic!("Expected a DiceExpr::Roll, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_term_roll_keep_lowest_default() {
        let mut parser = Parser::new("4d6kl");
        let result = parser.parse_term().unwrap();
        if let DiceExpr::Roll(roll_spec) = result {
            assert_eq!(roll_spec.count, 4);
            assert_eq!(roll_spec.sides, 6);
            assert_eq!(roll_spec.keep, Some(Keep::Lowest(1)));
        } else {
            panic!("Expected a DiceExpr::Roll, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_term_roll_keep_invalid() {
        let mut parser = Parser::new("4d6kr");
        let result = parser.parse_term();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_term_invalid_dice_count() {
        let mut parser = Parser::new("-2d6");
        let result = parser.parse_term();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_expr_sum() {
        let mut parser = Parser::new("5d6 + 3");
        let result = parser.parse_expr().unwrap();
        if let DiceExpr::Sum(lhs, rhs) = result {
            if let DiceExpr::Roll(roll_spec) = *lhs {
                assert_eq!(roll_spec.count, 5);
                assert_eq!(roll_spec.sides, 6);
            } else {
                panic!("Expected left operand to be DiceExpr::Roll, got {:?}", lhs);
            }

            if let DiceExpr::Literal(value) = *rhs {
                assert_eq!(value, 3);
            } else {
                panic!(
                    "Expected right operand to be DiceExpr::Literal, got {:?}",
                    rhs
                );
            }
        } else {
            panic!("Expected a DiceExpr::Sum, got {:?}", result);
        }
    }

     #[test]
    fn test_parse_expr_diff_highest() {
        let mut parser = Parser::new("5d6kh-3");
        let result = parser.parse_expr().unwrap();
        if let DiceExpr::Difference(lhs, rhs) = result {
            if let DiceExpr::Roll(roll_spec) = *lhs {
                assert_eq!(roll_spec.count, 5);
                assert_eq!(roll_spec.sides, 6);
                assert_eq!(roll_spec.keep, Some(Keep::Highest(1)));
            } else {
                panic!("Expected left operand to be DiceExpr::Roll, got {:?}", lhs);
            }

            if let DiceExpr::Literal(value) = *rhs {
                assert_eq!(value, 3);
            } else {
                panic!(
                    "Expected right operand to be DiceExpr::Literal, got {:?}",
                    rhs
                );
            }
        } else {
            panic!("Expected a DiceExpr::Sum, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_expr_difference() {
        let mut parser = Parser::new("10 - 2d6");
        let result = parser.parse_expr().unwrap();
        if let DiceExpr::Difference(lhs, rhs) = result {
            if let DiceExpr::Literal(value) = *lhs {
                assert_eq!(value, 10);
            } else {
                panic!(
                    "Expected left operand to be DiceExpr::Literal, got {:?}",
                    lhs
                );
            }

            if let DiceExpr::Roll(roll_spec) = *rhs {
                assert_eq!(roll_spec.count, 2);
                assert_eq!(roll_spec.sides, 6);
            } else {
                panic!("Expected right operand to be DiceExpr::Roll, got {:?}", rhs);
            }
        } else {
            panic!("Expected a DiceExpr::Difference, got {:?}", result);
        }
    }

    #[test]
    fn test_assosciativity() {
        let mut parser = Parser::new("7 - 3 + 2 - 2");
        let result = parser.parse_expr().unwrap();
        let mut r = Roller::default();
        let diff = r.roll_expr(&result).unwrap();

        assert_eq!(diff.total, 4)
    }

    #[test]
    fn test_parse_expr_nested() {
        let mut parser = Parser::new("2d6 + 3 - 1");
        let result = parser.parse_expr().unwrap();
        if let DiceExpr::Difference(lhs, rhs) = result {
            if let DiceExpr::Sum(lhs_inner, rhs_inner) = *lhs {
                if let DiceExpr::Roll(roll_spec) = *lhs_inner {
                    assert_eq!(roll_spec.count, 2);
                    assert_eq!(roll_spec.sides, 6);
                } else {
                    panic!(
                        "Expected left inner operand to be DiceExpr::Roll, got {:?}",
                        lhs_inner
                    );
                }

                if let DiceExpr::Literal(value) = *rhs_inner {
                    assert_eq!(value, 3);
                } else {
                    panic!(
                        "Expected right inner operand to be DiceExpr::Literal, got {:?}",
                        rhs_inner
                    );
                }
            } else {
                panic!(
                    "Expected left outer operand to be DiceExpr::Sum, got {:?}",
                    lhs
                );
            }

            if let DiceExpr::Literal(value) = *rhs {
                assert_eq!(value, 1);
            } else {
                panic!(
                    "Expected right outer operand to be DiceExpr::Literal, got {:?}",
                    rhs
                );
            }
        } else {
            panic!("Expected a DiceExpr::Difference, got {:?}", result);
        }
    }
}
