//! Small expression engine for event hook `when` rules.

pub mod ast;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod value;

pub use eval::{RuleContext, evaluate};
pub use parser::parse_rule;
