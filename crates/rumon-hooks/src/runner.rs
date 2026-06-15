//! Rule-based event hook command runner.

use std::collections::BTreeMap;
use std::process::{Command, Stdio};
use std::time::Instant;

use rumon_config::EventHookConfig;
use rumon_shared::{ChangeDetail, FileChange, display_path};

use crate::events::EventType;
use crate::matcher::{EventHookDecision, EventHookDiagnostic, EventHookMatcher};
use crate::result::HookOutput;

/// Output from a matched event hook execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHookExecution {
    /// Hook name.
    pub name: String,
    /// Event type exported to the hook.
    pub event_type: EventType,
    /// Captured command output.
    pub output: HookOutput,
}

/// Matches and executes rule-based event hooks.
#[derive(Clone, Debug)]
pub struct EventHookRunner {
    matcher: EventHookMatcher,
    profile: Option<String>,
}

impl EventHookRunner {
    /// Creates an event hook runner from config.
    ///
    /// Creates an event hook runner from config.
    #[must_use]
    pub fn new(hooks: &[EventHookConfig], profile: Option<String>) -> Self {
        Self {
            matcher: EventHookMatcher::new(hooks),
            profile,
        }
    }

    /// Returns hook rule diagnostics collected during compilation.
    #[must_use]
    pub fn diagnostics(&self) -> &[EventHookDiagnostic] {
        self.matcher.diagnostics()
    }

    /// Returns verbose match decisions for a filesystem change.
    #[must_use]
    pub fn decisions(&self, change: &FileChange) -> Vec<EventHookDecision> {
        self.matcher.decisions(change)
    }

    /// Runs every matching non-empty hook for a filesystem change.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when a hook command cannot be spawned or collected.
    pub fn run_matching(&self, change: &FileChange) -> std::io::Result<Vec<EventHookExecution>> {
        let mut outputs = Vec::new();
        for matched in self.matcher.matches(change) {
            if matched.config.cmd.trim().is_empty() {
                continue;
            }
            let started = Instant::now();
            let output = shell_command(&matched.config.cmd)
                .envs(environment(
                    change,
                    matched.event_type,
                    &matched.config.name,
                    self.profile.as_deref(),
                ))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            outputs.push(EventHookExecution {
                name: matched.config.name,
                event_type: matched.event_type,
                output: HookOutput {
                    command: matched.config.cmd,
                    exit_code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    timed_out: false,
                    elapsed: started.elapsed(),
                },
            });
        }
        Ok(outputs)
    }
}

fn environment(
    change: &FileChange,
    event_type: EventType,
    hook_name: &str,
    profile: Option<&str>,
) -> BTreeMap<String, String> {
    let (added, removed) = diff_stats(change);
    let mut variables = BTreeMap::new();
    variables.insert(
        "RUMON_EVENT_TYPE".to_string(),
        event_type.as_str().to_string(),
    );
    variables.insert("RUMON_PATH".to_string(), normalize_path(&change.path));
    variables.insert(
        "RUMON_OLD_PATH".to_string(),
        change
            .previous_path
            .as_ref()
            .map_or_else(String::new, normalize_path),
    );
    variables.insert(
        "RUMON_NEW_PATH".to_string(),
        if change.previous_path.is_some() {
            normalize_path(&change.path)
        } else {
            String::new()
        },
    );
    variables.insert("RUMON_ADDED_LINES".to_string(), added.to_string());
    variables.insert("RUMON_REMOVED_LINES".to_string(), removed.to_string());
    variables.insert(
        "RUMON_PROFILE".to_string(),
        profile.unwrap_or_default().to_string(),
    );
    variables.insert("RUMON_HOOK_NAME".to_string(), hook_name.to_string());
    variables
}

fn diff_stats(change: &FileChange) -> (usize, usize) {
    let Some(ChangeDetail::Text { preview, .. }) = &change.detail else {
        return (0, 0);
    };
    let added = preview
        .iter()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .count();
    let removed = preview
        .iter()
        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
        .count();
    (added, removed)
}

fn normalize_path(path: impl AsRef<std::path::Path>) -> String {
    display_path(path.as_ref())
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
    use std::path::PathBuf;

    use rumon_config::EventHookConfig;
    use rumon_shared::{ChangeDetail, ChangeKind, FileChange};

    use super::{EventHookRunner, environment};
    use crate::EventType;

    #[test]
    fn environment_exports_rename_paths() {
        let change = FileChange {
            path: PathBuf::from("src/new.rs"),
            previous_path: Some(PathBuf::from("src/old.rs")),
            kind: ChangeKind::Renamed,
            is_directory: false,
            detail: None,
        };
        let env = environment(&change, EventType::FileRenamed, "rename", Some("rust"));

        assert_eq!(
            env.get("RUMON_EVENT_TYPE"),
            Some(&"file_renamed".to_string())
        );
        assert_eq!(env.get("RUMON_OLD_PATH"), Some(&"src/old.rs".to_string()));
        assert_eq!(env.get("RUMON_NEW_PATH"), Some(&"src/new.rs".to_string()));
        assert_eq!(env.get("RUMON_PROFILE"), Some(&"rust".to_string()));
    }

    #[test]
    fn skips_empty_commands() {
        let runner = EventHookRunner::new(
            &[EventHookConfig {
                name: "skip".to_string(),
                events: vec!["file_modified".to_string()],
                paths: vec!["**/*".to_string()],
                when: String::new(),
                cmd: String::new(),
            }],
            None,
        );
        let change = FileChange {
            path: PathBuf::from("src/main.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: Some(ChangeDetail::Text {
                location: None,
                preview: vec!["+ added".to_string(), "- removed".to_string()],
                truncated: false,
            }),
        };

        let outputs = runner.run_matching(&change).expect("run hooks");

        assert!(outputs.is_empty());
    }
}
