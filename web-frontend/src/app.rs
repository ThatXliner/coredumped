//! Web application shell using xterm.js.

use std::cell::RefCell;
use std::rc::Rc;

use coredumped_core::game::{ActionCost, Intent, Mode};
use coredumped_core::render::render;
use coredumped_core::terminal::Frame;
use coredumped_core::world::World;

use wasm_bindgen::prelude::*;

use crate::storage;
use crate::XtermBridge;

struct WebState {
    world: World,
    frame: Frame,
    terminal: XtermBridge,
}

impl WebState {
    fn new(terminal: XtermBridge) -> Self {
        let cols = terminal.cols() as i32;
        let rows = terminal.rows() as i32;
        let frame = Frame::new(cols, rows);

        let world = if storage::has_save(0) {
            storage::load_world(0).unwrap_or_else(|e| {
                crate::log(&format!("Auto-load failed: {}", e));
                World::new_game()
            })
        } else {
            World::new_game()
        };

        Self {
            world,
            frame,
            terminal,
        }
    }

    fn tick(&mut self) {
        self.world.mark_visible_entities();
        self.world.mark_visible_tiles();
        self.world.refresh_rule_discovery();
        self.world.update_camera(
            coredumped_core::render::VIEWPORT_WIDTH,
            coredumped_core::render::VIEWPORT_HEIGHT,
        );

        self.frame.clear();
        self.world.render_frame = self.world.render_frame.wrapping_add(1);
        render(&mut self.frame, &self.world);
        self.flush_frame();
    }

    fn flush_frame(&self) {
        let output = self.frame.to_ansi_string();
        self.terminal.write(&output);
    }

    fn handle_key(&mut self, key: String) {
        let intent = parse_xterm_key(&key, &self.world);
        if !matches!(intent, Intent::Noop) {
            let cost = self.world.apply_intent(intent.clone());
            match cost {
                ActionCost::Quit => {}
                ActionCost::Tick => {
                    self.tick();
                    if let Err(e) = storage::save_world(0, &self.world) {
                        crate::log(&format!("Auto-save failed: {}", e));
                    }
                }
                ActionCost::Free => {
                    self.tick();
                    self.handle_save_load_intent(&intent);
                }
            }
            // Handle deferred intents (e.g., wipe confirmation)
            if let Some(deferred) = self.world.deferred_intent.take() {
                self.handle_save_load_intent(&deferred);
                self.tick();
            }
        }
    }

    fn handle_save_load_intent(&mut self, intent: &Intent) {
        match intent {
            Intent::SaveGame(slot) => {
                if let Err(e) = storage::save_world(*slot, &self.world) {
                    crate::log(&format!("Save failed: {}", e));
                }
            }
            Intent::LoadGame(slot) => match storage::load_world(*slot) {
                Ok(world) => {
                    self.world = world;
                    self.tick();
                }
                Err(e) => {
                    crate::log(&format!("Load failed: {}", e));
                }
            },
            Intent::WipeSave(slot) => {
                if storage::has_save(*slot) {
                    if let Err(e) = storage::delete_save(*slot) {
                        self.world
                            .event_log
                            .push(format!("Cannot delete save: {}", e));
                    } else {
                        self.world
                            .event_log
                            .push(format!("Save slot {} deleted.", slot));
                        self.world.quit_countdown = 3;
                    }
                } else {
                    self.world
                        .event_log
                        .push(format!("Save slot {} does not exist.", slot));
                }
            }
            _ => {}
        }
    }

    fn handle_resize(&mut self, cols: u32, rows: u32) {
        self.frame.resize(cols as i32, rows as i32);
        self.tick();
    }
}

fn parse_xterm_key(key: &str, world: &World) -> Intent {
    match world.mode {
        Mode::Normal => parse_normal_key(key, world),
        Mode::Inspector => parse_inspector_key(key),
        Mode::Keybindings => parse_keybindings_key(key),
        Mode::Memories => parse_memories_key(key),
        Mode::Console => parse_console_key(key),
        Mode::Dead => parse_dead_key(key),
    }
}

fn parse_normal_key(key: &str, world: &World) -> Intent {
    match key {
        "Escape" => {
            if world.bindings.contains_key("esc") {
                Intent::ExecuteBinding("esc".into())
            } else {
                Intent::Noop
            }
        }
        "ArrowUp" | "k" => {
            if world.bindings.contains_key("up") {
                Intent::ExecuteBinding("up".into())
            } else if world.bindings.contains_key("k") {
                Intent::ExecuteBinding("k".into())
            } else {
                Intent::Noop
            }
        }
        "ArrowDown" | "j" => {
            if world.bindings.contains_key("down") {
                Intent::ExecuteBinding("down".into())
            } else if world.bindings.contains_key("j") {
                Intent::ExecuteBinding("j".into())
            } else {
                Intent::Noop
            }
        }
        "ArrowLeft" | "h" => {
            if world.bindings.contains_key("left") {
                Intent::ExecuteBinding("left".into())
            } else if world.bindings.contains_key("h") {
                Intent::ExecuteBinding("h".into())
            } else {
                Intent::Noop
            }
        }
        "ArrowRight" | "l" => {
            if world.bindings.contains_key("right") {
                Intent::ExecuteBinding("right".into())
            } else if world.bindings.contains_key("l") {
                Intent::ExecuteBinding("l".into())
            } else {
                Intent::Noop
            }
        }
        "Enter" => {
            if world.bindings.contains_key("enter") {
                Intent::ExecuteBinding("enter".into())
            } else {
                Intent::Noop
            }
        }
        "Tab" => {
            if world.bindings.contains_key("tab") {
                Intent::ExecuteBinding("tab".into())
            } else {
                Intent::Noop
            }
        }
        "`" => Intent::ToggleConsole,
        c if c.len() == 1 => {
            let ch = c.chars().next().unwrap().to_ascii_lowercase();
            let binding_name = ch.to_string();
            if world.bindings.contains_key(&binding_name) {
                Intent::ExecuteBinding(binding_name)
            } else {
                Intent::Noop
            }
        }
        _ => Intent::Noop,
    }
}

fn parse_inspector_key(key: &str) -> Intent {
    match key {
        "Escape" | "i" | "I" => Intent::CloseOverlay,
        "`" => Intent::ToggleConsole,
        "ArrowUp" | "k" | "K" => Intent::InspectorScroll(-1),
        "ArrowDown" | "j" | "J" => Intent::InspectorScroll(1),
        "PageUp" => Intent::InspectorScroll(-8),
        "PageDown" => Intent::InspectorScroll(8),
        _ => Intent::Noop,
    }
}

fn parse_keybindings_key(key: &str) -> Intent {
    match key {
        "Escape" | "Tab" => Intent::CloseOverlay,
        "`" => Intent::ToggleConsole,
        "ArrowUp" | "k" | "K" => Intent::InspectorScroll(-1),
        "ArrowDown" | "j" | "J" => Intent::InspectorScroll(1),
        "PageUp" => Intent::InspectorScroll(-8),
        "PageDown" => Intent::InspectorScroll(8),
        _ => Intent::Noop,
    }
}

fn parse_memories_key(key: &str) -> Intent {
    match key {
        "Escape" | "m" | "M" => Intent::CloseOverlay,
        "`" => Intent::ToggleConsole,
        "ArrowUp" | "k" | "K" => Intent::InspectorScroll(-1),
        "ArrowDown" | "j" | "J" => Intent::InspectorScroll(1),
        "PageUp" => Intent::InspectorScroll(-8),
        "PageDown" => Intent::InspectorScroll(8),
        _ => Intent::Noop,
    }
}

fn parse_console_key(key: &str) -> Intent {
    match key {
        "Escape" => Intent::CloseOverlay,
        "`" => Intent::ToggleConsole,
        "Backspace" => Intent::ConsoleBackspace,
        "Delete" => Intent::ConsoleDelete,
        "Home" => Intent::ConsoleHome,
        "End" => Intent::ConsoleEnd,
        "Enter" => Intent::ConsoleSubmit,
        "ArrowUp" => Intent::ConsoleHistory(-1),
        "ArrowDown" => Intent::ConsoleHistory(1),
        "ArrowLeft" => Intent::ConsoleCursor(-1),
        "ArrowRight" => Intent::ConsoleCursor(1),
        "PageUp" => Intent::Scroll(-10),
        "PageDown" => Intent::Scroll(10),
        c if c.len() == 1 => {
            let ch = c.chars().next().unwrap();
            if ch.is_ascii() && !ch.is_control() {
                Intent::ConsoleInput(ch.to_ascii_lowercase())
            } else {
                Intent::Noop
            }
        }
        _ => Intent::Noop,
    }
}

fn parse_dead_key(key: &str) -> Intent {
    match key {
        "R" => Intent::Restart,
        "r" => Intent::Respawn,
        "Escape" | "q" | "Q" => Intent::Quit,
        _ => Intent::Noop,
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let terminal = XtermBridge::new("terminal");
    let state = Rc::new(RefCell::new(WebState::new(terminal)));

    // Initial render
    state.borrow_mut().tick();

    // Key callback
    let state_key = Rc::clone(&state);
    let key_callback = Closure::wrap(Box::new(move |key: String| {
        state_key.borrow_mut().handle_key(key);
    }) as Box<dyn FnMut(String)>);

    state.borrow().terminal.set_key_callback(&key_callback);
    key_callback.forget();

    // Resize callback
    let state_resize = Rc::clone(&state);
    let resize_callback = Closure::wrap(Box::new(move |cols: u32, rows: u32| {
        state_resize.borrow_mut().handle_resize(cols, rows);
    }) as Box<dyn FnMut(u32, u32)>);

    state
        .borrow()
        .terminal
        .set_resize_callback(&resize_callback);
    resize_callback.forget();

    Ok(())
}
