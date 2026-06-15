//! Terminal UI rendering and interaction model for Rumon.

mod app;
mod focus;
mod input;
mod keyboard;
mod layout;
mod ratatui_renderer;
mod screen;
mod terminal;
mod theme;

pub mod panels;
pub mod widgets;

pub use app::{TuiApp, TuiConfig};
pub use focus::{FocusTarget, next_focus, previous_focus};
pub use input::spawn_input_reader;
pub use keyboard::{KeyCommand, parse_key};
pub use layout::{Layout, Rect, split_layout};
pub use screen::TerminalScreen;
pub use terminal::{render_dashboard, terminal_size};
pub use theme::{ColorToken, Theme};
