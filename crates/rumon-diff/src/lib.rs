//! Diff engine boundary for Rumon.

mod binary;
mod column_diff;
mod formatter;
mod line_diff;
mod preview;
mod text;

pub use binary::{BinaryDiff, BinaryMetadata, diff_binary, metadata};
pub use column_diff::{ColumnChange, InlineDiff, inline_diff};
pub use formatter::{format_binary, format_lines};
pub use line_diff::{LineChange, diff_lines};
pub use preview::DiffPreview;
pub use text::{TextDiff, diff_text, is_text_path};
