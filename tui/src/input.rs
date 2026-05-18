//! Keyboard translation for the bracket-lib frontend.
//!
//! Bracket-lib gives us raw key codes. This module maps those keys into game
//! intents while respecting the current UI mode, so movement keys can be tick
//! actions in normal mode and free navigation/input inside overlays.

use bracket_lib::prelude::VirtualKeyCode;

use crate::{
    game::{Intent, Mode},
    world::World,
};

pub fn key_to_intent(key: VirtualKeyCode, shift: bool, world: &World) -> Intent {
    match world.mode {
        Mode::Normal => {
            if let Some(name) = key_to_binding_name(key, shift) {
                if world.bindings.contains_key(&name) {
                    return Intent::ExecuteBinding(name);
                }
            }
            Intent::Noop
        }
        Mode::Inspector => inspector_key_to_intent(key),
        Mode::Keybindings => keybindings_key_to_intent(key),
        Mode::Console => console_key_to_intent(key, shift),
        Mode::Dead => dead_key_to_intent(key, shift),
    }
}

fn key_to_binding_name(key: VirtualKeyCode, shift: bool) -> Option<String> {
    match key {
        VirtualKeyCode::Escape => Some("esc".into()),
        VirtualKeyCode::Tab => Some("tab".into()),
        VirtualKeyCode::Grave => Some("`".into()),
        VirtualKeyCode::Left => Some("left".into()),
        VirtualKeyCode::Right => Some("right".into()),
        VirtualKeyCode::Up => Some("up".into()),
        VirtualKeyCode::Down => Some("down".into()),
        VirtualKeyCode::Return => Some("enter".into()),
        VirtualKeyCode::Back => Some("backspace".into()),
        VirtualKeyCode::Space => Some(" ".into()),
        _ => key_to_console_char(key, shift).map(|c| c.to_string()),
    }
}

fn keybindings_key_to_intent(key: VirtualKeyCode) -> Intent {
    match key {
        VirtualKeyCode::Escape | VirtualKeyCode::Tab => Intent::CloseOverlay,
        VirtualKeyCode::Grave => Intent::ToggleConsole,
        VirtualKeyCode::Up | VirtualKeyCode::K => Intent::InspectorScroll(-1),
        VirtualKeyCode::Down | VirtualKeyCode::J => Intent::InspectorScroll(1),
        _ => Intent::Noop,
    }
}

fn inspector_key_to_intent(key: VirtualKeyCode) -> Intent {
    match key {
        VirtualKeyCode::Escape | VirtualKeyCode::I => Intent::CloseOverlay,
        VirtualKeyCode::Grave => Intent::ToggleConsole,
        VirtualKeyCode::Up | VirtualKeyCode::K => Intent::InspectorScroll(-1),
        VirtualKeyCode::Down | VirtualKeyCode::J => Intent::InspectorScroll(1),
        _ => Intent::Noop,
    }
}

fn console_key_to_intent(key: VirtualKeyCode, shift: bool) -> Intent {
    match (key, shift) {
        (VirtualKeyCode::Escape, _) => Intent::CloseOverlay,
        (VirtualKeyCode::Grave, _) => Intent::ToggleConsole,
        (VirtualKeyCode::Back, _) => Intent::ConsoleBackspace,
        (VirtualKeyCode::Return, false) => Intent::ConsoleSubmit,
        (VirtualKeyCode::Return, true) => Intent::ConsoleNewline,
        (VirtualKeyCode::Up, _) => Intent::ConsoleScroll(-1),
        (VirtualKeyCode::Down, _) => Intent::ConsoleScroll(1),
        _ => key_to_console_char(key, shift)
            .map(Intent::ConsoleInput)
            .unwrap_or(Intent::Noop),
    }
}

fn dead_key_to_intent(key: VirtualKeyCode, shift: bool) -> Intent {
    match (key, shift) {
        (VirtualKeyCode::R, false) => Intent::Respawn,
        (VirtualKeyCode::R, true) => Intent::Restart,
        (VirtualKeyCode::Escape, _) | (VirtualKeyCode::Q, _) => Intent::Quit,
        _ => Intent::Noop,
    }
}

fn key_to_console_char(key: VirtualKeyCode, shift: bool) -> Option<char> {
    match (key, shift) {
        (VirtualKeyCode::A, _) => Some('a'),
        (VirtualKeyCode::B, _) => Some('b'),
        (VirtualKeyCode::C, _) => Some('c'),
        (VirtualKeyCode::D, _) => Some('d'),
        (VirtualKeyCode::E, _) => Some('e'),
        (VirtualKeyCode::F, _) => Some('f'),
        (VirtualKeyCode::G, _) => Some('g'),
        (VirtualKeyCode::H, _) => Some('h'),
        (VirtualKeyCode::I, _) => Some('i'),
        (VirtualKeyCode::J, _) => Some('j'),
        (VirtualKeyCode::K, _) => Some('k'),
        (VirtualKeyCode::L, _) => Some('l'),
        (VirtualKeyCode::M, _) => Some('m'),
        (VirtualKeyCode::N, _) => Some('n'),
        (VirtualKeyCode::O, _) => Some('o'),
        (VirtualKeyCode::P, _) => Some('p'),
        (VirtualKeyCode::Q, _) => Some('q'),
        (VirtualKeyCode::R, _) => Some('r'),
        (VirtualKeyCode::S, _) => Some('s'),
        (VirtualKeyCode::T, _) => Some('t'),
        (VirtualKeyCode::U, _) => Some('u'),
        (VirtualKeyCode::V, _) => Some('v'),
        (VirtualKeyCode::W, _) => Some('w'),
        (VirtualKeyCode::X, _) => Some('x'),
        (VirtualKeyCode::Y, _) => Some('y'),
        (VirtualKeyCode::Z, _) => Some('z'),
        (VirtualKeyCode::Key0, false) => Some('0'),
        (VirtualKeyCode::Key0, true) => Some(')'),
        (VirtualKeyCode::Key1, false) => Some('1'),
        (VirtualKeyCode::Key1, true) => Some('!'),
        (VirtualKeyCode::Key2, false) => Some('2'),
        (VirtualKeyCode::Key2, true) => Some('@'),
        (VirtualKeyCode::Key3, false) => Some('3'),
        (VirtualKeyCode::Key3, true) => Some('#'),
        (VirtualKeyCode::Key4, false) => Some('4'),
        (VirtualKeyCode::Key4, true) => Some('$'),
        (VirtualKeyCode::Key5, false) => Some('5'),
        (VirtualKeyCode::Key5, true) => Some('%'),
        (VirtualKeyCode::Key6, false) => Some('6'),
        (VirtualKeyCode::Key6, true) => Some('^'),
        (VirtualKeyCode::Key7, false) => Some('7'),
        (VirtualKeyCode::Key7, true) => Some('&'),
        (VirtualKeyCode::Key8, false) => Some('8'),
        (VirtualKeyCode::Key8, true) => Some('*'),
        (VirtualKeyCode::Key9, false) => Some('9'),
        (VirtualKeyCode::Key9, true) => Some('('),
        (VirtualKeyCode::Space, _) => Some(' '),
        (VirtualKeyCode::Period, false) => Some('.'),
        (VirtualKeyCode::Period, true) => Some('>'),
        (VirtualKeyCode::Comma, false) => Some(','),
        (VirtualKeyCode::Comma, true) => Some('<'),
        (VirtualKeyCode::Minus, false) => Some('-'),
        (VirtualKeyCode::Minus, true) => Some('_'),
        (VirtualKeyCode::Equals, false) => Some('='),
        (VirtualKeyCode::Equals, true) => Some('+'),
        (VirtualKeyCode::Semicolon, false) => Some(';'),
        (VirtualKeyCode::Semicolon, true) => Some(':'),
        (VirtualKeyCode::Apostrophe, false) => Some('\''),
        (VirtualKeyCode::Apostrophe, true) => Some('"'),
        (VirtualKeyCode::Slash, false) => Some('/'),
        (VirtualKeyCode::Slash, true) => Some('?'),
        (VirtualKeyCode::Backslash, false) => Some('\\'),
        (VirtualKeyCode::Backslash, true) => Some('|'),
        (VirtualKeyCode::LBracket, false) => Some('['),
        (VirtualKeyCode::LBracket, true) => Some('{'),
        (VirtualKeyCode::RBracket, false) => Some(']'),
        (VirtualKeyCode::RBracket, true) => Some('}'),
        _ => None,
    }
}
