//! Rule expression parser.

use crate::rules::ast::{BinaryOp, Expr, UnaryOp};
use crate::rules::error::RuleError;
use crate::rules::lexer::{Token, lex};
use crate::rules::value::Value;

/// Parses a rule expression.
///
/// # Errors
///
/// Returns an error when expression syntax is invalid.
pub fn parse_rule(input: &str) -> Result<Expr, RuleError> {
    let mut parser = Parser {
        tokens: lex(input)?,
        position: 0,
    };
    let expr = parser.parse_expression(0)?;
    parser.expect_eof()?;
    Ok(expr)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn parse_expression(&mut self, min_precedence: u8) -> Result<Expr, RuleError> {
        let mut left = self.parse_unary()?;
        while let Some((op, precedence)) = self.current_binary_op() {
            if precedence < min_precedence {
                break;
            }
            self.advance();
            let right = self.parse_expression(precedence + 1)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, RuleError> {
        match self.current() {
            Token::Symbol(value) if value == "!" => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Token::Symbol(value) if value == "-" => {
                self.advance();
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, RuleError> {
        match self.current().clone() {
            Token::Ident(value) => {
                self.advance();
                Ok(Expr::Variable(value))
            }
            Token::String(value) => {
                self.advance();
                Ok(Expr::Literal(Value::String(value)))
            }
            Token::Number(value) => {
                self.advance();
                Ok(Expr::Literal(Value::Number(value)))
            }
            Token::Bool(value) => {
                self.advance();
                Ok(Expr::Literal(Value::Bool(value)))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Literal(Value::Null))
            }
            Token::Symbol(value) if value == "(" => {
                self.advance();
                let expr = self.parse_expression(0)?;
                self.expect_symbol(")")?;
                Ok(expr)
            }
            Token::Symbol(value) if value == "[" => self.parse_array(),
            token => Err(RuleError::new(format!(
                "expected expression, got {token:?}"
            ))),
        }
    }

    fn parse_array(&mut self) -> Result<Expr, RuleError> {
        self.expect_symbol("[")?;
        let mut items = Vec::new();
        if self.current_symbol("]") {
            self.advance();
            return Ok(Expr::Array(items));
        }
        loop {
            items.push(self.parse_expression(0)?);
            if self.current_symbol("]") {
                self.advance();
                break;
            }
            self.expect_symbol(",")?;
        }
        Ok(Expr::Array(items))
    }

    fn current_binary_op(&self) -> Option<(BinaryOp, u8)> {
        let Token::Symbol(value) = self.current() else {
            return None;
        };
        Some(match value.as_str() {
            "||" => (BinaryOp::Or, 1),
            "&&" => (BinaryOp::And, 2),
            "==" => (BinaryOp::Eq, 3),
            "!=" => (BinaryOp::Ne, 3),
            ">" => (BinaryOp::Gt, 3),
            ">=" => (BinaryOp::Ge, 3),
            "<" => (BinaryOp::Lt, 3),
            "<=" => (BinaryOp::Le, 3),
            "in" => (BinaryOp::In, 3),
            "contains" => (BinaryOp::Contains, 3),
            "starts_with" => (BinaryOp::StartsWith, 3),
            "ends_with" => (BinaryOp::EndsWith, 3),
            "matches" => (BinaryOp::Matches, 3),
            "+" => (BinaryOp::Add, 4),
            "-" => (BinaryOp::Sub, 4),
            "*" => (BinaryOp::Mul, 5),
            "/" => (BinaryOp::Div, 5),
            _ => return None,
        })
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    fn current_symbol(&self, expected: &str) -> bool {
        matches!(self.current(), Token::Symbol(value) if value == expected)
    }

    fn expect_symbol(&mut self, expected: &str) -> Result<(), RuleError> {
        if self.current_symbol(expected) {
            self.advance();
            Ok(())
        } else {
            Err(RuleError::new(format!("expected '{expected}'")))
        }
    }

    fn expect_eof(&self) -> Result<(), RuleError> {
        if matches!(self.current(), Token::Eof) {
            Ok(())
        } else {
            Err(RuleError::new("unexpected trailing tokens"))
        }
    }

    fn advance(&mut self) {
        self.position = self.position.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_rule;

    #[test]
    fn parses_arithmetic_and_boolean_expression() {
        let expr = parse_rule(r#"diff.added_lines + diff.removed_lines >= 30 && file.ext == "rs""#);

        assert!(expr.is_ok());
    }

    #[test]
    fn rejects_bad_syntax() {
        assert!(parse_rule("file.ext ==").is_err());
    }
}
