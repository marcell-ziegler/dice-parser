//! Parser for dice notation strings.
//!
//! This module provides the [`Parser`] type for converting dice notation strings
//! into [`DiceExpr`](crate::DiceExpr) AST nodes.
//!
//! # Supported Syntax
//!
//! - Dice rolls: `NdS` (e.g., `2d6`, `1d20`)
//! - Literals: Any integer (e.g., `5`, `-3`)
//! - Addition: `expr + expr`
//! - Subtraction: `expr - expr`
//!
//! # Examples
//!
//! ```
//! use dice_parser::Parser;
//!
//! let mut parser = Parser::new("2d6 + 3");
//! let expr = parser.parse().unwrap();
//! ```

use crate::{
    ast::{DiceExpr, RollSpec},
    error::{DiceError, ParseErrorKind},
};

/// A parser for dice notation strings.
///
/// The parser converts string input like `"2d6 + 3"` into a [`DiceExpr`](crate::DiceExpr) AST
/// that can be evaluated.
///
/// # Examples
///
/// ```
/// use dice_parser::Parser;
///
/// let mut parser = Parser::new("2d6 + 3");
/// let expr = parser.parse().unwrap();
/// let result = expr.roll().unwrap();
/// ```
///
/// # Syntax
///
/// The parser recognizes:
/// - **Dice rolls**: `NdS` where N is the count and S is the number of sides (e.g., `2d6`)
/// - **Literals**: Integer constants (e.g., `5`, `-3`)
/// - **Addition**: `expr + expr`
/// - **Subtraction**: `expr - expr`
/// - **Whitespace**: Ignored anywhere in the input
///
/// Operator precedence: Dice rolls and literals are parsed first, then left-to-right
/// evaluation of addition and subtraction (same precedence).
pub struct Parser<'a> {
    input: &'a str,
    byte_pos: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input string.
    ///
    /// # Examples
    ///
    /// ```
    /// use dice_parser::Parser;
    ///
    /// let parser = Parser::new("1d20 + 5");
    /// ```
    pub fn new(input: &'a str) -> Self {
        Self { input, byte_pos: 0 }
    }

    /// Peek at the `char` at `byte_pos` with an immutable borrow. `None` if no char under cursor.
    fn peek(&self) -> Option<char> {
        self.input[self.byte_pos..].chars().next()
    }

    /// Peek at the `char` two characters down.
    fn lookahead(&self) -> Option<char> {
        self.input[self.byte_pos..].chars().nth(1)
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

            return Ok(DiceExpr::Roll(RollSpec::new(
                // count is always >= 0 and <= u32::MAX
                count.try_into().unwrap(),
                sides,
                None,
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

    /// Parse the input string into a `DiceExpr`.
    ///
    /// This is the main entry point for parsing. It consumes the entire input and returns
    /// the resulting expression AST.
    ///
    /// # Examples
    ///
    /// ```
    /// use dice_parser::Parser;
    ///
    /// // Simple dice roll
    /// let mut parser = Parser::new("2d6");
    /// let expr = parser.parse().unwrap();
    /// ```
    ///
    /// ```
    /// use dice_parser::Parser;
    ///
    /// // Complex expression
    /// let mut parser = Parser::new("1d20 + 1d6 + 5");
    /// let expr = parser.parse().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`DiceError`](crate::DiceError) if:
    /// - The input contains invalid syntax
    /// - A number literal is out of range for i32/u32
    /// - There is trailing input after a valid expression
    ///
    /// ```
    /// use dice_parser::Parser;
    ///
    /// let mut parser = Parser::new("2d6 invalid");
    /// assert!(parser.parse().is_err());
    /// ```
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
    use crate::parser::Parser;

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
}
