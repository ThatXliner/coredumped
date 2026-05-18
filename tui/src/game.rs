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
    map::{Map, TileType},
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
    /// Execute a keybinding (checks bindings map).
    ExecuteBinding(String),
    Move(crate::entity::Direction),
    Wait,
    /// Scroll in overlays (inspector, keybindings).
    InspectorScroll(i32),
    ConsoleInput(char),
    ConsoleBackspace,
    ConsoleSubmit,
    CloseOverlay,
    ToggleConsole,
    ToggleKeybindings,
    Respawn,
    Restart,
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
            bindings: default_bindings(),
        }
    }

    /// Create a world with a procedurally generated dungeon starting at depth 1.
    pub fn new_game() -> Self {
        let depth = 1;

        let mut event_log = EventLog::new();
        event_log.push("Xlyph runtime booted.");
        event_log.push("Move with arrows or hjkl. ` opens the console. i inspects code.");
        event_log.push("Your flashlight ray-casts in the direction you last moved.");
        event_log.push("You are helpless. Find the wizard to learn the art of striking.");
        event_log.push(format!("Depth {depth}. Find the stairs down."));

        let registry = RuleRegistry::core();
        let mut ecs = Ecs::new();
        // Temporary position — build_level will move the player to the correct start
        let player_id = ecs.spawn_player(Position::new(0, 0));

        let glyph_env = setup_glyph_env();

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
            inspector_selection: 0,
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: default_bindings(),
        };

        crate::levels::build_level(&mut world, depth);
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
                self.finish_tick();
                ActionCost::Tick
            }
            Intent::ExecuteBinding(key) => {
                let before = self.turn;
                self.execute_binding(&key);
                if !self.running {
                    ActionCost::Quit
                } else if self.turn > before {
                    ActionCost::Tick
                } else {
                    ActionCost::Free
                }
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

    /// Spawn a depth-appropriate enemy at the given position.
    pub(crate) fn spawn_enemy_at(&mut self, pos: Position, depth: u32) {
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

    pub(crate) fn spawn_boss_at(&mut self, pos: Position) {
        self.ecs.spawn_ogre(pos);
    }

    fn descend(&mut self) {
        self.depth += 1;
        self.clear_all_enemies();
        crate::levels::build_level(self, self.depth);
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
        self.clear_all_enemies();
        crate::levels::build_level(self, self.depth);
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
        self.clear_all_enemies();
        self.ecs
            .set_hp(self.player_id, Hp::new(self.player_hp().max));
        crate::levels::build_level(self, self.depth);
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
                self.event_log.push_colored(line.to_string(), RGB::named(CYAN));
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

                        if last == glyph::kw("quit-terminal") {
                            self.console_output = "Terminal closed.".to_string();
                            self.event_log.push("Terminal closed.");
                            self.console_buffer.clear();
                            self.mode = Mode::Normal;
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
                self.finish_tick();
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
    m.insert(">".into(), "(descend!)".into());
    m.insert("<".into(), "(ascend!)".into());
    m.insert("i".into(), "(toggle-inspector!)".into());
    m.insert("`".into(), "(toggle-console!)".into());
    m.insert("tab".into(), "(toggle-keybindings!)".into());
    m.insert("esc".into(), "(quit!)".into());
    m.insert("q".into(), "(quit!)".into());
    m
}

fn setup_glyph_env() -> Env {
    let env = Env::extend(&glyph::default_env());

    macro_rules! reg {
        ($name:expr, $func:ident) => {
            env.bind(
                $name,
                Value::Builtin(glyph::BuiltinFn {
                    name: $name,
                    func: $func,
                }),
            );
        };
    }

    reg!("help", builtin_help);
    reg!("quit-terminal", builtin_quit_terminal);
    reg!("quit!", builtin_quit_bang);
    reg!("do-attack", builtin_do_attack);
    reg!("bind-key", builtin_bind_key);
    reg!("move!", builtin_move);
    reg!("wait!", builtin_wait);
    reg!("block!", builtin_block);
    reg!("toggle-inspector!", builtin_toggle_inspector);
    reg!("toggle-console!", builtin_toggle_console);
    reg!("toggle-keybindings!", builtin_toggle_keybindings);
    reg!("descend!", builtin_descend);
    reg!("ascend!", builtin_ascend);
    ai_builtins::register_all(&env);
    env
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
    world.running = false;
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
    world.apply_player_move(dir);
    world.finish_tick();
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
    world.blocking = true;
    world.event_log.push("You raise your guard.");
    world.finish_tick();
    Ok(Value::Nil)
}

fn builtin_toggle_inspector(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    world.mode = if world.mode == Mode::Inspector {
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
    let has_attack_binding = world
        .bindings
        .values()
        .any(|cmd| cmd.strip_prefix(':').is_some_and(|rest| ATTACK_SENTINEL_NAMES.contains(&rest)));
    if world.depth >= 3 && (!world.wizard_taught || !has_attack_binding) {
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

/// Keyword names returned by (do-attack) used as sentinel values that signal
/// "this binding produces an attack action" to both the binding executor and
/// the descend gate.
const ATTACK_SENTINEL_NAMES: &[&str] = &[
    "player-attack-facing",
    "player-attack-north",
    "player-attack-south",
    "player-attack-east",
    "player-attack-west",
];

/// Look up the sentinel keyword name for an optional direction (None = facing).
fn attack_sentinel_name(dir: Option<Direction>) -> &'static str {
    match dir {
        None => "player-attack-facing",
        Some(Direction::North) => "player-attack-north",
        Some(Direction::South) => "player-attack-south",
        Some(Direction::East) => "player-attack-east",
        Some(Direction::West) => "player-attack-west",
    }
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
        return Ok(glyph::kw(attack_sentinel_name(None)));
    }
    if args.len() != 1 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    match parse_attack_direction(&args[0]) {
        Some(dir) => Ok(glyph::kw(attack_sentinel_name(Some(dir)))),
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
    world: &mut World,
) -> glyph::EvalResult<Value> {
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
  (quit-terminal) — close the console overlay",
    );

    if world.player_can_attack {
        help.push_str(
            "\n  (do-attack :dir) — strike in direction (:north/:south/:east/:west/:facing)\n\
             \n  (bind-key :k (expr)) — bind a key to a Glyph expression",
        );
    }

    Ok(Value::String(help))
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
        world.console_buffer = "(bin".to_string();

        world.apply_intent(Intent::ConsoleSubmit);

        assert!(world.console_output.contains("syntax error"));
        assert!(!world.console_output.contains('\u{1b}'));
        assert_eq!(world.console_output_color, Some(RGB::named(RED)));
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
        world.bindings.insert("a".into(), "(do-attack)".into());

        world.apply_intent(Intent::ExecuteBinding("a".into()));

        assert_eq!(world.turn, 1);
        assert!(world.event_log.contains("don't know how to attack"));
    }

    #[test]
    fn attack_key_hits_enemy_in_facing_direction() {
        let mut world = world_with_single_enemy(Position::new(6, 5));
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
        let output = crate::levels::generate_wizard_box();
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

        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

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
            .insert("z".into(), ":player-attack-facing".into());
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

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

        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

        assert_eq!(cost, ActionCost::Free);
        assert_eq!(world.depth, 3);
        assert!(world.event_log.contains("barrier"));
    }

    #[test]
    fn console_bind_attack_allows_descend_at_depth_3() {
        let mut world = World::new();
        world.depth = 3;
        world.wizard_taught = true;
        world.clear_all_enemies();
        world.map.set_tile(world.player_pos(), TileType::StairsDown);

        // Bind (do-attack) to `z` via the console — the real code path
        // that evaluates the form before storing the keyword sentinel.
        world.mode = Mode::Console;
        world.console_buffer = "(bind-key :z (do-attack))".to_string();
        world.apply_intent(Intent::ConsoleSubmit);

        // Confirm the binding was stored as the evaluated sentinel
        assert_eq!(
            world.bindings.get("z").map(|s| s.as_str()),
            Some(":player-attack-facing")
        );

        // Now descend should work
        let cost = world.apply_intent(Intent::ExecuteBinding(">".into()));

        assert_eq!(cost, ActionCost::Tick);
        assert_eq!(world.depth, 4);
    }
}
