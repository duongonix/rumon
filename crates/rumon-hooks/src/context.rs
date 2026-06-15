//! Hook execution context.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rumon_shared::display_path;

use crate::lifecycle::HookPoint;

/// Runtime context injected into hook environment variables.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookContext {
    /// File that triggered the lifecycle event when available.
    pub file: Option<PathBuf>,
    /// Lifecycle event name.
    pub event: HookPoint,
    /// Current process id when available.
    pub process_pid: Option<u32>,
    /// Process exit code when available.
    pub exit_code: Option<i32>,
    /// Timestamp in seconds since Unix epoch.
    pub timestamp: u64,
}

impl HookContext {
    /// Creates context for a hook point.
    #[must_use]
    pub fn new(event: HookPoint) -> Self {
        Self {
            file: None,
            event,
            process_pid: None,
            exit_code: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_secs()),
        }
    }

    /// Converts the context into `RUMON_*` environment variables.
    #[must_use]
    pub fn environment(&self) -> BTreeMap<String, String> {
        let mut variables = BTreeMap::new();
        variables.insert(
            "RUMON_FILE".to_string(),
            self.file
                .as_ref()
                .map_or_else(String::new, |path| display_path(path)),
        );
        variables.insert("RUMON_EVENT".to_string(), self.event.as_str().to_string());
        variables.insert(
            "RUMON_PROCESS_PID".to_string(),
            self.process_pid
                .map_or_else(String::new, |pid| pid.to_string()),
        );
        variables.insert(
            "RUMON_EXIT_CODE".to_string(),
            self.exit_code
                .map_or_else(String::new, |code| code.to_string()),
        );
        variables.insert("RUMON_TIMESTAMP".to_string(), self.timestamp.to_string());
        variables
    }
}

#[cfg(test)]
mod tests {
    use super::HookContext;
    use crate::HookPoint;

    #[test]
    fn context_exports_event_name() {
        let context = HookContext::new(HookPoint::BeforeRestart);
        let environment = context.environment();

        assert_eq!(
            environment.get("RUMON_EVENT"),
            Some(&"before_restart".to_string())
        );
    }
}
