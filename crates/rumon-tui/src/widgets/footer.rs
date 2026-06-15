//! Footer widget.

use crate::theme::{ColorToken, bold_paint, paint};

/// Renders footer key bindings.
#[must_use]
pub fn render_footer() -> String {
    [
        key("q", "quit"),
        key("r", "restart"),
        key("c", "clear logs"),
        key("/", "search"),
        key("tab", "switch panel"),
        key("↑/↓", "navigate"),
        key("enter", "expand/collapse"),
    ]
    .join("   ")
}

fn key(binding: &str, action: &str) -> String {
    format!(
        "{} {}",
        paint(ColorToken::Info, binding),
        bold_paint(ColorToken::Text, action)
    )
}
