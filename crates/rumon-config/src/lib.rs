//! Configuration loading, merging, and validation for Rumon.

mod error;
mod event_hooks;
mod init;
mod loader;
mod merge;
mod schema;
mod validation;

pub use error::{ConfigError, ConfigResult};
pub use event_hooks::EventHookConfig;
pub use init::{BASIC_CONFIG_TEMPLATE, DEFAULT_CONFIG_FILE, InitConfigResult, init_config};
pub use loader::{load, load_defaults};
pub use schema::{ApiConfig, CliOverrides, Config, IpcConfig, RunConfig, TuiConfig, WatchConfig};
pub use validation::validate;
