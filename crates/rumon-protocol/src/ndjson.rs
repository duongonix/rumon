//! JSON and NDJSON writing helpers.

use std::io::{self, Write};

use crate::schema::RumonEvent;

/// Writes one event as NDJSON and flushes the writer.
///
/// # Errors
///
/// Returns serialization or writer errors.
pub fn write_ndjson_event(writer: &mut impl Write, event: &RumonEvent) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, event).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// Writes events as a JSON array.
///
/// # Errors
///
/// Returns serialization or writer errors.
pub fn write_json_array(writer: &mut impl Write, events: &[RumonEvent]) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, events).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}
