//! Shared event types.

use crate::models::{FileChange, LogEntry};

/// Watcher events published to the application event bus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WatchEvent {
    /// A filesystem change was detected.
    Changed(FileChange),
    /// The watcher encountered a recoverable error.
    Error(String),
}

/// Process events published by the runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessEvent {
    /// The configured process is starting.
    Starting,
    /// The configured process started.
    Started,
    /// The configured process is restarting.
    Restarting,
    /// The configured process stopped.
    Stopped,
    /// The configured process exited.
    Exited(Option<i32>),
    /// The configured process failed.
    Failed(String),
    /// A log line was received.
    Log(LogEntry),
}

/// Top-level application event transported by the event bus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppEvent {
    /// A watcher event.
    Watch(WatchEvent),
    /// A runner event.
    Process(ProcessEvent),
    /// A system message.
    System(String),
}
