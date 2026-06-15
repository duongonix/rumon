//! Managed child process runner.

use std::process::{Child, ExitStatus};
use std::sync::mpsc::Sender;

use rumon_shared::{AppEvent, ProcessEvent};

use crate::command::{RunnerConfig, spawn_command};
use crate::error::{RunnerResult, to_runner_error};
use crate::logs::attach_output;

/// Managed child process runner.
#[derive(Debug)]
pub struct CommandRunner {
    config: RunnerConfig,
    child: Option<Child>,
    events: Sender<AppEvent>,
}

impl CommandRunner {
    /// Creates a command runner.
    #[must_use]
    pub fn new(config: RunnerConfig, events: Sender<AppEvent>) -> Self {
        Self {
            config,
            child: None,
            events,
        }
    }

    /// Starts the configured process.
    ///
    /// # Errors
    ///
    /// Returns an error when the command cannot be spawned or output cannot be captured.
    pub fn start(&mut self) -> RunnerResult<()> {
        self.publish(ProcessEvent::Starting);
        let mut child = spawn_command(&self.config)?;
        attach_output(&mut child, self.events.clone())?;
        self.child = Some(child);
        self.publish(ProcessEvent::Started);
        Ok(())
    }

    /// Stops the current process if it is running.
    ///
    /// # Errors
    ///
    /// Returns an error when the child status cannot be read or the process cannot be stopped.
    pub fn stop(&mut self) -> RunnerResult<()> {
        let Some(mut child) = self.child.take() else {
            self.publish(ProcessEvent::Stopped);
            return Ok(());
        };

        if child
            .try_wait()
            .map_err(|error| to_runner_error(&error))?
            .is_none()
        {
            child.kill().map_err(|error| to_runner_error(&error))?;
        }
        let status = child.wait().map_err(|error| to_runner_error(&error))?;
        self.publish(ProcessEvent::Exited(status.code()));
        self.publish(ProcessEvent::Stopped);
        Ok(())
    }

    /// Restarts the configured process.
    ///
    /// # Errors
    ///
    /// Returns an error when stopping or starting the child process fails.
    pub fn restart(&mut self) -> RunnerResult<()> {
        self.publish(ProcessEvent::Restarting);
        self.stop()?;
        self.start()
    }

    /// Polls the child process and returns an exit status when it has exited.
    ///
    /// # Errors
    ///
    /// Returns an error when the child process status cannot be read.
    pub fn poll_exit(&mut self) -> RunnerResult<Option<ExitStatus>> {
        let Some(child) = &mut self.child else {
            return Ok(None);
        };

        let status = child.try_wait().map_err(|error| to_runner_error(&error))?;
        if let Some(status) = status {
            self.publish(ProcessEvent::Exited(status.code()));
            self.child = None;
            return Ok(Some(status));
        }

        Ok(None)
    }

    fn publish(&self, event: ProcessEvent) {
        let _ = self.events.send(AppEvent::Process(event));
    }
}

/// Creates an event that marks the user process as starting.
#[must_use]
pub const fn process_starting() -> AppEvent {
    AppEvent::Process(ProcessEvent::Starting)
}

#[cfg(test)]
mod tests {
    use super::process_starting;
    use crate::RunnerConfig;
    use rumon_shared::{AppEvent, ProcessEvent};
    use std::path::PathBuf;

    #[test]
    fn creates_starting_event() {
        assert_eq!(
            process_starting(),
            AppEvent::Process(ProcessEvent::Starting)
        );
    }

    #[test]
    fn runner_config_keeps_command() {
        let config = RunnerConfig {
            command: "cargo run".to_string(),
            cwd: PathBuf::from("."),
            kill_signal: "term".to_string(),
        };

        assert_eq!(config.command, "cargo run");
    }
}
