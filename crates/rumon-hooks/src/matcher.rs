//! Event hook matching against event types and path globs.

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use rumon_config::EventHookConfig;
use rumon_shared::{FileChange, display_path};

use crate::events::EventType;
use crate::rules::ast::Expr;
use crate::rules::{RuleContext, evaluate, parse_rule};

/// A hook configuration diagnostic that should be logged instead of crashing Rumon.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHookDiagnostic {
    /// Hook name.
    pub hook_name: String,
    /// Diagnostic message.
    pub message: String,
}

/// A hook that matched a filesystem event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchedEventHook {
    /// Original hook configuration.
    pub config: EventHookConfig,
    /// Primary event type exported in the hook environment.
    pub event_type: EventType,
}

/// Match decision used for verbose diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHookDecision {
    /// Hook name.
    pub hook_name: String,
    /// Whether the hook matched.
    pub matched: bool,
    /// Human-readable reason.
    pub reason: String,
}

/// Matches rule-based event hooks.
#[derive(Clone, Debug)]
pub struct EventHookMatcher {
    rules: Vec<CompiledEventHook>,
    diagnostics: Vec<EventHookDiagnostic>,
}

impl EventHookMatcher {
    /// Compiles configured hook glob and rule expressions.
    #[must_use]
    pub fn new(configs: &[EventHookConfig]) -> Self {
        let mut rules = Vec::new();
        let mut diagnostics = Vec::new();
        for config in configs {
            match CompiledEventHook::new(config.clone()) {
                Ok(rule) => rules.push(rule),
                Err(message) => diagnostics.push(EventHookDiagnostic {
                    hook_name: config.name.clone(),
                    message,
                }),
            }
        }
        Self { rules, diagnostics }
    }

    /// Returns diagnostics found while compiling hook rules.
    #[must_use]
    pub fn diagnostics(&self) -> &[EventHookDiagnostic] {
        &self.diagnostics
    }

    /// Returns all hooks matching the supplied change in config order.
    #[must_use]
    pub fn matches(&self, change: &FileChange) -> Vec<MatchedEventHook> {
        let matching_types = EventType::matching_types(change);
        self.rules
            .iter()
            .filter_map(|rule| {
                rule.matched_event_type(change, &matching_types)
                    .map(|event_type| (rule, event_type))
            })
            .map(|rule| MatchedEventHook {
                config: rule.0.config.clone(),
                event_type: rule.1,
            })
            .collect()
    }

    /// Returns verbose match decisions for all compiled rules.
    #[must_use]
    pub fn decisions(&self, change: &FileChange) -> Vec<EventHookDecision> {
        let matching_types = EventType::matching_types(change);
        self.rules
            .iter()
            .map(|rule| rule.decision(change, &matching_types))
            .collect()
    }
}

#[derive(Clone, Debug)]
struct CompiledEventHook {
    config: EventHookConfig,
    events: Vec<EventType>,
    paths: GlobSet,
    when: Option<Expr>,
}

impl CompiledEventHook {
    fn new(config: EventHookConfig) -> Result<Self, String> {
        let mut builder = GlobSetBuilder::new();
        for path in &config.paths {
            builder.add(
                Glob::new(&normalize_glob(path))
                    .map_err(|error| format!("invalid glob '{path}': {error}"))?,
            );
        }
        let when = if config.when.trim().is_empty() {
            None
        } else {
            Some(
                parse_rule(&config.when)
                    .map_err(|error| format!("invalid when expression: {error}"))?,
            )
        };
        Ok(Self {
            events: config
                .events
                .iter()
                .filter_map(|event| EventType::parse(event))
                .collect(),
            paths: builder
                .build()
                .map_err(|error| format!("invalid glob set: {error}"))?,
            when,
            config,
        })
    }

    fn matches(&self, change: &FileChange, matching_types: &[EventType]) -> bool {
        self.decision(change, matching_types).matched
    }

    fn matched_event_type(
        &self,
        change: &FileChange,
        matching_types: &[EventType],
    ) -> Option<EventType> {
        if !self.matches(change, matching_types) {
            return None;
        }
        self.events
            .iter()
            .copied()
            .find(|event| matching_types.contains(event))
    }

    fn decision(&self, change: &FileChange, matching_types: &[EventType]) -> EventHookDecision {
        let event_matches = self
            .events
            .iter()
            .any(|event| matching_types.contains(event));
        let path_matches = path_matches(&self.paths, &change.path)
            || change
                .previous_path
                .as_deref()
                .is_some_and(|path| path_matches(&self.paths, path));
        let when_matches = self.when.as_ref().is_none_or(|expr| {
            let event_type = EventType::from_change(change);
            evaluate(expr, &RuleContext::new(change.clone(), event_type)).unwrap_or(false)
        });
        let matched = event_matches && path_matches && when_matches;
        let reason = if matched {
            "matched".to_string()
        } else if !event_matches {
            "event did not match".to_string()
        } else if !path_matches {
            "path did not match".to_string()
        } else {
            "when evaluated false".to_string()
        };
        EventHookDecision {
            hook_name: self.config.name.clone(),
            matched,
            reason,
        }
    }
}

fn path_matches(globs: &GlobSet, path: &Path) -> bool {
    globs.is_match(normalize_path(path))
}

fn normalize_path(path: &Path) -> String {
    display_path(path)
}

fn normalize_glob(value: &str) -> String {
    value.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rumon_config::EventHookConfig;
    use rumon_shared::{ChangeKind, FileChange};

    use super::EventHookMatcher;

    #[test]
    fn matches_hook_by_event_and_glob() {
        let matcher = EventHookMatcher::new(&[EventHookConfig {
            name: "rust".to_string(),
            events: vec!["file_modified".to_string()],
            paths: vec!["src/**/*.rs".to_string()],
            when: String::new(),
            cmd: "cargo check".to_string(),
        }]);
        let change = FileChange {
            path: PathBuf::from("src/bin/main.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: None,
        };

        assert_eq!(matcher.matches(&change).len(), 1);
    }

    #[test]
    fn absolute_paths_under_current_dir_match_relative_globs() {
        let matcher = EventHookMatcher::new(&[EventHookConfig {
            name: "workspace rust".to_string(),
            events: vec!["file_modified".to_string()],
            paths: vec!["crates/**/*.rs".to_string()],
            when: String::new(),
            cmd: "cargo check".to_string(),
        }]);
        let change = FileChange {
            path: std::env::current_dir()
                .expect("cwd")
                .join("crates")
                .join("rumon-core")
                .join("src")
                .join("lib.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: None,
        };

        assert_eq!(matcher.matches(&change).len(), 1);
    }

    #[test]
    fn double_star_matches_files_directly_under_root() {
        let matcher = EventHookMatcher::new(&[EventHookConfig {
            name: "rust".to_string(),
            events: vec!["file_modified".to_string()],
            paths: vec!["src/**/*.rs".to_string()],
            when: String::new(),
            cmd: "cargo check".to_string(),
        }]);
        let change = FileChange {
            path: PathBuf::from("src/main.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: None,
        };

        assert_eq!(matcher.matches(&change).len(), 1);
    }

    #[test]
    fn rename_matches_old_or_new_path() {
        let matcher = EventHookMatcher::new(&[EventHookConfig {
            name: "rename".to_string(),
            events: vec!["file_renamed".to_string()],
            paths: vec!["old/**/*.rs".to_string()],
            when: String::new(),
            cmd: "echo renamed".to_string(),
        }]);
        let change = FileChange {
            path: PathBuf::from("new/main.rs"),
            previous_path: Some(PathBuf::from("old/main.rs")),
            kind: ChangeKind::Renamed,
            is_directory: false,
            detail: None,
        };

        assert_eq!(matcher.matches(&change).len(), 1);
    }

    #[test]
    fn when_expression_filters_matches() {
        let matcher = EventHookMatcher::new(&[EventHookConfig {
            name: "large".to_string(),
            events: vec!["file_modified".to_string()],
            paths: vec!["src/**/*.rs".to_string()],
            when: r#"file.ext == "rs" && diff.added_lines >= 2"#.to_string(),
            cmd: "cargo test".to_string(),
        }]);
        let change = FileChange {
            path: PathBuf::from("src/main.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: Some(rumon_shared::ChangeDetail::Text {
                location: None,
                preview: vec!["+ one".to_string(), "+ two".to_string()],
                truncated: false,
            }),
        };

        assert_eq!(matcher.matches(&change).len(), 1);
    }

    #[test]
    fn invalid_when_expression_becomes_diagnostic() {
        let matcher = EventHookMatcher::new(&[EventHookConfig {
            name: "bad".to_string(),
            events: vec!["file_modified".to_string()],
            paths: vec!["src/**/*.rs".to_string()],
            when: "file.ext ==".to_string(),
            cmd: "cargo test".to_string(),
        }]);

        assert_eq!(
            matcher
                .matches(&FileChange {
                    path: PathBuf::from("src/main.rs"),
                    previous_path: None,
                    kind: ChangeKind::Modified,
                    is_directory: false,
                    detail: None,
                })
                .len(),
            0
        );
        assert_eq!(matcher.diagnostics().len(), 1);
    }
}
