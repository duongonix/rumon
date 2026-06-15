//! Hook command executor.

use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::context::HookContext;
use crate::lifecycle::Hook;
use crate::result::HookOutput;

/// Executes configured lifecycle hooks.
#[derive(Clone, Debug)]
pub struct HookExecutor {
    poll_interval: Duration,
}

impl HookExecutor {
    /// Creates a hook executor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            poll_interval: Duration::from_millis(10),
        }
    }

    /// Creates a hook executor with a custom polling interval.
    #[must_use]
    pub const fn with_poll_interval(poll_interval: Duration) -> Self {
        Self { poll_interval }
    }

    /// Runs a hook command and captures output.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the command cannot be spawned or waited on.
    pub fn run(&self, hook: &Hook, context: &HookContext) -> std::io::Result<HookOutput> {
        run_hook_with_poll_interval(hook, context, self.poll_interval)
    }
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs a hook command and captures output.
///
/// # Errors
///
/// Returns an I/O error when the command cannot be spawned or waited on.
pub fn run_hook(hook: &Hook, context: &HookContext) -> std::io::Result<HookOutput> {
    run_hook_with_poll_interval(hook, context, Duration::from_millis(10))
}

fn run_hook_with_poll_interval(
    hook: &Hook,
    context: &HookContext,
    poll_interval: Duration,
) -> std::io::Result<HookOutput> {
    if !hook.is_enabled() {
        return Ok(HookOutput {
            command: hook.command.clone(),
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            timed_out: false,
            elapsed: hook.timeout.min(std::time::Duration::ZERO),
        });
    }

    let started = Instant::now();
    let mut child = shell_command(&hook.command)
        .envs(context.environment())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    loop {
        if child.try_wait()?.is_some() {
            let output = child.wait_with_output()?;
            return Ok(HookOutput {
                command: hook.command.clone(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: false,
                elapsed: started.elapsed(),
            });
        }

        if started.elapsed() >= hook.timeout {
            child.kill()?;
            let output = child.wait_with_output()?;
            return Ok(HookOutput {
                command: hook.command.clone(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: true,
                elapsed: started.elapsed(),
            });
        }

        thread::sleep(poll_interval);
    }
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("cmd");
    process.arg("/C").arg(command);
    process
}

#[cfg(not(windows))]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("sh");
    process.arg("-c").arg(command);
    process
}

#[cfg(test)]
mod tests {
    use super::HookExecutor;
    use crate::{Hook, HookContext, HookPoint};
    use std::time::Duration;

    #[test]
    fn captures_hook_stdout() {
        let hook = Hook::new(HookPoint::BeforeRestart, "echo rumon-hook");
        let context = HookContext::new(HookPoint::BeforeRestart);
        let output = HookExecutor::new()
            .run(&hook, &context)
            .expect("hook should run");

        assert!(output.succeeded());
        assert!(output.stdout.contains("rumon-hook"));
    }

    #[test]
    fn reports_hook_timeout() {
        let mut hook = Hook::new(HookPoint::BeforeRestart, sleep_command());
        hook.timeout = Duration::from_millis(10);
        let context = HookContext::new(HookPoint::BeforeRestart);
        let output = HookExecutor::with_poll_interval(Duration::from_millis(1))
            .run(&hook, &context)
            .expect("hook should run");

        assert!(output.timed_out);
    }

    #[cfg(windows)]
    fn sleep_command() -> &'static str {
        "powershell -NoProfile -Command Start-Sleep -Milliseconds 200"
    }

    #[cfg(not(windows))]
    fn sleep_command() -> &'static str {
        "sleep 0.2"
    }
}
