//! Runtime values for rule evaluation.

/// Runtime value used by the mini rule engine.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Null value.
    Null,
    /// Boolean.
    Bool(bool),
    /// Number.
    Number(f64),
    /// String.
    String(String),
    /// Array.
    Array(Vec<Value>),
}

impl Value {
    /// Converts the value to a boolean.
    #[must_use]
    pub fn truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(value) => *value,
            Self::Number(value) => *value != 0.0,
            Self::String(value) => !value.is_empty(),
            Self::Array(value) => !value.is_empty(),
        }
    }

    /// Returns the value as number when possible.
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the value as string when possible.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }
}
