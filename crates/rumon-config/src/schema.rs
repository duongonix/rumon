//! Configuration schema.

use std::path::PathBuf;

use crate::event_hooks::EventHookConfig;

/// Watcher configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WatchConfig {
    /// Paths watched for filesystem changes.
    pub paths: Vec<PathBuf>,
    /// Paths ignored by the watcher.
    pub ignore: Vec<PathBuf>,
    /// Extension allow-list without leading dots.
    pub extensions: Vec<String>,
    /// Debounce window in milliseconds.
    pub debounce_ms: u64,
    /// Whether subdirectories are watched recursively.
    pub recursive: bool,
    /// Whether symlinks are followed.
    pub follow_symlink: bool,
}

/// Runner configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunConfig {
    /// Command executed by the runner.
    pub cmd: String,
    /// Working directory for the command.
    pub cwd: PathBuf,
    /// Whether file changes restart the command.
    pub restart: bool,
    /// Delay before restarting the command.
    pub restart_delay_ms: u64,
    /// Termination strategy.
    pub kill_signal: String,
    /// Whether logs clear on restart.
    pub clear_logs_on_restart: bool,
}

/// TUI configuration used by later phases and `--no-tui` now.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct TuiConfig {
    /// Whether the TUI is enabled.
    pub enabled: bool,
    /// Left panel width percentage.
    pub left_panel_width: u16,
    /// Whether timestamps are shown.
    pub show_timestamp: bool,
    /// Whether file icons are shown.
    pub show_file_icons: bool,
    /// Whether logs auto-scroll.
    pub auto_scroll_logs: bool,
    /// Maximum log lines retained.
    pub max_log_lines: usize,
}

/// HTTP/API server configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiConfig {
    /// Whether API server is enabled from config.
    pub enabled: bool,
    /// Host address to bind.
    pub host: String,
    /// TCP port to bind.
    pub port: u16,
    /// Enabled transports.
    pub transport: Vec<String>,
    /// Event format name.
    pub event_format: String,
    /// Maximum buffered events.
    pub max_event_buffer: usize,
}

/// Local IPC daemon configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpcConfig {
    /// Whether IPC daemon is enabled from config.
    pub enabled: bool,
    /// IPC endpoint name.
    pub name: String,
    /// Optional IPC path.
    pub path: String,
}

/// Complete Rumon configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// Configuration version.
    pub version: u32,
    /// Optional active profile name.
    pub profile: Option<String>,
    /// Watcher settings.
    pub watch: WatchConfig,
    /// Runner settings.
    pub run: RunConfig,
    /// TUI settings.
    pub tui: TuiConfig,
    /// HTTP/API server settings.
    pub api: ApiConfig,
    /// Local IPC settings.
    pub ipc: IpcConfig,
    /// Rule-based commands executed for matching filesystem events.
    pub event_hooks: Vec<EventHookConfig>,
    /// Run once and exit without starting the watcher.
    pub once: bool,
    /// Enable verbose runtime logs.
    pub verbose: bool,
    /// Suppress non-critical runtime logs.
    pub quiet: bool,
}

/// CLI-supplied configuration overrides.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct CliOverrides {
    /// Optional config path.
    pub config_path: Option<PathBuf>,
    /// Additional watch paths.
    pub watch_paths: Vec<PathBuf>,
    /// Additional ignore paths.
    pub ignore_paths: Vec<PathBuf>,
    /// Additional extension filters.
    pub extensions: Vec<String>,
    /// Command override.
    pub command: Option<String>,
    /// Working directory override.
    pub cwd: Option<PathBuf>,
    /// Debounce override.
    pub debounce_ms: Option<u64>,
    /// Disable TUI.
    pub no_tui: bool,
    /// Clear logs on restart.
    pub clear_logs: bool,
    /// Run once and exit.
    pub once: bool,
    /// Disable automatic restart.
    pub no_restart: bool,
    /// Verbose output.
    pub verbose: bool,
    /// Quiet output.
    pub quiet: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            profile: None,
            watch: WatchConfig {
                paths: vec![
                    PathBuf::from("src"),
                    PathBuf::from("crates"),
                    PathBuf::from("rumon.toml"),
                ],
                ignore: vec![
                    PathBuf::from(".git"),
                    PathBuf::from("target"),
                    PathBuf::from("node_modules"),
                    PathBuf::from("dist"),
                ],
                extensions: Vec::new(),
                debounce_ms: 500,
                recursive: true,
                follow_symlink: false,
            },
            run: RunConfig {
                cmd: "cargo run".to_string(),
                cwd: PathBuf::from("."),
                restart: true,
                restart_delay_ms: 300,
                kill_signal: "term".to_string(),
                clear_logs_on_restart: false,
            },
            tui: TuiConfig {
                enabled: true,
                left_panel_width: 50,
                show_timestamp: true,
                show_file_icons: true,
                auto_scroll_logs: true,
                max_log_lines: 5_000,
            },
            api: ApiConfig {
                enabled: false,
                host: "127.0.0.1".to_string(),
                port: 3717,
                transport: vec![
                    "http".to_string(),
                    "sse".to_string(),
                    "ws".to_string(),
                    "ipc".to_string(),
                ],
                event_format: "json".to_string(),
                max_event_buffer: 1_000,
            },
            ipc: IpcConfig {
                enabled: false,
                name: "rumon".to_string(),
                path: String::new(),
            },
            event_hooks: Vec::new(),
            once: false,
            verbose: false,
            quiet: false,
        }
    }
}
