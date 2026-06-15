//! MIME type mapping.

use std::path::Path;

use crate::metadata::MediaKind;

/// Returns a MIME type from magic bytes or path extension.
#[must_use]
pub fn detect_mime(path: &Path, bytes: &[u8]) -> Option<(&'static str, MediaKind)> {
    detect_magic(bytes).or_else(|| detect_extension(path))
}

fn detect_magic(bytes: &[u8]) -> Option<(&'static str, MediaKind)> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(("image/png", MediaKind::Image));
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some(("image/jpeg", MediaKind::Image));
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(("image/gif", MediaKind::Image));
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return Some(("image/webp", MediaKind::Image));
    }
    if bytes.starts_with(b"BM") {
        return Some(("image/bmp", MediaKind::Image));
    }
    if bytes.starts_with(b"ID3") || bytes.starts_with(&[0xff, 0xfb]) {
        return Some(("audio/mpeg", MediaKind::Audio));
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        return Some(("audio/wav", MediaKind::Audio));
    }
    if bytes.starts_with(b"fLaC") {
        return Some(("audio/flac", MediaKind::Audio));
    }
    if bytes.get(4..8) == Some(b"ftyp") {
        return Some(("video/mp4", MediaKind::Video));
    }
    None
}

fn detect_extension(path: &Path) -> Option<(&'static str, MediaKind)> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("png") => Some(("image/png", MediaKind::Image)),
        Some("jpg" | "jpeg") => Some(("image/jpeg", MediaKind::Image)),
        Some("gif") => Some(("image/gif", MediaKind::Image)),
        Some("webp") => Some(("image/webp", MediaKind::Image)),
        Some("bmp") => Some(("image/bmp", MediaKind::Image)),
        Some("svg") => Some(("image/svg+xml", MediaKind::Image)),
        Some("mp3") => Some(("audio/mpeg", MediaKind::Audio)),
        Some("wav") => Some(("audio/wav", MediaKind::Audio)),
        Some("flac") => Some(("audio/flac", MediaKind::Audio)),
        Some("ogg") => Some(("audio/ogg", MediaKind::Audio)),
        Some("m4a") => Some(("audio/mp4", MediaKind::Audio)),
        Some("aac") => Some(("audio/aac", MediaKind::Audio)),
        Some("mp4") => Some(("video/mp4", MediaKind::Video)),
        Some("mov") => Some(("video/quicktime", MediaKind::Video)),
        Some("avi") => Some(("video/x-msvideo", MediaKind::Video)),
        Some("webm") => Some(("video/webm", MediaKind::Video)),
        Some("mkv") => Some(("video/x-matroska", MediaKind::Video)),
        _ => None,
    }
}
