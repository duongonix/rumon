//! TUI focus handling.

/// TUI focus targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusTarget {
    /// The changes panel is focused.
    Changes,
    /// The logs panel is focused.
    Logs,
    /// Search input is focused.
    Search,
    /// Help overlay is focused.
    Help,
}

/// Returns the next focus target for Tab navigation.
#[must_use]
pub const fn next_focus(current: FocusTarget) -> FocusTarget {
    match current {
        FocusTarget::Changes => FocusTarget::Logs,
        FocusTarget::Logs | FocusTarget::Search | FocusTarget::Help => FocusTarget::Changes,
    }
}

/// Returns the previous focus target for Shift+Tab navigation.
#[must_use]
pub const fn previous_focus(current: FocusTarget) -> FocusTarget {
    match current {
        FocusTarget::Changes | FocusTarget::Search | FocusTarget::Help => FocusTarget::Logs,
        FocusTarget::Logs => FocusTarget::Changes,
    }
}

#[cfg(test)]
mod tests {
    use super::{FocusTarget, next_focus, previous_focus};

    #[test]
    fn focus_cycles_between_main_panels() {
        assert_eq!(next_focus(FocusTarget::Changes), FocusTarget::Logs);
        assert_eq!(next_focus(FocusTarget::Logs), FocusTarget::Changes);
        assert_eq!(previous_focus(FocusTarget::Changes), FocusTarget::Logs);
    }
}
