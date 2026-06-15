//! Media metadata models.

use std::fmt::{self, Display, Formatter};

/// Supported media families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaKind {
    /// Image files such as PNG, JPEG, GIF, WebP, BMP, and SVG.
    Image,
    /// Audio files such as MP3, WAV, FLAC, OGG, M4A, and AAC.
    Audio,
    /// Video files such as MP4, MOV, AVI, `WebM`, and MKV.
    Video,
}

impl Display for MediaKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image => formatter.write_str("image"),
            Self::Audio => formatter.write_str("audio"),
            Self::Video => formatter.write_str("video"),
        }
    }
}

/// Image metadata extracted from file headers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageMetadata {
    /// Width in pixels.
    pub width: Option<u32>,
    /// Height in pixels.
    pub height: Option<u32>,
    /// Color information when known.
    pub color_type: Option<String>,
}

/// Audio metadata summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioMetadata {
    /// Duration in milliseconds when known.
    pub duration_ms: Option<u64>,
    /// Bitrate in bits per second when known.
    pub bitrate: Option<u32>,
    /// Channel count when known.
    pub channels: Option<u16>,
    /// Sample rate in hertz when known.
    pub sample_rate: Option<u32>,
}

/// Video metadata summary.
#[derive(Clone, Debug, PartialEq)]
pub struct VideoMetadata {
    /// Width in pixels when known.
    pub width: Option<u32>,
    /// Height in pixels when known.
    pub height: Option<u32>,
    /// Duration in milliseconds when known.
    pub duration_ms: Option<u64>,
    /// Codec name when known.
    pub codec: Option<String>,
    /// Frame rate when known.
    pub frame_rate: Option<f32>,
}

/// Detailed metadata for a media file.
#[derive(Clone, Debug, PartialEq)]
pub enum MediaDetails {
    /// Image details.
    Image(ImageMetadata),
    /// Audio details.
    Audio(AudioMetadata),
    /// Video details.
    Video(VideoMetadata),
}

/// Lightweight media summary for UI display.
#[derive(Clone, Debug, PartialEq)]
pub struct MediaSummary {
    /// Media family.
    pub kind: MediaKind,
    /// MIME type when known.
    pub mime_type: String,
    /// File size in bytes.
    pub file_size: u64,
    /// Detailed metadata.
    pub details: MediaDetails,
}

/// Metadata comparison result.
#[derive(Clone, Debug, PartialEq)]
pub struct MediaMetadataChange {
    /// Previous metadata.
    pub previous: Option<MediaSummary>,
    /// Current metadata.
    pub current: Option<MediaSummary>,
    /// Whether file size changed.
    pub file_size_changed: bool,
    /// Human-readable summary lines.
    pub summary_lines: Vec<String>,
}

impl MediaSummary {
    /// Returns a concise display summary.
    #[must_use]
    pub fn display_lines(&self) -> Vec<String> {
        let mut lines = vec![
            self.kind.to_string(),
            self.mime_type.clone(),
            format!("{} bytes", self.file_size),
        ];
        match &self.details {
            MediaDetails::Image(image) => {
                if let (Some(width), Some(height)) = (image.width, image.height) {
                    lines.push(format!("{width}x{height}"));
                }
                if let Some(color_type) = &image.color_type {
                    lines.push(color_type.clone());
                }
            }
            MediaDetails::Audio(audio) => {
                push_optional(&mut lines, "duration_ms", audio.duration_ms);
                push_optional(&mut lines, "bitrate", audio.bitrate);
                push_optional(&mut lines, "channels", audio.channels);
                push_optional(&mut lines, "sample_rate", audio.sample_rate);
            }
            MediaDetails::Video(video) => {
                if let (Some(width), Some(height)) = (video.width, video.height) {
                    lines.push(format!("{width}x{height}"));
                }
                push_optional(&mut lines, "duration_ms", video.duration_ms);
                if let Some(codec) = &video.codec {
                    lines.push(format!("codec {codec}"));
                }
                if let Some(frame_rate) = video.frame_rate {
                    lines.push(format!("frame_rate {frame_rate:.2}"));
                }
            }
        }
        lines
    }
}

/// Compares previous and current metadata.
#[must_use]
pub fn compare_metadata(
    previous: Option<MediaSummary>,
    current: Option<MediaSummary>,
) -> MediaMetadataChange {
    let file_size_changed = previous.as_ref().map(|summary| summary.file_size)
        != current.as_ref().map(|summary| summary.file_size);
    let mut summary_lines = Vec::new();
    match (&previous, &current) {
        (Some(previous), Some(current)) => {
            summary_lines.push(format!("{} -> {}", previous.mime_type, current.mime_type));
            if file_size_changed {
                summary_lines.push(format!(
                    "{} bytes -> {} bytes",
                    previous.file_size, current.file_size
                ));
            }
        }
        (None, Some(current)) => {
            summary_lines.push(format!("created {}", current.mime_type));
            summary_lines.push(format!("{} bytes", current.file_size));
        }
        (Some(previous), None) => {
            summary_lines.push(format!("deleted {}", previous.mime_type));
        }
        (None, None) => {}
    }

    MediaMetadataChange {
        previous,
        current,
        file_size_changed,
        summary_lines,
    }
}

fn push_optional<T: Display>(lines: &mut Vec<String>, label: &str, value: Option<T>) {
    if let Some(value) = value {
        lines.push(format!("{label} {value}"));
    }
}

#[cfg(test)]
mod tests {
    use super::{ImageMetadata, MediaDetails, MediaKind, MediaSummary, compare_metadata};

    #[test]
    fn summary_records_kind() {
        let summary = MediaSummary {
            kind: MediaKind::Image,
            mime_type: "image/png".to_string(),
            file_size: 42,
            details: MediaDetails::Image(ImageMetadata {
                width: Some(1),
                height: Some(1),
                color_type: None,
            }),
        };

        assert_eq!(summary.kind, MediaKind::Image);
    }

    #[test]
    fn comparison_reports_size_change() {
        let previous = MediaSummary {
            kind: MediaKind::Image,
            mime_type: "image/png".to_string(),
            file_size: 42,
            details: MediaDetails::Image(ImageMetadata {
                width: None,
                height: None,
                color_type: None,
            }),
        };
        let current = MediaSummary {
            file_size: 50,
            ..previous.clone()
        };

        let change = compare_metadata(Some(previous), Some(current));

        assert!(change.file_size_changed);
    }
}
