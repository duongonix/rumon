//! Ratatui terminal screen backend.

use std::io::{self, Stdout};

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::TuiApp;
use crate::ratatui_renderer::render_dashboard;

/// Ratatui terminal renderer for the Rumon dashboard.
#[derive(Debug)]
pub struct TerminalScreen {
    command: String,
    active: bool,
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
}

impl TerminalScreen {
    /// Creates a terminal screen for a command.
    #[must_use]
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            active: false,
            terminal: None,
        }
    }

    /// Enters the alternate screen.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when writing to stdout fails.
    pub fn enter(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.clear()?;
        self.active = true;
        self.terminal = Some(terminal);
        Ok(())
    }

    /// Draws the dashboard.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when writing to stdout fails.
    pub fn draw(&mut self, app: &TuiApp) -> io::Result<()> {
        let Some(terminal) = &mut self.terminal else {
            return Ok(());
        };
        terminal.draw(|frame| render_dashboard(frame, app, &self.command))?;
        Ok(())
    }

    /// Leaves the alternate screen.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when writing to stdout fails.
    pub fn leave(&mut self) -> io::Result<()> {
        if self.active {
            self.active = false;
            if let Some(mut terminal) = self.terminal.take() {
                terminal.show_cursor()?;
            }
            execute!(io::stdout(), Show, LeaveAlternateScreen)?;
        }
        Ok(())
    }
}

impl Drop for TerminalScreen {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}
