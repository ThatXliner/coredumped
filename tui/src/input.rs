//! Keyboard translation for the crossterm frontend.
//!
//! This module maps raw terminal key events into game intents while respecting
//! the current UI mode.

use coredumped_core::game::{Intent, Mode};
use coredumped_core::world::World;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn key_to_intent(event: KeyEvent, world: &World) -> Intent {
    let shift = event.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
    let alt = event.modifiers.contains(KeyModifiers::ALT);

    match world.mode {
        Mode::Normal => {
            match event.code {
                KeyCode::F(5) => return Intent::SaveGame(1),
                KeyCode::F(9) => return Intent::LoadGame(1),
                KeyCode::PageUp => return Intent::Scroll(-10),
                KeyCode::PageDown => return Intent::Scroll(10),
                _ => {}
            }
            if let Some(name) = key_to_binding_name(event.code) {
                if world.bindings.contains_key(&name) {
                    return Intent::ExecuteBinding(name);
                }
            }
            Intent::Noop
        }
        Mode::Inspector => inspector_key_to_intent(event.code),
        Mode::Keybindings => keybindings_key_to_intent(event.code),
        Mode::Memories => memories_key_to_intent(event.code),
        Mode::ReadingSign => sign_key_to_intent(event.code),
        Mode::Console => console_key_to_intent(event.code, shift, ctrl, alt),
        Mode::Dead => dead_key_to_intent(event.code, shift),
    }
}

fn key_to_binding_name(key: KeyCode) -> Option<String> {
    match key {
        KeyCode::Esc => Some("esc".into()),
        KeyCode::Tab | KeyCode::BackTab => Some("tab".into()),
        KeyCode::Left => Some("left".into()),
        KeyCode::Right => Some("right".into()),
        KeyCode::Up => Some("up".into()),
        KeyCode::Down => Some("down".into()),
        KeyCode::Enter => Some("enter".into()),
        KeyCode::Backspace => Some("backspace".into()),
        KeyCode::Char(c) => Some(binding_char(c).to_string()),
        _ => None,
    }
}

fn sign_key_to_intent(key: KeyCode) -> Intent {
    match key {
        KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Enter => Intent::CloseOverlay,
        KeyCode::PageUp => Intent::InspectorScroll(-8),
        KeyCode::PageDown => Intent::InspectorScroll(8),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => Intent::InspectorScroll(-1),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => Intent::InspectorScroll(1),
        _ => Intent::Noop,
    }
}

fn keybindings_key_to_intent(key: KeyCode) -> Intent {
    match key {
        KeyCode::Esc | KeyCode::Tab | KeyCode::BackTab => Intent::CloseOverlay,
        KeyCode::Char('`') => Intent::ToggleConsole,
        KeyCode::PageUp => Intent::InspectorScroll(-8),
        KeyCode::PageDown => Intent::InspectorScroll(8),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => Intent::InspectorScroll(-1),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => Intent::InspectorScroll(1),
        _ => Intent::Noop,
    }
}

fn memories_key_to_intent(key: KeyCode) -> Intent {
    match key {
        KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('M') => Intent::CloseOverlay,
        KeyCode::Char('`') => Intent::ToggleConsole,
        KeyCode::PageUp => Intent::InspectorScroll(-8),
        KeyCode::PageDown => Intent::InspectorScroll(8),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => Intent::InspectorScroll(-1),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => Intent::InspectorScroll(1),
        _ => Intent::Noop,
    }
}

fn inspector_key_to_intent(key: KeyCode) -> Intent {
    match key {
        KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('I') => Intent::CloseOverlay,
        KeyCode::Char('`') => Intent::ToggleConsole,
        KeyCode::PageUp => Intent::InspectorScroll(-8),
        KeyCode::PageDown => Intent::InspectorScroll(8),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => Intent::InspectorScroll(-1),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => Intent::InspectorScroll(1),
        _ => Intent::Noop,
    }
}

fn console_key_to_intent(key: KeyCode, shift: bool, ctrl: bool, alt: bool) -> Intent {
    if ctrl && matches!(key, KeyCode::Char('e') | KeyCode::Char('E')) {
        return Intent::OpenExternalEditor;
    }

    match key {
        KeyCode::Esc => Intent::CloseOverlay,
        KeyCode::Char('`') => Intent::ToggleConsole,
        KeyCode::Backspace if alt => Intent::ConsoleBackspaceWord,
        KeyCode::Backspace => {
            if ctrl {
                Intent::ConsoleBackspaceWord
            } else {
                Intent::ConsoleBackspace
            }
        }
        KeyCode::Delete => Intent::ConsoleDelete,
        KeyCode::Home => Intent::ConsoleHome,
        KeyCode::End => Intent::ConsoleEnd,
        KeyCode::PageUp => Intent::Scroll(-10),
        KeyCode::PageDown => Intent::Scroll(10),
        KeyCode::Enter if shift => Intent::ConsoleNewline,
        KeyCode::Enter => Intent::ConsoleSubmit,
        KeyCode::Up => Intent::ConsoleHistory(-1),
        KeyCode::Down => Intent::ConsoleHistory(1),
        KeyCode::Left if ctrl || alt => Intent::ConsoleMoveWord(-1),
        KeyCode::Right if ctrl || alt => Intent::ConsoleMoveWord(1),
        KeyCode::Left => Intent::ConsoleCursor(-1),
        KeyCode::Right => Intent::ConsoleCursor(1),
        KeyCode::Char('a') | KeyCode::Char('A') if ctrl => Intent::ConsoleHome,
        KeyCode::Char('u') | KeyCode::Char('U') if ctrl => Intent::ConsoleKillToStart,
        KeyCode::Char('k') | KeyCode::Char('K') if ctrl => Intent::ConsoleKillToEnd,
        KeyCode::Char('w') | KeyCode::Char('W') if ctrl => Intent::ConsoleBackspaceWord,
        KeyCode::Char('b') | KeyCode::Char('B') if alt => Intent::ConsoleMoveWord(-1),
        KeyCode::Char('f') | KeyCode::Char('F') if alt => Intent::ConsoleMoveWord(1),
        KeyCode::Char(c) if !ctrl => Intent::ConsoleInput(console_char(c)),
        _ => Intent::Noop,
    }
}

fn dead_key_to_intent(key: KeyCode, shift: bool) -> Intent {
    match key {
        KeyCode::Char('R') => Intent::Restart,
        KeyCode::Char('r') if shift => Intent::Restart,
        KeyCode::Char('r') => Intent::Respawn,
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => Intent::Quit,
        _ => Intent::Noop,
    }
}

fn binding_char(c: char) -> char {
    if c.is_ascii_alphabetic() {
        c.to_ascii_lowercase()
    } else {
        c
    }
}

fn console_char(c: char) -> char {
    if c.is_ascii_uppercase() {
        c.to_ascii_lowercase()
    } else {
        c
    }
}
