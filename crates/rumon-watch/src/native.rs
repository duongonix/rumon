//! Native filesystem watcher backed by notify and notify-debouncer-full.

use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use rumon_shared::{AppEvent, WatchEvent};

use crate::backend::{existing_roots, send_watch_error};
use crate::notify_mapper::KindCache;
use crate::snapshot::scan;
use crate::watcher::WatchOptions;

/// Native debounced filesystem watcher.
pub(crate) struct NativeWatcher {
    options: WatchOptions,
    debouncer: Debouncer<notify::RecommendedWatcher, RecommendedCache>,
    receiver: mpsc::Receiver<DebounceEventResult>,
    kind_cache: KindCache,
}

impl NativeWatcher {
    /// Creates and starts a native watcher.
    pub(crate) fn new(options: WatchOptions, debounce: Duration) -> Result<Self, String> {
        let snapshots = scan(&options).map_err(|error| error.to_string())?;
        let (sender, receiver) = mpsc::channel();
        let mut debouncer = new_debouncer(debounce, None, move |result| {
            let _ = sender.send(result);
        })
        .map_err(|error| error.to_string())?;

        let roots = existing_roots(&options);
        if roots.is_empty() {
            return Err("no existing watch paths".to_string());
        }

        for root in roots {
            debouncer
                .watch(&root, recursive_mode(options.recursive))
                .map_err(|error| format!("watch {}: {error}", root.display()))?;
        }

        Ok(Self {
            options,
            debouncer,
            receiver,
            kind_cache: KindCache::new(&snapshots),
        })
    }

    /// Runs the native watcher until the event bus disconnects.
    pub(crate) fn run(mut self, events: &Sender<AppEvent>) {
        let _keep_alive = &self.debouncer;
        while let Ok(result) = self.receiver.recv() {
            match result {
                Ok(debounced_events) => {
                    for debounced_event in debounced_events {
                        for event in self
                            .kind_cache
                            .map_event(&debounced_event.event, &self.options)
                        {
                            if events.send(event).is_err() {
                                return;
                            }
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        send_watch_error(events, error);
                    }
                }
            }
        }
        let _ = events.send(AppEvent::Watch(WatchEvent::Error(
            "native watcher stopped unexpectedly".to_string(),
        )));
    }
}

fn recursive_mode(recursive: bool) -> RecursiveMode {
    if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    }
}
