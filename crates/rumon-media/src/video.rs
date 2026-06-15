//! Video metadata extraction.

use crate::metadata::VideoMetadata;

/// Extracts lightweight video metadata from headers.
#[must_use]
pub fn extract_video_metadata(mime_type: &str, bytes: &[u8]) -> VideoMetadata {
    match mime_type {
        "video/mp4" | "video/quicktime" => mp4_metadata(bytes),
        _ => VideoMetadata {
            width: None,
            height: None,
            duration_ms: None,
            codec: None,
            frame_rate: None,
        },
    }
}

fn mp4_metadata(bytes: &[u8]) -> VideoMetadata {
    VideoMetadata {
        width: None,
        height: None,
        duration_ms: None,
        codec: codec_from_brands(bytes),
        frame_rate: None,
    }
}

fn codec_from_brands(bytes: &[u8]) -> Option<String> {
    if bytes.windows(4).any(|window| window == b"avc1") {
        return Some("H264".to_string());
    }
    if bytes
        .windows(4)
        .any(|window| window == b"hvc1" || window == b"hev1")
    {
        return Some("H265".to_string());
    }
    None
}
