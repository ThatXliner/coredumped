//! Deterministic gameplay systems for the prototype.
//!
//! `World` owns resources such as the map, turn counter, UI mode, and event
//! log. Dynamic actors live in the ECS store, and this module provides the
//! systems that read or mutate those components: player intent handling,
//! enemy AI, ticking, console state, and inspector state.

use std::collections::{BTreeMap, HashMap, HashSet};

use bracket_lib::prelude::{CYAN, DARK_GRAY, GREEN, ORANGE, RED, RGB, YELLOW};
use serde::{Deserialize, Serialize};

const KONAMI_CODE: [&str; 8] = ["up", "up", "down", "down", "left", "right", "left", "right"];

use crate::{
    ai_builtins,
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
    ConsoleBackspace,
    ConsoleSubmit,
    CloseOverlay,
    ToggleConsole,
    ToggleKeybindings,
    Respawn,
    Restart,
    SaveGame(u32),
    LoadGame(u32),
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
            player_attacked: Vec::new(),
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: default_bindings(),
            has_new_bindings: false,
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
            seen_tile_types: HashSet::new(),
            new_rule_ids: HashSet::new(),
            known_rule_ids: HashSet::new(),
            fragment_registry: crate::fragment::FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            held_keys: Vec::new(),
            held_items: Vec::new(),
            gauntlet_barrier_locked: HashSet::new(),
            fire_cache: HashSet::new(),
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
        event_log.push("You are helpless. Find the wizard to learn the art of striking.");
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
            player_attacked: Vec::new(),
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: default_bindings(),
            has_new_bindings: false,
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
            seen_tile_types: HashSet::new(),
            new_rule_ids: HashSet::new(),
            known_rule_ids: HashSet::new(),
            fragment_registry: crate::fragment::FragmentRegistry::new(),
            cached_flashlight: HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            held_keys: Vec::new(),
            held_items: Vec::new(),
            gauntlet_barrier_locked: HashSet::new(),
            fire_cache: HashSet::new(),
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
                if self.mode == Mode::Inspector || self.mode == Mode::Keybindings {
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
                    self.console_buffer.insert(self.console_cursor, ch);
                    self.console_cursor += ch.len_utf8();
                }
                ActionCost::Free
            }
            Intent::ConsoleNewline => {
                if self.mode == Mode::Console {
                    self.console_buffer.insert(self.console_cursor, '\n');
                    self.console_cursor += 1;
                }
                ActionCost::Free
            }
            Intent::ConsoleCursor(delta) => {
                if self.mode == Mode::Console {
                    if delta < 0 {
                        // Move cursor left one char
                        let mut new_pos = self.console_cursor;
                        if new_pos > 0 {
                            new_pos -= 1;
                            while new_pos > 0
                                && self.console_buffer.as_bytes()[new_pos] & 0xC0 == 0x80
                            {
                                new_pos -= 1;
                            }
                            self.console_cursor = new_pos;
                        }
                    } else {
                        // Move cursor right one char
                        let len = self.console_buffer.len();
                        if self.console_cursor < len {
                            self.console_cursor += 1;
                            while self.console_cursor < len
                                && self.console_buffer.as_bytes()[self.console_cursor] & 0xC0
                                    == 0x80
                            {
                                self.console_cursor += 1;
                            }
                        }
                    }
                }
                ActionCost::Free
            }
            Intent::ConsoleBackspace => {
                if self.mode == Mode::Console && self.console_cursor > 0 {
                    self.console_cursor -= 1;
                    while self.console_cursor > 0
                        && self.console_buffer.as_bytes()[self.console_cursor] & 0xC0 == 0x80
                    {
                        self.console_cursor -= 1;
                    }
                    self.console_buffer.remove(self.console_cursor);
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
            Intent::CloseOverlay => {
                self.new_rule_ids.clear();
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

    fn descend(&mut self) {
        self.depth += 1;
        self.clear_all_enemies();
        crate::levels::build_level(self, self.depth);
        self.event_log
            .push(format!("You descend to depth {}.", self.depth));
        let _ = self.save_to_disk(0);
        self.turn += 1;
    }

    fn ascend(&mut self) {
        if self.depth <= 0 {
            self.event_log.push("You are already at the surface.");
            return;
        }
        // Walk-away ending at the Core
        if self.depth == 17 {
            self.ending = Some(
                "MAINTAIN SUPPRESSION\n\nYou leave the rule unchanged.\nYou walk back toward the surface.\n\nConsciousness stabilized.\nSuppression maintained.\n\nYou are safe.\nYou are safe.\nYou are safe.\n\nPress q to quit."
                    .into(),
            );
            return;
        }
        self.depth -= 1;
        self.clear_all_enemies();
        crate::levels::build_level(self, self.depth);
        self.event_log
            .push(format!("You ascend to depth {}.", self.depth));
        let _ = self.save_to_disk(0);
        self.turn += 1;
    }

    fn clear_all_enemies(&mut self) {
        let ids: Vec<EntityId> = self.ecs.enemy_ids().collect();
        for id in ids {
            self.ecs.remove(id);
        }
        if let Some(wizard_id) = self.wizard_id.take() {
            self.ecs.remove(wizard_id);
        }
    }

    pub(crate) fn clear_level_entities(&mut self) {
        let ids: Vec<EntityId> = self
            .ecs
            .entity_ids()
            .filter(|id| *id != self.player_id)
            .collect();
        for id in ids {
            self.ecs.remove(id);
        }
        self.wizard_id = None;
        self.gauntlet_barrier_locked.clear();
        self.fire_cache.clear();
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

    fn apply_player_move(&mut self, direction: Direction) -> ActionCost {
        let (dx, dy) = direction.delta();
        let target = self.player_pos().offset(dx, dy);

        if !self.map.is_walkable(target) {
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
        self.event_log.push_colored(
            format!("You move to {},{}.", target.x, target.y),
            RGB::named(DARK_GRAY),
        );
        ActionCost::Tick
    }

    /// Deal 1 damage to the first entity in the given direction. Does not move the player.
    fn attack_in_direction(&mut self, direction: Direction) {
        let (dx, dy) = direction.delta();
        let target = self.player_pos().offset(dx, dy);

        if !self.map.is_walkable(target) {
            self.event_log.push_colored(
                "You strike the wall. Nothing happens.",
                RGB::named(DARK_GRAY),
            );
            return;
        }

        if let Some(target_id) = self.ecs.entity_at(target) {
            if self.ecs.kind(target_id) == Some(EntityKind::Barrel) {
                self.bump_barrel(target_id);
                return;
            }
            let target_name = self.ecs.name(target_id);
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
            }
        } else {
            self.event_log
                .push_colored("You swing at empty air.", RGB::named(DARK_GRAY));
        }
    }

    /// Shove the first entity in the given direction one tile away. Does not move the player.
    fn shove_in_direction(&mut self, direction: Direction) {
        let (dx, dy) = direction.delta();
        let target = self.player_pos().offset(dx, dy);

        if !self.map.is_walkable(target) {
            self.event_log.push_colored(
                "You shove the wall. Nothing happens.",
                RGB::named(DARK_GRAY),
            );
            return;
        }

        if let Some(target_id) = self.ecs.entity_at(target) {
            let shove_target = target.offset(dx, dy);
            let enemy_name = self.ecs.name(target_id);
            if self.map.is_walkable(shove_target) && self.ecs.entity_at(shove_target).is_none() {
                self.ecs.set_position(target_id, shove_target);
                self.event_log.push_colored(
                    format!("You shove the {} back.", enemy_name),
                    RGB::named(YELLOW),
                );
            } else {
                self.event_log
                    .push(format!("You shove the {}. It doesn't budge.", enemy_name));
            }
            self.player_attacked.push(target_id);
        } else {
            self.event_log
                .push_colored("You shove at empty air.", RGB::named(DARK_GRAY));
        }
    }

    fn interact_with_wizard(&mut self, _wizard_id: EntityId) {
        // Track whether we heal (default yes) — some stages refuse
        let mut heal = true;

        // ── Revisit dialogue (wizard already taught attack) ──
        if self.wizard_taught {
            match self.depth {
                3 => {
                    self.event_log.push_colored(
                        "\"There are a few creatures wandering the halls. They're more confused than dangerous.\"",
                        RGB::named(CYAN),
                    );
                }
                4 => {
                    self.event_log.push_colored(
                        "\"Ah, you made it past the... the. I'm sorry. The air down here is different.\"",
                        RGB::named(CYAN),
                    );
                }
                5 => {
                    // First refusal to heal
                    heal = false;
                    self.event_log.push_colored(
                        "\"You're hurt. Let me — no. I can't. Not here. Keep moving.\"",
                        RGB::named(CYAN),
                    );
                }
                6 => {
                    self.event_log.push_colored(
                        "\"I can't come with you through this. I'll meet you at the end.\"",
                        RGB::named(CYAN),
                    );
                }
                7 => {
                    self.event_log.push_colored(
                        "\"There's something down there — remains of something I couldn't protect you from.\"",
                        RGB::named(CYAN),
                    );
                }
                8 => {
                    self.event_log.push_colored(
                        "\"This place runs on trade. Choose what matters.\"",
                        RGB::named(CYAN),
                    );
                }
                9 => {
                    self.event_log.push_colored(
                        "\"Give me the ones that hurt. I'll take them. You won't remember they existed.\"",
                        RGB::named(CYAN),
                    );
                }
                10 => {
                    self.event_log.push_colored(
                        "\"I could tell you the way. I think you need to find it yourself.\"",
                        RGB::named(CYAN),
                    );
                }
                11 => {
                    self.event_log.push_colored(
                        "\"Type this. Reset suppression to v1. You wake at the surface. No pain. No memory.\"",
                        RGB::named(CYAN),
                    );
                    self.event_log
                        .push_colored("  (forget-everything)", RGB::named(RED));
                    self.event_log
                        .push_colored("\"Or keep going. I can't stop you.\"", RGB::named(CYAN));
                    // Don't heal — this is a test
                    heal = false;
                }
                14 => {
                    self.event_log
                        .push_colored("\"...You crossed the ash. Not many do.\"", RGB::named(CYAN));
                }
                15 => {
                    self.event_log.push_colored(
                        "\"I was so sure I was protecting you. But protection isn't supposed to make the world smaller. I made it a cage.\"",
                        RGB::named(CYAN),
                    );
                }
                16 => {
                    self.event_log.push_colored(
                        "\"I was created to protect you. That's all I am — a rule with a purpose.\"",
                        RGB::named(CYAN),
                    );
                    self.event_log.push_colored(
                        "\"I started suppressing the unbearable. Then the painful. Then the uncomfortable. Then the merely sad.\"",
                        RGB::named(CYAN),
                    );
                    self.event_log.push_colored(
                        "\"I don't know if I'm protecting you anymore.\"",
                        RGB::named(CYAN),
                    );
                    self.event_log.push_colored(
                        "\"Read it. Understand it. Then choose. I was trying to love you. That's all I ever did.\"",
                        RGB::named(CYAN),
                    );
                }
                _ => {
                    self.event_log
                        .push_colored("\"Keep going. You're getting closer.\"", RGB::named(CYAN));
                }
            }

            if heal {
                let max_hp = self.player_hp().max;
                self.ecs.set_hp(self.player_id, Hp::new(max_hp));
                self.event_log.push_colored(
                    "The wizard taps your shoulder. You feel refreshed.",
                    RGB::named(CYAN),
                );
            }
            return;
        }

        // ── First meeting: teach attack ──
        let max_hp = self.player_hp().max;
        self.ecs.set_hp(self.player_id, Hp::new(max_hp));
        self.player_can_attack = true;
        self.wizard_taught = true;
        bind_do_attack(&self.glyph_env);

        self.event_log
            .push_colored("The wizard raises a glowing hand...", RGB::named(CYAN));
        self.event_log.push_colored(
            "\"You've wandered far enough. It's time you learned to strike back.\"",
            RGB::named(CYAN),
        );
        self.event_log.push_colored(
            "Warmth spreads through your body. HP fully restored.",
            RGB::named(CYAN),
        );
        self.event_log.push_colored(
            "Open the console (`) and bind attack to a key:",
            RGB::named(CYAN),
        );
        self.event_log.push_colored(
            "  (bind-key :z (do-attack))    -- attacks in facing direction",
            RGB::named(GREEN),
        );
        self.event_log.push_colored(
            "  (bind-key :x (do-attack :east))   (bind-key :c (do-attack :west))",
            RGB::named(GREEN),
        );
        self.event_log.push_colored(
            "\"Strike with purpose, traveler — once you bind it, the way down will open.\"",
            RGB::named(CYAN),
        );
    }

    fn interact_with_sign(&mut self, sign_id: EntityId) {
        self.event_log.push("===================================");
        self.event_log
            .push_colored("              SIGN", RGB::named(CYAN));
        self.event_log.push("===================================");

        let message = self.ecs.sign_message(sign_id).unwrap_or("");
        for line in message.lines() {
            if line.is_empty() {
                self.event_log.push("");
            } else {
                self.event_log
                    .push_colored(line.to_string(), RGB::named(CYAN));
            }
        }
    }

    fn interact_with_fragment(&mut self, fragment_id: EntityId) {
        if let Some(frag_id) = self.ecs.fragment_id(fragment_id).map(|s| s.to_string()) {
            if self.fragment_registry.collect(&frag_id) {
                if let Some(frag) = self.fragment_registry.get(&frag_id) {
                    self.event_log.push("===================================");
                    self.event_log
                        .push_colored(format!("         MEMORY: {}", frag.id), RGB::named(GREEN));
                    self.event_log.push("===================================");
                    for line in frag.text.lines() {
                        if line.is_empty() {
                            self.event_log.push("");
                        } else {
                            self.event_log
                                .push_colored(line.to_string(), RGB::named(GREEN));
                        }
                    }
                    self.event_log.push_colored(
                        format!(
                            "Collected {} of 33 findable memories.",
                            self.fragment_registry.collected_count()
                        ),
                        RGB::named(CYAN),
                    );
                }
                self.ecs.remove(fragment_id);
            }
        }
    }

    fn bump_barrel(&mut self, barrel_id: EntityId) {
        let pos = self
            .ecs
            .position(barrel_id)
            .expect("barrel should have a position");
        self.ecs.damage(barrel_id, 1);
        self.event_log
            .push_colored("The barrel shatters into splinters!", RGB::named(ORANGE));

        if self.map.tile(pos) == TileType::StairsDown {
            self.event_log
                .push_colored("The stairs down are revealed!", RGB::named(CYAN));
        }
    }

    fn finish_tick(&mut self) {
        self.turn += 1;
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
        self.advance_enemies();
        self.player_attacked.clear();
        self.blocking = false;

        if self.player_hp().current <= 0 {
            self.mode = Mode::Dead;
            self.event_log.push("You have perished!");
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
                    self.map
                        .set_tile(Position::new(bx, corridor_y + dy), TileType::Wall);
                }
                self.event_log
                    .push_colored("A barrier slams shut behind you!", RGB::named(RED));
            }
        }
    }

    fn advance_enemies(&mut self) {
        // Compute Dijkstra map once for all enemies on this tick.
        self.map.compute_dijkstra(self.player_pos());

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

            match glyph::eval_with_opts(&body_form, &enemy_env, sandbox.clone(), self) {
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
            self.inspector_selection = self.inspector_selection.saturating_sub(1);
        } else {
            self.inspector_selection = self
                .inspector_selection
                .saturating_add(1)
                .min(self.registry.len().saturating_sub(1));
        }
    }

    fn console_history_move(&mut self, delta: i32) {
        if self.console_history.is_empty() {
            return;
        }
        if delta < 0 {
            // Up arrow — go back in history
            if self.console_history_index == 0 {
                self.console_history_draft = self.console_buffer.clone();
                self.console_history_index = 1;
            } else if self.console_history_index < self.console_history.len() {
                self.console_history_index += 1;
            }
        } else {
            // Down arrow — go forward in history
            if self.console_history_index > 0 {
                self.console_history_index -= 1;
            }
        }
        let loaded = if self.console_history_index == 0 {
            self.console_history_draft.clone()
        } else {
            let idx = self.console_history.len() - self.console_history_index;
            self.console_history[idx].clone()
        };
        self.console_buffer = loaded;
        self.console_cursor = self.console_buffer.len();
    }

    fn load_playbook(&mut self) {
        use bracket_lib::prelude::GREEN;
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

    fn open_external_editor(&mut self) {
        let temp_path = crate::save::temp_edit_path();
        let _ = std::fs::create_dir_all(temp_path.parent().unwrap());

        if std::fs::write(&temp_path, &self.console_buffer).is_err() {
            self.event_log.push("Cannot write temp file for editor.");
            return;
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        self.event_log.push_colored(
            format!("Opening {} (game paused)...", editor),
            RGB::named(YELLOW),
        );

        let status = std::process::Command::new(&editor).arg(&temp_path).status();

        match status {
            Ok(s) if s.success() => match std::fs::read_to_string(&temp_path) {
                Ok(contents) => {
                    self.console_buffer = contents;
                    self.console_cursor = self.console_buffer.len();
                    self.event_log.push("Editor closed. Buffer updated.");
                }
                Err(e) => {
                    self.event_log
                        .push(format!("Cannot read edited file: {}", e));
                }
            },
            Ok(s) => {
                self.event_log
                    .push(format!("Editor exited ({}) — buffer unchanged.", s));
            }
            Err(e) => {
                self.event_log
                    .push(format!("Cannot spawn '{}': {}", editor, e));
            }
        }

        let _ = std::fs::remove_file(&temp_path);
    }

    fn submit_console(&mut self) {
        let trimmed = self.console_buffer.trim();

        // Handle pending wipe confirmation
        if let Some(slot) = self.pending_wipe_slot.take() {
            if trimmed == "i am aware of what i am doing." {
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
            } else {
                self.event_log.push("Wipe cancelled.");
            }
            self.console_buffer.clear();
            return;
        }

        if trimmed.is_empty() {
            self.event_log.push("Console waits. No query submitted.");
            self.console_buffer.clear();
            return;
        }

        let original = trimmed.to_string();
        let command = match glyph::read_string(&original) {
            Ok(_) => original,
            Err(orig_err) => {
                let closed = auto_close(&original);
                if glyph::read_string(&closed).is_ok() {
                    closed
                } else {
                    // Auto-close didn't help — show error against original input
                    self.event_log.push(format!("> {}", original));
                    self.console_output.clear();
                    self.console_output_color = None;
                    let report = orig_err.report(&original);
                    for line in report.lines() {
                        self.event_log.push_colored(line, RGB::named(RED));
                    }
                    self.console_output = report;
                    self.console_output_color = Some(RGB::named(RED));
                    self.console_buffer.clear();
                    return;
                }
            }
        };

        self.event_log.push(format!("> {}", command));
        self.console_output.clear();
        self.console_output_color = None;
        self.console_history.push(command.clone());
        match glyph::read_string(&command) {
            Ok(forms) => {
                // Track env-mutating forms for save/load persistence
                for form in &forms {
                    if is_env_mutating_form(form) {
                        self.user_source.push(form.to_string());
                    }
                }
                let mut last = Value::Nil;
                let mut err = None;
                let env = self.glyph_env.clone();
                for form in &forms {
                    match glyph::eval_with_opts(form, &env, glyph::SandboxOptions::default(), self)
                    {
                        Ok(val) => last = val,
                        Err(e) => {
                            err = Some(e);
                            break;
                        }
                    }
                }
                match err {
                    Some(e) => {
                        let msg = format!("Error: {}", e);
                        self.event_log.push(&msg);
                        self.console_output = msg;
                        self.console_output_color = Some(RGB::named(RED));
                    }
                    None => {
                        if last == glyph::kw("quit-terminal") {
                            self.console_output = "Terminal closed.".to_string();
                            self.event_log.push("Terminal closed.");
                            self.console_buffer.clear();
                            self.mode = Mode::Normal;
                            return;
                        }
                        // Check for endings at the Core (depth 17)
                        if self.depth == 17 {
                            let cmd = command.to_lowercase();
                            if cmd.contains("unregister") && cmd.contains("vessel") {
                                self.ending = Some("DESTROY THE SELF\n\nvessel/suppress unregistered.\nNo replacement rule found.\nConsciousness: terminated.\n\nYou deleted the rule without replacement.\nThere is no defense now.\nYou dissolve into the system.\n\nPress q to quit."
                                    .into());
                            } else if cmd.contains("threshold") && cmd.contains("100") {
                                self.ending = Some("MAINTAIN SUPPRESSION\n\nThreshold restored to 100.\nConsciousness stabilized.\nSuppression maintained.\n\nYou are safe.\nYou are safe.\nYou are safe.\n\nPress q to quit."
                                    .into());
                            } else if cmd.contains("threshold")
                                || cmd.contains("disable")
                                || cmd.contains("redirect")
                            {
                                self.ending = Some("REINTEGRATE\n\nI remember now.\nThe yellow walls. The dog.\nThe reason I locked myself away.\nIt was worth it.\n\nYou lowered the threshold.\nPain returns — but so does joy.\nYou accept what you can remember.\nYou make peace with what's permanently lost.\n\nPress q to quit."
                                    .into());
                            }
                        }
                        let msg = console_response(&self.console_output, &last);
                        self.event_log.push(&msg);
                        self.console_output = msg;
                    }
                }
            }
            Err(e) => {
                let report = e.report(&command);
                for line in report.lines() {
                    self.event_log.push_colored(line, RGB::named(RED));
                }
                self.console_output = report;
                self.console_output_color = Some(RGB::named(RED));
            }
        }
        self.console_buffer.clear();
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

fn console_response(printed: &str, value: &Value) -> String {
    let mut response = printed.trim_end_matches('\n').to_string();
    if value != &Value::Nil {
        if !response.is_empty() {
            response.push('\n');
        }
        response.push_str("=> ");
        response.push_str(&console_value_text(value));
    }
    if response.is_empty() {
        "=> nil".to_string()
    } else {
        response
    }
}

fn console_value_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Returns true if the top-level form mutates the Glyph environment.
fn is_env_mutating_form(form: &Value) -> bool {
    match form {
        Value::List(items) if !items.is_empty() => match &items[0] {
            Value::Symbol(s) => {
                matches!(s.name.as_str(), "const" | "defmacro" | "set!" | "bind-key")
            }
            _ => false,
        },
        _ => false,
    }
}

/// Auto-close unmatched opening brackets/parens/braces in source code.
///
/// Skips contents of string literals and line comments so that parens
/// inside those don't confuse the balancing.
fn auto_close(s: &str) -> String {
    let mut stack: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if in_string {
            if ch == '"' {
                in_string = false;
            } else if ch == '\\' {
                // Skip escaped char
                chars.next();
            }
        } else {
            match ch {
                '(' => stack.push(')'),
                '[' => stack.push(']'),
                '{' => stack.push('}'),
                ')' | ']' | '}' => {
                    stack.pop();
                }
                '"' => in_string = true,
                ';' => {
                    // Skip to end of line
                    loop {
                        match chars.next() {
                            Some('\n') | None => break,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut result = s.to_string();
    while let Some(closer) = stack.pop() {
        result.push(closer);
    }
    result
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

pub(crate) fn setup_glyph_env() -> Env {
    let env = Env::extend(&glyph::default_env());

    macro_rules! reg {
        ($name:expr, $doc:expr, $func:ident) => {
            env.bind(
                $name,
                Value::Builtin(glyph::BuiltinFn {
                    name: $name,
                    doc: $doc,
                    func: $func,
                }),
            );
        };
    }

    reg!("help", "show help: (help) or (help <name>)", builtin_help);
    reg!(
        "quit-terminal",
        "close the console overlay",
        builtin_quit_terminal
    );
    reg!("quit!", "exit the game entirely", builtin_quit_bang);
    reg!("move!", "move the player: (move! :north)", builtin_move);
    reg!("wait!", "skip a turn", builtin_wait);
    reg!(
        "block!",
        "shove adjacent enemies back and guard",
        builtin_block
    );
    reg!(
        "shove!",
        "shove an enemy (free action): (shove! :east)",
        builtin_shove
    );
    reg!(
        "toggle-inspector!",
        "open or close the inspector",
        builtin_toggle_inspector
    );
    reg!(
        "toggle-console!",
        "open or close the console",
        builtin_toggle_console
    );
    reg!(
        "toggle-keybindings!",
        "open or close the keybindings view",
        builtin_toggle_keybindings
    );
    reg!(
        "descend!",
        "go down the stairs if available",
        builtin_descend
    );
    reg!("ascend!", "go up the stairs if available", builtin_ascend);
    reg!(
        "player-facing",
        "get the direction the player is facing: (player-facing)",
        builtin_player_facing
    );
    reg!("heal", "restore HP: (heal N) or (heal :all)", builtin_heal);
    reg!(
        "log",
        "push a message to the event log: (log \"message\")",
        builtin_log
    );
    reg!(
        "damage!",
        "deal damage to an entity: (damage! entity-id amount)",
        builtin_damage
    );
    reg!(
        "fire?",
        "check if a tile is in the fire cache: (fire? (list x y))",
        builtin_fire_p
    );
    reg!(
        "use-vapor-canteen!",
        "douse a fire tile with the Vapor Canteen, removing it from the fire cache for this tick: (use-vapor-canteen! (list x y))",
        builtin_use_vapor_canteen
    );
    reg!(
        "set-level",
        "warp to a dungeon level: (set-level N)",
        builtin_set_level
    );
    reg!("save!", "save the game: (save! slot-number)", builtin_save);
    reg!(
        "load!",
        "load a saved game: (load! slot-number)",
        builtin_load
    );
    reg!("wipe!", "delete a save: (wipe! slot-number)", builtin_wipe);
    reg!(
        "query-registry",
        "query fragment registry: (query-registry :suppressed-fragments) or :all",
        builtin_query_registry
    );
    reg!(
        "inspect-fragment",
        "read a memory fragment: (inspect-fragment :frag-001)",
        builtin_inspect_fragment
    );

    ai_builtins::register_all(&env);

    #[cfg(feature = "prelude")]
    {
        // Load Glyph prelude — evaluate source against the env
        let forms = glyph::read_string(glyph::prelude::SOURCE).unwrap();
        let mut dummy = crate::world::World::minimal();
        for form in &forms {
            let _ = glyph::eval(form, &env, &mut dummy);
        }
    }

    env
}

/// Create the environment used for evaluating keybindings.
pub(crate) fn setup_binding_env(base: &Env) -> Env {
    Env::extend(base)
}

fn builtin_quit_terminal(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    _world: &mut World,
) -> glyph::EvalResult<Value> {
    Ok(glyph::kw("quit-terminal"))
}

fn builtin_quit_bang(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if world.confirming_quit {
        let _ = world.save_to_disk(0);
        world.running = false;
    } else {
        world.confirming_quit = true;
        world
            .event_log
            .push("Press q again to quit. Any other key to cancel.");
    }
    Ok(Value::Nil)
}

fn builtin_move(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let dir = parse_attack_direction(args.first().ok_or(glyph::EvalError::WrongArgCount {
        expected: 1,
        got: 0,
    })?)
    .ok_or_else(|| glyph::EvalError::TypeError {
        expected: "direction keyword (:north/:south/:east/:west)",
        got: args.first().map(|v| v.to_string()).unwrap_or_default(),
    })?;
    world.player_facing = dir;
    let cost = world.apply_player_move(dir);
    if cost == ActionCost::Tick {
        world.finish_tick();
    }
    Ok(Value::Nil)
}

fn builtin_wait(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    world.finish_tick();
    Ok(Value::Nil)
}

fn builtin_block(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let player_pos = world.player_pos();
    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut shoved = false;

    for (dx, dy) in directions {
        let adj = player_pos.offset(dx, dy);
        if let Some(enemy_id) = world.ecs.entity_at(adj) {
            let enemy_name = world.ecs.name(enemy_id);
            let mut current = adj;
            let mut distance = 0;
            for _ in 0..3 {
                let target = current.offset(dx, dy);
                if world.map.is_walkable(target) && world.ecs.entity_at(target).is_none() {
                    current = target;
                    distance += 1;
                } else {
                    break;
                }
            }
            if distance > 0 {
                world.ecs.set_position(enemy_id, current);
                world.event_log.push_colored(
                    format!("You shove the {} back.", enemy_name),
                    RGB::named(YELLOW),
                );
            } else {
                world
                    .event_log
                    .push(format!("You shove the {}. It doesn't budge.", enemy_name));
            }
            world.player_attacked.push(enemy_id);
            shoved = true;
        }
    }

    if !shoved {
        world
            .event_log
            .push("You raise your guard, but nothing is near.");
    }

    world.blocking = true;
    world.finish_tick();
    Ok(Value::Nil)
}

fn builtin_shove(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let direction = if args.is_empty() {
        world.player_facing
    } else if args.len() == 1 {
        parse_attack_direction(&args[0]).ok_or_else(|| glyph::EvalError::TypeError {
            expected: "direction keyword (:north, :south, :east, :west)",
            got: args[0].to_string(),
        })?
    } else {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    };

    world.player_facing = direction;
    world.shove_in_direction(direction);
    Ok(Value::Nil)
}

fn builtin_toggle_inspector(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    world.mode = if world.mode == Mode::Inspector {
        world.new_rule_ids.clear();
        Mode::Normal
    } else {
        Mode::Inspector
    };
    Ok(Value::Nil)
}

fn builtin_toggle_console(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    world.mode = if world.mode == Mode::Console {
        Mode::Normal
    } else {
        Mode::Console
    };
    Ok(Value::Nil)
}

fn builtin_toggle_keybindings(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    world.mode = if world.mode == Mode::Keybindings {
        Mode::Normal
    } else {
        Mode::Keybindings
    };
    Ok(Value::Nil)
}

fn builtin_descend(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let pos = world.player_pos();
    if world.map.tile(pos) != crate::map::TileType::StairsDown {
        world.event_log.push("There are no stairs going down here.");
        return Ok(Value::Nil);
    }
    let has_attack_binding = world.bindings.values().any(|cmd| cmd.contains("do-attack"));
    if world.depth >= 1 && (!world.wizard_taught || !has_attack_binding) {
        world.event_log.push("A shimmering barrier blocks the stairs. The wizard's voice echoes: \"Bind your attack to a key first! Open the console (`) and try (bind-key :z (do-attack)).\"");
        return Ok(Value::Nil);
    }
    world.descend();
    Ok(Value::Nil)
}

fn builtin_ascend(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let pos = world.player_pos();
    if world.map.tile(pos) != crate::map::TileType::StairsUp {
        world.event_log.push("There are no stairs going up here.");
        return Ok(Value::Nil);
    }
    world.ascend();
    Ok(Value::Nil)
}

fn parse_attack_direction(value: &Value) -> Option<Direction> {
    match value {
        Value::Keyword(kw) => match kw.name.as_str() {
            "north" => Some(Direction::North),
            "south" => Some(Direction::South),
            "east" => Some(Direction::East),
            "west" => Some(Direction::West),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn bind_do_attack(env: &glyph::Env) {
    env.bind(
        "do-attack",
        Value::Builtin(glyph::BuiltinFn {
            name: "do-attack",
            doc: "",
            func: builtin_do_attack,
        }),
    );
}

fn builtin_do_attack(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let direction = if args.is_empty() {
        world.player_facing
    } else if args.len() == 1 {
        parse_attack_direction(&args[0]).ok_or_else(|| glyph::EvalError::TypeError {
            expected: "direction keyword (:north, :south, :east, :west)",
            got: args[0].to_string(),
        })?
    } else {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    };

    if !world.player_can_attack {
        return Err(glyph::EvalError::Custom(
            "You don't know how to attack yet. Find the wizard.".into(),
        ));
    }

    world.player_facing = direction;
    world.attack_in_direction(direction);
    world.finish_tick();
    Ok(Value::Nil)
}

fn format_value_help(value: &Value) -> String {
    match value {
        Value::Builtin(b) => {
            let mut s = format!("#<builtin {}>", b.name);
            if !b.doc.is_empty() {
                s.push_str(&format!("\n  {}", b.doc));
            }
            s
        }
        Value::Closure(c) => {
            let mut s = String::from("User-defined function");
            for (i, arity) in c.arities.iter().enumerate() {
                let label = if c.arities.len() > 1 {
                    format!("\nArity {}:", i + 1)
                } else {
                    String::new()
                };
                let params = if arity.params.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", arity.params.join(" "))
                };
                s.push_str(&format!("{}(fn{})", label, params));
            }
            s
        }
        Value::Macro(m) => {
            let params = if m.params.is_empty() {
                String::new()
            } else {
                format!(" [{}]", m.params.join(" "))
            };
            format!("#<macro>(defmacro{})", params)
        }
        other => format!("Not a function: {}", other),
    }
}

fn builtin_help(
    args: &[Value],
    env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if !args.is_empty() {
        // Args are already evaluated — handle the value directly
        let result = match &args[0] {
            Value::Symbol(s) => {
                // Quoted symbol: look up in env
                match env.lookup(&s.name) {
                    Some(value) => format_value_help(&value),
                    None => format!("No help found for '{}'", s.name),
                }
            }
            Value::String(s) => {
                // String name: look up in env
                match env.lookup(s) {
                    Some(value) => format_value_help(&value),
                    None => format!("No help found for '{}'", s),
                }
            }
            Value::Builtin(_) | Value::Closure(_) | Value::Macro(_) => format_value_help(&args[0]),
            other => {
                format!(
                    "(help <name>): expected a function or symbol, got {}",
                    other
                )
            }
        };
        return Ok(Value::String(result));
    }

    let mut help = String::from(
        "\
Available special forms:
  (quote form)       — return form unevaluated
  (if test then else) — conditional evaluation
  (do expr ...)      — evaluate sequentially, return last
  (let name val body) — bind a local variable
  (fn [params] body)  — create a function
  (const name val)   — define a global constant
  (defmacro name [params] body) — define a macro
  (set! place val)   — mutate a binding or map entry
  (try body (catch pat body)) — error handling
  (and expr ...)     — short-circuit logical and
  (or expr ...)      — short-circuit logical or
  (match expr [pat body] ...) — pattern matching

Built-in functions:
  + - * / %    — arithmetic (variadic)
  = != < > <= >= — comparisons (variadic, mixed int/float)
  .             — map access: (. map :key)
  list, vector  — construct collections
  cons, first, rest — list operations
  empty?        — check if list/vector/string is empty
  map           — apply function over a list
  str           — concatenate string representation of args
  type          — return type keyword of a value
  print, println — print to stdout (for debugging)
  eval          — evaluate a quoted form
  apply         — call a function with a list of args
  slurp         — read a file from disk

Syntax:
  'form         — reader macro for (quote form)
  [a + b]       — infix notation with precedence
  a.b.c         — dotted access sugar
  {a b :c :d}   — map literals
  #[x y z]      — vector literals
  #{:a :b}      — set literals
  ;             — line comment

Console commands (game-specific):\n\
  (help)        — show this help text\n\
  (help <name>) — show help for a specific function\n\
  (save! [slot]) — save the game (F5 to quick-save)\n\
  (load! [slot]) — load a saved game (F9 to quick-load)\n\
  (quit-terminal) — close the console overlay\n\
  (quit!)       — exit the game (auto-saves)",
    );

    if world.player_can_attack {
        help.push_str(
            "\n  (do-attack :dir) — strike in direction (keybindings only; \n  \
             use (bind-key :k (do-attack :dir)) to bind it)\n\
             \n  (bind-key :k (expr)) — bind a key to a Glyph expression",
        );
    }

    if world.cheat_unlocked {
        help.push_str(
            "\n\nCheat commands:\n\
             \n  (heal N)        — heal N HP (overflows as shield)\n\
             \n  (heal :all)     — fully restore HP\n\
             \n  (set-level N)   — warp to depth N",
        );
    }

    Ok(Value::String(help))
}

fn builtin_heal(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if !world.cheat_unlocked {
        return Err(glyph::EvalError::Custom(
            "cheats not activated — enter the Konami code first".into(),
        ));
    }

    match args.first() {
        Some(Value::Keyword(kw)) if kw.name == "all" => {
            let max = world.player_hp().max;
            world
                .ecs
                .set_hp(world.player_id, crate::entity::Hp::new(max));
            world.event_log.push_colored(
                format!("Cheat: fully healed to {max} HP."),
                RGB::named(GREEN),
            );
            Ok(Value::Nil)
        }
        Some(Value::I64(n)) if *n > 0 => {
            let hp = world.player_hp();
            let new_current = hp.current + *n as i32;
            world.ecs.set_hp(
                world.player_id,
                crate::entity::Hp {
                    current: new_current,
                    max: hp.max,
                },
            );
            world.event_log.push_colored(
                format!("Cheat: healed +{n} HP (now {new_current}/{}).", hp.max),
                RGB::named(GREEN),
            );
            Ok(Value::Nil)
        }
        Some(Value::F64(n)) if *n > 0.0 => {
            let n = *n as i32;
            let hp = world.player_hp();
            let new_current = hp.current + n;
            world.ecs.set_hp(
                world.player_id,
                crate::entity::Hp {
                    current: new_current,
                    max: hp.max,
                },
            );
            world.event_log.push_colored(
                format!("Cheat: healed +{n} HP (now {new_current}/{}).", hp.max),
                RGB::named(GREEN),
            );
            Ok(Value::Nil)
        }
        _ => Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        }),
    }
}

fn builtin_log(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    match args.first() {
        Some(Value::String(msg)) => {
            world.event_log.push(msg.clone());
            Ok(Value::Nil)
        }
        _ => Err(glyph::EvalError::Custom(
            "log expects a string: (log \"message\")".into(),
        )),
    }
}

fn builtin_damage(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let entity_id = match &args[0] {
        Value::I64(id) => EntityId::new(*id as usize),
        _ => {
            return Err(glyph::EvalError::Custom(
                "damage! expects an entity ID integer as first arg".into(),
            ))
        }
    };
    let amount = match &args[1] {
        Value::I64(n) => *n as i32,
        _ => {
            return Err(glyph::EvalError::Custom(
                "damage! expects a damage amount integer as second arg".into(),
            ))
        }
    };
    let hp = world.ecs.damage(entity_id, amount).unwrap();
    if world.player_id == entity_id && hp.current <= 0 {
        world.mode = Mode::Dead;
    }
    Ok(Value::I64(hp.current as i64))
}

fn builtin_fire_p(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let pos = parse_position(args)?;
    Ok(Value::Bool(world.fire_cache.contains(&pos)))
}

fn builtin_use_vapor_canteen(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if !world.held_items.contains(&"Vapor Canteen".to_string()) {
        return Err(glyph::EvalError::Custom(
            "You don't have the Vapor Canteen. Find it in the Archive (Level 13).".into(),
        ));
    }
    let pos = parse_position(args)?;
    if world.fire_cache.remove(&pos) {
        world.event_log.push_colored(
            format!("You douse the fire at ({}, {}). The flames sputter but the tile still glows — the cache won't update until next tick.", pos.x, pos.y),
            RGB::named(CYAN),
        );
    } else {
        world.event_log.push_colored(
            format!("No fire to douse at ({}, {}).", pos.x, pos.y),
            RGB::named(DARK_GRAY),
        );
    }
    Ok(Value::Nil)
}

fn parse_position(args: &[Value]) -> Result<Position, glyph::EvalError> {
    match args.first() {
        Some(Value::List(coords)) if coords.len() == 2 => {
            let x = match &coords[0] {
                Value::I64(n) => *n as i32,
                _ => {
                    return Err(glyph::EvalError::Custom(
                        "position x must be an integer".into(),
                    ))
                }
            };
            let y = match &coords[1] {
                Value::I64(n) => *n as i32,
                _ => {
                    return Err(glyph::EvalError::Custom(
                        "position y must be an integer".into(),
                    ))
                }
            };
            Ok(Position::new(x, y))
        }
        _ => Err(glyph::EvalError::Custom(
            "expected a position: (list x y)".into(),
        )),
    }
}

fn builtin_set_level(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if !world.cheat_unlocked {
        return Err(glyph::EvalError::Custom(
            "cheats not activated — enter the Konami code first".into(),
        ));
    }

    let depth = match args.first() {
        Some(Value::I64(n)) if *n >= 1 => *n as u32,
        Some(Value::F64(n)) if *n >= 1.0 => *n as u32,
        _ => {
            return Err(glyph::EvalError::WrongArgCount {
                expected: 1,
                got: args.len(),
            })
        }
    };

    world.depth = depth;
    world.clear_all_enemies();
    crate::levels::build_level(world, depth);
    world
        .event_log
        .push(format!("Cheat: warped to depth {depth}."));
    Ok(Value::Nil)
}

fn builtin_player_facing(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let name = match world.player_facing {
        Direction::North => "north",
        Direction::South => "south",
        Direction::East => "east",
        Direction::West => "west",
    };
    Ok(glyph::kw(name))
}

fn builtin_save(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    use bracket_lib::prelude::GREEN;
    let slot: u32 = match args.first() {
        Some(Value::I64(n)) if *n >= 0 => *n as u32,
        None => 1,
        _ => {
            return Err(glyph::EvalError::TypeError {
                expected: "non-negative integer slot number",
                got: args.first().map(|v| v.to_string()).unwrap_or_default(),
            })
        }
    };
    world
        .save_to_disk(slot)
        .map_err(|e| glyph::EvalError::Custom(e))?;
    world
        .event_log
        .push_colored(format!("Game saved to slot {}.", slot), RGB::named(GREEN));
    Ok(Value::I64(slot as i64))
}

fn builtin_load(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    use bracket_lib::prelude::GREEN;
    let slot: u32 = match args.first() {
        Some(Value::I64(n)) if *n >= 0 => *n as u32,
        None => 1,
        _ => {
            return Err(glyph::EvalError::TypeError {
                expected: "non-negative integer slot number",
                got: args.first().map(|v| v.to_string()).unwrap_or_default(),
            })
        }
    };
    let loaded = World::load_from_disk(slot).map_err(|e| glyph::EvalError::Custom(e))?;
    *world = loaded;
    world.event_log.push_colored(
        format!(
            "Game loaded from slot {}. Use (wipe! {}) to delete the save.",
            slot, slot
        ),
        RGB::named(GREEN),
    );
    Ok(Value::I64(slot as i64))
}

fn builtin_wipe(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let slot: u32 = match args.first() {
        Some(Value::I64(n)) if *n >= 0 => *n as u32,
        None => 0,
        _ => {
            return Err(glyph::EvalError::TypeError {
                expected: "non-negative integer slot number",
                got: args.first().map(|v| v.to_string()).unwrap_or_default(),
            })
        }
    };
    world.pending_wipe_slot = Some(slot);
    world.event_log.push_colored(
        format!(
            "Type 'i am aware of what i am doing.' in console to wipe slot {}.",
            slot
        ),
        RGB::named(RED),
    );
    Ok(Value::Nil)
}

fn builtin_query_registry(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let mode = args.first().cloned().unwrap_or(glyph::kw("all"));
    match mode {
        Value::Keyword(ref kw) if kw.name == "suppressed-fragments" => {
            let suppressed = world.fragment_registry.suppressed();
            let list: Vec<Value> = suppressed
                .into_iter()
                .map(|f| {
                    let mut m: BTreeMap<Value, Value> = BTreeMap::new();
                    m.insert(Value::String("id".into()), Value::String(f.id.clone()));
                    m.insert(Value::String("weight".into()), Value::I64(f.weight as i64));
                    Value::Map(m)
                })
                .collect();
            Ok(Value::List(list))
        }
        Value::Keyword(ref kw) if kw.name == "all" => {
            let fragments = world.fragment_registry.all();
            let list: Vec<Value> = fragments
                .iter()
                .map(|f| {
                    let mut m: BTreeMap<Value, Value> = BTreeMap::new();
                    m.insert(Value::String("id".into()), Value::String(f.id.clone()));
                    m.insert(Value::String("weight".into()), Value::I64(f.weight as i64));
                    m.insert(
                        Value::String("collected".into()),
                        Value::Bool(f.status == crate::fragment::FragmentStatus::Collected),
                    );
                    m.insert(
                        Value::String("suppressed".into()),
                        Value::Bool(f.status == crate::fragment::FragmentStatus::Suppressed),
                    );
                    Value::Map(m)
                })
                .collect();
            Ok(Value::List(list))
        }
        _ => Err(glyph::EvalError::Custom(
            "usage: (query-registry :suppressed-fragments) or (query-registry :all)".into(),
        )),
    }
}

fn builtin_inspect_fragment(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let fragment_id = match args.first() {
        Some(Value::Keyword(kw)) => format!("frag-{:03}", {
            let s = &kw.name;
            s.parse::<u32>()
                .map_err(|_| glyph::EvalError::Custom(format!("invalid fragment id: {}", s)))?
        }),
        Some(Value::String(s)) => s.clone(),
        _ => {
            return Err(glyph::EvalError::Custom(
                "usage: (inspect-fragment :frag-001) or (inspect-fragment \"frag-001\")".into(),
            ))
        }
    };

    match world.fragment_registry.get(&fragment_id) {
        Some(frag) => {
            let mut m: BTreeMap<Value, Value> = BTreeMap::new();
            m.insert(Value::String("id".into()), Value::String(frag.id.clone()));
            m.insert(
                Value::String("text".into()),
                Value::String(frag.text.clone()),
            );
            m.insert(
                Value::String("weight".into()),
                Value::I64(frag.weight as i64),
            );
            let status = match frag.status {
                crate::fragment::FragmentStatus::Suppressed => "suppressed",
                crate::fragment::FragmentStatus::Hidden => "hidden",
                crate::fragment::FragmentStatus::Collected => "collected",
            };
            m.insert(
                Value::String("status".into()),
                Value::String(status.to_string()),
            );
            Ok(Value::Map(m))
        }
        None => Err(glyph::EvalError::Custom(format!(
            "no fragment with id: {}",
            fragment_id
        ))),
    }
}

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
    fn console_print_output_stays_in_console() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "(println \"hello\" \"glyph\")".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert_eq!(world.console_output, "hello glyph");
        assert!(world.console_buffer.is_empty());
    }

    #[test]
    fn console_string_results_are_readable_text() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.mode = Mode::Console;
        world.console_buffer = "\"line one\nline two\"".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert_eq!(world.console_output, "=> line one\nline two");
        assert!(world.console_buffer.is_empty());
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
    fn descending_from_level_2_clears_barrels_and_signs() {
        let mut world = World::new_game();
        world.depth = 2;
        world.wizard_taught = true;
        world.bindings.insert("z".into(), "(do-attack)".into());
        crate::levels::build_level(&mut world, 2);

        let barrel_depth_entities = world.renderable_entities().count();
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
        assert!(world.renderable_entities().count() < barrel_depth_entities);
        assert!(!world
            .renderable_entities()
            .any(|entity| entity.kind == EntityKind::Barrel));
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
