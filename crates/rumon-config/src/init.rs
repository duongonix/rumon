//! Project configuration initialization.

use std::fs;
use std::path::{Path, PathBuf};

use crate::{ConfigError, ConfigResult};

/// Default Rumon configuration filename.
pub const DEFAULT_CONFIG_FILE: &str = "rumon.toml";

/// Basic starter configuration written by `rumon init`.
pub const BASIC_CONFIG_TEMPLATE: &str = r#"version = 1

[watch]
paths = ["."]
ignore = [".git", "target", "node_modules", "dist"]
extensions = []
debounce_ms = 500
recursive = true
follow_symlink = false

[run]
cmd = "cargo run"
cwd = "."
restart = true
restart_delay_ms = 300
kill_signal = "term"
clear_logs_on_restart = false

[tui]
left_panel_width = 50
show_timestamp = true
show_file_icons = true
auto_scroll_logs = true
max_log_lines = 5000

[api]
enabled = false
host = "127.0.0.1"
port = 3717
transport = ["http", "sse", "ws", "ipc"]
event_format = "json"
max_event_buffer = 1000

[ipc]
enabled = false
name = "rumon"
path = ""

# Example rule-based hook:
# [[event_hooks]]
# name = "rust source changed"
# events = ["file_created", "file_modified", "file_deleted"]
# paths = ["src/**/*.rs", "crates/**/*.rs"]
# when = "diff.added_lines + diff.removed_lines > 0"
# cmd = "cargo check"
"#;

/// Result of creating an initial configuration file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitConfigResult {
    /// Path to the config file.
    pub path: PathBuf,
    /// Whether a new file was written.
    pub created: bool,
}

/// Creates `rumon.toml` in `directory` unless it already exists.
///
/// # Errors
///
/// Returns an error when the directory cannot be read or the file cannot be written.
pub fn init_config(directory: &Path) -> ConfigResult<InitConfigResult> {
    let path = directory.join(DEFAULT_CONFIG_FILE);
    if path.exists() {
        return Ok(InitConfigResult {
            path,
            created: false,
        });
    }

    fs::write(&path, BASIC_CONFIG_TEMPLATE).map_err(|error| {
        ConfigError::new(format!(
            "failed to write config {}: {error}",
            path.display()
        ))
    })?;

    Ok(InitConfigResult {
        path,
        created: true,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{BASIC_CONFIG_TEMPLATE, DEFAULT_CONFIG_FILE, init_config};

    #[test]
    fn creates_basic_config_in_directory() {
        let dir =
            std::env::temp_dir().join(format!("rumon_init_config_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");

        let result = init_config(&dir).expect("init config");

        assert!(result.created);
        assert_eq!(result.path, dir.join(DEFAULT_CONFIG_FILE));
        let content = fs::read_to_string(result.path).expect("read config");
        assert_eq!(content, BASIC_CONFIG_TEMPLATE);

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn does_not_overwrite_existing_config() {
        let dir = std::env::temp_dir().join(format!(
            "rumon_init_existing_config_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join(DEFAULT_CONFIG_FILE);
        fs::write(&path, "version = 1\n").expect("seed config");

        let result = init_config(&dir).expect("init config");

        assert!(!result.created);
        assert_eq!(
            fs::read_to_string(&path).expect("read config"),
            "version = 1\n"
        );

        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
