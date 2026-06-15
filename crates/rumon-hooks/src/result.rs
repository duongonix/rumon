//! Hook execution results.

use std::time::Duration;

/// Hook execution output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookOutput {
    /// Command that was executed.
    pub command: String,
    /// Process exit code when available.
    pub exit_code: Option<i32>,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Whether the hook timed out.
    pub timed_out: bool,
    /// Elapsed execution time.
    pub elapsed: Duration,
}

impl HookOutput {
    /// Returns whether the hook finished successfully.
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out
    }

    /// Returns log lines suitable for Rumon UI/log panels.
    #[must_use]
    pub fn log_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("hook: {}", self.command)];
        if !self.stdout.is_empty() {
            lines.push(format!("stdout: {}", self.stdout.trim()));
        }
        if !self.stderr.is_empty() {
            lines.push(format!("stderr: {}", self.stderr.trim()));
        }
        if self.timed_out {
            lines.push("hook timed out".to_string());
        } else {
            lines.push(format!("exit code: {:?}", self.exit_code));
        }
        lines
    }
}
