//! Lifecycle hook models.

use std::time::Duration;

/// Built-in lifecycle hook points.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookPoint {
    /// Runs before the current process is terminated for restart.
    BeforeRestart,
    /// Runs after the replacement process starts.
    AfterRestart,
    /// Runs after a process exits successfully.
    OnSuccess,
    /// Runs after a process exits with failure.
    OnFailure,
}

impl HookPoint {
    /// Returns the configuration key for the hook point.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforeRestart => "before_restart",
            Self::AfterRestart => "after_restart",
            Self::OnSuccess => "on_success",
            Self::OnFailure => "on_failure",
        }
    }
}

/// A configured lifecycle hook.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hook {
    /// Hook point.
    pub point: HookPoint,
    /// Command to execute.
    pub command: String,
    /// Timeout for the hook command.
    pub timeout: Duration,
}

impl Hook {
    /// Creates a hook with the default timeout.
    #[must_use]
    pub fn new(point: HookPoint, command: impl Into<String>) -> Self {
        Self {
            point,
            command: command.into(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Returns whether the hook has a command to run.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        !self.command.trim().is_empty()
    }
}

/// Collection of lifecycle hooks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookSet {
    /// Hook executed before restart.
    pub before_restart: Option<Hook>,
    /// Hook executed after restart.
    pub after_restart: Option<Hook>,
    /// Hook executed on successful process exit.
    pub on_success: Option<Hook>,
    /// Hook executed on failed process exit.
    pub on_failure: Option<Hook>,
}

impl HookSet {
    /// Creates an empty hook set.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            before_restart: None,
            after_restart: None,
            on_success: None,
            on_failure: None,
        }
    }

    /// Returns a hook for the supplied lifecycle point.
    #[must_use]
    pub fn get(&self, point: HookPoint) -> Option<&Hook> {
        match point {
            HookPoint::BeforeRestart => self.before_restart.as_ref(),
            HookPoint::AfterRestart => self.after_restart.as_ref(),
            HookPoint::OnSuccess => self.on_success.as_ref(),
            HookPoint::OnFailure => self.on_failure.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Hook, HookPoint};

    #[test]
    fn hook_keeps_command() {
        let hook = Hook {
            point: HookPoint::BeforeRestart,
            command: "cargo fmt".to_string(),
            timeout: std::time::Duration::from_secs(30),
        };

        assert_eq!(hook.command, "cargo fmt");
    }

    #[test]
    fn hook_point_has_config_key() {
        assert_eq!(HookPoint::AfterRestart.as_str(), "after_restart");
    }
}
