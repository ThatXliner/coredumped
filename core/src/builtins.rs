//! Glyph builtin functions for the game console.
//!
//! This module contains all the builtin functions registered in the Glyph
//! environment, including game commands (move, attack, save/load), help pages,
//! and registry access.

use std::collections::BTreeMap;

use bracket_color::prelude::{CYAN, DARK_GRAY, GREEN, RED, RGB, YELLOW};

use crate::ai_builtins;
use crate::entity::{Direction, EntityId, EntityKind, Position};
use crate::game::{ActionCost, Mode};
use crate::glyph::{self, Env, Value};
use crate::world::World;

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
        "toggle-memories!",
        "open or close the collected memories view",
        builtin_toggle_memories
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
    reg!(
        "open-registry",
        "open a hidden registry handle",
        builtin_open_registry
    );
    reg!(
        "player",
        "query player state: (player :pos), (player :hp), (player :facing), (player :console-buffer)",
        builtin_player
    );
    reg!(
        "last-impact-force",
        "get the force of the last attack: (last-impact-force)",
        builtin_last_impact_force
    );
    reg!(
        "impact-payload",
        "get the payload bytes from the last impact (size = force × target mass)",
        builtin_impact_payload
    );
    reg!(
        "bytes",
        "allocate a byte buffer: (bytes 64) returns a list of zeros",
        builtin_bytes
    );
    reg!(
        "copy-bytes!",
        "copy src bytes into dest buffer: (copy-bytes! dest src)",
        builtin_copy_bytes
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

pub(crate) fn builtin_block(
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

pub(crate) fn builtin_shove(
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
        world.new_binding_keys.clear();
        Mode::Normal
    } else {
        world.has_new_bindings = false;
        Mode::Keybindings
    };
    Ok(Value::Nil)
}

fn builtin_toggle_memories(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    world.mode = if world.mode == Mode::Memories {
        Mode::Normal
    } else {
        Mode::Memories
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
        log::warn!(
            target: "xlyph::depth",
            "descend blocked reason=no_stairs turn={} depth={} pos=({},{}) tile={:?}",
            world.turn,
            world.depth,
            pos.x,
            pos.y,
            world.map.tile(pos)
        );
        world.event_log.push("There are no stairs going down here.");
        return Ok(Value::Nil);
    }
    let has_attack_binding = world.bindings.values().any(|cmd| cmd.contains("do-attack"));
    if world.depth >= 1 && (!world.wizard_taught || !has_attack_binding) {
        log::warn!(
            target: "xlyph::depth",
            "descend blocked reason=wizard_gate turn={} depth={} pos=({},{}) wizard_taught={} has_attack_binding={}",
            world.turn,
            world.depth,
            pos.x,
            pos.y,
            world.wizard_taught,
            has_attack_binding
        );
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
        log::warn!(
            target: "xlyph::depth",
            "ascend blocked reason=no_stairs turn={} depth={} pos=({},{}) tile={:?}",
            world.turn,
            world.depth,
            pos.x,
            pos.y,
            world.map.tile(pos)
        );
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
            doc: "attack in a direction: (do-attack), (do-attack :east), (do-attack :north 5)",
            func: builtin_do_attack,
        }),
    );
}

pub(crate) fn builtin_do_attack(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let (direction, force) = match args {
        [] => (world.player_facing, 1),
        [arg] => {
            if let Some(direction) = parse_attack_direction(arg) {
                (direction, 1)
            } else {
                (world.player_facing, parse_attack_force(arg)?)
            }
        }
        [dir, force] => {
            let direction =
                parse_attack_direction(dir).ok_or_else(|| glyph::EvalError::TypeError {
                    expected: "direction keyword (:north, :south, :east, :west)",
                    got: dir.to_string(),
                })?;
            (direction, parse_attack_force(force)?)
        }
        _ => {
            return Err(glyph::EvalError::WrongArgCount {
                expected: 2,
                got: args.len(),
            })
        }
    };

    if !world.player_can_attack {
        return Err(glyph::EvalError::Custom(
            "You don't know how to attack yet. Find the wizard.".into(),
        ));
    }

    world.player_facing = direction;
    world.attack_in_direction(direction, force);
    world.finish_tick();
    Ok(Value::Nil)
}

fn parse_attack_force(value: &Value) -> glyph::EvalResult<i32> {
    match value {
        Value::I64(n) if *n > 0 => Ok(*n as i32),
        Value::F64(n) if *n > 0.0 => Ok(*n as i32),
        other => Err(glyph::EvalError::TypeError {
            expected: "positive attack force number",
            got: other.to_string(),
        }),
    }
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
    if args.len() > 1 {
        return Ok(Value::String(
            "(help): expected zero args or one page/function name".into(),
        ));
    }

    if let Some(topic) = args.first() {
        if let Some(page) = help_page(topic, world) {
            return Ok(Value::String(page));
        }

        // Args are already evaluated — handle the value directly
        let result = match topic {
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
            Value::Builtin(_) | Value::Closure(_) | Value::Macro(_) => format_value_help(topic),
            other => {
                format!(
                    "(help <name>): expected a page, function, or symbol; got {}",
                    other
                )
            }
        };
        return Ok(Value::String(result));
    }

    Ok(Value::String(help_index(world)))
}

fn help_page(topic: &Value, world: &World) -> Option<String> {
    match topic {
        Value::I64(page) => help_page_by_number(*page, world),
        Value::Keyword(kw) => help_page_by_name(&kw.name, world),
        Value::String(name) => help_page_by_name(name, world),
        Value::Symbol(sym) => help_page_by_name(&sym.name, world),
        _ => None,
    }
}

fn help_page_by_number(page: i64, world: &World) -> Option<String> {
    match page {
        1 => Some(help_index(world)),
        2 => Some(help_forms()),
        3 => Some(help_builtins()),
        4 => Some(help_game_commands(world)),
        5 => Some(help_language_reference()),
        6 => Some(help_tutorial()),
        _ => Some(format!(
            "No help page {page}. Try (help), (help :language), or (help :tutorial)."
        )),
    }
}

fn help_page_by_name(name: &str, world: &World) -> Option<String> {
    match name.trim_start_matches(':').to_lowercase().as_str() {
        "1" | "index" | "contents" | "overview" => Some(help_index(world)),
        "2" | "forms" | "special-forms" => Some(help_forms()),
        "3" | "builtins" | "functions" => Some(help_builtins()),
        "4" | "game" | "commands" | "console" => Some(help_game_commands(world)),
        "5" | "language" | "reference" | "lang" => Some(help_language_reference()),
        "6" | "tutorial" | "learn" | "intro" => Some(help_tutorial()),
        _ => None,
    }
}

fn help_index(world: &World) -> String {
    let mut help = String::from(
        "\
Glyph help (page 1/6)

Use:
  (help)          — show this index
  (help 2)        — open a numbered page
  (help :language) — open a named page
  (help :tutorial) — learn by example
  (help 'map)     — show help for a function or macro

Available pages:
  1  :index       using help
  2  :forms       special forms
  3  :builtins    core functions
  4  :game        dungeon console commands
  5  :language    language reference
  6  :tutorial    short language tutorial

Tips:
  Page names can be keywords or strings: (help :game), (help \"game\").
  Function names are easiest quoted: (help 'bind-key), (help '+).",
    );

    if world.player_can_attack {
        help.push_str("\n  Attack is unlocked, so :game includes binding and attack commands.");
    } else {
        help.push_str("\n  Find the wizard to unlock attack and binding help.");
    }

    help
}

fn help_forms() -> String {
    String::from(
        "\
Glyph help (page 2/6): special forms

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

Examples:
  (let x 2 (+ x 3))
  (do (println \"hi\") :done)
  (if (> hp 0) :alive :dead)

Next: (help 3), (help :language), or (help :tutorial)",
    )
}

fn help_builtins() -> String {
    String::from(
        "\
Glyph help (page 3/6): built-in functions

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

Examples:
  (map (fn [x] (* x 2)) (list 1 2 3))
  (apply + (list 1 2 3))
  (. {:name \"xlyph\" :depth 4} :depth)

For details on a function:
  (help '+)
  (help 'map)
  (help \"println\")",
    )
}

fn help_game_commands(world: &World) -> String {
    let mut help = String::from(
        "\
Glyph help (page 4/6): game console commands

  (help)        — show this help text\n\
  (help <name>) — show help for a specific function\n\
  (save! [slot]) — save the game (F5 to quick-save)\n\
  (load! [slot]) — load a saved game (F9 to quick-load)\n\
  (quit-terminal) — close the console overlay\n\
  (quit!)       — exit the game (auto-saves)",
    );

    if world.player_can_attack {
        help.push_str(
            "\n  (do-attack :dir [force]) — strike in direction (keybindings only; \n  \
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

    help.push_str("\n\nNext: (help :language) or (help :tutorial)");
    help
}

fn help_language_reference() -> String {
    String::from(
        "\
Glyph help (page 5/6): language reference

Data:
  nil true false — constants
  42 -7 3.14    — numbers
  \"text\"        — strings
  :north        — keywords
  (list 1 2)    — lists
  #[1 2 3]      — vectors
  #{:a :b}      — sets
  {:hp 12}      — maps

Syntax:
  'form         — reader macro for (quote form)
  [a + b]       — infix notation with precedence
  a.b.c         — dotted access sugar
  ; comment     — line comment

Evaluation:
  Lists call functions: (+ 1 2)
  Special forms control evaluation: (if test then else)
  Functions evaluate their arguments before running.
  Quote data when you want the form itself: '(move! :north)

Errors stay in the console output. Game events go to the event log.",
    )
}

fn help_tutorial() -> String {
    String::from(
        "\
Glyph help (page 6/6): short tutorial

1. Try a value:
  (+ 1 2)
  (list :north :south)

2. Name a temporary value:
  (let hp 12
    (if (> hp 0) :standing :down))

3. Make a function:
  ((fn [x] (* x x)) 5)

4. Build data:
  {:dir :east :steps (list 1 2 3)}

5. Use game commands:
  (player-facing)
  (save!)

6. Bind a command after the wizard teaches you:
  (bind-key :z (do-attack :east))

More:
  (help :forms)
  (help :builtins)
  (help :game)
  (help 'bind-key)",
    )
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
    use bracket_color::prelude::GREEN;
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
    use bracket_color::prelude::GREEN;
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

fn registry_name_from_value(value: &Value) -> glyph::EvalResult<&str> {
    match value {
        Value::Keyword(kw) => Ok(kw.name.as_str()),
        Value::String(s) => Ok(s.as_str()),
        other => Err(glyph::EvalError::TypeError {
            expected: "registry or rule keyword",
            got: other.to_string(),
        }),
    }
}

fn rule_matches(rule: &crate::rules::Rule, requested: &str) -> bool {
    rule.id == requested
        || rule.name == requested
        || rule.id.replace('-', "/") == requested
        || rule.name.replace('/', "-") == requested
}

fn suppressed_fragment_list(world: &World) -> Value {
    let list: Vec<Value> = world
        .fragment_registry
        .suppressed()
        .into_iter()
        .map(|f| {
            let mut m: BTreeMap<Value, Value> = BTreeMap::new();
            m.insert(Value::String("id".into()), Value::String(f.id.clone()));
            m.insert(Value::String("weight".into()), Value::I64(f.weight as i64));
            Value::Map(m)
        })
        .collect();
    Value::List(list)
}

fn builtin_open_registry(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 1 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    match registry_name_from_value(&args[0])? {
        "suppressed-fragments" => Ok(Value::Builtin(glyph::BuiltinFn {
            name: "suppressed-fragments",
            doc: "registry handle: (handle :read)",
            func: builtin_suppressed_fragments_handle,
        })),
        "spawn-log" => Ok(Value::Builtin(glyph::BuiltinFn {
            name: "spawn-log",
            doc: "registry handle: (handle :write key value)",
            func: builtin_spawn_log_handle,
        })),
        "rule-registry" => {
            if world.registry_write_unlocked {
                Ok(Value::Builtin(glyph::BuiltinFn {
                    name: "rule-registry",
                    doc: "registry handle: (handle :read rule), (handle :write rule form), or (handle :unregister rule)",
                    func: builtin_rule_registry_handle,
                }))
            } else {
                Err(glyph::EvalError::Custom(
                    "Registry access denied: write-protect flag is set.".into(),
                ))
            }
        }
        other => Err(glyph::EvalError::Custom(format!(
            "unknown registry: {}",
            other
        ))),
    }
}

fn builtin_suppressed_fragments_handle(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    match args {
        [Value::Keyword(method)] if method.name == "read" => Ok(suppressed_fragment_list(world)),
        _ => Err(glyph::EvalError::Custom("usage: (handle :read)".into())),
    }
}

fn builtin_spawn_log_handle(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    match args.first() {
        Some(Value::Keyword(method)) if method.name == "write" => {
            world
                .event_log
                .push_colored("Spawn log accepted the write.", RGB::named(DARK_GRAY));
            Ok(Value::Nil)
        }
        _ => Err(glyph::EvalError::Custom(
            "usage: (handle :write key value)".into(),
        )),
    }
}

fn builtin_rule_registry_handle(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let method = match args.first() {
        Some(Value::Keyword(kw)) => kw.name.as_str(),
        Some(other) => {
            return Err(glyph::EvalError::TypeError {
                expected: "registry method keyword",
                got: other.to_string(),
            })
        }
        None => {
            return Err(glyph::EvalError::WrongArgCount {
                expected: 1,
                got: 0,
            })
        }
    };

    match method {
        "read" => {
            if args.len() != 2 {
                return Err(glyph::EvalError::WrongArgCount {
                    expected: 2,
                    got: args.len(),
                });
            }
            let requested = registry_name_from_value(&args[1])?;
            let rule = world
                .registry
                .iter()
                .find(|rule| rule_matches(rule, requested))
                .ok_or_else(|| glyph::EvalError::Custom(format!("unknown rule: {}", requested)))?;
            Ok(Value::String(rule.source_lines.join("\n")))
        }
        "write" => {
            if args.len() != 3 {
                return Err(glyph::EvalError::WrongArgCount {
                    expected: 3,
                    got: args.len(),
                });
            }
            let requested = registry_name_from_value(&args[1])?;
            let rule = world
                .registry
                .iter()
                .find(|rule| rule_matches(rule, requested))
                .ok_or_else(|| glyph::EvalError::Custom(format!("unknown rule: {}", requested)))?;
            let rule_name = rule.name;
            world.event_log.push_colored(
                format!("Registry write accepted for {}.", rule_name),
                RGB::named(CYAN),
            );
            Ok(Value::String(format!("{} patched", rule_name)))
        }
        "unregister" => {
            if args.len() != 2 {
                return Err(glyph::EvalError::WrongArgCount {
                    expected: 2,
                    got: args.len(),
                });
            }
            let requested = registry_name_from_value(&args[1])?;
            let rule = world
                .registry
                .iter()
                .find(|rule| rule_matches(rule, requested))
                .ok_or_else(|| glyph::EvalError::Custom(format!("unknown rule: {}", requested)))?;
            let rule_name = rule.name;
            world.event_log.push_colored(
                format!("Registry unregistered {}.", rule_name),
                RGB::named(RED),
            );
            Ok(Value::String(format!("{} unregistered", rule_name)))
        }
        _ => Err(glyph::EvalError::Custom(
            "usage: (handle :read rule), (handle :write rule form), or (handle :unregister rule)"
                .into(),
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
        Some(Value::Keyword(kw)) if kw.name.starts_with("frag-") => kw.name.clone(),
        Some(Value::Keyword(kw)) => {
            let s = &kw.name;
            format!(
                "frag-{:03}",
                s.parse::<u32>().map_err(|_| {
                    glyph::EvalError::Custom(format!("invalid fragment id: {}", s))
                })?
            )
        }
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

fn builtin_player(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let key = match args.first() {
        Some(Value::Keyword(kw)) => kw.name.as_str(),
        Some(other) => {
            return Err(glyph::EvalError::TypeError {
                expected: "keyword",
                got: other.to_string(),
            })
        }
        None => {
            return Err(glyph::EvalError::Custom(
                "usage: (player :pos), (player :hp), (player :facing), (player :console-buffer)"
                    .into(),
            ))
        }
    };

    match key {
        "pos" => {
            let pos = world.player_pos();
            Ok(Value::List(vec![
                Value::I64(pos.x as i64),
                Value::I64(pos.y as i64),
            ]))
        }
        "hp" => {
            let hp = world.player_hp();
            Ok(Value::I64(hp.current as i64))
        }
        "max-hp" => {
            let hp = world.player_hp();
            Ok(Value::I64(hp.max as i64))
        }
        "facing" => {
            let dir = match world.player_facing {
                Direction::North => "north",
                Direction::South => "south",
                Direction::East => "east",
                Direction::West => "west",
            };
            Ok(glyph::kw(dir))
        }
        "console-buffer" => Ok(Value::String(world.console_buffer.clone())),
        "depth" => Ok(Value::I64(world.depth as i64)),
        "turn" => Ok(Value::I64(world.turn as i64)),
        _ => Err(glyph::EvalError::Custom(format!(
            "unknown player attribute: {}. Try :pos, :hp, :facing, :console-buffer, :depth, :turn",
            key
        ))),
    }
}

fn builtin_last_impact_force(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    Ok(Value::I64(world.last_impact_force as i64))
}

fn builtin_impact_payload(
    _args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let mass = match world.last_impact_target {
        Some(EntityKind::Rage) => 8,
        Some(_) => 4,
        None => 0,
    };
    let size = world.last_impact_force * mass;
    Ok(Value::List(vec![Value::I64(0); size as usize]))
}

fn builtin_bytes(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    _world: &mut World,
) -> glyph::EvalResult<Value> {
    let size = match args.first() {
        Some(Value::I64(n)) if *n >= 0 => *n as usize,
        Some(other) => {
            return Err(glyph::EvalError::TypeError {
                expected: "positive integer",
                got: other.to_string(),
            })
        }
        None => {
            return Err(glyph::EvalError::WrongArgCount {
                expected: 1,
                got: 0,
            })
        }
    };
    Ok(Value::List(vec![Value::I64(0); size]))
}

fn builtin_copy_bytes(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    let (dest, src) = match args {
        [dest, src] => (dest, src),
        _ => {
            return Err(glyph::EvalError::WrongArgCount {
                expected: 2,
                got: args.len(),
            })
        }
    };

    let dest_len = match dest {
        Value::List(v) => v.len(),
        _ => {
            return Err(glyph::EvalError::TypeError {
                expected: "list (byte buffer)",
                got: dest.to_string(),
            })
        }
    };

    let src_len = match src {
        Value::List(v) => v.len(),
        _ => {
            return Err(glyph::EvalError::TypeError {
                expected: "list (byte buffer)",
                got: src.to_string(),
            })
        }
    };

    if src_len > dest_len {
        if world.last_impact_target == Some(EntityKind::Rage) && !world.registry_write_unlocked {
            world.registry_write_unlocked = true;
            world.event_log.push_colored(
                "Buffer overflow! The impact payload overruns its buffer. Registry write-protect disabled.",
                RGB::named(CYAN),
            );
        }
    }

    Ok(Value::Nil)
}

