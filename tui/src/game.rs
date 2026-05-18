//! Deterministic gameplay systems for the prototype.
//!
//! `World` owns resources such as the map, turn counter, UI mode, and event
//! log. Dynamic actors live in the ECS store, and this module provides the
//! systems that read or mutate those components: player intent handling,
//! enemy AI, ticking, console state, and inspector state.

use std::collections::HashMap;

use bracket_lib::prelude::{
    a_star_search, NavigationPath, CYAN, DARK_GRAY, GREEN, ORANGE, RED, RGB,
};

use crate::{
    ai_builtins,
    ecs::Ecs,
    entity::{Direction, EntityId, EntityKind, EntityView, Hp, Position},
    event_log::EventLog,
    glyph::{self, Env, Value},
    map::{Map, MapGenOutput, TileType, MAP_HEIGHT, MAP_WIDTH},
    rules::RuleRegistry,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionCost {
    Free,
    Tick,
    Quit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Intent {
    Move(Direction),
    Wait,
    ToggleInspector,
    ToggleConsole,
    InspectorScroll(i32),
    ConsoleInput(char),
    ConsoleBackspace,
    ConsoleSubmit,
    CloseOverlay,
    Descend,
    Ascend,
    Block,
    Attack,
    Respawn,
    Restart,
    ToggleKeybindings,
    ExecuteBinding(String),
    Quit,
    Noop,
}

use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        event_log.push("Your flashlight ray-casts in the direction you last moved.");
        event_log.push("You are helpless. Find the wizard to learn the art of striking.");

        let registry = RuleRegistry::core();

        let mut ecs = Ecs::new();
        let player_id = ecs.spawn_player(Position::new(5, 5));
        ecs.spawn_slime(Position::new(19, 5));
        ecs.spawn_slime(Position::new(47, 18));

        let glyph_env = setup_glyph_env();

        Self {
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
            inspector_selection: 0,
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: HashMap::new(),
        }
    }

    /// Create a world with a procedurally generated dungeon starting at depth 1.
    pub fn new_game() -> Self {
        let depth = 1;
        let gen = Self::generate_level(depth);
        let map = gen.map;
        let player_start = gen.player_start;
        let combat_spawns = gen.combat_spawns.clone();
        let boss_spawns = gen.boss_spawns.clone();

        let mut event_log = EventLog::new();
        event_log.push("Xlyph runtime booted.");
        event_log.push("Move with arrows or hjkl. ` opens the console. i inspects code.");
        event_log.push("Your flashlight ray-casts in the direction you last moved.");
        event_log.push("You are helpless. Find the wizard to learn the art of striking.");
        event_log.push(format!("Depth {depth}. Find the stairs down."));

        let registry = RuleRegistry::core();
        let mut ecs = Ecs::new();
        let player_id = ecs.spawn_player(player_start);

        let glyph_env = setup_glyph_env();

        let mut world = Self {
            map,
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
            inspector_selection: 0,
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: HashMap::new(),
        };

        world.spawn_level_enemies_from(&combat_spawns, &boss_spawns);
        world
    }

    pub fn apply_intent(&mut self, intent: Intent) -> ActionCost {
        match intent {
            Intent::Move(direction) => {
                self.player_facing = direction;
                self.apply_player_move(direction);
                self.finish_tick();
                ActionCost::Tick
            }
            Intent::Wait => {
                self.event_log.push("You wait (one tick advanced).");
                self.finish_tick();
                ActionCost::Tick
            }
            Intent::ToggleInspector => {
                self.mode = if self.mode == Mode::Inspector {
                    Mode::Normal
                } else {
                    Mode::Inspector
                };
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
            Intent::InspectorScroll(delta) => {
                if self.mode == Mode::Inspector {
                    self.scroll_inspector(delta);
                }
                ActionCost::Free
            }
            Intent::ConsoleInput(ch) => {
                if self.mode == Mode::Console {
                    self.console_buffer.push(ch);
                }
                ActionCost::Free
            }
            Intent::ConsoleBackspace => {
                if self.mode == Mode::Console {
                    self.console_buffer.pop();
                }
                ActionCost::Free
            }
            Intent::ConsoleSubmit => {
                if self.mode == Mode::Console {
                    self.submit_console();
                }
                ActionCost::Free
            }
            Intent::CloseOverlay => {
                self.mode = Mode::Normal;
                ActionCost::Free
            }
            Intent::ToggleKeybindings => {
                self.mode = if self.mode == Mode::Keybindings {
                    Mode::Normal
                } else {
                    Mode::Keybindings
                };
                ActionCost::Free
            }
            Intent::Descend => {
                if self.map.tile(self.player_pos()) == TileType::StairsDown {
                    let has_attack_binding =
                        self.bindings.values().any(|cmd| cmd.contains("do-attack"));
                    if self.depth >= 3 && (!self.wizard_taught || !has_attack_binding) {
                        self.event_log.push("A shimmering barrier blocks the stairs. The wizard's voice echoes: \"Bind your attack to a key first! Open the console (`) and try (bind-key :z (do-attack :facing)).\"");
                        ActionCost::Free
                    } else {
                        self.descend();
                        ActionCost::Tick
                    }
                } else {
                    self.event_log.push("There are no stairs going down here.");
                    ActionCost::Free
                }
            }
            Intent::Ascend => {
                if self.map.tile(self.player_pos()) == TileType::StairsUp {
                    self.ascend();
                    ActionCost::Tick
                } else {
                    self.event_log.push("There are no stairs going up here.");
                    ActionCost::Free
                }
            }
            Intent::Attack => {
                if !self.player_can_attack {
                    self.event_log
                        .push("You flail uselessly. Find the wizard to learn how to fight!");
                } else {
                    self.attack_in_direction(self.player_facing);
                }
                self.finish_tick();
                ActionCost::Tick
            }
            Intent::Block => {
                self.blocking = true;
                self.event_log.push("You raise your guard.");
                self.finish_tick();
                ActionCost::Tick
            }
            Intent::Respawn => {
                self.respawn();
                ActionCost::Free
            }
            Intent::Restart => {
                self.restart();
                ActionCost::Free
            }
            Intent::ExecuteBinding(command) => {
                self.execute_binding(&command);
                ActionCost::Tick
            }
            Intent::Quit => {
                self.running = false;
                ActionCost::Quit
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

    pub fn enemy_ai_path(&self, enemy_id: EntityId) -> NavigationPath {
        let enemy_pos = self
            .ecs
            .position(enemy_id)
            .expect("enemy should always have a Position component");

        a_star_search(
            self.map.idx(enemy_pos),
            self.map.idx(self.player_pos()),
            &self.map,
        )
    }

    /// Generate a level appropriate for the given depth.
    /// Depth 1, 3, 5... = rooms-and-corridors; depth 2, 4, 6... = caves.
    /// Depth 3 is the wizard's tutorial chamber.
    fn generate_level(depth: u32) -> MapGenOutput {
        if depth == 3 {
            Map::generate_wizard_box()
        } else if depth % 2 == 0 {
            Map::generate_cave(depth)
        } else {
            Map::generate(MAP_WIDTH, MAP_HEIGHT, depth)
        }
    }

    /// Spawn enemies on a freshly generated level.
    fn spawn_level_enemies_from(&mut self, combat_spawns: &[Position], boss_spawns: &[Position]) {
        if self.depth == 3 {
            // Depth 3 is the wizard's chamber: no enemies, just the wizard
            self.spawn_wizard_near_player();
            return;
        }

        if self.depth == 4 {
            // Depth 4 is the Barrel Depths: mix of barrels and enemies
            for pos in combat_spawns {
                let hash = (pos.x.wrapping_mul(31).wrapping_add(pos.y.wrapping_mul(17))) as u32;
                if hash % 3 == 0 {
                    self.spawn_enemy_at(*pos, self.depth);
                } else {
                    self.ecs.spawn_barrel(*pos);
                }
            }
            for pos in boss_spawns {
                self.spawn_boss_at(*pos);
            }
            // Place a barrel on the stairs
            let stairs = find_stairs_down(&self.map);
            self.ecs.spawn_barrel(stairs);
            // Spawn sign near player
            self.spawn_sign_near_player();
            self.spawn_wizard_near_player();
            return;
        }

        for pos in combat_spawns {
            self.spawn_enemy_at(*pos, self.depth);
        }
        for pos in boss_spawns {
            self.spawn_boss_at(*pos);
        }

        // Spawn wizard on depth 4+
        if self.depth >= 4 {
            self.spawn_wizard_near_player();
        }
    }

    fn spawn_wizard_near_player(&mut self) {
        let player_pos = self.player_pos();
        let candidates = [
            player_pos.offset(2, 0),
            player_pos.offset(-2, 0),
            player_pos.offset(0, 2),
            player_pos.offset(0, -2),
            player_pos.offset(3, 0),
            player_pos.offset(-3, 0),
            player_pos.offset(0, 3),
            player_pos.offset(0, -3),
        ];
        let wizard_pos = candidates
            .iter()
            .copied()
            .find(|&p| self.map.is_walkable(p) && self.ecs.entity_at(p).is_none())
            .unwrap_or(player_pos.offset(2, 0));
        self.wizard_id = Some(self.ecs.spawn_wizard(wizard_pos));
    }

    fn spawn_sign_near_player(&mut self) {
        let player_pos = self.player_pos();
        let candidates = [
            player_pos.offset(3, 0),
            player_pos.offset(-3, 0),
            player_pos.offset(0, 3),
            player_pos.offset(0, -3),
            player_pos.offset(4, 0),
            player_pos.offset(-4, 0),
            player_pos.offset(0, 4),
            player_pos.offset(0, -4),
        ];
        if let Some(&pos) = candidates
            .iter()
            .find(|&&p| self.map.is_walkable(p) && self.ecs.entity_at(p).is_none())
        {
            self.ecs.spawn_sign(pos);
        }
    }

    /// Spawn a depth-appropriate enemy at the given position.
    fn spawn_enemy_at(&mut self, pos: Position, depth: u32) {
        let hash = (pos.x.wrapping_mul(31).wrapping_add(pos.y.wrapping_mul(17))) as u32;
        let roll = (hash % 100) as i32;
        match depth {
            1..=3 => {
                // Before checkpoint: only slimes — simple enemies for the helpless phase
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

    fn spawn_boss_at(&mut self, pos: Position) {
        self.ecs.spawn_ogre(pos);
    }

    fn descend(&mut self) {
        self.depth += 1;
        let gen = Self::generate_level(self.depth);
        let player_start = gen.player_start;
        let combat_spawns = gen.combat_spawns;
        let boss_spawns = gen.boss_spawns;
        self.clear_all_enemies();
        self.ecs.set_position(self.player_id, player_start);
        self.map = gen.map;
        self.spawn_level_enemies_from(&combat_spawns, &boss_spawns);
        self.event_log
            .push(format!("You descend to depth {}.", self.depth));
        self.turn += 1;
    }

    fn ascend(&mut self) {
        if self.depth <= 1 {
            self.event_log.push("You are already at the surface.");
            return;
        }
        self.depth -= 1;
        let gen = Self::generate_level(self.depth);
        let player_start = gen.player_start;
        let combat_spawns = gen.combat_spawns;
        let boss_spawns = gen.boss_spawns;
        self.clear_all_enemies();
        self.ecs.set_position(self.player_id, player_start);
        self.map = gen.map;
        self.spawn_level_enemies_from(&combat_spawns, &boss_spawns);
        self.event_log
            .push(format!("You ascend to depth {}.", self.depth));
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

    fn respawn(&mut self) {
        let gen = Self::generate_level(self.depth);
        self.map = gen.map;
        self.ecs.set_position(self.player_id, gen.player_start);
        self.clear_all_enemies();
        self.ecs
            .set_hp(self.player_id, Hp::new(self.player_hp().max));
        self.spawn_level_enemies_from(&gen.combat_spawns, &gen.boss_spawns);
        self.mode = Mode::Normal;
        self.player_facing = Direction::East;
        self.event_log.push("You gasp back into existence!");
    }

    fn restart(&mut self) {
        *self = World::new_game();
    }

    fn apply_player_move(&mut self, direction: Direction) {
        let (dx, dy) = direction.delta();
        let target = self.player_pos().offset(dx, dy);

        if !self.map.is_walkable(target) {
            self.event_log
                .push("You bump into a wall. Time still moves.");
            return;
        }

        if let Some(target_id) = self.ecs.entity_at(target) {
            match self.ecs.kind(target_id) {
                Some(EntityKind::Wizard) => {
                    self.interact_with_wizard(target_id);
                    return;
                }
                Some(EntityKind::Sign) => {
                    self.interact_with_sign(target_id);
                    return;
                }
                Some(EntityKind::Barrel) => {
                    self.bump_barrel(target_id);
                    return;
                }
                _ => {}
            }

            if !self.player_can_attack {
                self.event_log.push(format!(
                    "You helplessly shove the {}. Find the wizard to learn how to fight!",
                    self.ecs.name(target_id)
                ));
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
            return;
        }

        self.ecs.set_position(self.player_id, target);
        self.event_log.push_colored(
            format!("You move to {},{}.", target.x, target.y),
            RGB::named(DARK_GRAY),
        );
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

    fn interact_with_wizard(&mut self, _wizard_id: EntityId) {
        if self.wizard_taught {
            self.event_log.push_colored(
                "The wizard smiles. \"You already know the art of striking.\"",
                RGB::named(CYAN),
            );
            let max_hp = self.player_hp().max;
            self.ecs.set_hp(self.player_id, Hp::new(max_hp));
            self.event_log.push_colored(
                "The wizard taps your shoulder. You feel refreshed.",
                RGB::named(CYAN),
            );
            return;
        }

        let max_hp = self.player_hp().max;
        self.ecs.set_hp(self.player_id, Hp::new(max_hp));
        self.player_can_attack = true;
        self.wizard_taught = true;

        self.event_log
            .push_colored("The wizard raises a glowing hand...", RGB::named(CYAN));
        self.event_log.push_colored(
            "\"Ah, a lost soul! Let me mend your wounds.\"",
            RGB::named(CYAN),
        );
        self.event_log.push_colored(
            "Warmth spreads through your body. HP fully restored.",
            RGB::named(CYAN),
        );
        self.event_log.push_colored(
            "\"Now — you are not helpless. I teach you the art of striking.\"",
            RGB::named(CYAN),
        );
        self.event_log.push_colored(
            "Open the console (`) and bind attack to a key:",
            RGB::named(CYAN),
        );
        self.event_log
            .push_colored("  (bind-key :z (do-attack :facing))", RGB::named(GREEN));
        self.event_log.push_colored(
            "  (bind-key :x (do-attack :east))   (bind-key :c (do-attack :west))",
            RGB::named(GREEN),
        );
        self.event_log.push_colored(
            "\"Strike with purpose, traveler — once you bind it, the way down will open.\"",
            RGB::named(CYAN),
        );
    }

    fn interact_with_sign(&mut self, _sign_id: EntityId) {
        self.event_log.push("===================================");
        self.event_log
            .push_colored("              SIGN", RGB::named(CYAN));
        self.event_log.push("===================================");
        self.event_log
            .push_colored("Welcome to the Barrel Depths!", RGB::named(CYAN));
        self.event_log.push("");
        self.event_log.push("Each (do-attack) costs 1 tick. But");
        self.event_log.push("you can chain them with (do ...):");
        self.event_log.push_colored(
            "  (do (do-attack :north) (do-attack :south))",
            RGB::named(GREEN),
        );
        self.event_log.push("That attacks twice — 2 ticks total.");
        self.event_log.push("Bind the full combo to ONE key:");
        self.event_log.push_colored(
            "  (bind-key :x (do (do-attack :north) (do-attack :south)",
            RGB::named(GREEN),
        );
        self.event_log.push_colored(
            "                  (do-attack :east)  (do-attack :west)))",
            RGB::named(GREEN),
        );
        self.event_log
            .push("Now clear these barrels and find the exit!");
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
        self.advance_enemies();
        self.blocking = false;

        if self.player_hp().current <= 0 {
            self.mode = Mode::Dead;
            self.event_log.push("You have perished!");
        }
    }

    fn advance_enemies(&mut self) {
        let enemy_ids: Vec<EntityId> = self.ecs.enemy_ids().collect();

        let sandbox = glyph::SandboxOptions::default();

        for enemy_id in enemy_ids {
            if !self.ecs.is_alive(enemy_id) {
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

    fn submit_console(&mut self) {
        let command = self.console_buffer.trim().to_string();
        if command.is_empty() {
            self.event_log.push("Console waits. No query submitted.");
            self.console_buffer.clear();
            return;
        }
        self.event_log.push(format!("> {}", command));
        self.console_output.clear();
        self.console_output_color = None;
        match glyph::read_string(&command) {
            Ok(forms) => {
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
                        // Check for do-attack sentinel
                        if let Value::Keyword(ref kw) = last {
                            let dir = match kw.name.as_str() {
                                "player-attack-north" => Some(Direction::North),
                                "player-attack-south" => Some(Direction::South),
                                "player-attack-east" => Some(Direction::East),
                                "player-attack-west" => Some(Direction::West),
                                "player-attack-facing" => Some(self.player_facing),
                                _ => None,
                            };
                            if let Some(direction) = dir {
                                if !self.player_can_attack {
                                    self.console_output =
                                        "You don't know how to attack yet. Find the wizard.".into();
                                    self.event_log.push("You flail uselessly.");
                                } else {
                                    self.player_facing = direction;
                                    self.attack_in_direction(direction);
                                    self.finish_tick();
                                    self.console_output =
                                        format!("You attack {:?}. Turn {}.", direction, self.turn);
                                }
                                self.console_buffer.clear();
                                return;
                            }
                        }

                        if last == glyph::kw("quit") {
                            self.console_output = "Quitting. Goodbye.".to_string();
                            self.event_log.push("Quitting. Goodbye.");
                            self.console_buffer.clear();
                            self.running = false;
                            return;
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

        let env = self.glyph_env.clone();
        let mut last = Value::Nil;
        let mut err = None;
        for form in &forms {
            match glyph::eval_with_opts(form, &env, glyph::SandboxOptions::default(), self) {
                Ok(val) => last = val,
                Err(e) => {
                    err = Some(e);
                    break;
                }
            }
        }

        if let Some(e) = err {
            self.event_log.push(format!("Binding error: {}", e));
            return;
        }

        // Check for do-attack sentinel
        if let Value::Keyword(ref kw) = last {
            let dir = match kw.name.as_str() {
                "player-attack-north" => Some(Direction::North),
                "player-attack-south" => Some(Direction::South),
                "player-attack-east" => Some(Direction::East),
                "player-attack-west" => Some(Direction::West),
                "player-attack-facing" => Some(self.player_facing),
                _ => None,
            };
            if let Some(direction) = dir {
                if !self.player_can_attack {
                    self.event_log
                        .push("You don't know how to attack yet. Find the wizard.");
                } else {
                    self.player_facing = direction;
                    self.attack_in_direction(direction);
                }
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

fn find_stairs_down(map: &Map) -> Position {
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Position::new(x, y);
            if map.tile(pos) == TileType::StairsDown {
                return pos;
            }
        }
    }
    Position::new(2, 2)
}

fn setup_glyph_env() -> Env {
    let env = Env::extend(&glyph::default_env());
    env.bind(
        "help",
        Value::Builtin(glyph::BuiltinFn {
            name: "help",
            func: builtin_help,
        }),
    );
    env.bind(
        "quit",
        Value::Builtin(glyph::BuiltinFn {
            name: "quit",
            func: builtin_quit,
        }),
    );
    env.bind(
        "do-attack",
        Value::Builtin(glyph::BuiltinFn {
            name: "do-attack",
            func: builtin_do_attack,
        }),
    );
    env.bind(
        "bind-key",
        Value::Builtin(glyph::BuiltinFn {
            name: "bind-key",
            func: builtin_bind_key,
        }),
    );
    ai_builtins::register_all(&env);
    env
}

fn builtin_quit(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    _world: &mut World,
) -> glyph::EvalResult<Value> {
    Ok(glyph::kw("quit"))
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

fn builtin_do_attack(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    _world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.is_empty() {
        return Ok(glyph::kw("player-attack-facing"));
    }
    if args.len() != 1 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    match parse_attack_direction(&args[0]) {
        Some(dir) => {
            let sentinel = match dir {
                Direction::North => "player-attack-north",
                Direction::South => "player-attack-south",
                Direction::East => "player-attack-east",
                Direction::West => "player-attack-west",
            };
            Ok(glyph::kw(sentinel))
        }
        None => Err(glyph::EvalError::TypeError {
            expected: "direction keyword (:north, :south, :east, :west)",
            got: format!("{}", args[0]),
        }),
    }
}

fn builtin_bind_key(
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
    let key = match &args[0] {
        Value::Keyword(kw) => kw.name.clone(),
        other => {
            return Err(glyph::EvalError::TypeError {
                expected: "keyword (e.g. :z, :x)",
                got: other.to_string(),
            })
        }
    };
    if key.is_empty() || key.len() != 1 {
        return Err(glyph::EvalError::TypeError {
            expected: "single-character keyword (e.g. :z, :x)",
            got: format!(":{}", key),
        });
    }
    let command = args[1].to_string();
    world.bindings.insert(key.clone(), command.clone());
    world.event_log.push_colored(
        format!("Bound key '{}' to: {}", key, command),
        RGB::named(GREEN),
    );
    Ok(args[1].clone())
}

fn builtin_help(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    _world: &mut World,
) -> glyph::EvalResult<Value> {
    Ok(Value::String(
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

Console commands (game-specific):
  (help)        — show this help text
  (quit)        — exit the game
  (do-attack :dir) — strike in direction (:north/:south/:east/:west/:facing)
  (bind-key :k (expr)) — bind a key to a Glyph expression"
            .into(),
    ))
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

        let cost = world.apply_intent(Intent::ToggleInspector);

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.turn, 0);
        assert_eq!(world.mode, Mode::Inspector);
    }

    #[test]
    fn console_toggle_and_typing_are_free() {
        let mut world = world_with_single_enemy(Position::new(20, 5));

        assert_eq!(world.apply_intent(Intent::ToggleConsole), ActionCost::Free);
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
        world.console_buffer = "(bin".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world.console_output.contains("syntax error"));
        assert!(!world.console_output.contains('\u{1b}'));
        assert_eq!(world.console_output_color, Some(RGB::named(RED)));
        assert!(world.console_buffer.is_empty());
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

        world.apply_intent(Intent::Move(Direction::East));

        let enemy_after = single_enemy(&world);
        assert_eq!(enemy_after.hp.current, initial_hp);
        assert!(world.event_log.contains("helplessly shove"));
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
    fn helpless_attack_key_flails() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.player_can_attack = false;

        world.apply_intent(Intent::Attack);

        assert_eq!(world.turn, 1);
        assert!(world.event_log.contains("flail"));
    }

    #[test]
    fn attack_key_hits_enemy_in_facing_direction() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
        world.player_can_attack = true;
        world.player_facing = Direction::East;

        world.apply_intent(Intent::Attack);

        assert_eq!(world.turn, 1);
        assert_eq!(world.player_pos(), Position::new(5, 5)); // didn't move
        assert_eq!(single_enemy(&world).hp.current, 2); // took 1 damage
        assert!(world.event_log.contains("strike"));
    }

    #[test]
    fn attack_key_swings_at_empty_air() {
        let mut world = world_with_single_enemy(Position::new(20, 5));
        world.player_can_attack = true;
        world.player_facing = Direction::North;

        world.apply_intent(Intent::Attack);

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

        world.apply_intent(Intent::Move(Direction::East));

        assert!(world.player_can_attack);
        assert!(world.wizard_taught);
        assert_eq!(world.player_hp().current, 12);
        assert!(world.event_log.contains("art of striking"));
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
        assert!(world.event_log.contains("already know"));
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

    #[test]
    fn do_attack_builtin_returns_direction_sentinel() {
        let mut world = World::minimal();
        let env = setup_glyph_env();
        let forms = crate::glyph::read_string("(do-attack :east)").unwrap();
        let result = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();
        assert_eq!(result, crate::glyph::kw("player-attack-east"));
    }

    #[test]
    fn do_attack_builtin_no_args_returns_facing_sentinel() {
        let mut world = World::minimal();
        let env = setup_glyph_env();
        let forms = crate::glyph::read_string("(do-attack)").unwrap();
        let result = crate::glyph::eval_with_opts(
            &forms[0],
            &env,
            crate::glyph::SandboxOptions::default(),
            &mut world,
        )
        .unwrap();
        assert_eq!(result, crate::glyph::kw("player-attack-facing"));
    }

    #[test]
    fn do_attack_rejects_non_direction() {
        let mut world = World::minimal();
        let env = setup_glyph_env();
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
        assert_eq!(world.depth, 1);
        assert_eq!(world.player_hp().current, 12);
        assert!(!world.player_can_attack);
    }

    // --- Depth 3 / wizard gating tests ---

    #[test]
    fn wizard_box_has_no_enemies() {
        let output = Map::generate_wizard_box();
        assert!(output.combat_spawns.is_empty());
        assert!(output.boss_spawns.is_empty());
    }

    #[test]
    fn descend_blocked_at_depth_3_without_wizard() {
        let mut world = World::new();
        world.depth = 3;
        world.wizard_taught = false;
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        let cost = world.apply_intent(Intent::Descend);

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.depth, 3);
        assert!(world.event_log.contains("barrier"));
    }

    #[test]
    fn descend_allowed_at_depth_3_with_wizard_and_binding() {
        let mut world = World::new();
        world.depth = 3;
        world.wizard_taught = true;
        world
            .bindings
            .insert("z".into(), "(do-attack :facing)".into());
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        let cost = world.apply_intent(Intent::Descend);

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.depth, 4);
    }

    #[test]
    fn descend_blocked_when_taught_but_not_bound() {
        let mut world = World::new();
        world.depth = 3;
        world.wizard_taught = true;
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        let cost = world.apply_intent(Intent::Descend);

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.depth, 3);
        assert!(world.event_log.contains("barrier"));
    }

    #[test]
    fn depth_3_level_generates_wizard_box() {
        // generate_level(3) should return a wizard box with no enemies
        let output = World::generate_level(3);
        assert!(output.combat_spawns.is_empty());
        assert!(output.boss_spawns.is_empty());
    }
}
