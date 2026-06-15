//! JSON and NDJSON stream runtimes.

use std::io::{self, Write};
use std::sync::mpsc;
use std::time::Duration;

use rumon_config::Config;
use rumon_protocol::{app_event_to_rumon_event, write_json_array, write_ndjson_event};
use rumon_watch::{WatchOptions, spawn_watcher};

use crate::{CoreError, CoreResult, Runtime};

/// Machine-readable stream format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamFormat {
    /// JSON array snapshot.
    Json,
    /// Realtime newline-delimited JSON.
    Ndjson,
}

/// Runs watcher output as JSON or NDJSON.
///
/// # Errors
///
/// Returns watcher bus or writer errors.
pub fn run_watch_stream(config: &Config, format: StreamFormat) -> CoreResult<u8> {
    if config.once {
        let events = collect_once_events(config)?;
        let mut stdout = io::stdout();
        match format {
            StreamFormat::Json => write_json_array(&mut stdout, &events)
                .map_err(|error| CoreError::new(error.to_string()))?,
            StreamFormat::Ndjson => {
                for event in events {
                    write_ndjson_event(&mut stdout, &event)
                        .map_err(|error| CoreError::new(error.to_string()))?;
                }
            }
        }
        return Ok(0);
    }

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
    let mut stdout = io::stdout();

    loop {
        let event = runtime
            .bus
            .recv_timeout(Duration::from_secs(3600))
            .map_err(|error| CoreError::new(error.to_string()))?;
        let enriched = runtime.apply_and_return(event);
        if let Some(protocol) = app_event_to_rumon_event(&enriched, config.profile.as_deref()) {
            match format {
                StreamFormat::Json => {
                    serde_json::to_writer(&mut stdout, &serde_json::json!({ "event": protocol }))
                        .map_err(|error| CoreError::new(error.to_string()))?;
                    stdout
                        .write_all(b"\n")
                        .and_then(|()| stdout.flush())
                        .map_err(|error| CoreError::new(error.to_string()))?;
                }
                StreamFormat::Ndjson => write_ndjson_event(&mut stdout, &protocol)
                    .map_err(|error| CoreError::new(error.to_string()))?,
            }
        }
    }
}

fn collect_once_events(config: &Config) -> CoreResult<Vec<rumon_protocol::RumonEvent>> {
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
    let window = Duration::from_millis(config.watch.debounce_ms.saturating_mul(2).max(250));
    let deadline = std::time::Instant::now() + window;
    let mut events = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match runtime
            .bus
            .recv_timeout(remaining.min(Duration::from_millis(100)))
        {
            Ok(event) => {
                let enriched = runtime.apply_and_return(event);
                if let Some(protocol) =
                    app_event_to_rumon_event(&enriched, config.profile.as_deref())
                {
                    events.push(protocol);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(CoreError::new("event bus disconnected"));
            }
        }
    }
    Ok(events)
}
