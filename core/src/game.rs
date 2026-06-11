//! Deterministic gameplay systems for the prototype.
//!
//! `World` owns resources such as the map, turn counter, UI mode, and event
//! log. Dynamic actors live in the ECS store, and this module provides the
//! systems that read or mutate those components: player intent handling,
//! enemy AI, ticking, console state, and inspector state.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use bracket_color::prelude::{CYAN, DARK_GRAY, GREEN, ORANGE, RED, RGB, YELLOW};
use bracket_random::prelude::RandomNumberGenerator;
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
    ReadingSign,
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
            run_seed: RandomNumberGenerator::new().next_u64(),
            turn: 0,
            turn_at_depth_start: 0,
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
            sign_text: String::new(),
            sign_scroll: 0,
            read_signs: HashSet::new(),
            fragment_registry: crate::fragment::FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            registry_write_unlocked: false,
            suppression_lifted: false,
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
        event_log.push("Move with arrows or hjkl.  Bump into signs to read them.");
        event_log.push("You are helpless. Find the wizard (W) to learn the art of striking.");
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
            run_seed: RandomNumberGenerator::new().next_u64(),
            turn: 0,
            turn_at_depth_start: 0,
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
            sign_text: String::new(),
            sign_scroll: 0,
            read_signs: HashSet::new(),
            fragment_registry: crate::fragment::FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            registry_write_unlocked: false,
            suppression_lifted: false,
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
                } else if self.mode == Mode::ReadingSign {
                    self.scroll_sign(delta);
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

    /// Turn count shown to the player: turns elapsed on the current depth.
    /// Counts up from 0 when a level is entered and is unaffected by respawns.
    pub fn turn_in_level(&self) -> u64 {
        self.turn.saturating_sub(self.turn_at_depth_start)
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
        self.suppression_lifted = false;
        self.registry = RuleRegistry::core();
        self.held_keys.clear();
        self.held_items.clear();
        self.gauntlet_barrier_locked.clear();
        crate::player_profile::PlayerProfile::delete();
    }

    fn respawn(&mut self) {
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
        // Evaluate tile-effect rules (e.g. fire/burn). Patched bodies run
        // instead of the default; unregistered rules don't fire at all.
        let body_form = self
            .registry
            .tile_rule(self.map.tile(target))
            .and_then(|r| self.registry.active_body(r.id));
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

            // Unregistered rules return no body: the enemy stands inert.
            let body_form = match self.registry.active_body(rule_name) {
                Some(body) => body,
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
    pub(crate) fn spawn_slime(&mut self, pos: Position) -> EntityId {
        self.ecs.spawn_slime(pos)
    }

    #[cfg(test)]
    pub(crate) fn set_player_pos(&mut self, pos: Position) {
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

    fn scroll_sign(&mut self, delta: i32) {
        if delta < 0 {
            self.sign_scroll = self
                .sign_scroll
                .saturating_sub(delta.unsigned_abs() as usize);
        } else {
            self.sign_scroll = self.sign_scroll.saturating_add(delta as usize);
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

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
