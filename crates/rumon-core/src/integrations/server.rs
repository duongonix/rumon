//! HTTP, SSE, and WebSocket server mode.

use std::convert::Infallible;
use std::time::Instant;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures_util::{SinkExt, Stream, StreamExt};
use rumon_config::Config;
use rumon_protocol::{ProtocolEventBus, RumonEvent, StatusInfo};
use serde_json::json;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::integrations::watcher::spawn_protocol_watcher;
use crate::{CoreError, CoreResult};

/// Runs the HTTP integration server.
///
/// # Errors
///
/// Returns bind or server runtime errors.
pub fn run_api_server(config: &Config, host: Option<String>, port: Option<u16>) -> CoreResult<u8> {
    let host = host.unwrap_or_else(|| config.api.host.clone());
    let port = port.unwrap_or(config.api.port);
    let address = format!("{host}:{port}");
    let bus = ProtocolEventBus::new(config.api.max_event_buffer);
    let (tx, _) = broadcast::channel(config.api.max_event_buffer.max(16));
    spawn_protocol_watcher(config.clone(), bus.clone());
    spawn_broadcast_bridge(bus.clone(), tx.clone());

    let state = ServerState {
        config: config.clone(),
        started_at: Instant::now(),
        bus,
        tx,
    };
    eprintln!("rumon server: listening on http://{address}");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| CoreError::new(format!("failed to start server runtime: {error}")))?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(&address)
            .await
            .map_err(|error| CoreError::new(format!("failed to bind {address}: {error}")))?;
        axum::serve(listener, router(state))
            .await
            .map_err(|error| CoreError::new(format!("server failed: {error}")))
    })?;
    Ok(0)
}

#[derive(Clone)]
struct ServerState {
    config: Config,
    started_at: Instant,
    bus: ProtocolEventBus,
    tx: broadcast::Sender<RumonEvent>,
}

fn router(state: ServerState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/events", get(events))
        .route("/events/stream", get(events_stream))
        .route("/ws/events", get(ws_events))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}

async fn status(State(state): State<ServerState>) -> Json<StatusInfo> {
    Json(status_info(&state))
}

async fn events(
    State(state): State<ServerState>,
    Query(query): Query<EventQuery>,
) -> Json<serde_json::Value> {
    Json(events_json(&state, &query))
}

async fn events_stream(
    State(state): State<ServerState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.tx.subscribe()).filter_map(|event| async move {
        let event = event.ok()?;
        let event_name = event_name(&event);
        let data = serde_json::to_string(&event).ok()?;
        Some(Ok(Event::default().event(event_name).data(data)))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn ws_events(State(state): State<ServerState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade
        .on_upgrade(move |socket| websocket_client(socket, state))
        .into_response()
}

async fn websocket_client(socket: WebSocket, state: ServerState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();
    loop {
        tokio::select! {
            event = rx.recv() => {
                let Ok(event) = event else {
                    break;
                };
                let Ok(data) = serde_json::to_string(&event) else {
                    continue;
                };
                if sender.send(Message::Text(data.into())).await.is_err() {
                    break;
                }
            }
            message = receiver.next() => {
                match message {
                    Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                    Some(Ok(_)) => {}
                }
            }
        }
    }
}

#[derive(Debug, Default, serde::Deserialize)]
struct EventQuery {
    limit: Option<usize>,
    #[serde(rename = "type")]
    event_type: Option<String>,
    path: Option<String>,
}

fn status_info(state: &ServerState) -> StatusInfo {
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

fn events_json(state: &ServerState, query: &EventQuery) -> serde_json::Value {
    let events = state
        .bus
        .recent(query.limit.unwrap_or(100))
        .into_iter()
        .filter(|event| {
            query.event_type.as_ref().is_none_or(|expected| {
                serde_json::to_value(&event.event_type)
                    .ok()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned))
                    .is_some_and(|value| value == *expected)
            })
        })
        .filter(|event| {
            query.path.as_ref().is_none_or(|needle| {
                event
                    .path
                    .as_ref()
                    .or(event.new_path.as_ref())
                    .is_some_and(|path| path.contains(needle))
            })
        })
        .collect::<Vec<_>>();
    json!({ "events": events })
}

fn event_name(event: &RumonEvent) -> &'static str {
    match event.event_type {
        rumon_protocol::RumonEventType::FileCreated => "file_created",
        rumon_protocol::RumonEventType::FileModified => "file_modified",
        rumon_protocol::RumonEventType::FileDeleted => "file_deleted",
        rumon_protocol::RumonEventType::FileRenamed => "file_renamed",
        rumon_protocol::RumonEventType::FolderCreated => "folder_created",
        rumon_protocol::RumonEventType::FolderDeleted => "folder_deleted",
        rumon_protocol::RumonEventType::FolderRenamed => "folder_renamed",
        rumon_protocol::RumonEventType::MetadataChanged => "metadata_changed",
        rumon_protocol::RumonEventType::ContentChanged => "content_changed",
        rumon_protocol::RumonEventType::PermissionChanged => "permission_changed",
    }
}

fn spawn_broadcast_bridge(bus: ProtocolEventBus, tx: broadcast::Sender<RumonEvent>) {
    std::thread::spawn(move || {
        let rx = bus.subscribe();
        while let Ok(event) = rx.recv() {
            let _ = tx.send(event);
        }
    });
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rumon_protocol::{ProtocolEventBus, RumonEvent, RumonEventType};

    use super::{EventQuery, ServerState, event_name, events_json};

    #[test]
    fn formats_sse_event_name() {
        assert_eq!(event_name(&sample_event("src/a.rs")), "file_modified");
    }

    #[test]
    fn filters_recent_events_by_type_and_path() {
        let bus = ProtocolEventBus::new(10);
        let matching = sample_event("src/a.rs");
        let other = RumonEvent {
            path: Some("assets/a.png".to_string()),
            event_type: RumonEventType::FileCreated,
            ..sample_event("assets/a.png")
        };
        bus.publish(&matching);
        bus.publish(&other);
        let (tx, _) = tokio::sync::broadcast::channel(10);
        let state = ServerState {
            config: rumon_config::Config::default(),
            started_at: Instant::now(),
            bus,
            tx,
        };
        let query = EventQuery {
            limit: Some(10),
            event_type: Some("file_modified".to_string()),
            path: Some("src".to_string()),
        };

        let response = events_json(&state, &query);

        assert_eq!(response["events"].as_array().expect("events").len(), 1);
        assert_eq!(response["events"][0]["path"], "src/a.rs");
    }

    fn sample_event(path: &str) -> RumonEvent {
        RumonEvent {
            id: "evt_test".to_string(),
            event_type: RumonEventType::FileModified,
            timestamp: "2026-06-13T18:30:00Z".to_string(),
            profile: "rust".to_string(),
            path: Some(path.to_string()),
            old_path: None,
            new_path: None,
            file: None,
            diff: None,
            metadata: None,
            media: None,
        }
    }
}
