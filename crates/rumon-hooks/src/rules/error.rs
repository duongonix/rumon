//! Rule engine errors.

/// Error produced while parsing or evaluating a rule expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleError {
    /// Human-readable error message.
    pub message: String,
}

impl RuleError {
    /// Creates a rule error.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuleError {}
