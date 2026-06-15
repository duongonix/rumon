//! Remote nodes panel rendering.

use crate::theme::{ColorToken, bold_paint, dim, paint};

/// Remote node summary rendered by the TUI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemotePanelNode {
    /// Node display name.
    pub name: String,
    /// Connection state label.
    pub state: String,
    /// Recent event count.
    pub events: usize,
    /// Recent log count.
    pub logs: usize,
}

/// Renders remote monitor nodes.
#[must_use]
pub fn render_remote_panel(nodes: &[RemotePanelNode], height: usize) -> Vec<String> {
    let mut lines = vec![
        bold_paint(ColorToken::Info, "Remote Nodes"),
        dim("Multi-machine monitor"),
    ];
    if nodes.is_empty() {
        lines.push(String::new());
        lines.push(dim("No remote nodes connected"));
        lines.push(dim("Run: rumon remote connect --token <TOKEN>"));
        return fit_height(lines, height);
    }

    for node in nodes {
        lines.push(format!(
            "{} {}  {}  {} events  {} logs",
            state_icon(&node.state),
            paint(ColorToken::Text, &node.name),
            state_label(&node.state),
            paint(ColorToken::Info, node.events.to_string()),
            paint(ColorToken::Muted, node.logs.to_string())
        ));
    }

    fit_height(lines, height)
}

fn state_icon(state: &str) -> String {
    match state {
        "connected" => paint(ColorToken::Success, "●"),
        "connecting" => paint(ColorToken::Warning, "●"),
        "disconnected" => paint(ColorToken::Muted, "●"),
        _ => paint(ColorToken::Error, "●"),
    }
}

fn state_label(state: &str) -> String {
    match state {
        "connected" => paint(ColorToken::Success, state),
        "connecting" => paint(ColorToken::Warning, state),
        "disconnected" => paint(ColorToken::Muted, state),
        _ => paint(ColorToken::Error, state),
    }
}

fn fit_height(mut lines: Vec<String>, height: usize) -> Vec<String> {
    lines.truncate(height);
    while lines.len() < height {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::{RemotePanelNode, render_remote_panel};

    #[test]
    fn renders_remote_node() {
        let output = render_remote_panel(
            &[RemotePanelNode {
                name: "node-a".to_string(),
                state: "connected".to_string(),
                events: 2,
                logs: 3,
            }],
            6,
        )
        .join("\n");

        assert!(output.contains("node-a"));
        assert!(output.contains("connected"));
    }
}
