# Code Architecture

This document describes the implemented Rust architecture for CoreDumped and
the Xlyph terminal runtime. It is about the current code, not the long-term
concept architecture in [game-architecture.md](./game-architecture.md).

For a shorter contributor guide, see [PROJECT.md](./PROJECT.md).

## System Overview

The repository contains one Cargo workspace member, `coredumped`, whose package
and binary name is `xlyph-tui`.

At runtime the app is a terminal event loop around one mutable `World`:

```text
main.rs
  -> app::run_with_options
  -> State::tick
     -> mark discovery state
     -> render current World into Frame
     -> poll terminal events
     -> input::key_to_intent
     -> World::apply_intent
        -> direct gameplay mutation, Glyph evaluation, or UI update
```

The architecture is intentionally small. There is no async runtime, no external
ECS framework, no separate scripting process, and no renderer-owned game state.
Most systems mutate `World` directly, while rendering reads `World` and writes
only to the terminal `Frame`.

## Module Ownership

| Module | Owns |
| --- | --- |
| `main.rs` | CLI parsing, logger initialization, terminal app launch. |
| `app.rs` | Crossterm session lifecycle, event loop, terminal resize, mouse state, frame flush. |
| `world.rs` | The `World` struct, cached pathing/visibility helpers, rule discovery. |
| `game.rs` | Intent handling, turn systems, combat, console, keybindings, game builtins. |
| `input.rs` | Translation from `crossterm::KeyEvent` to `Intent`. |
| `render.rs` | Read-only projection of `World` into a `Frame`. |
| `terminal.rs` | Cell buffer, color conversion, emoji/wide-text terminal support. |
| `ecs.rs` | Stable entity ids and component stores. |
| `entity.rs` | Entity component primitives and entity kind metadata. |
| `map.rs` | Tiles, pathfinding integration, flashlight ray casting, procedural room generation. |
| `levels/` | Authored depth builders, shared level helpers, procedural fallback. |
| `glyph/` | Reader, evaluator, values, lexical environment, prelude, syntax highlighting. |
| `rules.rs` | Static inspectable rule registry and parsed AI/tile rule bodies. |
| `ai_builtins.rs` | Glyph host functions used by enemy AI rules. |
| `save.rs` | Serializable world snapshots, save/load disk paths, external editor temp path. |
| `playbook.rs` | User-authored Glyph load paths under `~/.xlyph/playbooks/current`. |
| `player_profile.rs` | Player-owned state persisted outside individual save slots. |
| `fragment.rs` | Memory fragment registry and collection status. |
| `event_log.rs` | Ring-buffer log displayed by the UI. |
| `no_hit.rs` | Bounded no-hit route search over cloned `World` states. |
| `diagnostics.rs` | File-backed logging to `~/.xlyph/logs/xlyph.log`. |

`lib.rs` exposes the library boundary for tests and for the binary. It reexports
the core types (`World`, `Ecs`, `EntityId`, `Map`, `RuleRegistry`, `Intent`,
`ActionCost`, and no-hit analysis types) while keeping some implementation
modules crate-private.

## Startup Lifecycle

Startup begins in `main.rs`:

1. Parse `--wipe` and `--ascii-only` with `clap`.
2. Initialize file logging through `diagnostics::init_file_logger`.
3. If `--wipe` is present, delete auto-save/profile data with `save::wipe`.
4. Call `app::run_with_options`.

`app::State::new_with_options` then decides whether to load slot 0 or create a
new game:

```text
if ~/.xlyph/saves/slot-0.json exists
  World::load_from_disk(0)
else
  World::new_game()
```

`World::new_game` creates a player entity, initializes Glyph environments,
builds depth 0 through `levels::build_level`, and loads the current playbook if
one exists.

## App Shell

`app.rs` is the only module that owns terminal side effects:

- Enters and leaves the alternate screen.
- Enables raw mode and mouse capture.
- Polls `crossterm` events.
- Tracks terminal size.
- Maintains the `Frame`.
- Calls `render`.

`State::tick` has three phases:

1. Update derived discovery state:
   `mark_visible_entities`, `mark_visible_tiles`, `refresh_rule_discovery`.
2. Clear and render the current world into the frame, then flush it.
3. Poll one terminal event and apply the resulting intent.

The app shell does not contain game rules. When it receives input, it delegates
key interpretation to `input.rs` and game mutation to `World::apply_intent`.

## Input Model

Raw keyboard input is converted to `Intent` values. The current `Mode` decides
which keymap is active:

```text
Mode::Normal      -> default bindings, F5/F9 save/load, log scroll
Mode::Inspector   -> inspector scroll, close, console toggle
Mode::Keybindings -> keybinding overlay scroll, close, console toggle
Mode::Console     -> editor controls, history, cursor movement, submit
Mode::Dead        -> respawn, restart, quit
```

In normal mode, most keys are not hard-coded to actions. They are normalized to
binding names such as `left`, `right`, `h`, `q`, or ``` ` ```, then looked up in
`World::bindings`. A binding stores Glyph source, for example:

```lisp
(move! :west)
(toggle-console!)
(descend!)
```

This means normal controls, custom keybindings, and playbook-defined controls
all flow through the Glyph evaluator.

## World State

`World` is the single source of simulation truth. It owns:

- `Map`
- `Ecs`
- `RuleRegistry`
- player entity id, facing, HP access, and current depth
- turn counter and current `Mode`
- event log
- console input/output/cursor/history state
- Glyph runtime environments
- keybindings and user source persistence
- discovered entities, tiles, and rules
- collected fragments and special campaign items
- per-level callbacks such as wizard interaction
- tick-local caches for flashlight, fire, and Dijkstra pathing

`World::minimal` exists for tests and for contexts that need a placeholder
world, such as evaluating the Glyph prelude during environment setup.

Most behavior lives in `impl World` blocks split across `world.rs`, `game.rs`,
and `save.rs`:

- `world.rs` handles cached pathing/visibility and rule discovery.
- `game.rs` handles gameplay systems, input intents, console state, builtins,
  and tick advancement.
- `save.rs` handles serialization and restoration.

## Intent Handling And Action Cost

`World::apply_intent` is the main mutation gate. It returns:

```rust
pub enum ActionCost {
    Free,
    Tick,
    Quit,
}
```

Important rules:

- UI actions are free: console typing, scrolling, overlay toggles, history.
- World-changing gameplay usually ticks: movement, waiting, attacks, barriers,
  stairs, enemy advancement.
- Some interactions are intentionally free, such as collecting fragments or
  picking up certain items.
- Keybindings are evaluated as Glyph. Their cost is inferred by comparing the
  turn before and after binding execution.

This cost model is the main turn-order contract. Tests exercise the invariant
that UI-only actions do not advance enemies while tick actions do.

## Turn Execution

Tick actions call `finish_tick` either directly or through a builtin that mutates
the world. A tick:

1. Increments `World::turn`.
2. Rebuilds the fire cache from map tiles.
3. Applies depth-specific gauntlet barriers.
4. Advances enemy AI once.
5. Repairs enemy positions if a rule or barrier left one invalid.
6. Clears tick-local player attack/blocking flags.
7. Moves to `Mode::Dead` if player HP is depleted.

Enemy AI is data-driven by entity kind:

```text
EntityKind::rule_name
  -> RuleRegistry::get(rule_name)
  -> Rule::body_form
  -> glyph::eval_with_opts(body_form, enemy_env, sandbox, world)
```

Each enemy gets an environment extended from the game Glyph environment with
`*self*` and `*player*` bound to entity ids. AI builtins such as `adjacent?`,
`attack!`, `step-toward!`, `random-step!`, `flee-step!`, `hp`, and `manhattan`
are Rust functions registered into that environment.

## ECS

The ECS is deliberately tiny. `EntityId` is a stable `usize` wrapper. `Ecs`
stores components in ordered maps and marker sets:

```text
entities:       BTreeSet<EntityId>
kinds:          BTreeMap<EntityId, EntityKind>
positions:      BTreeMap<EntityId, Position>
hp:             BTreeMap<EntityId, Hp>
alive:          BTreeSet<EntityId>
enemy_ai:       BTreeSet<EntityId>
render_glyphs:  BTreeMap<EntityId, RenderGlyph>
sign_messages:  BTreeMap<EntityId, String>
fragment_ids:   BTreeMap<EntityId, String>
```

There is no system dispatcher. Game code queries and mutates the store directly.
`EntityView` is a read-only snapshot used by render, tests, AI helpers, and
analysis tools.

`Ecs::set_position` enforces basic occupancy rules by rejecting moves into an
occupied tile. Higher-level walkability and special-case rules live in `World`
and `Map`.

## Entity Model

`entity.rs` defines the data primitives:

- `EntityId`: stable id wrapper.
- `Direction`: cardinal movement with `delta`.
- `Position`: grid coordinate with offsets and Manhattan distance.
- `Hp`: current/max hit points.
- `EntityKind`: player, enemy types, wizard, barrels, signs, fragments, items.
- `RenderGlyph`: display character for a kind.
- `EntityView`: render/query snapshot.

`EntityKind::rule_name` maps AI-capable kinds to rule ids in `RuleRegistry`.
Kinds without AI return an empty rule name.

## Map, Pathing, And Visibility

`Map` owns tile data and implements bracket pathfinding traits:

- `Algorithm2D`
- `BaseMap`

Tile types are:

```rust
Floor
Wall
StairsDown
StairsUp
Fire
```

Map responsibilities:

- Bounds checks and index/position conversion.
- Walkability checks.
- Pathfinding exits for bracket pathfinding.
- Static map construction.
- Procedural room generation with region-based room placement, MST corridors,
  extra loop edges, room typing, spawn points, and stairs.
- Flashlight visibility through a cone filter plus Bresenham ray tracing.

`World` caches flashlight tiles based on player position and facing. Discovery
uses that cache to mark visible entity kinds and tile types, which in turn
drives rule visibility in the inspector.

`World` also caches one Dijkstra map per AI target for a tick. `ai_builtins`
uses `World::dijkstra_best_step` for pathing toward the player.

## Levels

Authored campaign levels live in `coredumped/src/levels/depth_*.rs`. The
dispatcher is `levels::build_level`:

1. Clear all level entities except the player.
2. Select the builder for the requested depth.
3. Apply the generated map and entity placement.
4. Rebuild the fire cache.
5. Log placement diagnostics.

Each builder receives `&mut World`, so levels can:

- Apply a map.
- Move the player start.
- Place enemies, signs, fragments, items, stairs, and hazards.
- Configure `on_wizard_interact` callbacks for depth-specific dialogue or
  behavior.

Depths beyond the authored campaign fall through to `levels/procedural.rs`.

## Rule Registry

`RuleRegistry` stores inspectable rules as Rust structs:

```rust
pub struct Rule {
    pub id: &'static str,
    pub name: &'static str,
    pub phase: RulePhase,
    pub cost: RuleCost,
    pub source_lines: &'static [&'static str],
    pub body_form: Value,
}
```

`source_lines` are the readable source shown to the player. `body_form` is the
parsed Glyph expression actually evaluated for runtime rules. Most rules are
AI rules; `fire/burn` is a tile-effect rule; `vessel-suppress` is currently
inspected narrative/rule text rather than evaluated behavior.

Rule discovery is based on what the player has seen:

- `RuleRegistry::ALWAYS_VISIBLE` makes `flashlight` visible immediately.
- Seen entity kinds reveal their AI rule.
- Seen fire tiles reveal `fire/burn`.
- Newly visible rules are added to `World::new_rule_ids` and logged.

The inspector renders only known/discovered rules, with new rules highlighted
until the inspector is closed.

## Glyph Runtime

Glyph is a Lisp runtime embedded directly in the game process.

### Reader

`glyph/reader.rs` parses source into `Value` forms. It handles:

- Lists, maps, sets, and vector/list sugar.
- Quote sugar.
- Dot access sugar.
- Infix expressions.
- Keywords, symbols, strings, numbers, comments.

Reader errors carry byte offsets and can render ariadne diagnostics for console
output.

### Values And Environment

`glyph/value.rs` defines runtime data and errors:

- `Value`: nil, bool, integers, floats, strings, symbols, keywords, lists,
  vectors/maps/sets as implemented values, builtins, closures, macros.
- `ReadError` and `EvalError`.
- `SandboxOptions`: recursion budget, optional virtual filesystem, and `recur`
  permission.
- `BuiltinFn`: Rust function pointer plus docs.

`glyph/env.rs` is a lexical environment backed by shared mutable environment
nodes. Environments can be extended for closures, macros, keybindings, console
evaluation, and enemy AI.

### Evaluator

`glyph/eval.rs` evaluates values with a `World` parameter available to builtins.
It supports:

- Self-evaluating values.
- Symbol lookup.
- Special forms such as `quote`, `if`, `do`, `let`, `fn`, `const`,
  `defmacro`, `set!`, `try`, `and`, `or`, `match`, `bind-key`, and `recur`.
- Macro expansion before normal function application.
- Closures and multi-arity functions.
- Tail recursion via `recur`.
- Builtins for arithmetic, collections, printing, eval/apply/map/range, and
  host-provided game behavior.

Every builtin has this shape:

```rust
fn(&[Value], &Env, &SandboxOptions, &mut World) -> EvalResult<Value>
```

That shared signature is the bridge between language code and game state.

### Environments

`setup_glyph_env` builds the main game environment:

1. Extend `glyph::default_env`.
2. Register game builtins such as `move!`, `wait!`, `block!`, `shove!`,
   `descend!`, `save!`, `load!`, `query-registry`, and `inspect-fragment`.
3. Register AI builtins from `ai_builtins`.
4. Load the Glyph prelude when the default `prelude` feature is enabled.

`setup_binding_env` extends the main game environment for keybinding
evaluation. This separates binding execution from console history/cursor state
while keeping the same builtins available.

## Console And Keybindings

The console stores editable input in `World::console_buffer` and tracks cursor,
history, output, and output scroll.

Submitting console input:

1. Handles pending wipe confirmation, if any.
2. Parses the buffer with `glyph::read_string`.
3. Attempts auto-closing delimiters when parsing fails.
4. Stores environment-mutating forms in `World::user_source`.
5. Evaluates all forms in the main Glyph environment.
6. Displays either the final value, printed output, or an error diagnostic.
7. Applies ending logic on depth 17 for selected registry commands.

`bind-key` is a special form implemented inside the evaluator. It writes a key
name and expression source into `World::bindings`, marks new bindings for the
keybinding overlay, and participates in save/load through `user_source`.

Default movement and UI controls are just initial entries in the same bindings
map.

## Save, Load, Profile, And Playbooks

Persistent data is rooted at `~/.xlyph`:

```text
~/.xlyph/saves/slot-0.json
~/.xlyph/saves/slot-N.json
~/.xlyph/profile.json
~/.xlyph/playbooks/current/init.glyph
~/.xlyph/playbooks/current/lib/*.glyph
~/.xlyph/tmp/console-input.glyph
~/.xlyph/logs/xlyph.log
```

`SaveData` is an explicit serializable snapshot containing map tiles, entities,
turn/depth/player state, event log entries, bindings, user source, fragments,
ending state, special items, and level-specific state.

Loading does not deserialize a live Glyph environment. It:

1. Rebuilds a minimal world.
2. Restores map and ECS snapshots.
3. Restores simple game state fields.
4. Rebuilds `glyph_env` and `binding_env`.
5. Re-registers learned builtins such as `do-attack`.
6. Replays `user_source` into the fresh environment.

This makes user-authored state source-replayable rather than depending on
serialized closures.

Playbooks use the same model. On new game startup, `load_playbook` reads
`init.glyph` and sorted `lib/*.glyph` files, evaluates them into both the main
and binding environments, and appends successful forms to `user_source`.

## Rendering

Rendering is read-only with respect to `World`.

`render.rs` draws:

- Map tiles.
- Flashlight-lit terrain.
- Entities, using emoji where supported and ASCII otherwise.
- Side panel with player status, controls, rules, and keybinding summaries.
- Event log.
- Inspector overlay.
- Console overlay with syntax-highlighted text and diagnostics.
- Keybindings overlay.
- Death screen.
- Mouse hover tooltip.

`terminal.rs` owns the `Frame` abstraction. A frame is a cell buffer with RGB
foreground/background colors and either a single char, a static text span, or a
skip marker for wide text continuation cells.

Frame flush converts each cell into crossterm commands. Wide glyph support is
important for emoji entities; trailing cells are marked as `Skip` so later
drawing can clear or overwrite the whole span safely.

## Diagnostics

`diagnostics.rs` installs a simple file logger. Logs go to
`~/.xlyph/logs/xlyph.log` and include timestamp, level, target, and message.

Current diagnostic targets include:

- level construction
- depth transitions
- AI movement/pathing
- rejected ECS moves
- entity overlap detection

The in-game event log tells the player what happened; diagnostics are for
debugging simulation bugs.

## Analysis Tools

`no_hit.rs` is not part of the live game loop. It answers whether a route to an
exit can be found without taking damage by cloning `World` and replaying real
movement/wait intents in a bounded breadth-first search.

The search key includes player position, player HP, and sorted enemy state. It
uses `World::apply_intent`, so it exercises the same turn rules as gameplay.

## Main Data Flow

```text
Terminal key/mouse event
  -> input::key_to_intent
  -> Intent
  -> World::apply_intent
     -> direct mutation
     -> or binding Glyph source
     -> or console Glyph source
     -> or save/load
  -> ActionCost
  -> State::tick renders updated World
```

Enemy AI data flow:

```text
finish_tick
  -> advance_enemies
  -> EntityKind::rule_name
  -> RuleRegistry::get
  -> Glyph body_form
  -> eval_with_opts
  -> ai_builtins mutate World
```

Rule discovery data flow:

```text
render tick prelude
  -> World::ensure_lit_tiles
  -> mark_visible_entities / mark_visible_tiles
  -> RuleRegistry::visible_ids
  -> known_rule_ids / new_rule_ids
  -> inspector rendering
```

Save/load data flow:

```text
World
  -> SaveData
  -> JSON on disk
  -> SaveData
  -> World::minimal
  -> restored map/ECS/state
  -> rebuilt Glyph envs
  -> replayed user_source
```

## Extension Points

Add a new enemy:

1. Add a variant to `EntityKind`.
2. Add glyph/name/rule mapping in `entity.rs`.
3. Add a spawn helper in `Ecs`.
4. Add save/load conversion in `save.rs`.
5. Add a `Rule` in `RuleRegistry` if it has AI.
6. Register placement in authored/procedural levels.
7. Add tests around spawn walkability and behavior.

Add a game command:

1. Implement a builtin in `game.rs`.
2. Register it in `setup_glyph_env`.
3. Decide whether it should call `finish_tick`.
4. Add help text.
5. Add tests for cost, state mutation, and save/load implications.

Add a Glyph builtin:

1. Implement it in `glyph/eval.rs` for pure language behavior, or in `game.rs`
   / `ai_builtins.rs` when it needs world access.
2. Register it in the appropriate environment.
3. Update `glyph-reference.md` or `language-spec.md` when user-visible.
4. Add reader/evaluator tests.

Add a level:

1. Create a new `levels/depth_XX_name.rs` builder.
2. Register it in `levels/mod.rs`.
3. Use helpers for map application, wizard placement, and fragment placement.
4. Add invariant tests for walkability, exits, fragments, and enemies.

## Architectural Invariants

- `World` is the simulation source of truth.
- `render.rs` must stay read-only.
- `app.rs` owns terminal side effects.
- All input should become an `Intent` before mutating the world.
- Gameplay mutation should make tick cost explicit.
- Enemy AI, console expressions, and keybindings should use the same Glyph
  evaluator.
- Save/load should rebuild runtime environments and replay source rather than
  serializing closures.
- Authored rules shown in the inspector should correspond to runtime behavior,
  except for intentionally inspect-only narrative rules.
- Entity placement should avoid blocked or occupied tiles.
- Tests should prefer direct `World` and Glyph calls over terminal rendering.
