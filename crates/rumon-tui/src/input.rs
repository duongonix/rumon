//! TUI input command stream.

use std::io;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::keyboard::{KeyCommand, parse_key};

/// Starts a background input reader that emits TUI commands.
#[must_use]
pub fn spawn_input_reader() -> Receiver<KeyCommand> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _raw_mode = RawModeGuard::enter().ok();
        loop {
            match read_key_command() {
                Ok(Some(command)) => {
                    if sender.send(command).is_err() {
                        break;
                    }
                }
                Ok(None) => {}
                Err(_) => break,
            }
        }
    });
    receiver
}

fn read_key_command() -> io::Result<Option<KeyCommand>> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(None);
    }
    let Event::Key(key) = event::read()? else {
        return Ok(None);
    };
    Ok(command_from_key(key))
}

fn command_from_key(key: KeyEvent) -> Option<KeyCommand> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(KeyCommand::Quit)
        }
        KeyCode::Char(character) => parse_key(&character.to_string()),
        KeyCode::Tab => Some(KeyCommand::NextPanel),
        KeyCode::BackTab => Some(KeyCommand::PreviousPanel),
        KeyCode::Up => Some(KeyCommand::ScrollUp),
        KeyCode::Down => Some(KeyCommand::ScrollDown),
        KeyCode::PageUp => Some(KeyCommand::PageUp),
        KeyCode::PageDown => Some(KeyCommand::PageDown),
        KeyCode::Home => Some(KeyCommand::Home),
        KeyCode::End => Some(KeyCommand::End),
        KeyCode::Enter => Some(KeyCommand::ToggleSelected),
        KeyCode::Esc => Some(KeyCommand::Cancel),
        _ => None,
    }
}

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::command_from_key;
    use crate::KeyCommand;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    #[test]
    fn maps_arrow_keys_to_scroll_commands() {
        assert_eq!(
            command_from_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Some(KeyCommand::ScrollUp)
        );
        assert_eq!(
            command_from_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Some(KeyCommand::ScrollDown)
        );
    }

    #[test]
    fn maps_enter_to_toggle_selected() {
        assert_eq!(
            command_from_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(KeyCommand::ToggleSelected)
        );
    }

    #[test]
    fn ignores_key_release_events() {
        assert_eq!(
            command_from_key(KeyEvent::new_with_kind(
                KeyCode::Down,
                KeyModifiers::NONE,
                KeyEventKind::Release
            )),
            None
        );
    }
}
