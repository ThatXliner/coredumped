//! Deterministic gameplay systems for the prototype.
//!
//! `World` owns resources such as the map, turn counter, UI mode, and event
//! log. Dynamic actors live in the ECS store, and this module provides the
//! systems that read or mutate those components: player intent handling,
//! enemy AI, ticking, console state, and inspector state.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use bracket_color::prelude::{CYAN, DARK_GRAY, GREEN, ORANGE, RED, RGB, YELLOW};
use serde::{Deserialize, Serialize};

const KONAMI_CODE: [&str; 8] = ["up", "up", "down", "down", "left", "right", "left", "right"];

use crate::{
    ecs::Ecs,
    entity::{Direction, EntityId, EntityKind, EntityView, Hp, Position},
    event_log::EventLog,
    glyph::{self, Env, Value},
    map::{Map, TileType},
    rules::RuleRegistry,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionCost {
    Free,
    Tick,
    Quit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Execute a keybinding (checks bindings map).
    ExecuteBinding(String),
    Move(crate::entity::Direction),
    Wait,
    /// Scroll in overlays (inspector, keybindings).
    InspectorScroll(i32),
    ConsoleInput(char),
    ConsoleNewline,
    ConsoleHistory(i32),
    ConsoleCursor(i32),
    ConsoleMoveWord(i32),
    ConsoleBackspace,
    ConsoleBackspaceWord,
    ConsoleDelete,
    ConsoleHome,
    ConsoleEnd,
    ConsoleKillToStart,
    ConsoleKillToEnd,
    ConsoleSubmit,
    Scroll(i32),
    CloseOverlay,
    ToggleConsole,
    ToggleKeybindings,
    ToggleMemories,
    Respawn,
    Restart,
    SaveGame(u32),
    LoadGame(u32),
    WipeSave(u32),
    OpenExternalEditor,
    Quit,
    Noop,
}

use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Normal,
    Inspector,
    Console,
    Dead,
    Keybindings,
    Memories,
}

impl World {
    pub fn new() -> Self {
        let mut event_log = EventLog::new();
        event_log.push("Xlyph runtime booted.");
        event_log.push("Move with arrows or hjkl. ` opens the console. i inspects code.");
        event_log.push("Ctrl+E in console opens external editor for multi-line input.");
        event_log.push("Your flashlight ray-casts in the direction you last moved.");
        event_log.push("You are helpless. Find the wizard to learn the art of striking.");

        let registry = RuleRegistry::core();

        let mut ecs = Ecs::new();
        let player_id = ecs.spawn_player(Position::new(5, 5));
        ecs.spawn_slime(Position::new(19, 5));
        ecs.spawn_slime(Position::new(47, 18));

        let glyph_env = setup_glyph_env();
        let binding_env = setup_binding_env(&glyph_env);

        let mut world = Self {
            map: Map::new_static(),
            ecs,
            registry,
            player_id,
            player_facing: Direction::East,
            depth: 0,
            turn: 0,
            mode: Mode::Normal,
            event_log,
            console_buffer: String::new(),
            console_output: String::new(),
            console_output_color: None,
            glyph_env,
            binding_env,
            inspector_selection: 0,
            memory_scroll: 0,
            player_attacked: Vec::new(),
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: default_bindings(),
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
            deferred_intent: None,
            quit_countdown: 0,
            seen_entity_kinds: HashSet::new(),
            seen_tile_types: HashSet::new(),
            new_rule_ids: HashSet::new(),
            known_rule_ids: HashSet::new(),
            fragment_registry: crate::fragment::FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            registry_write_unlocked: false,
            last_impact_force: 0,
            last_impact_target: None,
            held_keys: Vec::new(),
            held_items: Vec::new(),
            gauntlet_barrier_locked: HashSet::new(),
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
        };

        world.load_playbook();
        world
    }

    /// Create a world with a procedurally generated dungeon starting at depth 0.
    pub fn new_game() -> Self {
        let depth = 0;

        let mut event_log = EventLog::new();
        event_log.push("Xlyph runtime booted.");
        event_log.push("Move with arrows or hjkl. ` opens the console. i inspects code.");
        event_log.push("Ctrl+E in console opens external editor for multi-line input.");
        event_log.push("Your flashlight ray-casts in the direction you last moved.");
        event_log.push("You are helpless. Find the wizard to learn the art of striking. Bump into signs to read them");
        event_log.push(format!("Depth {depth}. Find the stairs down."));

        let registry = RuleRegistry::core();
        let mut ecs = Ecs::new();
        // Temporary position — build_level will move the player to the correct start
        let player_id = ecs.spawn_player(Position::new(0, 0));

        let glyph_env = setup_glyph_env();
        let binding_env = setup_binding_env(&glyph_env);

        let mut world = Self {
            map: Map::new_static(),
            ecs,
            registry,
            player_id,
            player_facing: Direction::East,
            depth,
            turn: 0,
            mode: Mode::Normal,
            event_log,
            console_buffer: String::new(),
            console_output: String::new(),
            console_output_color: None,
            glyph_env,
            binding_env,
            inspector_selection: 0,
            memory_scroll: 0,
            player_attacked: Vec::new(),
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: default_bindings(),
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
            deferred_intent: None,
            quit_countdown: 0,
            seen_entity_kinds: HashSet::new(),
            seen_tile_types: HashSet::new(),
            new_rule_ids: HashSet::new(),
            known_rule_ids: HashSet::new(),
            fragment_registry: crate::fragment::FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            registry_write_unlocked: false,
            last_impact_force: 0,
            last_impact_target: None,
            held_keys: Vec::new(),
            held_items: Vec::new(),
            gauntlet_barrier_locked: HashSet::new(),
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
        };

        crate::levels::build_level(&mut world, depth);
        world.load_playbook();
        world
    }

    pub fn apply_intent(&mut self, intent: Intent) -> ActionCost {
        // Any action other than q when confirming quit should cancel it
        if self.confirming_quit {
            match &intent {
                Intent::ExecuteBinding(name) if name == "q" => {}
                _ => {
                    self.confirming_quit = false;
                }
            }
        }

        match intent {
            Intent::Move(direction) => {
                self.player_facing = direction;
                let cost = self.apply_player_move(direction);
                if cost == ActionCost::Tick {
                    self.finish_tick();
                }
                cost
            }
            Intent::Wait => {
                self.finish_tick();
                ActionCost::Tick
            }
            Intent::ExecuteBinding(key) => {
                let before = self.turn;
                self.execute_binding(&key);
                self.check_konami(&key);
                if !self.running {
                    ActionCost::Quit
                } else if self.turn > before {
                    ActionCost::Tick
                } else {
                    ActionCost::Free
                }
            }
            Intent::InspectorScroll(delta) => {
                if self.mode == Mode::Memories {
                    self.scroll_memories(delta);
                } else if self.mode == Mode::Inspector || self.mode == Mode::Keybindings {
                    self.scroll_inspector(delta);
                }
                ActionCost::Free
            }
            Intent::ConsoleHistory(delta) => {
                if self.mode == Mode::Console {
                    self.console_history_move(delta);
                }
                ActionCost::Free
            }
            Intent::ConsoleInput(ch) => {
                if self.mode == Mode::Console {
                    self.console_insert(ch);
                }
                ActionCost::Free
            }
            Intent::ConsoleNewline => {
                if self.mode == Mode::Console {
                    self.console_insert('\n');
                }
                ActionCost::Free
            }
            Intent::ConsoleCursor(delta) => {
                if self.mode == Mode::Console {
                    self.console_move_cursor(delta);
                }
                ActionCost::Free
            }
            Intent::ConsoleMoveWord(delta) => {
                if self.mode == Mode::Console {
                    self.console_move_word(delta);
                }
                ActionCost::Free
            }
            Intent::ConsoleBackspace => {
                if self.mode == Mode::Console {
                    self.console_backspace();
                }
                ActionCost::Free
            }
            Intent::ConsoleBackspaceWord => {
                if self.mode == Mode::Console {
                    self.console_backspace_word();
                }
                ActionCost::Free
            }
            Intent::ConsoleDelete => {
                if self.mode == Mode::Console {
                    self.console_delete();
                }
                ActionCost::Free
            }
            Intent::ConsoleHome => {
                if self.mode == Mode::Console {
                    self.console_cursor = 0;
                }
                ActionCost::Free
            }
            Intent::ConsoleEnd => {
                if self.mode == Mode::Console {
                    self.console_cursor = self.console_buffer.len();
                }
                ActionCost::Free
            }
            Intent::ConsoleKillToStart => {
                if self.mode == Mode::Console {
                    self.console_kill_to_start();
                }
                ActionCost::Free
            }
            Intent::ConsoleKillToEnd => {
                if self.mode == Mode::Console {
                    self.console_kill_to_end();
                }
                ActionCost::Free
            }
            Intent::ConsoleSubmit => {
                if self.mode == Mode::Console {
                    self.submit_console();
                    self.console_history_index = 0;
                    self.console_history_draft.clear();
                    self.console_cursor = 0;
                }
                ActionCost::Free
            }
            Intent::Scroll(delta) => {
                self.scroll_view(delta);
                ActionCost::Free
            }
            Intent::CloseOverlay => {
                match self.mode {
                    Mode::Inspector => self.new_rule_ids.clear(),
                    Mode::Keybindings => self.new_binding_keys.clear(),
                    _ => {}
                }
                self.mode = Mode::Normal;
                ActionCost::Free
            }
            Intent::ToggleConsole => {
                self.mode = if self.mode == Mode::Console {
                    Mode::Normal
                } else {
                    Mode::Console
                };
                ActionCost::Free
            }
            Intent::ToggleKeybindings => {
                self.mode = if self.mode == Mode::Keybindings {
                    Mode::Normal
                } else {
                    self.has_new_bindings = false;
                    Mode::Keybindings
                };
                ActionCost::Free
            }
            Intent::ToggleMemories => {
                self.mode = if self.mode == Mode::Memories {
                    Mode::Normal
                } else {
                    Mode::Memories
                };
                ActionCost::Free
            }
            Intent::Respawn => {
                self.respawn();
                ActionCost::Free
            }
            Intent::Restart => {
                self.restart();
                ActionCost::Free
            }
            Intent::Quit => {
                let _ = self.save_to_disk(0);
                self.event_log.push("Game saved.");
                self.running = false;
                ActionCost::Quit
            }
            Intent::SaveGame(slot) => {
                if let Err(e) = self.save_to_disk(slot) {
                    self.event_log.push(format!("Save failed: {}", e));
                } else {
                    self.event_log
                        .push_colored(format!("Game saved to slot {}.", slot), RGB::named(GREEN));
                }
                ActionCost::Free
            }
            Intent::LoadGame(slot) => {
                match World::load_from_disk(slot) {
                    Ok(world) => {
                        *self = world;
                    }
                    Err(e) => {
                        self.event_log.push(format!("Load failed: {}", e));
                    }
                }
                ActionCost::Free
            }
            Intent::WipeSave(slot) => {
                let path = crate::save::save_path(slot);
                if path.exists() {
                    if let Err(e) = std::fs::remove_file(&path) {
                        self.event_log.push(format!("Cannot delete save: {}", e));
                    } else {
                        crate::player_profile::PlayerProfile::delete();
                        self.event_log
                            .push_colored(format!("Save slot {} deleted.", slot), RGB::named(RED));
                        self.quit_countdown = 3;
                    }
                } else {
                    self.event_log
                        .push(format!("Save slot {} does not exist.", slot));
                }
                ActionCost::Free
            }
            Intent::OpenExternalEditor => {
                if self.mode == Mode::Console {
                    self.open_external_editor();
                }
                ActionCost::Free
            }
            Intent::Noop => ActionCost::Free,
        }
    }

    pub fn player_pos(&self) -> Position {
        self.ecs
            .position(self.player_id)
            .expect("player should always have a Position component")
    }

    pub fn player_hp(&self) -> Hp {
        self.ecs
            .hp(self.player_id)
            .expect("player should always have an Hp component")
    }

    pub fn entity_at(&self, pos: Position) -> Option<EntityView> {
        self.ecs.entity_at(pos).and_then(|id| self.ecs.view(id))
    }

    pub fn living_enemies(&self) -> impl Iterator<Item = EntityView> + '_ {
        self.ecs
            .enemy_ids()
            .filter_map(|id| self.ecs.view(id))
            .filter(|enemy| enemy.alive)
    }

    pub fn renderable_entities(&self) -> impl Iterator<Item = EntityView> + '_ {
        self.ecs.renderable_entities()
    }

    /// Spawn a depth-appropriate enemy at the given position.
    pub(crate) fn spawn_enemy_at(&mut self, pos: Position, depth: u32) {
        let hash = (pos.x.wrapping_mul(31).wrapping_add(pos.y.wrapping_mul(17))) as u32;
        let roll = (hash % 100) as i32;
        match depth {
            0..=3 => {
                // Tutorial depths: only slimes — simple enemies for early game
                self.ecs.spawn_slime(pos);
            }
            _ => {
                if roll < 30 {
                    self.ecs.spawn_slime(pos);
                } else if roll < 65 {
                    self.ecs.spawn_goblin(pos);
                } else if roll < 90 {
                    self.ecs.spawn_bat(pos);
                } else {
                    self.ecs.spawn_ogre(pos);
                }
            }
        }
    }

    pub(crate) fn spawn_boss_at(&mut self, pos: Position) {
        self.ecs.spawn_ogre(pos);
    }

    fn wipe_player_state(&mut self) {
        self.bindings = default_bindings();
        self.user_source.clear();
        self.console_history.clear();
        self.console_buffer.clear();
        self.console_output.clear();
        self.console_output_color = None;
        self.player_can_attack = false;
        self.wizard_taught = false;
        self.cheat_unlocked = false;
        self.player_attacked.clear();
        self.blocking = false;
        self.konami_index = 0;
        self.pending_wipe_slot = None;
        self.quit_countdown = 0;
        self.confirming_quit = false;
        self.seen_entity_kinds.clear();
        self.seen_tile_types.clear();
        self.new_rule_ids.clear();
        self.known_rule_ids.clear();
        self.ending = None;
        self.registry_write_unlocked = false;
        self.held_keys.clear();
        self.held_items.clear();
        self.gauntlet_barrier_locked.clear();
        crate::player_profile::PlayerProfile::delete();
    }

    fn respawn(&mut self) {
        self.wipe_player_state();
        self.clear_all_enemies();
        self.ecs
            .set_hp(self.player_id, Hp::new(self.player_hp().max));
        crate::levels::build_level(self, self.depth);
        self.mode = Mode::Normal;
        self.player_facing = Direction::East;
        self.event_log.push("You gasp back into existence!");
    }

    fn restart(&mut self) {
        self.wipe_player_state();
        *self = World::new_game();
    }

    fn counting_room_locked_door(pos: Position) -> bool {
        matches!((pos.x, pos.y), (16, 12) | (28, 12) | (16, 18) | (28, 18))
    }

    fn activate_pressure_plate(&mut self, pos: Position) {
        // Barrel room pressure plate at (38, 29) - toggles door from room 7
        if self.depth == 2 && pos == Position::new(38, 29) {
            let door_pos = Position::new(35, 28);
            let door_closed = self.map.tile(door_pos) == TileType::Wall;
            if door_closed {
                self.map.set_tile(door_pos, TileType::Floor);
                self.map.set_tile(door_pos.offset(0, 1), TileType::Floor);
                self.event_log
                    .push_colored("Click. The door opens.", RGB::named(GREEN));
            } else {
                self.map.set_tile(door_pos, TileType::Wall);
                self.map.set_tile(door_pos.offset(0, 1), TileType::Wall);
                self.event_log
                    .push_colored("Click. The door slides shut behind you.", RGB::named(GREEN));
            }
        }
    }

    fn try_open_counting_room_door(&mut self, target: Position) -> bool {
        if self.depth != 8 || !Self::counting_room_locked_door(target) {
            return false;
        }

        if self.held_keys.pop().is_some() {
            self.map.set_tile(target, TileType::Floor);
            self.event_log.push_colored(
                "The key dissolves in your hand. The locked door opens.",
                RGB::named(CYAN),
            );
        } else {
            self.event_log.push_colored(
                "The door is locked. Somewhere nearby, a key-goblin is carrying what you need.",
                RGB::named(YELLOW),
            );
        }
        true
    }

    pub(crate) fn award_counting_room_key(&mut self, target_kind: EntityKind) {
        if self.depth == 8 && target_kind == EntityKind::Goblin && self.held_keys.len() < 3 {
            let key_id = format!("memory-key-{}", self.held_keys.len() + 1);
            self.held_keys.push(key_id);
            self.event_log.push_colored(
                "A memory-key clatters to the floor. You pick it up.",
                RGB::named(CYAN),
            );
        }
    }

    pub(crate) fn apply_player_move(&mut self, direction: Direction) -> ActionCost {
        let (dx, dy) = direction.delta();
        let target = self.player_pos().offset(dx, dy);

        if !self.map.is_walkable(target) {
            if self.try_open_counting_room_door(target) {
                return ActionCost::Tick;
            }
            self.event_log
                .push("You bump into a wall. Time still moves.");
            return ActionCost::Tick;
        }

        if let Some(target_id) = self.ecs.entity_at(target) {
            match self.ecs.kind(target_id) {
                Some(EntityKind::Wizard) => {
                    self.interact_with_wizard(target_id);
                    return ActionCost::Tick;
                }
                Some(EntityKind::Sign) => {
                    self.interact_with_sign(target_id);
                    return ActionCost::Tick;
                }
                Some(EntityKind::Fragment) => {
                    self.interact_with_fragment(target_id);
                    return ActionCost::Free;
                }
                Some(EntityKind::ShadeEcho) => {
                    self.held_items.push("Shade Echo".to_string());
                    self.event_log.push_colored(
                        "You pick up the Shade Echo. It shivers faintly in your hand.",
                        RGB::named(CYAN),
                    );
                    self.ecs.remove(target_id);
                    return ActionCost::Free;
                }
                Some(EntityKind::VaporCanteen) => {
                    self.held_items.push("Vapor Canteen".to_string());
                    self.event_log.push_colored(
                        "You pick up the Vapor Canteen. The liquid inside feels cold.",
                        RGB::named(CYAN),
                    );
                    self.ecs.remove(target_id);
                    return ActionCost::Free;
                }
                Some(EntityKind::Barrel) => {
                    self.bump_barrel(target_id);
                    return ActionCost::Tick;
                }
                _ => {}
            }

            if !self.player_can_attack {
                let enemy_pos = self.ecs.position(target_id).unwrap();
                let shove_target = enemy_pos.offset(dx, dy);
                if self.map.is_walkable(shove_target) && self.ecs.entity_at(shove_target).is_none()
                {
                    self.ecs.set_position(target_id, shove_target);
                    self.event_log.push_colored(
                        format!("You shove the {} back.", self.ecs.name(target_id)),
                        RGB::named(YELLOW),
                    );
                } else {
                    self.event_log.push(format!(
                        "You shove the {}. It doesn't budge.",
                        self.ecs.name(target_id)
                    ));
                }
                return ActionCost::Free;
            }

            let target_name = self.ecs.name(target_id);
            let target_kind = self.ecs.kind(target_id).unwrap_or(EntityKind::Slime);
            let hp = self
                .ecs
                .damage(target_id, 1)
                .expect("combat targets should have an Hp component");

            self.event_log.push_colored(
                format!("You strike the {target_name} for 1 damage."),
                RGB::named(ORANGE),
            );

            if hp.current <= 0 {
                self.event_log.push_colored(
                    format!("The {target_name} collapses into inert code."),
                    RGB::named(ORANGE),
                );
                self.award_counting_room_key(target_kind);
            }
            return ActionCost::Tick;
        }

        self.ecs.set_position(self.player_id, target);
        // Evaluate tile-effect rules (e.g. fire/burn)
        let body_form = self
            .registry
            .tile_rule(self.map.tile(target))
            .map(|r| r.body_form.clone());
        if let Some(body_form) = body_form {
            let tile_env = Env::extend(&self.glyph_env);
            tile_env.bind("*player*", Value::I64(self.player_id.raw() as i64));
            tile_env.bind(
                "*pos*",
                Value::List(vec![
                    Value::I64(target.x as i64),
                    Value::I64(target.y as i64),
                ]),
            );
            let _ = glyph::eval_with_opts(
                &body_form,
                &tile_env,
                glyph::SandboxOptions::default(),
                self,
            );
        }
        // Pressure plate handling
        if self.map.tile(target) == TileType::PressurePlate {
            self.activate_pressure_plate(target);
        }
        self.event_log.push_colored(
            format!("You move to {},{}.", target.x, target.y),
            RGB::named(DARK_GRAY),
        );
        ActionCost::Tick
    }

    pub(crate) fn finish_tick(&mut self) {
        self.turn += 1;
        self.log_entity_overlaps("tick-start");
        // Rebuild fire cache at tick start — Vapor Canteen mutations
        // from the previous tick are now baked in.
        self.fire_cache.clear();
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let pos = Position::new(x, y);
                if self.map.tile(pos) == TileType::Fire {
                    self.fire_cache.insert(pos);
                }
            }
        }
        self.check_gauntlet_barriers();
        self.shift_maze_walls();
        self.log_entity_overlaps("after-barriers");
        self.advance_enemies();
        self.log_entity_overlaps("after-ai");
        self.repair_all_enemy_positions();
        self.log_entity_overlaps("after-repair");
        self.player_attacked.clear();
        self.blocking = false;

        if self.player_hp().current <= 0 {
            self.mode = Mode::Dead;
            self.event_log.push("You have perished!");
        }
    }

    fn shift_maze_walls(&mut self) {
        if self.depth != 10 || self.maze_shifting_walls.is_empty() {
            return;
        }

        // Check for exploit: if console buffer contains (quote :still), freeze maze
        // This is the "eval injection" — the maze/shift rule reads the buffer
        // without checking if it was submitted.
        if !self.maze_shift_frozen {
            let buffer = self.console_buffer.trim();
            if buffer.contains("(quote :still)") || buffer.contains("':still") || buffer == ":still"
            {
                self.maze_shift_frozen = true;
                self.event_log.push_colored(
                    "The walls shudder... and stop. The maze holds its breath.",
                    RGB::named(CYAN),
                );
                return;
            }
        }

        if self.maze_shift_frozen {
            return;
        }

        // Toggle walls based on turn parity
        let is_wall_phase = self.turn % 2 == 0;
        let walls: Vec<Position> = self.maze_shifting_walls.iter().copied().collect();
        for pos in walls {
            let new_tile = if is_wall_phase {
                TileType::Wall
            } else {
                TileType::Floor
            };
            // Don't shift if player or enemy is standing there
            if self.ecs.entity_at(pos).is_none() {
                self.map.set_tile(pos, new_tile);
            }
        }
    }

    fn log_entity_overlaps(&self, phase: &str) {
        let mut by_pos: BTreeMap<(i32, i32), Vec<EntityView>> = BTreeMap::new();
        for entity in self.ecs.renderable_entities() {
            by_pos
                .entry((entity.pos.x, entity.pos.y))
                .or_default()
                .push(entity);
        }

        for ((x, y), entities) in by_pos {
            if entities.len() < 2 {
                continue;
            }

            let occupants = entities
                .iter()
                .map(|entity| format!("{}#{}", entity.name(), entity.id.raw()))
                .collect::<Vec<_>>()
                .join(", ");
            log::error!(
                target: "xlyph::overlap",
                "phase={phase} turn={} depth={} pos=({x},{y}) occupants=[{occupants}]",
                self.turn,
                self.depth,
            );
        }
    }

    fn check_gauntlet_barriers(&mut self) {
        if self.depth != 6 {
            return;
        }
        let barrier_xs = [7, 13, 19, 25, 31, 37, 43, 49];
        let corridor_y = crate::map::MAP_HEIGHT / 2;
        let player_x = self.player_pos().x;

        for &bx in &barrier_xs {
            if player_x > bx && self.gauntlet_barrier_locked.insert(bx) {
                for dy in -2..=2 {
                    let pos = Position::new(bx, corridor_y + dy);
                    if let Some(entity_id) = self.ecs.entity_at(pos) {
                        if let Some(evacuation_pos) =
                            self.gauntlet_barrier_evacuation_pos(bx, corridor_y)
                        {
                            self.ecs.set_position(entity_id, evacuation_pos);
                        }
                    }
                    self.map.set_tile(pos, TileType::Wall);
                }
                self.event_log
                    .push_colored("A barrier slams shut behind you!", RGB::named(RED));
            }
        }
    }

    fn gauntlet_barrier_evacuation_pos(&self, bx: i32, corridor_y: i32) -> Option<Position> {
        for distance in 1..=8 {
            for x in [bx - distance, bx + distance] {
                if matches!(x, 7 | 13 | 19 | 25 | 31 | 37 | 43 | 49) {
                    continue;
                }
                let candidate = Position::new(x, corridor_y);
                if self.map.is_walkable(candidate) && self.ecs.entity_at(candidate).is_none() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn enemy_can_occupy(&self, enemy_id: EntityId, pos: Position) -> bool {
        self.map.is_walkable(pos) && self.ecs.entity_at_except(pos, enemy_id).is_none()
    }

    fn nearest_enemy_open_tile(&self, enemy_id: EntityId, start: Position) -> Option<Position> {
        if self.map.width <= 0 || self.map.height <= 0 {
            return None;
        }

        let start = Position::new(
            start.x.clamp(0, self.map.width - 1),
            start.y.clamp(0, self.map.height - 1),
        );
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(pos) = queue.pop_front() {
            if self.enemy_can_occupy(enemy_id, pos) {
                return Some(pos);
            }

            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let next = pos.offset(dx, dy);
                if self.map.contains(next) && visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }

        None
    }

    fn repair_enemy_position(&mut self, enemy_id: EntityId, fallback: Position) {
        let Some(pos) = self.ecs.position(enemy_id) else {
            return;
        };

        if self.enemy_can_occupy(enemy_id, pos) {
            return;
        }

        if self.enemy_can_occupy(enemy_id, fallback) {
            self.ecs.set_position(enemy_id, fallback);
        } else if let Some(pos) = self.nearest_enemy_open_tile(enemy_id, fallback) {
            self.ecs.set_position(enemy_id, pos);
        }
    }

    fn repair_all_enemy_positions(&mut self) {
        let enemy_ids: Vec<EntityId> = self.ecs.enemy_ids().collect();
        for enemy_id in enemy_ids {
            if !self.ecs.is_alive(enemy_id) {
                continue;
            }
            let Some(pos) = self.ecs.position(enemy_id) else {
                continue;
            };
            self.repair_enemy_position(enemy_id, pos);
        }
    }

    fn advance_enemies(&mut self) {
        self.clear_dijkstra_cache();
        let enemy_ids: Vec<EntityId> = self.ecs.enemy_ids().collect();

        let sandbox = glyph::SandboxOptions::default();

        for enemy_id in enemy_ids {
            if !self.ecs.is_alive(enemy_id) {
                continue;
            }

            // Enemy the player struck or shoved this tick doesn't get to act.
            if self.player_attacked.contains(&enemy_id) {
                continue;
            }

            let previous_pos = match self.ecs.position(enemy_id) {
                Some(pos) => pos,
                None => continue,
            };

            let rule_name = match self.ecs.kind(enemy_id) {
                Some(kind) => kind.rule_name(),
                None => continue,
            };

            if rule_name.is_empty() {
                continue;
            }

            let body_form = match self.registry.get(rule_name) {
                Some(rule) => rule.body_form.clone(),
                None => continue,
            };

            let enemy_env = Env::extend(&self.glyph_env);
            enemy_env.bind("*self*", Value::I64(enemy_id.raw() as i64));
            enemy_env.bind("*player*", Value::I64(self.player_id.raw() as i64));

            let result = glyph::eval_with_opts(&body_form, &enemy_env, sandbox.clone(), self);
            if self.ecs.is_alive(enemy_id) {
                self.repair_enemy_position(enemy_id, previous_pos);
            }

            match result {
                Ok(_) => {}
                Err(err) => {
                    self.event_log.push(format!(
                        "AI error in '{}' for {}: {}",
                        rule_name,
                        self.ecs.name(enemy_id),
                        err
                    ));
                }
            }
        }
    }

    #[cfg(test)]
    fn spawn_slime(&mut self, pos: Position) -> EntityId {
        self.ecs.spawn_slime(pos)
    }

    #[cfg(test)]
    fn set_player_pos(&mut self, pos: Position) {
        self.ecs.set_position(self.player_id, pos);
    }

    fn scroll_inspector(&mut self, delta: i32) {
        if delta < 0 {
            self.inspector_selection = self
                .inspector_selection
                .saturating_sub(delta.unsigned_abs() as usize);
        } else {
            self.inspector_selection = self
                .inspector_selection
                .saturating_add(delta as usize)
                .min(self.registry.len().saturating_sub(1));
        }
    }

    fn scroll_memories(&mut self, delta: i32) {
        if delta < 0 {
            self.memory_scroll = self
                .memory_scroll
                .saturating_sub(delta.unsigned_abs() as usize);
        } else {
            self.memory_scroll = self.memory_scroll.saturating_add(delta as usize);
        }
    }

    fn scroll_view(&mut self, delta: i32) {
        let target = match self.mode {
            Mode::Console => &mut self.console_output_scroll,
            Mode::Memories => {
                self.scroll_memories(delta);
                return;
            }
            Mode::Inspector | Mode::Keybindings => {
                self.scroll_inspector(delta);
                return;
            }
            _ => &mut self.event_log_scroll,
        };

        if delta < 0 {
            *target = target.saturating_add(delta.unsigned_abs() as usize);
        } else {
            *target = target.saturating_sub(delta as usize);
        }
    }

    fn load_playbook(&mut self) {
        use bracket_color::prelude::GREEN;
        if !crate::playbook::has_playbook() {
            return;
        }

        let glyph_env = self.glyph_env.clone();
        let binding_env = self.binding_env.clone();
        let mut loaded = false;

        // Load init.glyph
        if let Some(init_source) = crate::playbook::load_init_glyph() {
            match crate::glyph::read_string(&init_source) {
                Ok(forms) => {
                    for form in &forms {
                        let source = form.to_string();
                        match crate::glyph::eval_with_opts(
                            form,
                            &glyph_env,
                            crate::glyph::SandboxOptions::default(),
                            self,
                        ) {
                            Ok(_) => {
                                self.user_source.push(source.clone());
                                let _ = crate::glyph::eval_with_opts(
                                    form,
                                    &binding_env,
                                    crate::glyph::SandboxOptions::default(),
                                    self,
                                );
                            }
                            Err(e) => {
                                self.event_log.push(format!("Playbook init error: {}", e));
                            }
                        }
                    }
                    loaded = true;
                }
                Err(e) => {
                    self.event_log
                        .push(format!("Playbook init.glyph parse error: {}", e));
                }
            }
        }

        // Load lib/*.glyph files
        for lib_file in crate::playbook::list_lib_files() {
            match std::fs::read_to_string(&lib_file) {
                Ok(source) => match crate::glyph::read_string(&source) {
                    Ok(forms) => {
                        for form in &forms {
                            let source = form.to_string();
                            match crate::glyph::eval_with_opts(
                                form,
                                &glyph_env,
                                crate::glyph::SandboxOptions::default(),
                                self,
                            ) {
                                Ok(_) => {
                                    self.user_source.push(source.clone());
                                    let _ = crate::glyph::eval_with_opts(
                                        form,
                                        &binding_env,
                                        crate::glyph::SandboxOptions::default(),
                                        self,
                                    );
                                }
                                Err(e) => {
                                    self.event_log.push(format!(
                                        "Playbook lib '{}' error: {}",
                                        lib_file.display(),
                                        e
                                    ));
                                }
                            }
                        }
                        loaded = true;
                    }
                    Err(e) => {
                        self.event_log.push(format!(
                            "Playbook lib '{}' parse error: {}",
                            lib_file.display(),
                            e
                        ));
                    }
                },
                Err(e) => {
                    self.event_log.push(format!(
                        "Cannot read playbook lib '{}': {}",
                        lib_file.display(),
                        e
                    ));
                }
            }
        }

        if loaded {
            self.event_log
                .push_colored("Playbook loaded.", RGB::named(GREEN));
        }
    }

    fn check_konami(&mut self, key: &str) {
        if self.cheat_unlocked {
            return;
        }
        // Only direction keys advance/affect the Konami sequence
        if !matches!(key, "up" | "down" | "left" | "right") {
            return;
        }
        if KONAMI_CODE.get(self.konami_index) == Some(&key) {
            self.konami_index += 1;
            if self.konami_index >= KONAMI_CODE.len() {
                self.cheat_unlocked = true;
                self.event_log.push_colored(
                    "Cheat codes activated! (heal) and (set-level) now available in the console.",
                    RGB::named(GREEN),
                );
            }
        } else {
            // Wrong direction — reset, but if the key is "up" it starts a new attempt
            self.konami_index = if key == "up" { 1 } else { 0 };
        }
    }

    fn execute_binding(&mut self, key: &str) {
        let command = match self.bindings.get(key) {
            Some(cmd) => cmd.clone(),
            None => return,
        };

        let forms = match glyph::read_string(&command) {
            Ok(f) => f,
            Err(e) => {
                self.event_log
                    .push(format!("Binding error: {}", e.report(&command)));
                return;
            }
        };

        let env = self.binding_env.clone();
        for form in &forms {
            if let Err(e) =
                glyph::eval_with_opts(form, &env, glyph::SandboxOptions::default(), self)
            {
                self.event_log.push(format!("Binding error: {}", e));
                return;
            }
        }
    }
}

fn default_bindings() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("h".into(), "(move! :west)".into());
    m.insert("j".into(), "(move! :south)".into());
    m.insert("k".into(), "(move! :north)".into());
    m.insert("l".into(), "(move! :east)".into());
    m.insert("left".into(), "(move! :west)".into());
    m.insert("right".into(), "(move! :east)".into());
    m.insert("up".into(), "(move! :north)".into());
    m.insert("down".into(), "(move! :south)".into());
    m.insert(".".into(), "(wait!)".into());
    m.insert("b".into(), "(block!)".into());
    m.insert("v".into(), "(shove!)".into());
    m.insert(">".into(), "(descend!)".into());
    m.insert("<".into(), "(ascend!)".into());
    m.insert("i".into(), "(toggle-inspector!)".into());
    m.insert("`".into(), "(toggle-console!)".into());
    m.insert("tab".into(), "(toggle-keybindings!)".into());
    m.insert("q".into(), "(quit!)".into());
    m
}

// Builtin functions moved to builtins.rs
pub(crate) use crate::builtins::{bind_do_attack, setup_binding_env, setup_glyph_env};
#[cfg(test)]
pub(crate) use crate::builtins::{builtin_block, builtin_do_attack, builtin_shove};

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TileType;

    fn world_with_single_enemy(enemy_pos: Position) -> World {
        let mut world = World::new();
        world.set_player_pos(Position::new(5, 5));
        world.ecs.set_hp(world.player_id, Hp::new(12));
        world.clear_all_enemies();
        world.spawn_slime(enemy_pos);
        world.turn = 0;
        world.event_log = EventLog::new();
        world.mode = Mode::Normal;
        world.console_buffer.clear();
        world
    }

    fn single_enemy(world: &World) -> EntityView {
        world
            .living_enemies()
            .next()
            .expect("test world should have exactly one enemy")
    }

    #[test]
    fn player_movement_increments_turn() {
        let mut world = world_with_single_enemy(Position::new(20, 5));

        let cost = world.apply_intent(Intent::Move(Direction::East));

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.turn, 1);
        assert_eq!(world.player_pos(), Position::new(6, 5));
        assert_eq!(world.player_facing, Direction::East);
    }

    #[test]
    fn bumping_wall_increments_turn_and_logs_it() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.set_player_pos(Position::new(1, 1));

        let cost = world.apply_intent(Intent::Move(Direction::West));

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.turn, 1);
        assert_eq!(world.player_pos(), Position::new(1, 1));
        assert_eq!(world.player_facing, Direction::West);
        assert!(world.event_log.contains("bump into a wall"));
    }

    #[test]
    fn waiting_increments_turn() {
        let mut world = world_with_single_enemy(Position::new(20, 5));

        let cost = world.apply_intent(Intent::Wait);

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.turn, 1);
    }

    #[test]
    fn inspector_toggle_is_free() {
        let mut world = world_with_single_enemy(Position::new(20, 5));

        let cost = world.apply_intent(Intent::ExecuteBinding("i".into()));

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.turn, 0);
        assert_eq!(world.mode, Mode::Inspector);
    }

    #[test]
    fn memories_toggle_is_free() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world
            .bindings
            .insert("m".into(), "(toggle-memories!)".into());

        let cost = world.apply_intent(Intent::ExecuteBinding("m".into()));

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.turn, 0);
        assert_eq!(world.mode, Mode::Memories);
    }

    #[test]
    fn memories_scroll_uses_memory_offset() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Memories;

        world.apply_intent(Intent::InspectorScroll(3));

        assert_eq!(world.memory_scroll, 3);
        assert_eq!(world.inspector_selection, 0);
    }

    #[test]
    fn closing_keybindings_does_not_acknowledge_new_rules() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Keybindings;
        world.new_rule_ids.insert("slime-hunt".into());
        world.new_binding_keys.insert("z".into());

        let cost = world.apply_intent(Intent::CloseOverlay);

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.mode, Mode::Normal);
        assert!(world.new_rule_ids.contains("slime-hunt"));
        assert!(world.new_binding_keys.is_empty());
    }

    #[test]
    fn closing_inspector_does_not_acknowledge_new_bindings() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Inspector;
        world.new_rule_ids.insert("slime-hunt".into());
        world.has_new_bindings = true;
        world.new_binding_keys.insert("z".into());

        let cost = world.apply_intent(Intent::CloseOverlay);

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.mode, Mode::Normal);
        assert!(world.new_rule_ids.is_empty());
        assert!(world.has_new_bindings);
        assert!(world.new_binding_keys.contains("z"));
    }

    #[test]
    fn opening_keybindings_keeps_new_binding_rows_marked_until_close() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.has_new_bindings = true;
        world.new_binding_keys.insert("z".into());

        let cost = world.apply_intent(Intent::ExecuteBinding("tab".into()));

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.mode, Mode::Keybindings);
        assert!(!world.has_new_bindings);
        assert!(world.new_binding_keys.contains("z"));

        world.apply_intent(Intent::CloseOverlay);

        assert!(world.new_binding_keys.is_empty());
    }

    #[test]
    fn console_toggle_and_typing_are_free() {
        let mut world = world_with_single_enemy(Position::new(20, 5));

        assert_eq!(
            world.apply_intent(Intent::ExecuteBinding("`".into())),
            ActionCost::Free
        );
        assert_eq!(
            world.apply_intent(Intent::ConsoleInput('x')),
            ActionCost::Free
        );
        assert_eq!(
            world.apply_intent(Intent::ConsoleInput('y')),
            ActionCost::Free
        );

        assert_eq!(world.turn, 0);
        assert_eq!(world.console_buffer, "xy");
    }

    #[test]
    fn console_readline_word_motion_and_deletion_are_free() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "alpha beta gamma".to_string();
        world.console_cursor = world.console_buffer.len();

        assert_eq!(
            world.apply_intent(Intent::ConsoleMoveWord(-1)),
            ActionCost::Free
        );
        assert_eq!(world.console_cursor, "alpha beta ".len());

        world.apply_intent(Intent::ConsoleBackspaceWord);

        assert_eq!(world.console_buffer, "alpha gamma");
        assert_eq!(world.console_cursor, "alpha ".len());
        assert_eq!(world.turn, 0);
    }

    #[test]
    fn console_readline_kill_and_delete_edit_the_buffer() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "alpha beta".to_string();
        world.console_cursor = "alpha ".len();

        world.apply_intent(Intent::ConsoleKillToStart);

        assert_eq!(world.console_buffer, "beta");
        assert_eq!(world.console_cursor, 0);

        world.console_buffer = "alpha beta".to_string();
        world.console_cursor = "alpha".len();
        world.apply_intent(Intent::ConsoleKillToEnd);

        assert_eq!(world.console_buffer, "alpha");
        assert_eq!(world.console_cursor, "alpha".len());

        world.console_buffer = "éx".to_string();
        world.console_cursor = 0;
        world.apply_intent(Intent::ConsoleDelete);

        assert_eq!(world.console_buffer, "x");
    }

    #[test]
    fn scroll_intent_targets_active_scrollback() {
        let mut world = world_with_single_enemy(Position::new(20, 5));

        world.apply_intent(Intent::Scroll(-3));
        assert_eq!(world.event_log_scroll, 3);
        world.apply_intent(Intent::Scroll(1));
        assert_eq!(world.event_log_scroll, 2);

        world.mode = Mode::Console;
        world.apply_intent(Intent::Scroll(-5));
        assert_eq!(world.console_output_scroll, 5);
        world.apply_intent(Intent::Scroll(2));
        assert_eq!(world.console_output_scroll, 3);
    }

    #[test]
    fn console_print_output_stays_in_console() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(println \"hello\" \"glyph\")".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert_eq!(world.console_output, "hello glyph");
        assert!(!world
            .event_log
            .entries()
            .iter()
            .any(|entry| entry.text == "hello glyph"));
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn console_string_results_are_readable_text() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "\"line one\nline two\"".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert_eq!(world.console_output, "=> line one\nline two");
        assert!(!world
            .event_log
            .entries()
            .iter()
            .any(|entry| entry.text == "=> line one\nline two"));
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn console_help_output_stays_out_of_event_log() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(help)".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world.console_output.starts_with("=> Glyph help (page 1/6)"));
        assert!(!world
            .event_log
            .entries()
            .iter()
            .any(|entry| entry.text.starts_with("=> Glyph help")));
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn console_help_supports_numbered_and_named_pages() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(help 5)".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world
            .console_output
            .starts_with("=> Glyph help (page 5/6): language reference"));

        world.console_buffer = "(help :tutorial)".to_string();
        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world
            .console_output
            .starts_with("=> Glyph help (page 6/6): short tutorial"));
    }

    #[test]
    fn console_syntax_errors_are_tui_colored_without_ansi_codes() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "\"unclosed".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world.console_output.contains("syntax error"));
        assert!(!world.console_output.contains('\u{1b}'));
        assert_eq!(world.console_output_color, Some(RGB::named(RED)));
        assert!(!world.event_log.contains("syntax error"));
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn console_auto_closes_parentheses() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(+ 1 2".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert_eq!(world.console_output, "=> 3");
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn console_auto_closes_nested_parens() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(+ (* 2 3".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert_eq!(world.console_output, "=> 6");
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn console_auto_close_handles_mixed_brackets() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(first (list 1 2 3".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert_eq!(world.console_output, "=> 1");
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn quit_terminal_closes_console_without_killing_game() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(quit-terminal)".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world.running);
        assert_eq!(world.mode, Mode::Normal);
        assert_eq!(world.console_output, "Terminal closed.");
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn quit_is_not_a_console_builtin() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(quit)".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world.running);
        assert_eq!(world.mode, Mode::Console);
        assert!(world.console_output.contains("unbound symbol: quit"));
        assert_eq!(world.console_output_color, Some(RGB::named(RED)));
    }

    #[test]
    fn enemy_advances_after_each_tick_action() {
        let mut world = world_with_single_enemy(Position::new(10, 5));

        world.apply_intent(Intent::Wait);

        assert_eq!(world.turn, 1);
        // Slimes may wander or path — either way they move from start
        assert_ne!(single_enemy(&world).pos, Position::new(10, 5));
    }

    #[test]
    fn adjacent_enemy_attacks_instead_of_moving() {
        let mut world = world_with_single_enemy(Position::new(6, 5));

        world.apply_intent(Intent::Wait);

        assert_eq!(world.turn, 1);
        assert_eq!(single_enemy(&world).pos, Position::new(6, 5));
        assert_eq!(world.player_hp().current, 11);
        assert!(world.event_log.contains("attacks you for 1 damage"));
    }

    #[test]
    fn enemy_pathing_respects_walls() {
        let mut world = world_with_single_enemy(Position::new(10, 9));
        world.set_player_pos(Position::new(10, 7));

        world.apply_intent(Intent::Wait);

        assert_eq!(world.turn, 1);
        assert_ne!(single_enemy(&world).pos, Position::new(10, 8));
        assert_eq!(world.map.tile(Position::new(10, 8)), TileType::Wall);
    }

    fn builtin_step_onto_wizard_for_test(
        args: &[Value],
        _env: &Env,
        _opts: &glyph::SandboxOptions,
        world: &mut World,
    ) -> glyph::EvalResult<Value> {
        let Some(Value::I64(raw_id)) = args.first() else {
            return Err(glyph::EvalError::WrongArgCount {
                expected: 1,
                got: args.len(),
            });
        };
        let entity_id = EntityId::new(*raw_id as usize);
        let wizard_id = world
            .wizard_id
            .expect("test world should have a wizard entity");
        let wizard_pos = world
            .ecs
            .position(wizard_id)
            .expect("wizard should have a position");
        world.ecs.set_position(entity_id, wizard_pos);
        Ok(Value::Bool(true))
    }

    #[test]
    fn enemy_ai_cannot_finish_on_wizard_tile() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        let old_enemy_id = world.living_enemies().next().unwrap().id;
        world.ecs.remove(old_enemy_id);
        world.set_player_pos(Position::new(8, 5));
        let enemy_id = world.ecs.spawn_goblin(Position::new(5, 5));
        let wizard_pos = Position::new(6, 5);
        let wizard_id = world.ecs.spawn_wizard(wizard_pos);
        world.wizard_id = Some(wizard_id);
        world.glyph_env.bind(
            "step-toward!",
            Value::Builtin(glyph::BuiltinFn {
                name: "step-toward!",
                doc: "",
                func: builtin_step_onto_wizard_for_test,
            }),
        );

        world.apply_intent(Intent::Wait);

        assert_eq!(world.ecs.position(wizard_id), Some(wizard_pos));
        assert_ne!(world.ecs.position(enemy_id), Some(wizard_pos));
        assert_eq!(world.ecs.entity_at(wizard_pos), Some(wizard_id));
    }

    #[test]
    fn goblin_pathing_cannot_step_onto_wizard_tile() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        let old_enemy_id = world.living_enemies().next().unwrap().id;
        world.ecs.remove(old_enemy_id);
        world.set_player_pos(Position::new(8, 5));
        let goblin_id = world.ecs.spawn_goblin(Position::new(5, 5));
        let wizard_pos = Position::new(6, 5);
        let wizard_id = world.ecs.spawn_wizard(wizard_pos);
        world.wizard_id = Some(wizard_id);

        world.apply_intent(Intent::Wait);

        assert_eq!(world.ecs.position(goblin_id), Some(Position::new(5, 5)));
        assert_eq!(world.ecs.position(wizard_id), Some(wizard_pos));
        assert_eq!(world.ecs.entity_at(wizard_pos), Some(wizard_id));
    }

    #[test]
    fn enemy_position_write_cannot_take_wizard_tile() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.set_player_pos(Position::new(8, 5));
        let enemy_id = world.living_enemies().next().unwrap().id;
        let wizard_pos = Position::new(6, 5);
        let wizard_id = world.ecs.spawn_wizard(wizard_pos);
        world.wizard_id = Some(wizard_id);

        assert!(!world.ecs.set_position(enemy_id, wizard_pos));
        assert_eq!(world.ecs.position(wizard_id), Some(wizard_pos));
        assert_ne!(world.ecs.position(enemy_id), Some(wizard_pos));
        assert_eq!(world.ecs.entity_at(wizard_pos), Some(wizard_id));
    }

    #[test]
    fn gauntlet_barrier_does_not_trap_enemy_inside_wall() {
        let mut world = World::new_game();
        world.depth = 6;
        crate::levels::build_level(&mut world, 6);
        world.clear_all_enemies();

        let corridor_y = crate::map::MAP_HEIGHT / 2;
        let enemy_id = world.spawn_slime(Position::new(13, corridor_y));
        world.set_player_pos(Position::new(14, corridor_y));

        world.check_gauntlet_barriers();

        let enemy_pos = world
            .ecs
            .position(enemy_id)
            .expect("enemy should still have a position");
        assert!(world.map.is_walkable(enemy_pos));
        assert_eq!(
            world.map.tile(Position::new(13, corridor_y)),
            TileType::Wall
        );
    }

    #[test]
    fn flashlight_lights_facing_direction_and_stops_at_walls() {
        let world = world_with_single_enemy(Position::new(20, 5));
        let lit = world
            .map
            .flashlight_tiles(world.player_pos(), Direction::East);

        assert!(lit.contains(&Position::new(8, 5)));
        assert!(!lit.contains(&Position::new(2, 5)));
        assert!(lit.contains(&Position::new(8, 8)));
        assert!(!lit.contains(&Position::new(8, 9)));
    }

    // --- Helpless phase tests ---

    #[test]
    fn helpless_player_bump_deals_no_damage() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = false;
        let enemy = single_enemy(&world);
        let initial_hp = enemy.hp.current;

        let cost = world.apply_intent(Intent::Move(Direction::East));

        let enemy_after = single_enemy(&world);
        assert_eq!(enemy_after.hp.current, initial_hp);
        assert!(world.event_log.contains("shove the slime"));
        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.turn, 0);
    }

    #[test]
    fn helpless_shove_moves_enemy_from_tile() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = false;

        let cost = world.apply_intent(Intent::Move(Direction::East));

        // Enemy pushed off original tile (AI doesn't act on Free shove)
        let enemy = single_enemy(&world);
        assert_ne!(enemy.pos, Position::new(6, 5));
        assert!(world.event_log.contains("shove the slime"));
        assert_eq!(cost, ActionCost::Free);
    }

    #[test]
    fn helpless_shove_blocked_by_wall() {
        let mut world = world_with_single_enemy(Position::new(1, 5));
        world.player_can_attack = false;
        world.set_player_pos(Position::new(2, 5));
        let enemy_id = world.living_enemies().next().unwrap().id;
        world.ecs.set_position(enemy_id, Position::new(1, 5));

        let cost = world.apply_intent(Intent::Move(Direction::West));

        // Enemy can't move further west (map border), shove blocked
        let enemy = single_enemy(&world);
        assert_eq!(enemy.pos, Position::new(1, 5));
        assert!(world.event_log.contains("doesn't budge"));
        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.turn, 0);
    }

    #[test]
    fn armed_player_bump_deals_damage() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;

        world.apply_intent(Intent::Move(Direction::East));

        let enemy_after = single_enemy(&world);
        assert_eq!(enemy_after.hp.current, 2); // Slime starts at 3
        assert!(world.event_log.contains("strike"));
    }

    #[test]
    fn attack_key_hits_enemy_in_facing_direction() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.glyph_env.bind(
            "do-attack",
            Value::Builtin(glyph::BuiltinFn {
                name: "do-attack",
                doc: "",
                func: builtin_do_attack,
            }),
        );
        world.player_can_attack = true;
        world.player_facing = Direction::East;
        world.bindings.insert("a".into(), "(do-attack)".into());

        world.apply_intent(Intent::ExecuteBinding("a".into()));

        assert_eq!(world.turn, 1);
        assert_eq!(world.player_pos(), Position::new(5, 5)); // didn't move
        assert_eq!(single_enemy(&world).hp.current, 2); // took 1 damage
        assert!(world.event_log.contains("strike"));
    }

    #[test]
    fn attack_key_swings_at_empty_air() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.glyph_env.bind(
            "do-attack",
            Value::Builtin(glyph::BuiltinFn {
                name: "do-attack",
                doc: "",
                func: builtin_do_attack,
            }),
        );
        world.player_can_attack = true;
        world.player_facing = Direction::North;
        world.bindings.insert("a".into(), "(do-attack)".into());

        world.apply_intent(Intent::ExecuteBinding("a".into()));

        assert_eq!(world.turn, 1);
        assert!(world.event_log.contains("empty air"));
    }

    // --- Wizard tests ---

    #[test]
    fn wizard_teaches_and_heals() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        let wizard_id = world.ecs.spawn_wizard(Position::new(6, 5));
        world.wizard_id = Some(wizard_id);
        world.ecs.set_hp(
            world.player_id,
            Hp {
                current: 3,
                max: 12,
            },
        );
        world.player_can_attack = false;
        world.wizard_taught = false;
        world.depth = 1; // wizard teaches attack at depth 1+

        world.apply_intent(Intent::Move(Direction::East));

        assert!(world.player_can_attack);
        assert!(world.wizard_taught);
        assert_eq!(world.player_hp().current, 12);
        assert!(world.event_log.contains("strike back"));
    }

    #[test]
    fn wizard_revisit_heals_but_does_not_reteach() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        let wizard_id = world.ecs.spawn_wizard(Position::new(6, 5));
        world.wizard_id = Some(wizard_id);
        world.player_can_attack = true;
        world.wizard_taught = true;
        world.ecs.set_hp(
            world.player_id,
            Hp {
                current: 5,
                max: 12,
            },
        );

        world.apply_intent(Intent::Move(Direction::East));

        assert!(world.ecs.is_alive(wizard_id));
        assert_eq!(world.player_hp().current, 12);
        assert!(world.event_log.contains("refreshed"));
    }

    #[test]
    fn wizard_at_depth_0_intros_but_does_not_teach_attack() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        let wizard_id = world.ecs.spawn_wizard(Position::new(6, 5));
        world.wizard_id = Some(wizard_id);
        world.player_can_attack = false;
        world.wizard_taught = false;
        world.depth = 0;

        world.apply_intent(Intent::Move(Direction::East));

        assert!(!world.player_can_attack); // not taught yet
        assert!(!world.wizard_taught); // depth 0 doesn't set this
        assert!(world.event_log.contains("you're awake"));
    }

    #[test]
    fn bumping_wizard_does_not_damage_it() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        let wizard_id = world.ecs.spawn_wizard(Position::new(6, 5));
        world.wizard_id = Some(wizard_id);
        world.player_can_attack = true;

        world.apply_intent(Intent::Move(Direction::East));

        assert!(world.ecs.is_alive(wizard_id));
        assert_eq!(world.ecs.hp(wizard_id).unwrap().current, 20);
    }

    // --- do-attack builtin tests ---

    fn setup_do_attack_test_env() -> Env {
        let env = setup_glyph_env();
        env.bind(
            "do-attack",
            Value::Builtin(glyph::BuiltinFn {
                name: "do-attack",
                doc: "",
                func: builtin_do_attack,
            }),
        );
        env
    }

    #[test]
    fn do_attack_builtin_performs_attack() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;
        world.player_facing = Direction::East;
        let env = setup_do_attack_test_env();
        let forms = crate::glyph::read_string("(do-attack :east)").unwrap();
        let result = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();
        assert_eq!(result, Value::Nil);
        assert_eq!(world.turn, 1);
        assert_eq!(single_enemy(&world).hp.current, 2); // Slime starts at 3
    }

    #[test]
    fn do_attack_builtin_no_args_uses_facing() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;
        world.player_facing = Direction::East;
        let env = setup_do_attack_test_env();
        let forms = crate::glyph::read_string("(do-attack)").unwrap();
        let result = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();
        assert_eq!(result, Value::Nil);
        assert_eq!(world.turn, 1);
        assert_eq!(single_enemy(&world).hp.current, 2);
    }

    #[test]
    fn do_attack_rejects_non_direction() {
        let mut world = World::minimal();
        let env = setup_do_attack_test_env();
        let forms = crate::glyph::read_string("(do-attack :up)").unwrap();
        let result = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        );
        assert!(result.is_err());
    }

    #[test]
    fn charged_rage_attack_unlocks_registry_write() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.clear_all_enemies();
        world.ecs.spawn_rage(Position::new(6, 5));
        world.player_can_attack = true;
        let env = setup_do_attack_test_env();

        // Step 1: Hit rage with force > 12 (stores impact info)
        let attack = crate::glyph::read_string("(do-attack :east 13)").unwrap();
        crate::glyph::eval_with_opts(
            &attack[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        // Attack stores impact but doesn't unlock yet
        assert!(!world.registry_write_unlocked);
        assert_eq!(world.last_impact_force, 13);
        assert_eq!(world.last_impact_target, Some(EntityKind::Rage));

        // Step 2: Trigger overflow via copy-bytes!
        // Payload size = 13 * 8 (rage mass) = 104 bytes > 64 byte buffer
        let overflow =
            crate::glyph::read_string("(copy-bytes! (bytes 64) (impact-payload))").unwrap();
        crate::glyph::eval_with_opts(
            &overflow[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert!(world.registry_write_unlocked);
        assert!(world.event_log.contains("Buffer overflow"));
    }

    #[test]
    fn rule_registry_denies_access_before_unlock() {
        let mut world = World::new();
        let env = setup_glyph_env();
        let forms = crate::glyph::read_string("(open-registry :rule-registry)").unwrap();

        let err = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap_err();

        assert!(err.to_string().contains("write-protect flag is set"));
    }

    #[test]
    fn unlocked_rule_registry_handle_accepts_vessel_write() {
        let mut world = World::new();
        world.registry_write_unlocked = true;
        let env = setup_glyph_env();
        let forms = crate::glyph::read_string(
            "(let r (open-registry :rule-registry) (r :write :vessel/suppress '(set! *threshold* 0)))",
        )
        .unwrap();

        let result = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert_eq!(result, Value::String("vessel/suppress patched".into()));
    }

    #[test]
    fn inspect_fragment_accepts_full_keyword_id() {
        let mut world = World::new();
        let env = setup_glyph_env();
        let forms = crate::glyph::read_string("(inspect-fragment :frag-001)").unwrap();

        let result = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert!(matches!(result, Value::Map(_)));
    }

    #[test]
    fn counting_room_locked_door_spends_key() {
        let mut world = World::new_game();
        crate::levels::build_level(&mut world, 8);
        world.depth = 8;
        world.set_player_pos(Position::new(15, 12));
        world.held_keys.push("memory-key-1".into());

        let cost = world.apply_intent(Intent::Move(Direction::East));

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.map.tile(Position::new(16, 12)), TileType::Floor);
        assert!(world.held_keys.is_empty());
        assert!(world.event_log.contains("locked door opens"));
    }

    // --- Death & respawn tests ---

    #[test]
    fn player_dies_when_hp_reaches_zero() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.ecs.set_hp(
            world.player_id,
            Hp {
                current: 1,
                max: 12,
            },
        );

        world.apply_intent(Intent::Wait);

        assert_eq!(world.mode, Mode::Dead);
        assert!(world.event_log.contains("perished"));
    }

    #[test]
    fn death_mode_does_not_kill_on_nonfatal_damage() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.ecs.set_hp(
            world.player_id,
            Hp {
                current: 12,
                max: 12,
            },
        );

        world.apply_intent(Intent::Wait);

        assert_eq!(world.mode, Mode::Normal);
        assert_eq!(world.player_hp().current, 11);
    }

    #[test]
    fn respawn_restores_hp_and_regenerates_level() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.ecs.set_hp(
            world.player_id,
            Hp {
                current: 1,
                max: 12,
            },
        );

        // Kill the player
        world.apply_intent(Intent::Wait);
        assert_eq!(world.mode, Mode::Dead);

        // Respawn
        world.apply_intent(Intent::Respawn);

        assert_eq!(world.mode, Mode::Normal);
        assert_eq!(world.player_hp().current, 12);
        assert!(world.event_log.contains("gasp back"));
    }

    #[test]
    fn restart_creates_fresh_game() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.depth = 5;

        world.apply_intent(Intent::Restart);

        assert_eq!(world.mode, Mode::Normal);
        assert_eq!(world.depth, 0);
        assert_eq!(world.player_hp().current, 12);
        assert!(!world.player_can_attack);
    }

    // --- Depth 1 / wizard gating tests ---

    #[test]
    fn wizard_box_has_no_enemies() {
        let output = crate::levels::generate_wizard_box();
        assert!(output.combat_spawns.is_empty());
        assert!(output.boss_spawns.is_empty());
    }

    #[test]
    fn descend_blocked_at_depth_1_without_wizard() {
        let mut world = World::new();
        world.depth = 1;
        world.wizard_taught = false;
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.depth, 1);
        assert!(world.event_log.contains("barrier"));
    }

    #[test]
    fn descend_allowed_at_depth_1_with_wizard_and_binding() {
        let mut world = World::new();
        world.depth = 1;
        world.wizard_taught = true;
        world.bindings.insert("z".into(), "(do-attack)".into());
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.depth, 2);
    }

    #[test]
    fn descend_blocked_when_taught_but_not_bound() {
        let mut world = World::new();
        world.depth = 1;
        world.wizard_taught = true;
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.depth, 1);
        assert!(world.event_log.contains("barrier"));
    }

    #[test]
    fn console_bind_attack_allows_descend_at_depth_1() {
        let mut world = World::new();
        world.depth = 1;
        world.wizard_taught = true;
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        // Bind (do-attack) to `z` via the console — bind-key is now a
        // special form, so the second argument is stored unevaluated.
        world.mode = Mode::Console;
        world.console_buffer = "(bind-key :z (do-attack))".to_string();
        world.apply_intent(Intent::ConsoleSubmit);

        // Confirm the binding was stored as the source form, not a sentinel
        assert_eq!(
            world.bindings.get("z").map(|s| s.as_str()),
            Some("(do-attack)")
        );

        // Now descend should work
        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.depth, 2);
    }

    #[test]
    fn descending_from_level_2_clears_level_2_entities() {
        let mut world = World::new_game();
        world.depth = 2;
        world.wizard_taught = true;
        world.bindings.insert("z".into(), "(do-attack)".into());
        crate::levels::build_level(&mut world, 2);

        let depth_2_entities = world.renderable_entities().count();
        assert!(world
            .renderable_entities()
            .any(|entity| entity.kind == EntityKind::Barrel));
        assert!(world
            .renderable_entities()
            .any(|entity| entity.kind == EntityKind::Sign));

        let stairs_down = crate::levels::find_stairs_down(&world.map);
        if let Some(barrel_id) = world.ecs.entity_at(stairs_down) {
            world.ecs.remove(barrel_id);
        }
        world.ecs.set_position(world.player_id, stairs_down);

        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.depth, 3);
        // Level 3 has fewer entities than level 2's barrel room
        assert!(world.renderable_entities().count() < depth_2_entities);
    }

    // --- Player-first strike tests ---

    #[test]
    fn attacking_enemy_trades_damage() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;

        world.apply_intent(Intent::Move(Direction::East));

        // Player deals 1 damage, enemy retaliates for 1
        assert_eq!(single_enemy(&world).hp.current, 2);
        assert_eq!(world.player_hp().current, 11);
        assert!(world.event_log.contains("strike the slime"));
        assert!(world.event_log.contains("attacks you for 1 damage"));
    }

    #[test]
    fn unattacked_enemy_still_attacks() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;

        // Wait instead of attack — enemy should still attack
        world.apply_intent(Intent::Wait);

        assert_eq!(world.player_hp().current, 11); // took 1 damage
        assert!(world.event_log.contains("attacks you for 1 damage"));
    }

    // --- Block-as-shove tests ---

    fn setup_block_test_env() -> Env {
        let env = setup_glyph_env();
        env.bind(
            "block!",
            Value::Builtin(glyph::BuiltinFn {
                name: "block!",
                doc: "",
                func: builtin_block,
            }),
        );
        env
    }

    #[test]
    fn block_shoves_adjacent_enemy() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;
        let env = setup_block_test_env();
        let enemy_pos_before = single_enemy(&world).pos;

        let forms = crate::glyph::read_string("(block!)").unwrap();
        crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        let enemy_after = single_enemy(&world);
        assert_ne!(enemy_after.pos, enemy_pos_before);
        assert_eq!(enemy_after.pos, Position::new(9, 5)); // shoved 3 tiles east
        assert!(world.event_log.contains("shove the slime back"));
        assert_eq!(world.turn, 1);
    }

    #[test]
    fn block_shove_blocked_by_wall() {
        let mut world = world_with_single_enemy(Position::new(1, 5));
        world.player_can_attack = true;
        world.set_player_pos(Position::new(2, 5));
        let enemy_id = world.living_enemies().next().unwrap().id;
        world.ecs.set_position(enemy_id, Position::new(1, 5));
        let env = setup_block_test_env();

        let forms = crate::glyph::read_string("(block!)").unwrap();
        crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert_eq!(single_enemy(&world).pos, Position::new(1, 5)); // didn't move
        assert!(world.event_log.contains("doesn't budge"));
    }

    #[test]
    fn block_with_no_adjacent_enemies() {
        let mut world = world_with_single_enemy(Position::new(10, 5));
        world.player_can_attack = true;
        let env = setup_block_test_env();

        let forms = crate::glyph::read_string("(block!)").unwrap();
        crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert!(world.event_log.contains("nothing is near"));
        assert_eq!(world.turn, 1);
    }

    // --- Shove builtin tests ---

    fn setup_shove_test_env() -> Env {
        let env = setup_glyph_env();
        env.bind(
            "shove!",
            Value::Builtin(glyph::BuiltinFn {
                name: "shove!",
                doc: "",
                func: builtin_shove,
            }),
        );
        env
    }

    #[test]
    fn shove_builtin_pushes_enemy() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;
        world.player_facing = Direction::East;
        let env = setup_shove_test_env();

        let forms = crate::glyph::read_string("(shove! :east)").unwrap();
        crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert_eq!(single_enemy(&world).pos, Position::new(7, 5));
        assert!(world.event_log.contains("shove the slime back"));
        assert_eq!(world.turn, 0); // shove costs no tick
    }

    #[test]
    fn shove_builtin_uses_facing_when_no_args() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;
        world.player_facing = Direction::East;
        let env = setup_shove_test_env();

        let forms = crate::glyph::read_string("(shove!)").unwrap();
        crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert_eq!(single_enemy(&world).pos, Position::new(7, 5));
        assert_eq!(world.turn, 0); // shove costs no tick
    }

    #[test]
    fn shove_at_empty_air_logs_message() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.player_can_attack = true;
        world.player_facing = Direction::North;
        let env = setup_shove_test_env();

        let forms = crate::glyph::read_string("(shove! :north)").unwrap();
        crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();

        assert!(world.event_log.contains("empty air"));
        assert_eq!(world.turn, 0); // shove costs no tick
    }

    #[cfg(feature = "prelude")]
    #[test]
    fn prelude_functions_work_in_console() {
        let mut world = World::new();
        world.mode = Mode::Console;

        // Glyph range (shadows Rust builtin)
        world.console_buffer = "(range 5)".to_string();
        world.apply_intent(Intent::ConsoleSubmit);
        assert_eq!(world.console_output, "=> (0 1 2 3 4)");

        // Glyph filter
        world.console_buffer = "(filter (fn [x] (> x 3)) (range 10))".to_string();
        world.apply_intent(Intent::ConsoleSubmit);
        assert_eq!(world.console_output, "=> (4 5 6 7 8 9)");

        // Glyph reduce
        world.console_buffer = "(reduce + 0 (range 5))".to_string();
        world.apply_intent(Intent::ConsoleSubmit);
        assert_eq!(world.console_output, "=> 10");

        // Glyph some
        world.console_buffer = "(some (fn [x] (= x 3)) (range 10))".to_string();
        world.apply_intent(Intent::ConsoleSubmit);
        assert_eq!(world.console_output, "=> true");
    }
}
