//! The `World` struct — the central game state.
//!
//! Defined here so the glyph module can reference it in `BuiltinFn`'s
//! signature without creating circular `use` confusion.

use std::collections::HashMap;

use bracket_lib::prelude::RGB;

use crate::{
    ecs::Ecs,
    entity::{Direction, EntityId},
    event_log::EventLog,
    game::Mode,
    glyph::Env,
    map::Map,
    rules::RuleRegistry,
};

#[derive(Clone, Debug)]
pub struct World {
    pub map: Map,
    pub ecs: Ecs,
    pub registry: RuleRegistry,
    pub player_id: EntityId,
    pub player_facing: Direction,
    pub depth: u32,
    pub turn: u64,
    pub mode: Mode,
    pub event_log: EventLog,
    pub console_buffer: String,
    pub console_output: String,
    pub console_output_color: Option<RGB>,
    pub glyph_env: Env,
    pub binding_env: Env,
    pub inspector_selection: usize,
    pub blocking: bool,
    pub running: bool,
    pub player_can_attack: bool,
    pub wizard_taught: bool,
    pub wizard_id: Option<EntityId>,
    pub bindings: HashMap<String, String>,

    /// Tracks progress through the Konami code (↑↑↓↓←→←→).
    pub konami_index: usize,
    /// Set to true when the full Konami code is entered.
    pub cheat_unlocked: bool,

    /// History of submitted console commands (most recent last).
    pub console_history: Vec<String>,
    /// Position in history: 0 = at new input, 1 = at most recent entry, etc.
    pub console_history_index: usize,
    /// Saved buffer when user first presses up to browse history.
    pub console_history_draft: String,
    /// Byte-offset cursor position within console_buffer.
    pub console_cursor: usize,

    /// Set to true when q is pressed to confirm quitting.
    pub confirming_quit: bool,
}

impl World {
    /// Minimal World for tests and contexts where no real game state is needed.
    pub fn minimal() -> Self {
        World {
            map: Map::new_static(),
            ecs: Ecs::new(),
            registry: RuleRegistry::core(),
            player_id: EntityId::new(0),
            player_facing: Direction::East,
            depth: 0,
            turn: 0,
            mode: Mode::Normal,
            event_log: EventLog::new(),
            console_buffer: String::new(),
            console_output: String::new(),
            console_output_color: None,
            glyph_env: Env::extend(&crate::glyph::default_env()),
            binding_env: Env::extend(&crate::glyph::default_env()),
            inspector_selection: 0,
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: HashMap::new(),
            konami_index: 0,
            cheat_unlocked: false,
            console_history: Vec::new(),
            console_history_index: 0,
            console_history_draft: String::new(),
            console_cursor: 0,
            confirming_quit: false,
        }
    }
}

// `impl Default for World` lives in game.rs next to the other World methods.
