//! Scroll helpers.

/// Clamps a scroll offset to a collection length.
#[must_use]
pub fn clamp_scroll(offset: usize, len: usize) -> usize {
    offset.min(len.saturating_sub(1))
}
