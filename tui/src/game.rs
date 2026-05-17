//! Deterministic gameplay systems for the prototype.
//!
//! `World` owns resources such as the map, turn counter, UI mode, and event
//! log. Dynamic actors live in the ECS store, and this module provides the
//! systems that read or mutate those components: player intent handling,
//! enemy AI, ticking, console state, and inspector state.

use bracket_lib::prelude::{a_star_search, NavigationPath};

use crate::{
    ecs::Ecs,
    entity::{Direction, EntityId, EntityView, Hp, Position},
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
    Quit,
    Noop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Inspector,
    Console,
}

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
    pub glyph_env: Env,
    pub inspector_scroll: usize,
    pub running: bool,
}

impl World {
    pub fn new() -> Self {
        let mut event_log = EventLog::new();
        event_log.push("Xlyph runtime booted.");
        event_log.push("Move with arrows or hjkl. ` opens the console. i inspects code.");
        event_log.push("Your flashlight ray-casts in the direction you last moved.");

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
            glyph_env,
            inspector_scroll: 0,
            running: true,
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
            glyph_env,
            inspector_scroll: 0,
            running: true,
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
                self.event_log
                    .push("You wait and listen to the engine tick.");
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
            Intent::Descend => {
                if self.map.tile(self.player_pos()) == TileType::StairsDown {
                    self.descend();
                    ActionCost::Tick
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
    fn generate_level(depth: u32) -> MapGenOutput {
        if depth % 2 == 0 {
            Map::generate_cave(depth)
        } else {
            Map::generate(MAP_WIDTH, MAP_HEIGHT, depth)
        }
    }

    /// Spawn enemies on a freshly generated level.
    fn spawn_level_enemies_from(&mut self, combat_spawns: &[Position], boss_spawns: &[Position]) {
        for pos in combat_spawns {
            self.spawn_enemy_at(*pos, self.depth);
        }
        for pos in boss_spawns {
            self.spawn_boss_at(*pos);
        }
    }

    /// Spawn a depth-appropriate enemy at the given position.
    fn spawn_enemy_at(&mut self, pos: Position, depth: u32) {
        let hash = (pos.x.wrapping_mul(31).wrapping_add(pos.y.wrapping_mul(17))) as u32;
        let roll = (hash % 100) as i32;
        match depth {
            1 => {
                self.ecs.spawn_slime(pos);
            }
            2..=3 => {
                if roll < 60 {
                    self.ecs.spawn_slime(pos);
                } else if roll < 90 {
                    self.ecs.spawn_goblin(pos);
                } else {
                    self.ecs.spawn_bat(pos);
                }
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
            let target_name = self.ecs.name(target_id);
            let hp = self
                .ecs
                .damage(target_id, 1)
                .expect("combat targets should have an Hp component");

            self.event_log
                .push(format!("You strike the {target_name} for 1 damage."));

            if hp.current <= 0 {
                self.event_log
                    .push(format!("The {target_name} collapses into inert code."));
            }
            return;
        }

        self.ecs.set_position(self.player_id, target);
        self.event_log
            .push(format!("You move to {},{}.", target.x, target.y));
    }

    fn finish_tick(&mut self) {
        self.turn += 1;
        self.advance_enemies();
    }

    fn advance_enemies(&mut self) {
        let enemy_ids: Vec<EntityId> = self.ecs.enemy_ids().collect();

        for enemy_id in enemy_ids {
            if !self.ecs.is_alive(enemy_id) {
                continue;
            }

            let enemy_pos = self
                .ecs
                .position(enemy_id)
                .expect("enemy should always have a Position component");

            if enemy_pos.manhattan_distance(self.player_pos()) == 1 {
                self.ecs.damage(self.player_id, 1);
                self.event_log.push(format!(
                    "The {} attacks exactly as slime-hunt says.",
                    self.ecs.name(enemy_id)
                ));
                continue;
            }

            if let Some(next_pos) = self.next_step_toward_player(enemy_id) {
                self.ecs.set_position(enemy_id, next_pos);
                self.event_log.push(format!(
                    "The {} steps toward the player.",
                    self.ecs.name(enemy_id)
                ));
            }
        }
    }

    fn next_step_toward_player(&self, enemy_id: EntityId) -> Option<Position> {
        let path = self.enemy_ai_path(enemy_id);

        if !path.success || path.steps.len() < 2 {
            return None;
        }

        let next_pos = self.map.position_for_idx(path.steps[1]);
        if next_pos == self.player_pos() || self.ecs.entity_at_except(next_pos, enemy_id).is_some()
        {
            return None;
        }

        Some(next_pos)
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
            self.inspector_scroll = self
                .inspector_scroll
                .saturating_sub(delta.unsigned_abs() as usize);
        } else {
            self.inspector_scroll = self.inspector_scroll.saturating_add(delta as usize);
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
        match glyph::read_string(&command) {
            Ok(forms) => {
                let mut last = Value::Nil;
                let mut err = None;
                for form in &forms {
                    match glyph::eval_with_opts(
                        form,
                        &self.glyph_env,
                        glyph::SandboxOptions::default(),
                    ) {
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
                    }
                    None => {
                        if last == glyph::kw("quit") {
                            self.console_output = "Quitting. Goodbye.".to_string();
                            self.event_log.push("Quitting. Goodbye.");
                            self.console_buffer.clear();
                            self.running = false;
                            return;
                        }
                        let msg = format!("=> {}", last);
                        self.event_log.push(&msg);
                        self.console_output = msg;
                    }
                }
            }
            Err(e) => {
                let report = e.report(&command);
                for line in report.lines() {
                    self.event_log.push(line);
                }
                self.console_output = report;
            }
        }
        self.console_buffer.clear();
    }
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
    env
}

fn builtin_quit(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    Ok(glyph::kw("quit"))
}

fn builtin_help(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
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
  (quit)        — exit the game"
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
    fn enemy_advances_after_each_tick_action() {
        let mut world = world_with_single_enemy(Position::new(10, 5));

        world.apply_intent(Intent::Wait);

        assert_eq!(world.turn, 1);
        assert_eq!(single_enemy(&world).pos, Position::new(9, 5));
    }

    #[test]
    fn adjacent_enemy_attacks_instead_of_moving() {
        let mut world = world_with_single_enemy(Position::new(6, 5));

        world.apply_intent(Intent::Wait);

        assert_eq!(world.turn, 1);
        assert_eq!(single_enemy(&world).pos, Position::new(6, 5));
        assert_eq!(world.player_hp().current, 11);
        assert!(world
            .event_log
            .contains("attacks exactly as slime-hunt says"));
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
}
