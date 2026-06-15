//! Centralized application state.

use crate::events::{AppEvent, ProcessEvent, WatchEvent};
use crate::models::{FileChange, LogEntry, LogKind, ProcessState};

/// Central application state owned by `rumon-core`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppState {
    /// Recent file changes.
    pub changes: Vec<FileChange>,
    /// Recent logs.
    pub logs: Vec<LogEntry>,
    /// Current process state.
    pub process: ProcessState,
    /// Number of process restarts requested by Rumon.
    pub restart_count: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            changes: Vec::new(),
            logs: Vec::new(),
            process: ProcessState::Stopped,
            restart_count: 0,
        }
    }
}

impl AppState {
    /// Applies an application event to centralized state.
    pub fn apply(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Watch(WatchEvent::Changed(change)) => self.changes.push(change.clone()),
            AppEvent::Watch(WatchEvent::Error(message)) | AppEvent::System(message) => {
                self.logs
                    .push(LogEntry::new(LogKind::System, message.clone()));
            }
            AppEvent::Process(ProcessEvent::Starting) => self.process = ProcessState::Starting,
            AppEvent::Process(ProcessEvent::Started) => self.process = ProcessState::Running,
            AppEvent::Process(ProcessEvent::Restarting) => {
                self.process = ProcessState::Restarting;
                self.restart_count = self.restart_count.saturating_add(1);
            }
            AppEvent::Process(ProcessEvent::Stopped) => self.process = ProcessState::Stopped,
            AppEvent::Process(ProcessEvent::Exited(Some(0))) => {
                self.process = ProcessState::Stopped;
            }
            AppEvent::Process(ProcessEvent::Exited(_) | ProcessEvent::Failed(_)) => {
                self.process = ProcessState::Failed;
            }
            AppEvent::Process(ProcessEvent::Log(entry)) => self.logs.push(entry.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::{AppEvent, ProcessEvent, ProcessState};

    #[test]
    fn default_state_starts_stopped() {
        let state = AppState::default();

        assert_eq!(state.process, ProcessState::Stopped);
        assert!(state.changes.is_empty());
        assert!(state.logs.is_empty());
    }

    #[test]
    fn process_events_update_state() {
        let mut state = AppState::default();

        state.apply(&AppEvent::Process(ProcessEvent::Started));

        assert_eq!(state.process, ProcessState::Running);
    }

    #[test]
    fn restarting_increments_restart_count() {
        let mut state = AppState::default();

        state.apply(&AppEvent::Process(ProcessEvent::Restarting));

        assert_eq!(state.restart_count, 1);
    }
}
