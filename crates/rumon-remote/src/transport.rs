//! Remote TCP transport.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::error::RemoteResult;
use crate::protocol::RemoteFrame;

/// Opens a listening TCP socket.
///
/// # Errors
///
/// Returns an error when the address cannot be bound.
pub fn bind(address: &str) -> RemoteResult<TcpListener> {
    Ok(TcpListener::bind(address)?)
}

/// Connects to a remote TCP socket.
///
/// # Errors
///
/// Returns an error when the connection cannot be opened.
pub fn connect(address: impl ToSocketAddrs) -> RemoteResult<TcpStream> {
    Ok(TcpStream::connect(address)?)
}

/// Sends one frame over the stream.
///
/// # Errors
///
/// Returns an error when writing fails.
pub fn send_frame(stream: &mut TcpStream, frame: &RemoteFrame) -> RemoteResult<()> {
    writeln!(stream, "{}", frame.encode())?;
    stream.flush()?;
    Ok(())
}

/// Reads one frame from the buffered stream reader.
///
/// # Errors
///
/// Returns an error when reading fails or the frame is invalid.
pub fn read_frame(reader: &mut BufReader<TcpStream>) -> RemoteResult<Option<RemoteFrame>> {
    let mut line = String::new();
    let bytes = reader.read_line(&mut line)?;
    if bytes == 0 {
        return Ok(None);
    }
    Ok(Some(RemoteFrame::decode(&line)?))
}

/// Creates a buffered reader from a stream.
///
/// # Errors
///
/// Returns an error when the stream cannot be cloned.
pub fn frame_reader(stream: &TcpStream) -> RemoteResult<BufReader<TcpStream>> {
    Ok(BufReader::new(stream.try_clone()?))
}
