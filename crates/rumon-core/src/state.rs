//! Core runtime state wrapper.

use std::sync::mpsc::Sender;

use rumon_config::WatchConfig;
use rumon_shared::{AppEvent, AppState};

use crate::change_detail::ChangeDetailInspector;
use crate::event_bus::EventBus;

/// Core runtime container.
#[derive(Debug)]
pub struct Runtime {
    pub(crate) state: AppState,
    events: Vec<AppEvent>,
    pub(crate) bus: EventBus,
    change_details: ChangeDetailInspector,
}

impl Runtime {
    /// Creates an empty runtime with default application state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            events: Vec::new(),
            bus: EventBus::new(),
            change_details: ChangeDetailInspector::default(),
        }
    }

    /// Publishes an event into the runtime event bus.
    pub fn publish(&self, event: AppEvent) {
        let _ = self.bus.sender.send(event);
    }

    /// Applies an event to centralized runtime state.
    pub fn apply(&mut self, event: AppEvent) {
        let _ = self.apply_and_return(event);
    }

    /// Applies an event and returns the enriched event stored in state.
    #[must_use]
    pub fn apply_and_return(&mut self, event: AppEvent) -> AppEvent {
        let enriched = self.change_details.enrich_event(event);
        self.state.apply(&enriched);
        self.events.push(enriched.clone());
        enriched
    }

    /// Seeds file detail snapshots from the current watch configuration.
    pub fn seed_change_details(&mut self, config: &WatchConfig) {
        self.change_details.seed_from_watch_config(config);
    }

    /// Returns the current application state.
    #[must_use]
    pub const fn state(&self) -> &AppState {
        &self.state
    }

    /// Returns queued events.
    #[must_use]
    pub fn events(&self) -> &[AppEvent] {
        &self.events
    }

    /// Returns a sender for external event producers.
    #[must_use]
    pub fn sender(&self) -> Sender<AppEvent> {
        self.bus.sender()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Runtime;
    use rumon_shared::{AppEvent, ProcessEvent, ProcessState};

    #[test]
    fn runtime_records_events() {
        let mut runtime = Runtime::new();
        runtime.apply(AppEvent::System("ready".to_string()));

        assert_eq!(runtime.events().len(), 1);
    }

    #[test]
    fn runtime_updates_state_from_events() {
        let mut runtime = Runtime::new();
        runtime.apply(AppEvent::Process(ProcessEvent::Started));

        assert_eq!(runtime.state().process, ProcessState::Running);
    }
}
