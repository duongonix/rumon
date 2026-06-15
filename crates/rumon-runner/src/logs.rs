//! Output capture helpers.

use std::io::{BufRead, BufReader};
use std::process::Child;
use std::sync::mpsc::Sender;
use std::thread;

use rumon_shared::{AppEvent, LogEntry, LogKind, ProcessEvent};

use crate::command::{RunnerConfig, spawn_command};
use crate::error::{RunnerError, RunnerResult, to_runner_error};

/// Attaches output streams to the event bus.
pub(crate) fn attach_output(child: &mut Child, events: Sender<AppEvent>) -> RunnerResult<()> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RunnerError::new("failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RunnerError::new("failed to capture stderr"))?;

    spawn_log_thread(stdout, LogKind::Stdout, events.clone());
    spawn_log_thread(stderr, LogKind::Stderr, events);
    Ok(())
}

/// Runs a command once and returns its process exit code.
///
/// # Errors
///
/// Returns an error when the command cannot be spawned or waited on.
pub fn run_once(config: &RunnerConfig) -> RunnerResult<i32> {
    let mut child = spawn_command(config)?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(stdout) = stdout {
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                println!("{line}");
            }
        });
    }

    if let Some(stderr) = stderr {
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                eprintln!("{line}");
            }
        });
    }

    let status = child.wait().map_err(|error| to_runner_error(&error))?;
    Ok(status.code().unwrap_or(1))
}

fn spawn_log_thread(
    reader: impl std::io::Read + Send + 'static,
    kind: LogKind,
    events: Sender<AppEvent>,
) {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let entry = LogEntry::new(kind.clone(), line);
            let _ = events.send(AppEvent::Process(ProcessEvent::Log(entry)));
        }
    });
}
