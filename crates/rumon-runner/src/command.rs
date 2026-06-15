//! Command spawning helpers.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use crate::error::{RunnerError, RunnerResult};

/// Runtime command settings used by the runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerConfig {
    /// Command string to execute.
    pub command: String,
    /// Working directory for the command.
    pub cwd: PathBuf,
    /// Kill strategy requested by configuration.
    pub kill_signal: String,
}

/// Spawns a shell command with captured output.
pub(crate) fn spawn_command(config: &RunnerConfig) -> RunnerResult<Child> {
    let mut command = shell_command(&config.command);
    command
        .current_dir(&config.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().map_err(|error| {
        RunnerError::new(format!(
            "failed to spawn `{}` in {}: {error}",
            config.command,
            config.cwd.display()
        ))
    })
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
