//! Shared watcher-to-protocol publisher.

use std::thread;
use std::time::Duration;

use rumon_config::Config;
use rumon_protocol::{ProtocolEventBus, app_event_to_rumon_event};
use rumon_shared::{AppEvent, WatchEvent};
use rumon_watch::{WatchOptions, spawn_watcher};

use crate::Runtime;

pub(super) fn spawn_protocol_watcher(config: Config, bus: ProtocolEventBus) {
    thread::spawn(move || {
        let mut runtime = Runtime::new();
        runtime.seed_change_details(&config.watch);
        let _watcher = spawn_watcher(
            WatchOptions {
                paths: config.watch.paths.clone(),
                ignore: config.watch.ignore.clone(),
                extensions: config.watch.extensions.clone(),
                recursive: config.watch.recursive,
                follow_symlink: config.watch.follow_symlink,
            },
            Duration::from_millis(config.watch.debounce_ms),
            runtime.sender(),
        );

        while let Ok(event) = runtime.bus.recv_timeout(Duration::from_secs(3600)) {
            if matches!(event, AppEvent::Watch(WatchEvent::Error(_))) {
                continue;
            }
            let enriched = runtime.apply_and_return(event);
            if let Some(protocol) = app_event_to_rumon_event(&enriched, config.profile.as_deref()) {
                bus.publish(&protocol);
            }
        }
    });
}
