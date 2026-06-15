//! Machine-readable Rumon protocol types and helpers.

mod bus;
mod event;
mod ndjson;
mod schema;

pub use bus::ProtocolEventBus;
pub use event::{app_event_to_rumon_event, event_type_for_change};
pub use ndjson::{write_json_array, write_ndjson_event};
pub use schema::{
    DiffInfo, FileInfo, MediaInfo, MetadataInfo, RumonEvent, RumonEventType, StatusInfo,
};
