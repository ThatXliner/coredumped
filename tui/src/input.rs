//! Keyboard translation for the bracket-lib frontend.
//!
//! Bracket-lib gives us raw key codes. This module maps those keys into game
//! intents while respecting the current UI mode, so movement keys can be tick
//! actions in normal mode and free navigation/input inside overlays.

use bracket_lib::prelude::VirtualKeyCode;

use crate::{
    entity::Direction,
    game::{Intent, Mode, World},
};

pub fn key_to_intent(key: VirtualKeyCode, world: &World) -> Intent {
    match world.mode {
        Mode::Normal => normal_key_to_intent(key),
        Mode::Inspector => inspector_key_to_intent(key),
        Mode::Console => console_key_to_intent(key),
    }
}

fn normal_key_to_intent(key: VirtualKeyCode) -> Intent {
    match key {
        VirtualKeyCode::Left | VirtualKeyCode::H => Intent::Move(Direction::West),
        VirtualKeyCode::Right | VirtualKeyCode::L => Intent::Move(Direction::East),
        VirtualKeyCode::Up | VirtualKeyCode::K => Intent::Move(Direction::North),
        VirtualKeyCode::Down | VirtualKeyCode::J => Intent::Move(Direction::South),
        VirtualKeyCode::Period => Intent::Wait,
        VirtualKeyCode::I => Intent::ToggleInspector,
        VirtualKeyCode::Grave => Intent::ToggleConsole,
        VirtualKeyCode::Escape | VirtualKeyCode::Q => Intent::Quit,
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

fn console_key_to_intent(key: VirtualKeyCode) -> Intent {
    match key {
        VirtualKeyCode::Escape => Intent::CloseOverlay,
        VirtualKeyCode::Grave => Intent::ToggleConsole,
        VirtualKeyCode::Back => Intent::ConsoleBackspace,
        VirtualKeyCode::Return => Intent::ConsoleSubmit,
        _ => key_to_console_char(key)
            .map(Intent::ConsoleInput)
            .unwrap_or(Intent::Noop),
    }
}

fn key_to_console_char(key: VirtualKeyCode) -> Option<char> {
    match key {
        VirtualKeyCode::A => Some('a'),
        VirtualKeyCode::B => Some('b'),
        VirtualKeyCode::C => Some('c'),
        VirtualKeyCode::D => Some('d'),
        VirtualKeyCode::E => Some('e'),
        VirtualKeyCode::F => Some('f'),
        VirtualKeyCode::G => Some('g'),
        VirtualKeyCode::H => Some('h'),
        VirtualKeyCode::I => Some('i'),
        VirtualKeyCode::J => Some('j'),
        VirtualKeyCode::K => Some('k'),
        VirtualKeyCode::L => Some('l'),
        VirtualKeyCode::M => Some('m'),
        VirtualKeyCode::N => Some('n'),
        VirtualKeyCode::O => Some('o'),
        VirtualKeyCode::P => Some('p'),
        VirtualKeyCode::Q => Some('q'),
        VirtualKeyCode::R => Some('r'),
        VirtualKeyCode::S => Some('s'),
        VirtualKeyCode::T => Some('t'),
        VirtualKeyCode::U => Some('u'),
        VirtualKeyCode::V => Some('v'),
        VirtualKeyCode::W => Some('w'),
        VirtualKeyCode::X => Some('x'),
        VirtualKeyCode::Y => Some('y'),
        VirtualKeyCode::Z => Some('z'),
        VirtualKeyCode::Key0 => Some('0'),
        VirtualKeyCode::Key1 => Some('1'),
        VirtualKeyCode::Key2 => Some('2'),
        VirtualKeyCode::Key3 => Some('3'),
        VirtualKeyCode::Key4 => Some('4'),
        VirtualKeyCode::Key5 => Some('5'),
        VirtualKeyCode::Key6 => Some('6'),
        VirtualKeyCode::Key7 => Some('7'),
        VirtualKeyCode::Key8 => Some('8'),
        VirtualKeyCode::Key9 => Some('9'),
        VirtualKeyCode::Space => Some(' '),
        VirtualKeyCode::Period => Some('.'),
        VirtualKeyCode::Comma => Some(','),
        VirtualKeyCode::Minus => Some('-'),
        VirtualKeyCode::Equals => Some('='),
        VirtualKeyCode::Semicolon => Some(';'),
        VirtualKeyCode::Apostrophe => Some('\''),
        VirtualKeyCode::Slash => Some('/'),
        VirtualKeyCode::Backslash => Some('\\'),
        VirtualKeyCode::LBracket => Some('['),
        VirtualKeyCode::RBracket => Some(']'),
        _ => None,
    }
}
