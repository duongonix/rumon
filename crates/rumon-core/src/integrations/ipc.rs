//! Local IPC daemon mode.

use std::io::{BufRead, BufReader, Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{ListenerNonblockingMode, ListenerOptions, ToFsName};
use rumon_config::Config;
use rumon_protocol::{ProtocolEventBus, StatusInfo};
use serde_json::{Value, json};

use crate::integrations::watcher::spawn_protocol_watcher;
use crate::{CoreError, CoreResult};

/// Runs the local IPC daemon.
///
/// The current backend uses loopback NDJSON sockets. The protocol is transport-neutral, so a
/// platform named-pipe/UDS backend can replace the listener without changing handlers.
///
/// # Errors
///
/// Returns bind or connection setup errors.
pub fn run_ipc_daemon(config: &Config) -> CoreResult<u8> {
    let path = ipc_path(&config.ipc.path);
    let name = ipc_name(&path)?;
    let listener = ListenerOptions::new()
        .name(name)
        .nonblocking(ListenerNonblockingMode::Accept)
        .reclaim_name(true)
        .create_sync()
        .map_err(|error| CoreError::new(format!("failed to bind ipc {path}: {error}")))?;
    let bus = ProtocolEventBus::new(config.api.max_event_buffer);
    let state = IpcState {
        config: config.clone(),
        started_at: Instant::now(),
        bus: bus.clone(),
        shutdown: Arc::new(AtomicBool::new(false)),
    };
    spawn_protocol_watcher(config.clone(), bus);
    eprintln!("rumon daemon: ipc listening on {path}");

    while !state.shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok(stream) => {
                let state = state.clone();
                std::thread::spawn(move || {
                    let _ = handle_client(stream, &state);
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(error) => eprintln!("rumon daemon: client failed: {error}"),
        }
    }
    Ok(0)
}

#[derive(Clone)]
struct IpcState {
    config: Config,
    started_at: Instant,
    bus: ProtocolEventBus,
    shutdown: Arc<AtomicBool>,
}

fn handle_client<Stream>(stream: Stream, state: &IpcState) -> std::io::Result<()>
where
    Stream: Read + Write,
{
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let request = serde_json::from_str::<Value>(&line).unwrap_or_else(|error| {
            json!({
                "id": null,
                "method": "__parse_error",
                "error": error.to_string(),
            })
        });
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("__parse_error");
        match method {
            "ping" => write_json_line(
                reader.get_mut(),
                &json!({ "id": id, "ok": true, "result": { "pong": true } }),
            )?,
            "status" => write_json_line(
                reader.get_mut(),
                &json!({ "id": id, "ok": true, "result": status(state) }),
            )?,
            "recent_events" => {
                let limit = request
                    .get("params")
                    .and_then(|params| params.get("limit"))
                    .and_then(Value::as_u64)
                    .and_then(|value| usize::try_from(value).ok())
                    .unwrap_or(100);
                write_json_line(
                    reader.get_mut(),
                    &json!({ "id": id, "ok": true, "result": { "events": state.bus.recent(limit) } }),
                )?;
            }
            "subscribe_events" => {
                write_json_line(reader.get_mut(), &json!({ "id": id, "ok": true }))?;
                let rx = state.bus.subscribe();
                while let Ok(event) = rx.recv() {
                    if write_json_line(
                        reader.get_mut(),
                        &json!({ "type": "event", "event": event }),
                    )
                    .is_err()
                    {
                        break;
                    }
                }
                break;
            }
            "shutdown" => {
                write_json_line(reader.get_mut(), &json!({ "id": id, "ok": true }))?;
                state.shutdown.store(true, Ordering::Relaxed);
                break;
            }
            _ => write_json_line(
                reader.get_mut(),
                &json!({ "id": id, "ok": false, "error": "unknown_method" }),
            )?,
        }
    }
    Ok(())
}

fn status(state: &IpcState) -> StatusInfo {
    StatusInfo {
        name: "rumon".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        profile: state
            .config
            .profile
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        running: true,
        watching: true,
        watch_paths: state
            .config
            .watch
            .paths
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        event_count: state.bus.len(),
        uptime_ms: state.started_at.elapsed().as_millis(),
    }
}

fn write_json_line(stream: &mut impl Write, value: &Value) -> std::io::Result<()> {
    serde_json::to_writer(&mut *stream, value).map_err(std::io::Error::other)?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn ipc_path(path: &str) -> String {
    if !path.is_empty() {
        return path.to_string();
    }
    if cfg!(windows) {
        r"\\.\pipe\rumon".to_string()
    } else {
        "/tmp/rumon.sock".to_string()
    }
}

#[cfg(windows)]
fn ipc_name(path: &str) -> CoreResult<interprocess::local_socket::Name<'_>> {
    use interprocess::os::windows::local_socket::NamedPipe;

    path.to_fs_name::<NamedPipe>()
        .map_err(|error| CoreError::new(format!("invalid ipc path {path}: {error}")))
}

#[cfg(unix)]
fn ipc_name(path: &str) -> CoreResult<interprocess::local_socket::Name<'_>> {
    use interprocess::os::unix::local_socket::UdSocket;

    path.to_fs_name::<UdSocket>()
        .map_err(|error| CoreError::new(format!("invalid ipc path {path}: {error}")))
}
