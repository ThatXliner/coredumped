# Project Documentation

CoreDumped is a terminal roguelike built on the Xlyph runtime. The game presents
rules as readable Glyph source, then lets the player inspect and eventually
change parts of the system from inside the dungeon.

This document is the contributor-facing guide. For the player-facing overview,
see [README.md](./README.md). For Glyph itself, see
[glyph-reference.md](./glyph-reference.md) and
[language-spec.md](./language-spec.md).

## Project Shape

The repository is a single Cargo workspace:

```text
.
|-- Cargo.toml                 # workspace manifest
|-- coredumped/
|   |-- Cargo.toml             # package: xlyph-tui
|   `-- src/
|       |-- main.rs            # CLI entrypoint
|       |-- app.rs             # terminal event loop
|       |-- world.rs           # central game state
|       |-- game.rs            # gameplay systems and Glyph game builtins
|       |-- input.rs           # key/mouse events to intents
|       |-- render.rs          # read-only terminal rendering
|       |-- terminal.rs        # frame buffer and terminal helpers
|       |-- ecs.rs             # entity/component storage
|       |-- entity.rs          # entity ids, positions, kinds, HP, glyphs
|       |-- map.rs             # terrain, pathing, visibility, map generation
|       |-- levels/            # authored depths plus procedural fallback
|       |-- glyph/             # reader, evaluator, env, prelude, highlighting
|       |-- rules.rs           # inspectable rule registry
|       |-- ai_builtins.rs     # host functions used by AI rules
|       |-- save.rs            # save/load snapshots
|       |-- playbook.rs        # user-authored Glyph auto-load folder
|       |-- fragment.rs        # memory fragment data and collection status
|       |-- no_hit.rs          # route analysis helper
|       `-- diagnostics.rs     # file-backed logs
|-- README.md
|-- glyph-reference.md
|-- language-spec.md
|-- level-design.md
`-- game-architecture.md       # concept/vision document
```

The crate name on disk is `coredumped`; the Cargo package and binary are
`xlyph-tui`.

## Common Commands

Run these from the repository root:

```bash
cargo run -p xlyph-tui
cargo run -p xlyph-tui -- --wipe
cargo run -p xlyph-tui -- --ascii-only
cargo check
cargo test
cargo fmt
```

The binary options are:

| Option | Purpose |
| --- | --- |
| `--wipe` | Delete the auto-save and player profile before launch. |
| `--ascii-only` / `--ascii` | Force ASCII entity glyphs even if emoji support is detected. |

## Runtime Flow

The app is intentionally direct:

```text
main.rs
  -> app::run_with_options
  -> State::tick
  -> input::key_to_intent
  -> World::apply_intent
  -> render::render
```

`app.rs` is the only module that owns the crossterm alternate screen, raw mode,
event polling, mouse capture, and frame flushing. Everything else is ordinary
Rust state and pure-ish logic, which keeps tests away from the terminal.

`input.rs` converts keys into `Intent` values based on the current `Mode`.
`World::apply_intent` mutates the world and returns an `ActionCost`. Rendering
then projects the updated `World` into a `Frame`.

## Core Concepts

### World

`World` is the central state object. It owns the map, ECS, rule registry, player
id, current depth, turn counter, UI mode, event log, Glyph environments,
bindings, save-relevant player state, discovery state, and per-level callbacks.

Most gameplay logic is implemented as methods on `World` in `game.rs`. This is
deliberately small and explicit: there is no separate scheduler or systems
framework between an intent and the world mutation it causes.

### Intent And Tick Cost

All player input becomes an `Intent`. Applying an intent returns:

```rust
pub enum ActionCost {
    Free,
    Tick,
    Quit,
}
```

The important invariant is that gameplay actions cost `Tick`, while interface
actions cost `Free`. A tick advances the turn, updates tile effects, runs enemy
AI once, clears per-tick flags, and persists auto-save state when appropriate.
Opening the inspector, editing console text, browsing history, and scrolling
logs do not advance the simulation.

### Modes

`Mode` controls input routing and overlays:

| Mode | Purpose |
| --- | --- |
| `Normal` | Movement, waiting, interaction, keybindings, quit handling. |
| `Inspector` | Browse known rules and source. |
| `Console` | Enter and evaluate Glyph. |
| `Keybindings` | Show active bindings and newly learned commands. |
| `Dead` | Death overlay, respawn, restart, or quit. |

### ECS

`Ecs` is a small in-house component store. `EntityId` is a stable handle, and
component maps are keyed by id. Marker sets track living entities and AI-driven
entities. There is no ECS framework; game systems read and write the component
stores directly.

Queries usually return `EntityView`, a snapshot that is convenient for rendering,
AI, tests, and route analysis.

### Map And Levels

`Map` owns terrain, walkability, pathfinding exits, flashlight visibility, and
procedural room generation. The default dimensions are `MAP_WIDTH` by
`MAP_HEIGHT`.

Authored campaign levels live in `coredumped/src/levels/depth_*.rs`. The level
dispatcher is `levels::build_level`, which clears level entities, calls the
builder for the requested depth, rebuilds caches, and logs the result. Depths
after the authored campaign fall back to `levels/procedural.rs`.

To add or adjust a level:

1. Create or edit a builder in `coredumped/src/levels/`.
2. Use helpers from `levels/helpers.rs` to apply maps and place recurring items.
3. Register the builder in `levels/mod.rs`.
4. Add focused tests when placement, exits, fragments, or special rules can
   regress.

### Rendering

`render.rs` is a read-only projection of `World`. It draws the map, entities,
side panel, event log, inspector, console, keybindings overlay, death overlay,
and hover tooltip into the frame buffer.

Do not mutate simulation state from rendering. If the UI needs derived state,
prefer computing it from `World` or updating `World` before rendering in
`State::tick`.

### Glyph Runtime

Glyph is embedded directly in Rust. The runtime lives in `coredumped/src/glyph/`:

| File | Responsibility |
| --- | --- |
| `reader.rs` | Parse source text into canonical values. |
| `value.rs` | Runtime value types, errors, builtin metadata, sandbox options. |
| `env.rs` | Lexical environment and binding storage. |
| `eval.rs` | Evaluator, special forms, builtins, macro expansion. |
| `prelude.rs` | Optional prelude loaded through the default feature. |
| `highlight.rs` | Syntax highlighting for in-game source display. |

Game-facing builtins are registered in `setup_glyph_env` in `game.rs`. AI-facing
builtins are registered from `ai_builtins.rs`. Builtins receive `&mut World`
through the evaluator, so there is no FFI boundary or separate scripting bridge.

Console expressions, keybindings, and AI rules use the same evaluator. Enemy AI
rules come from `RuleRegistry`, which stores display source and parsed rule
bodies for the inspector and runtime.

### Rules

`rules.rs` defines:

| Type | Purpose |
| --- | --- |
| `Rule` | A named rule with phase, cost, source lines, parsed body, and visibility metadata. |
| `RuleRegistry` | The current set of inspectable rules. |
| `RulePhase` | Where the rule participates, such as AI or render discovery. |
| `RuleCost` | Whether a rule is understood as tick-costing or free. |

The inspector only shows rules the player has discovered. `World` tracks known
rule ids and newly discovered rule ids, then the renderer highlights new entries.

### Save, Profile, And Playbooks

Project data lives under `~/.xlyph`:

| Path | Purpose |
| --- | --- |
| `~/.xlyph/saves/slot-0.json` | Auto-save slot. |
| `~/.xlyph/saves/slot-N.json` | Manual save slots. |
| `~/.xlyph/profile.json` | Player-owned state such as bindings, macros, learned abilities, and console history. |
| `~/.xlyph/playbooks/current/init.glyph` | User Glyph loaded on game start. |
| `~/.xlyph/playbooks/current/lib/*.glyph` | Additional playbook files loaded in sorted order. |
| `~/.xlyph/tmp/console-input.glyph` | Temporary file used by external editor input. |
| `~/.xlyph/logs/xlyph.log` | Diagnostics log for the latest run. |

Save files persist a serializable snapshot of world state. Glyph runtime state is
restored by replaying stored user source forms such as `const`, `defmacro`,
`set!`, and `bind-key` onto a fresh environment.

## Development Guidelines

Preserve these invariants when changing behavior:

- Gameplay actions that change world state should return `ActionCost::Tick`.
- UI-only actions should return `ActionCost::Free`.
- Rendering should not mutate `World`.
- The app shell should remain the only crossterm event-loop boundary.
- Save data should be explicit and versioned through `SaveData`.
- User-authored Glyph state should be replayable from source.
- Authored levels should leave player, exits, fragments, and enemy starts on
  walkable tiles.
- Rule source shown in the inspector should match the behavior the runtime uses.

## Testing Notes

Most tests are inline module tests. The useful test targets are:

```bash
cargo test
cargo test -p xlyph-tui game::
cargo test -p xlyph-tui glyph::
cargo test -p xlyph-tui levels::
```

For gameplay changes, add tests around `World::apply_intent` where possible. For
language changes, test the reader/evaluator directly. For level work, prefer
placement and invariant tests over renderer tests.

## Documentation Map

| Document | Use it for |
| --- | --- |
| [README.md](./README.md) | Player-facing overview, controls, current status. |
| [PROJECT.md](./PROJECT.md) | Contributor-facing project structure and runtime guide. |
| [glyph-reference.md](./glyph-reference.md) | Guided Glyph tutorial and examples. |
| [language-spec.md](./language-spec.md) | Formal-ish implemented Glyph semantics. |
| [level-design.md](./level-design.md) | Narrative and level design notes. |
| [game-architecture.md](./game-architecture.md) | Broader concept architecture and long-term vision. |
