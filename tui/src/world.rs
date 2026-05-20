//! The `World` struct — the central game state.
//!
//! Defined here so the glyph module can reference it in `BuiltinFn`'s
//! signature without creating circular `use` confusion.

use std::collections::{HashMap, HashSet};

use bracket_lib::prelude::RGB;

use crate::{
    ecs::Ecs,
    entity::{Direction, EntityId, EntityKind, Position},
    event_log::EventLog,
    fragment::FragmentRegistry,
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
    /// Enemies the player struck or shoved this tick — skipped during enemy AI.
    pub player_attacked: Vec<EntityId>,
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

    /// Ordered list of env-mutating Glyph source forms (const, defmacro, set!,
    /// bind-key). Saved and replayed on load to restore user
    /// definitions/overrides.
    pub user_source: Vec<String>,

    /// Slot number awaiting wipe confirmation. Set by (wipe!).
    pub pending_wipe_slot: Option<u32>,

    /// Countdown to quit after wiping. 0 = not counting down.
    pub quit_countdown: u32,

    /// Entity kinds the player has seen (via flashlight or interaction).
    pub seen_entity_kinds: HashSet<EntityKind>,

    /// Memory fragment registry — tracks all 42 fragments and collected status.
    pub fragment_registry: FragmentRegistry,

    /// Cached flashlight tiles (invalidated when player position/facing changes).
    pub(crate) cached_flashlight: HashSet<Position>,
    pub(crate) cached_flashlight_pos: Position,
    pub(crate) cached_flashlight_facing: Direction,

    /// Ending text — set when the player triggers an ending at the Core.
    pub ending: Option<String>,

    /// Tracks which key IDs the player holds (from killing key-goblins in Level 8).
    pub held_keys: Vec<String>,

    /// Tracks special items found (Shade Echo, Vapor Canteen).
    pub held_items: Vec<String>,

    /// Tracks which gauntlet barriers have been locked (Level 6).
    pub gauntlet_barrier_locked: HashSet<i32>,
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
            player_attacked: Vec::new(),
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
            user_source: Vec::new(),
            pending_wipe_slot: None,
            quit_countdown: 0,
            seen_entity_kinds: HashSet::new(),
            fragment_registry: FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            held_keys: Vec::new(),
            held_items: Vec::new(),
            gauntlet_barrier_locked: HashSet::new(),
        }
    }

    /// Ensure flashlight cache is up to date. Call before reading `cached_flashlight`.
    fn ensure_lit_tiles(&mut self) {
        let pos = self.player_pos();
        let facing = self.player_facing;
        if pos != self.cached_flashlight_pos || facing != self.cached_flashlight_facing {
            self.cached_flashlight = self.map.flashlight_tiles(pos, facing);
            self.cached_flashlight_pos = pos;
            self.cached_flashlight_facing = facing;
        }
    }

    /// Mark entity kinds in the flashlight cone as seen.
    pub fn mark_visible_entities(&mut self) {
        self.ensure_lit_tiles();
        let mut newly_seen: Vec<EntityKind> = Vec::new();
        for entity in self.ecs.renderable_entities() {
            if self.cached_flashlight.contains(&entity.pos) || entity.kind == EntityKind::Player {
                newly_seen.push(entity.kind);
            }
        }
        for kind in newly_seen {
            self.seen_entity_kinds.insert(kind);
        }
    }
}

// `impl Default for World` lives in game.rs next to the other World methods.
