//! Rule-based event hook configuration.

/// Rule that runs a command when a filesystem event and path glob match.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventHookConfig {
    /// Human-readable hook name.
    pub name: String,
    /// Supported event type strings.
    pub events: Vec<String>,
    /// Glob paths matched against the changed path.
    pub paths: Vec<String>,
    /// Optional rule expression evaluated after event and path matching.
    pub when: String,
    /// Shell command executed when the rule matches.
    pub cmd: String,
}
