//! Parsed CLI commands.

use rumon_config::CliOverrides;

/// Parsed CLI command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    /// Run Rumon with the supplied overrides.
    Run(CliOverrides),
    /// Explicit TUI command.
    Tui(CliOverrides),
    /// Machine-readable watch command.
    Watch {
        /// CLI overrides.
        overrides: CliOverrides,
        /// Output format.
        format: WatchOutput,
    },
    /// HTTP integration server command.
    Server {
        /// CLI overrides.
        overrides: CliOverrides,
        /// Host override.
        host: Option<String>,
        /// Port override.
        port: Option<u16>,
    },
    /// Local IPC daemon command.
    Daemon {
        /// CLI overrides.
        overrides: CliOverrides,
        /// Whether IPC was explicitly requested.
        ipc: bool,
    },
    /// Create a starter rumon.toml in the current project.
    Init,
    /// Run a remote monitor subcommand.
    Remote(RemoteCommand),
    /// Print help.
    Help,
    /// Print version.
    Version,
}

/// Watch output format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WatchOutput {
    /// JSON output.
    Json,
    /// NDJSON output.
    Ndjson,
}

/// Parsed remote monitor command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteCommand {
    /// Run a remote agent that serves one client connection.
    Agent {
        /// Address to bind.
        address: String,
        /// Node name.
        node: String,
        /// Authentication token.
        token: String,
    },
    /// Connect to a remote agent.
    Connect {
        /// Address to connect.
        address: String,
        /// Local node name.
        node: String,
        /// Authentication token.
        token: String,
    },
}
