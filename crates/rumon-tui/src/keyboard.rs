//! Keyboard command mapping.

/// TUI commands generated from keyboard input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyCommand {
    /// Quit Rumon.
    Quit,
    /// Restart the process.
    Restart,
    /// Clear logs.
    ClearLogs,
    /// Start or focus search.
    Search,
    /// Move to the next search match.
    NextMatch,
    /// Move to the previous search match.
    PreviousMatch,
    /// Switch focus to the next panel.
    NextPanel,
    /// Switch focus to the previous panel.
    PreviousPanel,
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll one page up.
    PageUp,
    /// Scroll one page down.
    PageDown,
    /// Jump to the top.
    Home,
    /// Jump to the bottom and resume following logs.
    End,
    /// Expand or collapse the selected change block.
    ToggleSelected,
    /// Toggle help.
    ToggleHelp,
    /// Cancel active overlay.
    Cancel,
}

/// Parses a key label into a TUI command.
#[must_use]
pub fn parse_key(key: &str) -> Option<KeyCommand> {
    match key {
        "q" | "ctrl+c" => Some(KeyCommand::Quit),
        "r" => Some(KeyCommand::Restart),
        "c" => Some(KeyCommand::ClearLogs),
        "/" => Some(KeyCommand::Search),
        "n" => Some(KeyCommand::NextMatch),
        "N" | "shift+n" => Some(KeyCommand::PreviousMatch),
        "tab" => Some(KeyCommand::NextPanel),
        "shift+tab" => Some(KeyCommand::PreviousPanel),
        "up" => Some(KeyCommand::ScrollUp),
        "down" => Some(KeyCommand::ScrollDown),
        "pgup" => Some(KeyCommand::PageUp),
        "pgdn" => Some(KeyCommand::PageDown),
        "home" => Some(KeyCommand::Home),
        "end" => Some(KeyCommand::End),
        "enter" => Some(KeyCommand::ToggleSelected),
        "?" => Some(KeyCommand::ToggleHelp),
        "esc" => Some(KeyCommand::Cancel),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyCommand, parse_key};

    #[test]
    fn parses_core_shortcuts() {
        assert_eq!(parse_key("q"), Some(KeyCommand::Quit));
        assert_eq!(parse_key("tab"), Some(KeyCommand::NextPanel));
        assert_eq!(parse_key("unknown"), None);
    }
}
