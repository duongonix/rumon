//! Rule expression AST.

use crate::rules::value::Value;

/// Expression node.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// Literal value.
    Literal(Value),
    /// Variable path such as `file.ext`.
    Variable(String),
    /// Unary expression.
    Unary {
        /// Operator.
        op: UnaryOp,
        /// Inner expression.
        expr: Box<Expr>,
    },
    /// Binary expression.
    Binary {
        /// Left expression.
        left: Box<Expr>,
        /// Operator.
        op: BinaryOp,
        /// Right expression.
        right: Box<Expr>,
    },
    /// Array literal.
    Array(Vec<Expr>),
}

/// Unary operators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    /// Logical not.
    Not,
    /// Numeric negation.
    Neg,
}

/// Binary operators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    /// Equality.
    Eq,
    /// Inequality.
    Ne,
    /// Greater than.
    Gt,
    /// Greater or equal.
    Ge,
    /// Less than.
    Lt,
    /// Less or equal.
    Le,
    /// Logical and.
    And,
    /// Logical or.
    Or,
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
    /// Membership.
    In,
    /// String contains.
    Contains,
    /// String prefix.
    StartsWith,
    /// String suffix.
    EndsWith,
    /// Regex match.
    Matches,
}
