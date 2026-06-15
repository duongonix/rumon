//! Configuration merging helpers.

use std::path::PathBuf;

use crate::error::{ConfigError, ConfigResult};
use crate::event_hooks::EventHookConfig;
use crate::schema::{CliOverrides, Config};

/// Applies CLI overrides to a resolved configuration.
pub fn merge_cli(config: &mut Config, overrides: CliOverrides) {
    if !overrides.watch_paths.is_empty() {
        config.watch.paths = overrides.watch_paths;
    }
    if !overrides.ignore_paths.is_empty() {
        config.watch.ignore = overrides.ignore_paths;
    }
    if !overrides.extensions.is_empty() {
        config.watch.extensions = overrides
            .extensions
            .into_iter()
            .map(|extension| extension.trim_start_matches('.').to_string())
            .collect();
    }
    if let Some(command) = overrides.command {
        config.run.cmd = command;
    }
    if let Some(cwd) = overrides.cwd {
        config.run.cwd = cwd;
    }
    if let Some(debounce_ms) = overrides.debounce_ms {
        config.watch.debounce_ms = debounce_ms;
    }
    if overrides.no_tui {
        config.tui.enabled = false;
    }
    if overrides.clear_logs {
        config.run.clear_logs_on_restart = true;
    }
    if overrides.once {
        config.once = true;
    }
    if overrides.no_restart {
        config.run.restart = false;
    }
    if overrides.verbose {
        config.verbose = true;
    }
    if overrides.quiet {
        config.quiet = true;
    }
}

/// Merges the supported TOML subset into a configuration.
pub fn merge_toml_subset(config: &mut Config, content: &str) -> ConfigResult<()> {
    let mut section = String::new();
    let mut lines = content.lines().peekable();
    while let Some(raw_line) = lines.next() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line
            .strip_prefix("[[")
            .and_then(|value| value.strip_suffix("]]"))
        {
            section = name.trim().to_string();
            if section == "event_hooks" {
                config.event_hooks.push(EventHookConfig::default());
            }
            continue;
        }
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        {
            section = name.trim().to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = collect_value(value.trim(), &mut lines);
        apply_value(config, &section, key.trim(), &value)?;
    }
    Ok(())
}

fn collect_value<'a>(
    value: &'a str,
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> String {
    if value.starts_with("'''") && (value == "'''" || !value.trim_end().ends_with("'''")) {
        let mut collected = value.to_string();
        for raw_line in lines.by_ref() {
            collected.push('\n');
            collected.push_str(raw_line);
            if raw_line.trim_end().ends_with("'''") {
                break;
            }
        }
        return collected;
    }

    if !value.starts_with('[') || value.ends_with(']') {
        return value.to_string();
    }

    let mut collected = value.to_string();
    for raw_line in lines.by_ref() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if !line.is_empty() {
            collected.push(' ');
            collected.push_str(line);
        }
        if line.ends_with(']') {
            break;
        }
    }
    collected
}

fn apply_value(config: &mut Config, section: &str, key: &str, value: &str) -> ConfigResult<()> {
    match (section, key) {
        ("", "version") => config.version = parse_u32(value, key)?,
        ("", "profile") => config.profile = Some(parse_string(value)),
        ("watch", "paths") => config.watch.paths = parse_path_array(value),
        ("watch", "ignore") => config.watch.ignore = parse_path_array(value),
        ("watch", "extensions") => config.watch.extensions = parse_string_array(value),
        ("watch", "debounce_ms") => config.watch.debounce_ms = parse_u64(value, key)?,
        ("watch", "recursive") => config.watch.recursive = parse_bool(value, key)?,
        ("watch", "follow_symlink") => config.watch.follow_symlink = parse_bool(value, key)?,
        ("run", "cmd") => config.run.cmd = parse_string(value),
        ("run", "cwd") => config.run.cwd = PathBuf::from(parse_string(value)),
        ("run", "restart") => config.run.restart = parse_bool(value, key)?,
        ("run", "restart_delay_ms") => config.run.restart_delay_ms = parse_u64(value, key)?,
        ("run", "kill_signal") => config.run.kill_signal = parse_string(value),
        ("run", "clear_logs_on_restart") => {
            config.run.clear_logs_on_restart = parse_bool(value, key)?;
        }
        ("tui", "left_panel_width") => config.tui.left_panel_width = parse_u16(value, key)?,
        ("tui", "show_timestamp") => config.tui.show_timestamp = parse_bool(value, key)?,
        ("tui", "show_file_icons") => config.tui.show_file_icons = parse_bool(value, key)?,
        ("tui", "auto_scroll_logs") => config.tui.auto_scroll_logs = parse_bool(value, key)?,
        ("tui", "max_log_lines") => config.tui.max_log_lines = parse_usize(value, key)?,
        ("api", "enabled") => config.api.enabled = parse_bool(value, key)?,
        ("api", "host") => config.api.host = parse_string(value),
        ("api", "port") => config.api.port = parse_u16(value, key)?,
        ("api", "transport") => config.api.transport = parse_string_array(value),
        ("api", "event_format") => config.api.event_format = parse_string(value),
        ("api", "max_event_buffer") => config.api.max_event_buffer = parse_usize(value, key)?,
        ("ipc", "enabled") => config.ipc.enabled = parse_bool(value, key)?,
        ("ipc", "name") => config.ipc.name = parse_string(value),
        ("ipc", "path") => config.ipc.path = parse_string(value),
        ("event_hooks", "name") => {
            if let Some(hook) = config.event_hooks.last_mut() {
                hook.name = parse_string(value);
            }
        }
        ("event_hooks", "events") => {
            if let Some(hook) = config.event_hooks.last_mut() {
                hook.events = parse_string_array(value);
            }
        }
        ("event_hooks", "paths") => {
            if let Some(hook) = config.event_hooks.last_mut() {
                hook.paths = parse_string_array(value);
            }
        }
        ("event_hooks", "when") => {
            if let Some(hook) = config.event_hooks.last_mut() {
                hook.when = parse_string(value);
            }
        }
        ("event_hooks", "cmd") => {
            if let Some(hook) = config.event_hooks.last_mut() {
                hook.cmd = parse_string(value);
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_string(value: &str) -> String {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix("'''")
        .and_then(|value| value.strip_suffix("'''"))
    {
        return inner.trim().to_string();
    }
    value.trim_matches('"').to_string()
}

fn parse_string_array(value: &str) -> Vec<String> {
    let inner = value.trim().trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(parse_string)
        .filter(|value| !value.is_empty())
        .collect()
}

fn parse_path_array(value: &str) -> Vec<PathBuf> {
    parse_string_array(value)
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

fn parse_bool(value: &str, key: &str) -> ConfigResult<bool> {
    value
        .parse()
        .map_err(|_| ConfigError::new(format!("{key} must be a boolean")))
}

fn parse_u16(value: &str, key: &str) -> ConfigResult<u16> {
    value
        .parse()
        .map_err(|_| ConfigError::new(format!("{key} must be a number")))
}

fn parse_u32(value: &str, key: &str) -> ConfigResult<u32> {
    value
        .parse()
        .map_err(|_| ConfigError::new(format!("{key} must be a number")))
}

fn parse_u64(value: &str, key: &str) -> ConfigResult<u64> {
    value
        .parse()
        .map_err(|_| ConfigError::new(format!("{key} must be a number")))
}

fn parse_usize(value: &str, key: &str) -> ConfigResult<usize> {
    value
        .parse()
        .map_err(|_| ConfigError::new(format!("{key} must be a number")))
}

#[cfg(test)]
mod tests {
    use super::merge_toml_subset;
    use crate::Config;
    use std::path::PathBuf;

    #[test]
    fn toml_subset_overrides_nested_values() {
        let mut config = Config::default();

        merge_toml_subset(
            &mut config,
            r#"
            profile = "node"

            [run]
            cmd = "npm run dev"

            [watch]
            paths = ["app", "public"]
            extensions = ["ts", "tsx"]
            debounce_ms = 750
            "#,
        )
        .expect("TOML subset should parse");

        assert_eq!(config.profile, Some("node".to_string()));
        assert_eq!(config.run.cmd, "npm run dev");
        assert_eq!(
            config.watch.paths,
            vec![PathBuf::from("app"), PathBuf::from("public")]
        );
        assert_eq!(config.watch.extensions, vec!["ts", "tsx"]);
        assert_eq!(config.watch.debounce_ms, 750);
    }

    #[test]
    fn toml_subset_parses_multiline_arrays() {
        let mut config = Config::default();

        merge_toml_subset(
            &mut config,
            r#"
            [watch]
            extensions = [
              "js",
              "ts",
              "tsx"
            ]
            "#,
        )
        .expect("TOML subset should parse");

        assert_eq!(config.watch.extensions, vec!["js", "ts", "tsx"]);
    }

    #[test]
    fn toml_subset_parses_event_hooks_in_order() {
        let mut config = Config::default();

        merge_toml_subset(
            &mut config,
            r#"
            [[event_hooks]]
            name = "rust"
            events = [
              "file_created",
              "file_modified"
            ]
            paths = ["src/**/*.rs"]
            when = '''
            diff.added_lines > 0 &&
            file.ext == "rs"
            '''
            cmd = "cargo check"

            [[event_hooks]]
            name = "config"
            events = ["file_modified"]
            paths = ["rumon.toml"]
            cmd = ""
            "#,
        )
        .expect("TOML subset should parse event hooks");

        assert_eq!(config.event_hooks.len(), 2);
        assert_eq!(config.event_hooks[0].name, "rust");
        assert!(config.event_hooks[0].when.contains("diff.added_lines"));
        assert_eq!(
            config.event_hooks[0].events,
            vec!["file_created", "file_modified"]
        );
        assert_eq!(config.event_hooks[1].cmd, "");
    }
}
