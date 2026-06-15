//! Configuration validation.

use crate::error::{ConfigError, ConfigResult};
use crate::schema::Config;

/// Validates a complete configuration.
///
/// # Errors
///
/// Returns an error when required values are missing, invalid, or refer to unavailable watch paths.
pub fn validate(config: &Config) -> ConfigResult<()> {
    if config.run.cmd.trim().is_empty() {
        return Err(ConfigError::new("run.cmd must not be empty"));
    }

    if config.watch.paths.is_empty() && !config.once {
        return Err(ConfigError::new(
            "watch.paths must contain at least one path",
        ));
    }

    if config.watch.debounce_ms == 0 {
        return Err(ConfigError::new("watch.debounce_ms must be positive"));
    }

    if !(1..=99).contains(&config.tui.left_panel_width) {
        return Err(ConfigError::new(
            "tui.left_panel_width must be between 1 and 99",
        ));
    }

    if !matches!(
        config.run.kill_signal.as_str(),
        "term" | "kill" | "interrupt"
    ) {
        return Err(ConfigError::new(
            "run.kill_signal must be term, kill, or interrupt",
        ));
    }

    for path in &config.watch.paths {
        if !path.exists() && !config.once {
            return Err(ConfigError::new(format!(
                "watch path does not exist: {}",
                path.display()
            )));
        }
    }

    for hook in &config.event_hooks {
        if hook.name.trim().is_empty() {
            return Err(ConfigError::new("event_hooks.name must not be empty"));
        }
        if hook.events.is_empty() {
            return Err(ConfigError::new(format!(
                "event hook '{}' must include at least one event",
                hook.name
            )));
        }
        if hook.paths.is_empty() {
            return Err(ConfigError::new(format!(
                "event hook '{}' must include at least one path glob",
                hook.name
            )));
        }
        for event in &hook.events {
            if !is_supported_event_type(event) {
                return Err(ConfigError::new(format!(
                    "unsupported event hook event type: {event}"
                )));
            }
        }
    }

    Ok(())
}

fn is_supported_event_type(event: &str) -> bool {
    matches!(
        event,
        "file_created"
            | "file_modified"
            | "file_deleted"
            | "file_renamed"
            | "folder_created"
            | "folder_deleted"
            | "folder_renamed"
            | "metadata_changed"
            | "content_changed"
            | "permission_changed"
    )
}

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::Config;

    #[test]
    fn validation_rejects_empty_commands() {
        let mut config = Config::default();
        config.run.cmd.clear();

        assert!(validate(&config).is_err());
    }

    #[test]
    fn validation_rejects_unknown_event_hook_types() {
        let mut config = Config::default();
        config.event_hooks.push(crate::EventHookConfig {
            name: "bad".to_string(),
            events: vec!["nope".to_string()],
            paths: vec!["**/*".to_string()],
            when: String::new(),
            cmd: "echo nope".to_string(),
        });

        assert!(validate(&config).is_err());
    }
}
