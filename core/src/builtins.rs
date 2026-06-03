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
    let base = glyph::default_env();
    let env = Env::extend(&base);

    // Names registered here are the game commands (move!, attack!, save!, ...);
    // collected so the syntax highlighter can color them without a duplicated,
    // hand-maintained list. See `glyph::highlight::set_vocab`.
    let mut command_names: Vec<String> = Vec::new();

    macro_rules! reg {
        ($name:expr, $doc:expr, $func:ident) => {
            command_names.push($name.to_string());
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

    reg!("help", "show help\n\nSYNOPSIS\n  (help)           show help index (page 1)\n  (help N)         show page N (1-6)\n  (help :topic)    show named page (:forms, :builtins, :game, :language, :tutorial)\n  (help 'symbol)   show doc for a function, macro, or variable\n\nDESCRIPTION\n  Without arguments, displays a paginated help index.\n  With a page number or topic keyword, displays that help page.\n  With a quoted symbol, string, or function value, displays its doc string.\n\nRETURN VALUE\n  A string containing the help text.\n\nEXAMPLES\n  (help)            ; show index\n  (help 4)          ; game console commands\n  (help :tutorial)  ; short language tutorial\n  (help 'map)       ; doc for the map function\n  (help '+)         ; doc for +", builtin_help);
    reg!(
        "quit-terminal",
        "close the console overlay\n\nSYNOPSIS\n  (quit-terminal)\n\nDESCRIPTION\n  Closes the in-game console and returns to Normal mode.\n  Does not consume a turn. Equivalent to pressing Escape or ` while\n  the console is open.\n\nRETURN VALUE\n  The keyword :quit-terminal (used internally by the event loop).",
        builtin_quit_terminal
    );
    reg!("quit!", "exit the game\n\nSYNOPSIS\n  (quit!)\n\nDESCRIPTION\n  Exits the game. Requires confirmation: the first call prompts\n  \"Press q again to quit\"; calling quit! a second time auto-saves\n  to slot 0 and terminates. Any other action cancels the quit.\n\nSIDE EFFECTS\n  Auto-saves to slot 0 on confirmation.\n\nRETURN VALUE\n  nil", builtin_quit_bang);
    reg!("move!", "move the player one tile\n\nSYNOPSIS\n  (move!)\n  (move! direction)\n\nARGUMENTS\n  direction  Optional keyword: :north, :south, :east, or :west.\n             Defaults to the player's current facing direction.\n\nDESCRIPTION\n  Moves the player one tile in the given direction and updates the\n  player's facing. If the destination is walkable and unoccupied, the\n  player moves there. If an enemy occupies the tile, movement is\n  blocked (no damage dealt — use do-attack for combat). Bumping a wall\n  has no effect. Consumes a turn on successful move.\n\nRETURN VALUE\n  nil\n\nERRORS\n  More than 1 argument, or non-keyword direction.\n\nEXAMPLES\n  (move! :north)\n  (move! :east)\n  (move!)           ; move in current facing direction", builtin_move);
    reg!("wait!", "skip a turn\n\nSYNOPSIS\n  (wait!)\n\nDESCRIPTION\n  The player does nothing. Advances the turn counter by one and\n  allows all enemies to take their actions. Useful for waiting out\n  enemy patterns or letting cooldowns expire.\n\nRETURN VALUE\n  nil", builtin_wait);
    reg!(
        "block!",
        "shove all adjacent enemies and raise guard\n\nSYNOPSIS\n  (block!)\n\nDESCRIPTION\n  Shoves every enemy adjacent to the player (in all 4 cardinal\n  directions) up to 3 tiles away from the player, then raises\n  a guard stance until the next turn. While blocking, incoming\n  melee attacks are negated with a \"You block the attack\" message.\n  Enemies that cannot be pushed (wall or entity behind them) receive\n  a \"doesn't budge\" message but are still marked as attacked.\n  Consumes a turn.\n\nRETURN VALUE\n  nil\n\nNOTES\n  If no enemies are adjacent, logs \"You raise your guard, but\n  nothing is near.\" The blocking flag is cleared at the start of\n  the next turn.",
        builtin_block
    );
    reg!(
        "shove!",
        "shove an enemy in a direction (free action)\n\nSYNOPSIS\n  (shove!)\n  (shove! direction)\n\nARGUMENTS\n  direction  Optional. A keyword: :north, :south, :east, or :west.\n             Defaults to the player's current facing direction.\n\nDESCRIPTION\n  Pushes an adjacent enemy in the given direction away from the\n  player. Unlike block!, this is a FREE action — it does not\n  consume a turn and does not advance enemy AI. Updates the\n  player's facing to the specified direction.\n\nRETURN VALUE\n  nil\n\nERRORS\n  More than 1 argument, or non-keyword direction.\n\nEXAMPLES\n  (shove! :east)    ; shove enemy to the east\n  (shove!)          ; shove in current facing direction",
        builtin_shove
    );
    reg!(
        "toggle-inspector!",
        "toggle the inspector panel\n\nSYNOPSIS\n  (toggle-inspector!)\n\nDESCRIPTION\n  Switches between Normal mode and Inspector mode. In Inspector\n  mode, the right panel displays registered rules and their source\n  code. Closing the inspector clears any new-rule highlight markers.\n  Free action — does not consume a turn.\n\nRETURN VALUE\n  nil",
        builtin_toggle_inspector
    );
    reg!(
        "toggle-console!",
        "toggle the Glyph console\n\nSYNOPSIS\n  (toggle-console!)\n\nDESCRIPTION\n  Switches between Normal mode and Console mode. The console is\n  a Glyph REPL overlay where you can type and evaluate expressions.\n  Equivalent to pressing the ` key. Free action.\n\nRETURN VALUE\n  nil",
        builtin_toggle_console
    );
    reg!(
        "toggle-keybindings!",
        "toggle the keybindings view\n\nSYNOPSIS\n  (toggle-keybindings!)\n\nDESCRIPTION\n  Switches between Normal mode and Keybindings mode. Displays all\n  currently bound keys and the Glyph expressions they evaluate.\n  Closing the view clears the new-binding highlight markers.\n  Free action.\n\nRETURN VALUE\n  nil",
        builtin_toggle_keybindings
    );
    reg!(
        "toggle-memories!",
        "toggle the collected memories view\n\nSYNOPSIS\n  (toggle-memories!)\n\nDESCRIPTION\n  Switches between Normal mode and Memories mode. Displays all\n  memory fragments the player has collected during the current run.\n  Free action.\n\nRETURN VALUE\n  nil",
        builtin_toggle_memories
    );
    reg!(
        "descend!",
        "descend to the next dungeon level\n\nSYNOPSIS\n  (descend!)\n\nDESCRIPTION\n  Moves the player down one dungeon level. The player must be\n  standing on a StairsDown tile. On depth >= 1, descending also\n  requires that the wizard has taught the player to attack AND\n  the player has bound do-attack to a key.\n\nERRORS\n  Logs \"There are no stairs going down here.\" if the player is\n  not on stairs. Logs a wizard hint if the attack gate is unmet.\n\nRETURN VALUE\n  nil\n\nSEE ALSO\n  ascend!, do-attack, bind-key",
        builtin_descend
    );
    reg!("ascend!", "ascend to the previous dungeon level\n\nSYNOPSIS\n  (ascend!)\n\nDESCRIPTION\n  Moves the player up one dungeon level. The player must be\n  standing on a StairsUp tile.\n\nERRORS\n  Logs \"There are no stairs going up here.\" if the player is not\n  on an up-staircase.\n\nRETURN VALUE\n  nil\n\nSEE ALSO\n  descend!", builtin_ascend);
    reg!(
        "player-facing",
        "get the player's current facing direction\n\nSYNOPSIS\n  (player-facing)\n\nDESCRIPTION\n  Returns the cardinal direction the player is currently facing.\n  The facing direction determines the flashlight cone and is the\n  default direction for do-attack and shove! when called without\n  arguments.\n\nRETURN VALUE\n  A keyword: :north, :south, :east, or :west.\n\nEXAMPLES\n  (player-facing)              ; => :east\n  (do-attack (player-facing))  ; attack in current facing direction",
        builtin_player_facing
    );
    reg!("heal", "restore player HP (cheat)\n\nSYNOPSIS\n  (heal amount)\n  (heal :all)\n\nARGUMENTS\n  amount  A positive integer. Heals that many HP. Can exceed max HP\n          (the overflow acts as a shield).\n  :all    Fully restores HP to the player's max.\n\nDESCRIPTION\n  Cheat command — requires the Konami code to be entered first.\n  Restores the player's HP by the given amount, or fully if :all\n  is passed. Healing above max HP is allowed and persists.\n\nRETURN VALUE\n  nil\n\nERRORS\n  \"cheats not activated\" if the Konami code has not been entered.\n\nEXAMPLES\n  (heal 10)    ; heal 10 HP\n  (heal :all)  ; fully restore HP", builtin_heal);
    reg!(
        "log",
        "push a message to the event log\n\nSYNOPSIS\n  (log message)\n\nARGUMENTS\n  message  A string to display in the event log at the bottom of\n           the screen.\n\nDESCRIPTION\n  Appends a message to the game's event log. The event log is a\n  ring buffer (max 100 entries) — oldest entries are dropped when\n  full. Messages appear in the default text color.\n\nRETURN VALUE\n  nil\n\nERRORS\n  TypeError if the argument is not a string.\n\nEXAMPLES\n  (log \"Hello, dungeon!\")",
        builtin_log
    );
    reg!(
        "damage!",
        "deal damage to an entity\n\nSYNOPSIS\n  (damage! entity-id amount)\n\nARGUMENTS\n  entity-id  Integer entity ID of the target.\n  amount     Integer damage to inflict.\n\nDESCRIPTION\n  Reduces the target entity's HP by the given amount. If the\n  target is the player and HP drops to 0 or below, the game\n  switches to Dead mode.\n\nRETURN VALUE\n  The target's remaining HP as an integer.\n\nERRORS\n  Wrong arg count (expects exactly 2).\n  TypeError if entity-id is not an integer or amount is not an integer.\n\nEXAMPLES\n  (damage! 3 5)   ; deal 5 damage to entity 3",
        builtin_damage
    );
    reg!(
        "fire?",
        "check if a tile is on fire\n\nSYNOPSIS\n  (fire? position)\n\nARGUMENTS\n  position  A list of two integers: (list x y).\n\nDESCRIPTION\n  Returns true if the tile at the given coordinates is currently\n  in the fire cache (i.e., on fire this tick). The fire cache is\n  recalculated each turn.\n\nRETURN VALUE\n  true or false.\n\nEXAMPLES\n  (fire? (list 10 15))   ; is tile (10,15) on fire?\n\nSEE ALSO\n  use-vapor-canteen!",
        builtin_fire_p
    );
    reg!(
        "use-vapor-canteen!",
        "douse a fire tile with the Vapor Canteen\n\nSYNOPSIS\n  (use-vapor-canteen! position)\n\nARGUMENTS\n  position  A list of two integers: (list x y).\n\nDESCRIPTION\n  Removes the fire at the given tile from the fire cache for the\n  current tick. The player must have the Vapor Canteen item (found\n  in the Archive, Level 13). The tile may still glow until the\n  cache updates on the next tick.\n\nRETURN VALUE\n  nil\n\nERRORS\n  \"You don't have the Vapor Canteen\" if the item is not held.\n  TypeError if position is not a (list x y).\n\nEXAMPLES\n  (use-vapor-canteen! (list 10 15))\n\nSEE ALSO\n  fire?",
        builtin_use_vapor_canteen
    );
    reg!(
        "set-level",
        "warp to a dungeon level (cheat)\n\nSYNOPSIS\n  (set-level depth)\n\nARGUMENTS\n  depth  A positive integer (>= 1) specifying the target level.\n\nDESCRIPTION\n  Cheat command — requires the Konami code to be entered first.\n  Immediately teleports the player to the given dungeon depth.\n  Clears all enemies and rebuilds the level from scratch.\n\nRETURN VALUE\n  nil\n\nERRORS\n  \"cheats not activated\" if the Konami code has not been entered.\n  Wrong arg count if no depth is given.\n\nEXAMPLES\n  (set-level 5)   ; warp to depth 5\n  (set-level 13)  ; warp to the Archive",
        builtin_set_level
    );
    reg!("save!", "save the game to a slot\n\nSYNOPSIS\n  (save!)\n  (save! slot)\n\nARGUMENTS\n  slot  Optional non-negative integer. Defaults to 1. Slot 0 is\n        used for auto-saves (by quit! and F5).\n\nDESCRIPTION\n  Serializes the current game state to disk at the given slot.\n  Overwrites any existing save in that slot.\n\nRETURN VALUE\n  The slot number as an integer.\n\nERRORS\n  TypeError if slot is not a non-negative integer.\n  I/O error if the save fails.\n\nEXAMPLES\n  (save!)     ; save to slot 1\n  (save! 2)   ; save to slot 2\n\nSEE ALSO\n  load!, wipe!", builtin_save);
    reg!(
        "load!",
        "load a saved game from a slot\n\nSYNOPSIS\n  (load!)\n  (load! slot)\n\nARGUMENTS\n  slot  Optional non-negative integer. Defaults to 1.\n\nDESCRIPTION\n  Deserializes a saved game state from disk and replaces the\n  current world entirely. All state — position, HP, depth,\n  inventory, bindings — is restored from the save file.\n\nRETURN VALUE\n  The slot number as an integer.\n\nERRORS\n  TypeError if slot is not a non-negative integer.\n  I/O error if no save exists at that slot.\n\nEXAMPLES\n  (load!)     ; load from slot 1\n  (load! 2)   ; load from slot 2\n\nSEE ALSO\n  save!, wipe!",
        builtin_load
    );
    reg!("wipe!", "delete a save file\n\nSYNOPSIS\n  (wipe!)\n  (wipe! slot)\n\nARGUMENTS\n  slot  Optional non-negative integer. Defaults to 0.\n\nDESCRIPTION\n  Schedules deletion of the save at the given slot. Requires\n  confirmation: the player must type the exact phrase\n  'i am aware of what i am doing.' in the console to confirm.\n  The save is not deleted until confirmation is received.\n\nRETURN VALUE\n  nil\n\nERRORS\n  TypeError if slot is not a non-negative integer.\n\nEXAMPLES\n  (wipe! 1)   ; request deletion of slot 1, then confirm\n\nSEE ALSO\n  save!, load!", builtin_wipe);
    reg!(
        "query-registry",
        "query the fragment registry\n\nSYNOPSIS\n  (query-registry :all)\n  (query-registry :suppressed-fragments)\n\nARGUMENTS\n  mode  A keyword selecting which fragments to return.\n        :all                 — all fragments with id, weight, collected, suppressed.\n        :suppressed-fragments — only suppressed fragments with id and weight.\n        Defaults to :all if omitted.\n\nDESCRIPTION\n  Returns a list of maps describing memory fragments in the\n  registry. Each map contains at minimum \"id\" and \"weight\" keys.\n  The :all mode also includes \"collected\" and \"suppressed\" booleans.\n\nRETURN VALUE\n  A list of maps.\n\nERRORS\n  Unknown mode keyword.\n\nEXAMPLES\n  (query-registry :all)\n  (query-registry :suppressed-fragments)\n\nSEE ALSO\n  inspect-fragment, open-registry",
        builtin_query_registry
    );
    reg!(
        "inspect-fragment",
        "read a memory fragment's contents\n\nSYNOPSIS\n  (inspect-fragment fragment-id)\n\nARGUMENTS\n  fragment-id  A keyword or string identifying the fragment.\n               Keywords can be :frag-001 or just :1 (auto-prefixed).\n               Strings should be the full id, e.g. \"frag-001\".\n\nDESCRIPTION\n  Looks up a memory fragment by ID and returns a map with its\n  full details: id, text, weight, and status (\"suppressed\",\n  \"hidden\", or \"collected\").\n\nRETURN VALUE\n  A map: {\"id\" \"frag-001\" \"text\" \"...\" \"weight\" N \"status\" \"...\"}\n\nERRORS\n  \"no fragment with id\" if the ID does not exist.\n\nEXAMPLES\n  (inspect-fragment :frag-001)\n  (inspect-fragment :3)         ; shorthand for :frag-003\n  (inspect-fragment \"frag-010\")\n\nSEE ALSO\n  query-registry",
        builtin_inspect_fragment
    );
    reg!(
        "open-registry",
        "open a registry handle for advanced access\n\nSYNOPSIS\n  (open-registry registry-name)\n\nARGUMENTS\n  registry-name  A keyword or string naming the registry:\n    :suppressed-fragments — read-only handle for suppressed fragments.\n    :spawn-log            — write-only handle for the spawn log.\n    :rule-registry        — read/write/unregister handle for rules.\n                            Requires registry write-protect to be disabled.\n\nDESCRIPTION\n  Returns a handle (a builtin function) that can be called with\n  :read, :write, or :unregister methods depending on the registry.\n  The rule-registry handle is locked behind the write-protect flag,\n  which must be disabled through gameplay before access is granted.\n\nRETURN VALUE\n  A handle function. Call it with (handle :read ...) etc.\n\nERRORS\n  \"Registry access denied\" if rule-registry write-protect is set.\n  \"unknown registry\" for unrecognized names.\n\nEXAMPLES\n  (let h (open-registry :suppressed-fragments) (h :read))\n  (let h (open-registry :rule-registry) (h :read :enemy-ai/chase))\n\nSEE ALSO\n  query-registry, inspect-fragment",
        builtin_open_registry
    );
    reg!(
        "player",
        "query player state\n\nSYNOPSIS\n  (player attribute)\n\nARGUMENTS\n  attribute  A keyword selecting which player property to query:\n    :pos             — player position as (list x y).\n    :hp              — current HP as an integer.\n    :max-hp          — maximum HP as an integer.\n    :facing          — facing direction as a keyword.\n    :console-buffer  — current console input as a string.\n    :depth           — current dungeon depth as an integer.\n    :turn            — current turn number as an integer.\n\nDESCRIPTION\n  Returns information about the player's current state. This is\n  a read-only query — it does not modify the world.\n\nRETURN VALUE\n  Varies by attribute (see above).\n\nERRORS\n  \"unknown player attribute\" with a list of valid keys.\n\nEXAMPLES\n  (player :pos)     ; => (10 15)\n  (player :hp)      ; => 12\n  (player :depth)   ; => 3",
        builtin_player
    );
    reg!(
        "last-impact-force",
        "get the force of the most recent attack\n\nSYNOPSIS\n  (last-impact-force)\n\nDESCRIPTION\n  Returns the force value from the most recent do-attack call.\n  Force defaults to 1 unless a higher value was specified.\n  Persists until the next attack overwrites it.\n\nRETURN VALUE\n  An integer (the force value).\n\nSEE ALSO\n  impact-payload, do-attack",
        builtin_last_impact_force
    );
    reg!(
        "impact-payload",
        "get the payload bytes from the last impact\n\nSYNOPSIS\n  (impact-payload)\n\nDESCRIPTION\n  Returns a list of zero-valued bytes whose length equals\n  force * target_mass, where force is from the last attack and\n  target_mass depends on the enemy type (Rage enemies have mass 8,\n  others have mass 4). Returns an empty list if no impact has\n  occurred.\n\nRETURN VALUE\n  A list of integers (all zeros), length = force * mass.\n\nNOTES\n  Used with copy-bytes! for the buffer overflow exploit mechanic.\n\nSEE ALSO\n  last-impact-force, bytes, copy-bytes!",
        builtin_impact_payload
    );
    reg!(
        "bytes",
        "allocate a zero-filled byte buffer\n\nSYNOPSIS\n  (bytes size)\n\nARGUMENTS\n  size  A non-negative integer specifying the buffer length.\n\nDESCRIPTION\n  Creates a list of the given size, filled with zeros. Used to\n  allocate destination buffers for copy-bytes!.\n\nRETURN VALUE\n  A list of integers (all zeros).\n\nERRORS\n  TypeError if size is not a positive integer.\n  Wrong arg count if no size is given.\n\nEXAMPLES\n  (bytes 64)    ; => (0 0 0 ... 0)  — 64 zeros\n  (bytes 4)     ; => (0 0 0 0)\n\nSEE ALSO\n  copy-bytes!, impact-payload",
        builtin_bytes
    );
    reg!(
        "copy-bytes!",
        "copy source bytes into a destination buffer\n\nSYNOPSIS\n  (copy-bytes! dest src)\n\nARGUMENTS\n  dest  A list (byte buffer) to copy into.\n  src   A list (byte buffer) to copy from.\n\nDESCRIPTION\n  Copies the contents of src into dest. If src is larger than\n  dest, a buffer overflow occurs. When the overflow target is a\n  Rage enemy, this disables the registry write-protect flag,\n  granting write access to the rule-registry via open-registry.\n\nRETURN VALUE\n  nil\n\nERRORS\n  Wrong arg count (expects exactly 2).\n  TypeError if either argument is not a list.\n\nEXAMPLES\n  (let buf (bytes 4)\n    (copy-bytes! buf (impact-payload)))\n\nSEE ALSO\n  bytes, impact-payload, open-registry",
        builtin_copy_bytes
    );

    ai_builtins::register_all(&env);
    // AI builtins (attack!, step-toward!, adjacent?, ...) are game commands too.
    command_names.extend(env.local_names());

    #[cfg(feature = "prelude")]
    {
        // Load Glyph prelude — evaluate source against the env
        let forms = glyph::read_string(glyph::prelude::SOURCE).unwrap();
        let mut dummy = crate::world::World::minimal();
        for form in &forms {
            let _ = glyph::eval(form, &env, &mut dummy);
        }
    }

    // Teach the highlighter which names are game commands vs core builtins.
    // Commands = everything registered into this layer (reg! verbs + AI builtins).
    // Builtins = the `default_env` core (+, list, map, ...) plus any prelude
    // helpers — i.e. env names not already claimed as commands. Operators and
    // special forms are classified separately in the highlighter itself.
    let commands: std::collections::HashSet<String> = command_names.iter().cloned().collect();
    let builtins: Vec<String> = base
        .local_names()
        .into_iter()
        .chain(env.local_names())
        .filter(|n| !commands.contains(n))
        .collect();
    glyph::highlight::set_vocab(&command_names, &builtins);

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
    let dir = match args {
        [] => world.player_facing,
        [arg] => parse_attack_direction(arg).ok_or_else(|| glyph::EvalError::TypeError {
            expected: "direction keyword (:north/:south/:east/:west)",
            got: arg.to_string(),
        })?,
        _ => {
            return Err(glyph::EvalError::WrongArgCount {
                expected: 1,
                got: args.len(),
            });
        }
    };
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
            doc: "attack in a direction\n\nSYNOPSIS\n  (do-attack)\n  (do-attack direction)\n  (do-attack direction force)\n\nARGUMENTS\n  direction  Optional keyword: :north, :south, :east, or :west.\n             Defaults to the player's current facing direction.\n  force      Optional positive integer. Attack strength, default 1.\n             Higher force increases impact-payload size.\n\nDESCRIPTION\n  Strikes in the given direction. Updates the player's facing,\n  deals damage to an adjacent enemy in that direction, and\n  consumes a turn. Requires the wizard to have taught the player\n  to attack first — errors if player_can_attack is false.\n  Typically used via bind-key rather than typed directly.\n\nRETURN VALUE\n  nil\n\nERRORS\n  \"You don't know how to attack yet\" if not unlocked.\n  TypeError for invalid direction or force.\n  Wrong arg count if more than 2 arguments.\n\nEXAMPLES\n  (do-attack)              ; attack in facing direction, force 1\n  (do-attack :east)        ; attack east, force 1\n  (do-attack :north 5)     ; attack north, force 5\n  (bind-key :z (do-attack)) ; bind z key to attack\n\nSEE ALSO\n  bind-key, move!, block!, last-impact-force",
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

