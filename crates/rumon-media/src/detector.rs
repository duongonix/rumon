//! Media detection and extraction pipeline.

use std::path::Path;

use crate::audio::extract_audio_metadata;
use crate::image::extract_image_metadata;
use crate::metadata::{MediaDetails, MediaKind, MediaSummary};
use crate::mime::detect_mime;
use crate::video::extract_video_metadata;

/// Detects and extracts media metadata from a path and bytes.
#[must_use]
pub fn detect_media(path: &Path, bytes: &[u8]) -> Option<MediaSummary> {
    let (mime_type, kind) = detect_mime(path, bytes)?;
    let details = match kind {
        MediaKind::Image => MediaDetails::Image(extract_image_metadata(mime_type, bytes)),
        MediaKind::Audio => MediaDetails::Audio(extract_audio_metadata(mime_type, bytes)),
        MediaKind::Video => MediaDetails::Video(extract_video_metadata(mime_type, bytes)),
    };
    Some(MediaSummary {
        kind,
        mime_type: mime_type.to_string(),
        file_size: bytes.len() as u64,
        details,
    })
}

#[cfg(test)]
mod tests {
    use super::detect_media;
    use crate::MediaKind;
    use std::path::Path;

    #[test]
    fn detects_png_from_magic_bytes() {
        let bytes = b"\x89PNG\r\n\x1a\n";
        let summary = detect_media(Path::new("file.bin"), bytes).expect("png should be detected");

        assert_eq!(summary.kind, MediaKind::Image);
        assert_eq!(summary.mime_type, "image/png");
    }
}
