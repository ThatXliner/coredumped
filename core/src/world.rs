//! The `World` struct — the central game state.
//!
//! Defined here so the glyph module can reference it in `BuiltinFn`'s
//! signature without creating circular `use` confusion.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use bracket_color::prelude::RGB;
use bracket_pathfinding::prelude::{BaseMap, DijkstraMap};

use crate::{
    ecs::Ecs,
    entity::{Direction, EntityId, EntityKind, Position},
    event_log::EventLog,
    fragment::FragmentRegistry,
    game::Mode,
    glyph::Env,
    map::{Map, TileType},
    rules::RuleRegistry,
};

const AI_DIJKSTRA_MAX_DEPTH: f32 = 200.0;

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
    pub memory_scroll: usize,
    /// Enemies the player struck or shoved this tick — skipped during enemy AI.
    pub player_attacked: Vec<EntityId>,
    pub blocking: bool,
    pub running: bool,
    pub player_can_attack: bool,
    pub wizard_taught: bool,
    pub wizard_id: Option<EntityId>,
    pub bindings: HashMap<String, String>,

    /// Set to true when a new binding is added via bind-key.
    /// Cleared when the keybindings overlay is opened.
    pub has_new_bindings: bool,

    /// Binding keys added since the last time the keybindings overlay was closed.
    /// Mirrors new_rule_ids for the rules inspector.
    pub new_binding_keys: HashSet<String>,

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
    /// Output scrollback offset for the console overlay. 0 means follow newest.
    pub console_output_scroll: usize,
    /// Event-log scrollback offset. 0 means follow newest.
    pub event_log_scroll: usize,

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

    /// Tile types the player has seen (via flashlight).
    pub seen_tile_types: HashSet<TileType>,

    /// Rule ids that recently became visible but haven't been acknowledged.
    /// Cleared when the inspector is closed.
    pub new_rule_ids: HashSet<String>,

    /// All rule ids that have ever been discovered by the player.
    pub known_rule_ids: HashSet<String>,

    /// Memory fragment registry — tracks all 42 fragments and collected status.
    pub fragment_registry: FragmentRegistry,

    /// Cached flashlight tiles (invalidated when player position/facing changes).
    pub(crate) cached_flashlight: HashSet<Position>,
    pub(crate) cached_flashlight_pos: Position,
    pub(crate) cached_flashlight_facing: Direction,

    /// Ending text — set when the player triggers an ending at the Core.
    pub ending: Option<String>,

    /// Set when the Rage impact overflow has disabled registry write-protect.
    pub registry_write_unlocked: bool,

    /// Tracks which key IDs the player holds (from killing key-goblins in Level 8).
    pub held_keys: Vec<String>,

    /// Tracks special items found (Shade Echo, Vapor Canteen).
    pub held_items: Vec<String>,

    /// Tracks which gauntlet barriers have been locked (Level 6).
    pub gauntlet_barrier_locked: HashSet<i32>,

    /// When true, enemies cannot enter the barrel room (depth 2, room 8).
    pub barrel_room_protected: bool,

    /// Fire-tile cache rebuilt at tick start. Vapor Canteen can mutate mid-tick.
    pub fire_cache: HashSet<Position>,

    /// Positions of walls that can shift in the Maze of Regret (depth 10).
    /// These walls toggle between Wall and Floor each tick unless frozen.
    pub maze_shifting_walls: HashSet<Position>,

    /// When true, maze walls stop shifting. Set by the console injection exploit.
    pub maze_shift_frozen: bool,

    /// Tick-local distance field reused by AI pathfinding builtins.
    pub(crate) dijkstra_cache_target_idx: Option<usize>,
    pub(crate) dijkstra_cache_map: Vec<f32>,

    /// Per-level wizard dialogue callback. Set by each level builder.
    /// Called when player bumps wizard post-teaching.
    /// Return `true` to heal player, `false` to refuse.
    pub on_wizard_interact: Option<fn(&mut World) -> bool>,

    /// Camera position (top-left corner of viewport in map coordinates).
    pub camera_x: i32,
    pub camera_y: i32,

    /// Tiles the player has seen (fog of war). Persists across the current level.
    pub explored_tiles: HashSet<Position>,

    /// Frame counter for animations (incremented each render).
    pub render_frame: u64,
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
            memory_scroll: 0,
            player_attacked: Vec::new(),
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: HashMap::new(),
            has_new_bindings: false,
            new_binding_keys: HashSet::new(),
            konami_index: 0,
            cheat_unlocked: false,
            console_history: Vec::new(),
            console_history_index: 0,
            console_history_draft: String::new(),
            console_cursor: 0,
            console_output_scroll: 0,
            event_log_scroll: 0,
            confirming_quit: false,
            user_source: Vec::new(),
            pending_wipe_slot: None,
            quit_countdown: 0,
            seen_entity_kinds: HashSet::new(),
            seen_tile_types: HashSet::new(),
            new_rule_ids: HashSet::new(),
            known_rule_ids: HashSet::new(),
            fragment_registry: FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            registry_write_unlocked: false,
            held_keys: Vec::new(),
            held_items: Vec::new(),
            gauntlet_barrier_locked: HashSet::new(),
            barrel_room_protected: false,
            fire_cache: HashSet::new(),
            maze_shifting_walls: HashSet::new(),
            maze_shift_frozen: false,
            dijkstra_cache_target_idx: None,
            dijkstra_cache_map: Vec::new(),
            on_wizard_interact: None,
            camera_x: 0,
            camera_y: 0,
            explored_tiles: HashSet::new(),
            render_frame: 0,
        }
    }

    pub(crate) fn clear_dijkstra_cache(&mut self) {
        self.dijkstra_cache_target_idx = None;
        self.dijkstra_cache_map.clear();
    }

    pub(crate) fn dijkstra_best_step(
        &mut self,
        from: Position,
        target: Position,
    ) -> Option<Position> {
        if from == target || !self.map.contains(from) || !self.map.contains(target) {
            return None;
        }

        let target_idx = self.map.idx(target);
        self.ensure_dijkstra_cache(target_idx);

        let from_idx = self.map.idx(from);
        let current_dist = self.dijkstra_cache_map[from_idx];
        if current_dist >= f32::MAX {
            return None;
        }

        self.map
            .get_available_exits(from_idx)
            .into_iter()
            .filter(|(idx, _)| self.dijkstra_cache_map[*idx] < current_dist)
            .min_by(|(left, _), (right, _)| {
                self.dijkstra_cache_map[*left]
                    .partial_cmp(&self.dijkstra_cache_map[*right])
                    .unwrap_or(Ordering::Equal)
            })
            .map(|(idx, _)| self.map.position_for_idx(idx))
    }

    fn ensure_dijkstra_cache(&mut self, target_idx: usize) {
        let map_len = (self.map.width * self.map.height) as usize;
        if self.dijkstra_cache_target_idx == Some(target_idx)
            && self.dijkstra_cache_map.len() == map_len
        {
            return;
        }

        let dm = DijkstraMap::new(
            self.map.width,
            self.map.height,
            &[target_idx],
            &self.map,
            AI_DIJKSTRA_MAX_DEPTH,
        );
        self.dijkstra_cache_map = dm.map;
        self.dijkstra_cache_map[target_idx] = 0.0;
        self.dijkstra_cache_target_idx = Some(target_idx);
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

    /// Mark tile types in the flashlight cone as seen and add to explored tiles.
    pub fn mark_visible_tiles(&mut self) {
        self.ensure_lit_tiles();
        for pos in &self.cached_flashlight {
            self.seen_tile_types.insert(self.map.tile(*pos));
            self.explored_tiles.insert(*pos);
        }
    }

    /// Scan currently visible rules and record any newly discovered ones.
    pub fn refresh_rule_discovery(&mut self) {
        let visible = self
            .registry
            .visible_ids(&self.seen_entity_kinds, &self.seen_tile_types);
        for id in visible {
            if !self.known_rule_ids.contains(&id) {
                self.known_rule_ids.insert(id.clone());
                self.new_rule_ids.insert(id);
                self.event_log.push("New rule detected. Press I to see it.");
            }
        }
    }

    /// Update camera to center on player, clamped to map bounds.
    pub fn update_camera(&mut self, viewport_width: i32, viewport_height: i32) {
        let pos = self.player_pos();
        let half_w = viewport_width / 2;
        let half_h = viewport_height / 2;

        self.camera_x = (pos.x - half_w).clamp(0, (self.map.width - viewport_width).max(0));
        self.camera_y = (pos.y - half_h).clamp(0, (self.map.height - viewport_height).max(0));
    }
}

// `impl Default for World` lives in game.rs next to the other World methods.
