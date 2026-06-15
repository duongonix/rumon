//! Shared protocol event bus.

use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::schema::RumonEvent;

/// Broadcasts protocol events and keeps a bounded recent buffer.
#[derive(Clone, Debug)]
pub struct ProtocolEventBus {
    inner: Arc<Mutex<Inner>>,
    max_buffer: usize,
}

#[derive(Debug)]
struct Inner {
    events: VecDeque<RumonEvent>,
    subscribers: Vec<Sender<RumonEvent>>,
}

impl ProtocolEventBus {
    /// Creates a new protocol bus.
    #[must_use]
    pub fn new(max_buffer: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                events: VecDeque::new(),
                subscribers: Vec::new(),
            })),
            max_buffer,
        }
    }

    /// Publishes an event to the buffer and current subscribers.
    pub fn publish(&self, event: &RumonEvent) {
        let mut inner = self.lock_inner();
        inner.events.push_back(event.clone());
        while inner.events.len() > self.max_buffer {
            inner.events.pop_front();
        }
        inner
            .subscribers
            .retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }

    /// Subscribes to future events.
    #[must_use]
    pub fn subscribe(&self) -> Receiver<RumonEvent> {
        let (tx, rx) = mpsc::channel();
        self.lock_inner().subscribers.push(tx);
        rx
    }

    /// Returns recent events, newest last.
    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<RumonEvent> {
        let inner = self.lock_inner();
        inner
            .events
            .iter()
            .skip(inner.events.len().saturating_sub(limit))
            .cloned()
            .collect()
    }

    /// Returns buffered event count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock_inner().events.len()
    }

    /// Returns whether the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock_inner(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
