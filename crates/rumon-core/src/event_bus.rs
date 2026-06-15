//! In-process event bus.

use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use rumon_shared::AppEvent;

/// In-process event bus.
#[derive(Debug)]
pub struct EventBus {
    pub(crate) sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
}

impl EventBus {
    /// Creates an event bus.
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }

    /// Returns a sender handle for producers.
    #[must_use]
    pub fn sender(&self) -> Sender<AppEvent> {
        self.sender.clone()
    }

    /// Receives the next event.
    ///
    /// # Errors
    ///
    /// Returns a timeout or disconnected error from the underlying channel.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<AppEvent, mpsc::RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    /// Attempts to receive one pending event without blocking.
    ///
    /// # Errors
    ///
    /// Returns empty or disconnected errors from the underlying channel.
    pub fn try_recv(&self) -> Result<AppEvent, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::EventBus;
    use rumon_shared::AppEvent;
    use std::time::Duration;

    #[test]
    fn event_bus_transports_events() {
        let bus = EventBus::new();
        bus.sender()
            .send(AppEvent::System("ready".to_string()))
            .expect("send should succeed");

        assert_eq!(
            bus.recv_timeout(Duration::from_millis(10))
                .expect("event should arrive"),
            AppEvent::System("ready".to_string())
        );
    }
}
