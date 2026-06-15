//! Media metadata boundary for Rumon.

mod audio;
mod detector;
mod image;
mod metadata;
mod mime;
mod video;

pub use detector::detect_media;
pub use metadata::{
    AudioMetadata, ImageMetadata, MediaDetails, MediaKind, MediaMetadataChange, MediaSummary,
    VideoMetadata, compare_metadata,
};
