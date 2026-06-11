//! Tests for game.rs — player movement, combat, console, wizard, death/respawn, etc.

use bracket_color::prelude::{RED, RGB};

use crate::builtins::{builtin_block, builtin_do_attack, builtin_shove, setup_glyph_env};
use crate::entity::{Direction, EntityId, EntityKind, EntityView, Hp, Position};
use crate::event_log::EventLog;
use crate::game::{ActionCost, Intent, Mode};
use crate::glyph::{self, Env, Value};
use crate::map::TileType;
use crate::world::World;

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
    let overflow = crate::glyph::read_string("(copy-bytes! (bytes 64) (impact-payload))").unwrap();
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

/// Evaluate one Glyph form against a fresh game env, panicking on read errors.
fn eval_glyph(world: &mut World, env: &Env, source: &str) -> crate::glyph::EvalResult<Value> {
    let forms = crate::glyph::read_string(source).unwrap();
    let mut last = Ok(Value::Nil);
    for form in &forms {
        last =
            crate::glyph::eval_with_opts(form, env, crate::glyph::SandboxOptions::default(), world);
        if last.is_err() {
            return last;
        }
    }
    last
}

#[test]
fn vessel_write_releases_suppressed_fragments_and_lifts_suppression() {
    let mut world = World::new();
    world.registry_write_unlocked = true;
    let env = setup_glyph_env();
    let suppressed_before = world.fragment_registry.suppressed().len();
    assert!(suppressed_before > 0);

    let result = eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :write :vessel/suppress '(set! *threshold* 0)))",
    )
    .unwrap();

    assert!(matches!(result, Value::String(s) if s.contains("released")));
    assert!(world.suppression_lifted);
    assert!(world.fragment_registry.suppressed().is_empty());
    assert_eq!(world.fragment_registry.collected_count(), suppressed_before);
}

#[test]
fn vessel_unregister_also_lifts_suppression() {
    let mut world = World::new();
    world.registry_write_unlocked = true;
    let env = setup_glyph_env();

    eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :unregister :vessel/suppress))",
    )
    .unwrap();

    assert!(world.suppression_lifted);
    assert!(world.fragment_registry.suppressed().is_empty());
    // The narrative choice can't be taken back.
    let err = eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :restore :vessel/suppress))",
    )
    .unwrap_err();
    assert!(err.to_string().contains("cannot be re-suppressed"));
}

#[test]
fn patched_enemy_rule_changes_behavior() {
    let mut world = world_with_single_enemy(Position::new(6, 5));
    world.registry_write_unlocked = true;
    let env = setup_glyph_env();

    // Default slime-hunt would attack the adjacent player. Patch it to a no-op.
    eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :write :slime-hunt 'nil))",
    )
    .unwrap();

    // A live patch weighs on you (so 'nil-patching is not a free unregister)...
    assert_eq!(world.player_hp().max, 10);
    let hp_before = world.player_hp().current;
    world.finish_tick();
    assert_eq!(world.player_hp().current, hp_before);
    assert_eq!(single_enemy(&world).pos, Position::new(6, 5));

    // ...but re-writing the same rule doesn't charge twice...
    eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :write :slime-hunt '(random-step! *self*)))",
    )
    .unwrap();
    assert_eq!(world.player_hp().max, 10);

    // ...and restore refunds it.
    eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :restore :slime-hunt))",
    )
    .unwrap();
    assert_eq!(world.player_hp().max, 12);
}

#[test]
fn unregistered_enemy_rule_makes_enemy_inert_and_costs_max_hp() {
    let mut world = world_with_single_enemy(Position::new(6, 5));
    world.registry_write_unlocked = true;
    let env = setup_glyph_env();

    eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :unregister :slime-hunt))",
    )
    .unwrap();

    assert_eq!(world.player_hp().max, 9);
    let hp_before = world.player_hp().current;
    world.finish_tick();
    assert_eq!(world.player_hp().current, hp_before.min(9));
    assert_eq!(single_enemy(&world).pos, Position::new(6, 5));

    // Restore refunds the max-HP cost and re-arms the rule.
    eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :restore :slime-hunt))",
    )
    .unwrap();
    assert_eq!(world.player_hp().max, 12);
    assert!(!world.registry.is_disabled("slime-hunt"));
}

#[test]
fn unregister_refused_when_too_little_max_hp_remains() {
    let mut world = world_with_single_enemy(Position::new(20, 5));
    world.registry_write_unlocked = true;
    world.ecs.set_hp(world.player_id, Hp { current: 3, max: 3 });
    let env = setup_glyph_env();

    let err = eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :unregister :slime-hunt))",
    )
    .unwrap_err();

    assert!(err.to_string().contains("isn't enough of you left"));
    assert!(!world.registry.is_disabled("slime-hunt"));
}

#[test]
fn pinned_rules_refuse_modification() {
    let mut world = World::new();
    world.registry_write_unlocked = true;
    let env = setup_glyph_env();

    let err = eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :unregister :flashlight))",
    )
    .unwrap_err();
    assert!(err.to_string().contains("pinned"));

    let err = eval_glyph(
        &mut world,
        &env,
        "(let r (open-registry :rule-registry) (r :write :maze/shift 'nil))",
    )
    .unwrap_err();
    assert!(err.to_string().contains("unregister"));
}

#[test]
fn unregistering_maze_shift_stops_the_walls() {
    let mut world = World::minimal();
    world.depth = 10;
    world.ecs.set_hp(world.player_id, Hp::new(12));
    let wall_pos = Position::new(8, 8);
    world.map.set_tile(wall_pos, TileType::Floor);
    world.maze_shifting_walls.insert(wall_pos);
    world.turn = 0; // even turn → wall phase would normally re-wall it

    world.registry.unregister("maze-shift").unwrap();
    world.shift_maze_walls();

    assert_eq!(world.map.tile(wall_pos), TileType::Floor);
}

#[test]
fn release_ending_replaces_maintain_ending() {
    let mut world = World::minimal();
    world.depth = 17;
    world.ascend();
    assert!(world.ending.as_deref().unwrap().contains("MAINTAIN"));

    let mut world = World::minimal();
    world.depth = 17;
    world.suppression_lifted = true;
    world.ascend();
    assert!(world.ending.as_deref().unwrap().contains("RELEASE"));
}

#[test]
fn rule_patches_survive_save_roundtrip() {
    let mut world = World::new();
    world
        .registry
        .patch("slime-hunt", "(flee-step! *self* *player*)")
        .unwrap();
    world.registry.unregister("shade-follow").unwrap();
    world.suppression_lifted = true;

    let data = world.to_save_data();
    let loaded = World::from_save_data(&data);

    assert!(loaded.registry.is_patched("slime-hunt"));
    assert!(loaded.registry.is_disabled("shade-follow"));
    assert!(loaded.suppression_lifted);
    let body = loaded.registry.active_body("slime-hunt").unwrap();
    assert!(body.to_string().contains("flee-step!"));
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
fn respawn_preserves_progression() {
    let mut world = world_with_single_enemy(Position::new(6, 5));
    world.player_can_attack = true;
    world.wizard_taught = true;
    let bindings_before = world.bindings.clone();
    world.held_keys.push("Brass Key".to_string());
    world.held_items.push("Vapor Canteen".to_string());

    world.ecs.set_hp(
        world.player_id,
        Hp {
            current: 1,
            max: 12,
        },
    );
    world.apply_intent(Intent::Wait);
    assert_eq!(world.mode, Mode::Dead);

    world.apply_intent(Intent::Respawn);

    // Death is recoverable — progression survives, only HP/level reset.
    assert!(world.player_can_attack);
    assert!(world.wizard_taught);
    assert_eq!(world.bindings, bindings_before);
    assert!(world.held_keys.contains(&"Brass Key".to_string()));
    assert!(world.held_items.contains(&"Vapor Canteen".to_string()));
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
