//! TUI state and command application.

use std::collections::BTreeSet;

use rumon_shared::AppState;

use crate::focus::{FocusTarget, next_focus, previous_focus};
use crate::keyboard::KeyCommand;

/// TUI runtime configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiConfig {
    /// Left panel width percentage.
    pub left_panel_width: u16,
    /// Whether timestamps are rendered in logs.
    pub show_timestamp: bool,
    /// Whether logs follow the latest entry.
    pub auto_scroll_logs: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            left_panel_width: 50,
            show_timestamp: true,
            auto_scroll_logs: true,
        }
    }
}

/// TUI application state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiApp {
    /// Shared application state snapshot.
    pub state: AppState,
    /// Active focus target.
    pub focus: FocusTarget,
    /// Selected change index.
    pub selected_change: usize,
    /// First visible change block index.
    pub change_scroll: usize,
    /// Collapsed change block indexes.
    pub collapsed_changes: BTreeSet<usize>,
    /// Log scroll offset.
    pub log_scroll: usize,
    /// Whether logs auto-scroll.
    pub follow_logs: bool,
    /// Optional search query.
    pub search_query: Option<String>,
    /// Whether help is visible.
    pub help_visible: bool,
    /// TUI config.
    pub config: TuiConfig,
}

impl TuiApp {
    /// Creates a TUI app from shared state.
    #[must_use]
    pub fn new(state: AppState, config: TuiConfig) -> Self {
        Self {
            state,
            focus: FocusTarget::Changes,
            selected_change: 0,
            change_scroll: 0,
            collapsed_changes: BTreeSet::new(),
            log_scroll: 0,
            follow_logs: config.auto_scroll_logs,
            search_query: None,
            help_visible: false,
            config,
        }
    }

    /// Replaces the shared state snapshot and keeps selection/scroll ergonomic.
    pub fn sync_state(&mut self, state: AppState) {
        self.state = state;
        self.selected_change = self
            .selected_change
            .min(self.state.changes.len().saturating_sub(1));
        if self.state.changes.is_empty() {
            self.change_scroll = 0;
        }
        self.collapsed_changes
            .retain(|index| *index < self.state.changes.len());
        if self.follow_logs {
            self.log_scroll = self.state.logs.len().saturating_sub(1);
        }
    }

    /// Applies a keyboard command and returns whether the process should quit.
    #[must_use]
    pub fn apply_command(&mut self, command: KeyCommand) -> bool {
        match command {
            KeyCommand::Quit => return true,
            KeyCommand::NextPanel => self.focus = next_focus(self.focus),
            KeyCommand::PreviousPanel => self.focus = previous_focus(self.focus),
            KeyCommand::ScrollUp => self.scroll_up(),
            KeyCommand::ScrollDown => self.scroll_down(),
            KeyCommand::PageUp => self.page_up(),
            KeyCommand::PageDown => self.page_down(),
            KeyCommand::Home => self.jump_home(),
            KeyCommand::End => self.jump_end(),
            KeyCommand::ToggleSelected => self.toggle_selected_change(),
            KeyCommand::Search => {
                self.focus = FocusTarget::Search;
                self.search_query.get_or_insert_with(String::new);
            }
            KeyCommand::ToggleHelp => {
                self.help_visible = !self.help_visible;
                self.focus = if self.help_visible {
                    FocusTarget::Help
                } else {
                    FocusTarget::Changes
                };
            }
            KeyCommand::Cancel => {
                self.help_visible = false;
                self.search_query = None;
                self.focus = FocusTarget::Changes;
            }
            KeyCommand::Restart
            | KeyCommand::ClearLogs
            | KeyCommand::NextMatch
            | KeyCommand::PreviousMatch => {}
        }
        false
    }

    fn scroll_up(&mut self) {
        match self.focus {
            FocusTarget::Changes => {
                self.change_scroll = self.change_scroll.saturating_sub(1);
            }
            FocusTarget::Logs | FocusTarget::Search | FocusTarget::Help => {
                self.follow_logs = false;
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
        }
    }

    fn scroll_down(&mut self) {
        match self.focus {
            FocusTarget::Changes => {
                self.change_scroll = self.change_scroll.saturating_add(1);
            }
            FocusTarget::Logs | FocusTarget::Search | FocusTarget::Help => {
                self.log_scroll = self.log_scroll.saturating_add(1);
            }
        }
    }

    fn page_up(&mut self) {
        match self.focus {
            FocusTarget::Changes => {
                self.change_scroll = self.change_scroll.saturating_sub(10);
            }
            FocusTarget::Logs | FocusTarget::Search | FocusTarget::Help => {
                self.follow_logs = false;
                self.log_scroll = self.log_scroll.saturating_sub(10);
            }
        }
    }

    fn page_down(&mut self) {
        match self.focus {
            FocusTarget::Changes => {
                self.change_scroll = self.change_scroll.saturating_add(10);
            }
            FocusTarget::Logs | FocusTarget::Search | FocusTarget::Help => {
                self.log_scroll = self.log_scroll.saturating_add(10);
            }
        }
    }

    fn jump_home(&mut self) {
        match self.focus {
            FocusTarget::Changes => {
                self.selected_change = 0;
                self.change_scroll = 0;
            }
            FocusTarget::Logs | FocusTarget::Search | FocusTarget::Help => {
                self.follow_logs = false;
                self.log_scroll = 0;
            }
        }
    }

    fn jump_end(&mut self) {
        match self.focus {
            FocusTarget::Changes => {
                self.selected_change = self.state.changes.len().saturating_sub(1);
                self.change_scroll = usize::MAX;
            }
            FocusTarget::Logs | FocusTarget::Search | FocusTarget::Help => {
                self.follow_logs = true;
                self.log_scroll = self.state.logs.len().saturating_sub(1);
            }
        }
    }

    fn toggle_selected_change(&mut self) {
        if self.focus != FocusTarget::Changes || self.state.changes.is_empty() {
            return;
        }
        if !self.collapsed_changes.remove(&self.selected_change) {
            self.collapsed_changes.insert(self.selected_change);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TuiApp, TuiConfig};
    use crate::{FocusTarget, KeyCommand};
    use rumon_shared::AppState;

    #[test]
    fn tab_switches_focus() {
        let mut app = TuiApp::new(AppState::default(), TuiConfig::default());

        let _ = app.apply_command(KeyCommand::NextPanel);

        assert_eq!(app.focus, FocusTarget::Logs);
    }

    #[test]
    fn changes_scroll_is_line_based() {
        let mut state = AppState::default();
        state.changes.push(rumon_shared::FileChange {
            path: "src/main.rs".into(),
            previous_path: None,
            kind: rumon_shared::ChangeKind::Modified,
            is_directory: false,
            detail: None,
        });
        let mut app = TuiApp::new(state, TuiConfig::default());

        let _ = app.apply_command(KeyCommand::ScrollDown);

        assert_eq!(app.change_scroll, 1);
    }
}
