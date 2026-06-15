//! Configuration loading pipeline.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rumon_profiles::{extract_profile_name, load_profile_from_dir};

use crate::error::{ConfigError, ConfigResult};
use crate::merge::{merge_cli, merge_toml_subset};
use crate::schema::{CliOverrides, Config};
use crate::validation::validate;

/// Loads configuration using defaults, optional TOML file, environment variables, and CLI overrides.
///
/// # Errors
///
/// Returns an error when the config file cannot be read, an override cannot be parsed, or the
/// merged configuration is invalid.
pub fn load(overrides: CliOverrides) -> ConfigResult<Config> {
    let mut config = Config::default();
    let config_path = overrides.config_path.clone().or_else(|| {
        Path::new("rumon.toml")
            .exists()
            .then(|| PathBuf::from("rumon.toml"))
    });

    if let Some(path) = config_path {
        let content = read_config_file(&path)?;
        merge_selected_profile(&mut config, &content, config_base_dir(&path))?;
        merge_toml_subset(&mut config, &content)?;
    }

    merge_env(&mut config)?;
    merge_cli(&mut config, overrides);
    validate(&config)?;
    Ok(config)
}

/// Loads default configuration without external sources.
#[must_use]
pub fn load_defaults() -> Config {
    Config::default()
}

fn read_config_file(path: &Path) -> ConfigResult<String> {
    fs::read_to_string(path).map_err(|error| {
        ConfigError::new(format!("failed to read config {}: {error}", path.display()))
    })
}

fn merge_selected_profile(config: &mut Config, content: &str, base_dir: &Path) -> ConfigResult<()> {
    let Some(profile_name) = extract_profile_name(content) else {
        return Ok(());
    };
    let profile = load_profile_from_dir(&profile_name, base_dir)
        .map_err(|error| ConfigError::new(format!("failed to load profile: {error}")))?;
    merge_toml_subset(config, &profile.content)?;
    config.profile = Some(profile.name);
    Ok(())
}

fn config_base_dir(path: &Path) -> &Path {
    path.parent().unwrap_or_else(|| Path::new("."))
}

fn merge_env(config: &mut Config) -> ConfigResult<()> {
    if let Ok(command) = env::var("RUMON_RUN_CMD") {
        config.run.cmd = command;
    }
    if let Ok(debounce) = env::var("RUMON_WATCH_DEBOUNCE_MS") {
        config.watch.debounce_ms = debounce
            .parse()
            .map_err(|_| ConfigError::new("RUMON_WATCH_DEBOUNCE_MS must be a number"))?;
    }
    if let Ok(max_log_lines) = env::var("RUMON_TUI_MAX_LOG_LINES") {
        config.tui.max_log_lines = max_log_lines
            .parse()
            .map_err(|_| ConfigError::new("RUMON_TUI_MAX_LOG_LINES must be a number"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load, load_defaults};
    use crate::CliOverrides;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn defaults_match_rust_project() {
        let config = load_defaults();

        assert_eq!(config.run.cmd, "cargo run");
        assert_eq!(
            config.watch.paths,
            vec![
                PathBuf::from("src"),
                PathBuf::from("crates"),
                PathBuf::from("rumon.toml")
            ]
        );
    }

    #[test]
    fn cli_overrides_are_default_constructible() {
        let overrides = CliOverrides::default();

        assert!(overrides.command.is_none());
        assert!(overrides.watch_paths.is_empty());
    }

    #[test]
    fn profile_loads_before_user_and_cli_overrides() {
        let path = std::env::temp_dir().join("rumon_profile_override_test.toml");
        fs::write(
            &path,
            r#"
            profile = "rust"

            [run]
            cmd = "cargo test"
            "#,
        )
        .expect("write config");

        let config = load(CliOverrides {
            config_path: Some(path.clone()),
            command: Some("cargo check".to_string()),
            ..CliOverrides::default()
        })
        .expect("config should load");

        let _ = fs::remove_file(path);

        assert_eq!(config.profile, Some("rust".to_string()));
        assert_eq!(config.run.cmd, "cargo check");
        assert_eq!(config.watch.extensions, vec!["rs", "toml"]);
    }

    #[test]
    fn custom_profile_loads_next_to_config_file() {
        let base_dir = std::env::temp_dir().join("rumon_config_custom_profile_test");
        let profiles_dir = base_dir.join("profiles");
        fs::create_dir_all(&profiles_dir).expect("create profiles dir");
        fs::write(
            profiles_dir.join("backend.toml"),
            r#"
            [run]
            cmd = "cargo test"

            [watch]
            paths = ["."]
            extensions = [
              "rs",
              "toml"
            ]
            "#,
        )
        .expect("write custom profile");
        let config_path = base_dir.join("rumon.toml");
        fs::write(&config_path, r#"profile = "backend""#).expect("write config");

        let config = load(CliOverrides {
            config_path: Some(config_path),
            ..CliOverrides::default()
        })
        .expect("config should load");

        let _ = fs::remove_dir_all(base_dir);

        assert_eq!(config.profile, Some("backend".to_string()));
        assert_eq!(config.run.cmd, "cargo test");
        assert_eq!(config.watch.extensions, vec!["rs", "toml"]);
    }
}
