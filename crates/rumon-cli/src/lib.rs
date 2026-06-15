//! Command line parsing for Rumon.

mod args;
mod bootstrap;
mod commands;
mod error;

pub use args::parse_args;
pub use bootstrap::print_help;
pub use commands::{CliCommand, RemoteCommand, WatchOutput};
pub use error::CliError;
