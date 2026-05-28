# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

CoreDumped is a text-graphical roguelike about inspecting and eventually rewriting the rules that govern the dungeon, built on the **Xlyph** engine. The current beta is a playable vertical slice using `crossterm` for the terminal frontend and small bracket crates for color, geometry, pathfinding, and random numbers. The long-term vision embeds a custom Lisp (Glyph) as the run-time for rules, queries, and player-authored patches, but the playable prototype today is pure Rust.

## Commands

```bash
# Run the game (TUI)
cargo run -p coredumped-tui

# Build check (no run)
cargo check

# All tests
cargo test

# Format
cargo fmt

# Build web version
cd web-frontend && bun run build:wasm  # WASM only
cd web-frontend && bun run build:all   # WASM + Vite bundle
```

## Workspace Structure

The workspace has three crates:

| Crate | Package Name | Purpose |
|-------|--------------|---------|
| `core/` | `coredumped-core` | Game engine: ECS, map, rules, rendering logic, save/load |
| `tui/` | `coredumped-tui` | Terminal frontend using crossterm |
| `web-frontend/` | `coredumped-web` | WebAssembly frontend using xterm.js |

## Architecture

### Core (`core/src/`)

Platform-agnostic game engine. Both TUI and web frontends depend on this crate.

**Simulation core** — `game.rs` + `console.rs` + `builtins.rs`
- `World` owns the map, turn counter, UI mode, event log, console buffer, player-facing direction, inspector scroll, and an `Ecs` store. It is the single source of truth.
- `Intent` is the action enum produced by the input layer. `ActionCost` classifies every intent as `Free` (no time passes), `Tick` (advances turn + enemies), or `Quit`.
- `Mode` (Normal / Inspector / Console / Dead / Keybindings / Memories) determines how keys are routed and what overlays draw.
- Gameplay systems (player movement, wall bump, melee attack, enemy AI step, tick advancement, inspector scroll) live directly on `World` as methods. Enemy pathing uses bracket pathfinding helpers.
- Console input handling (buffer ops, cursor movement, history, submission) is in `console.rs`.
- Glyph builtin functions (move, attack, save/load, help pages, registry access) are in `builtins.rs`.

**ECS** — `ecs.rs` and `entity.rs`
- Custom in-house ECS: `EntityId` is a stable `usize` handle. Component stores are `BTreeMap<EntityId, T>` for position, HP, kind, render glyph. Marker sets (`BTreeSet<EntityId>`) track alive entities and enemy AI membership.
- `EntityView` is a read-only snapshot returned by queries. No systems abstraction — game logic reads/writes ECS directly.

**Map** — `map.rs`
- Fixed 55×30 static map. Implements `Algorithm2D` and `BaseMap` for pathfinding.
- Flashlight ray-caster: selects tiles within a radius cone in the facing direction, then Bresenham-traces each ray until a wall.

**Rendering** — `render.rs`
- Read-only projection of `World`. Draws the map (floor `.` / wall `#`), flashlight-lit tiles in warm colors, entities as colored glyphs, right-side panel (turn/hp/mode/controls/inspector), bottom event log, console overlay, and entity tooltip on mouse hover.

**Terminal abstraction** — `terminal.rs`
- `Frame` is a cell buffer with `set()`, `print_color()`, and `to_ansi_string()`. Platform frontends consume `Frame` for rendering.

**Rules** — `rules.rs`
- `RuleRegistry` stores `Rule` structs with id, name, phase (EnemyAi / Render), cost (Tick / Free), and static source lines. Displayed in the inspector panel.

**Event log** — `event_log.rs`
- Append-only ring buffer capped at 100 lines. Game systems push human-readable strings; the renderer shows the most recent entries.

### TUI (`tui/src/`)

Crossterm-based terminal frontend.

- `app.rs` — Event loop: reads terminal events, translates keys to intents, applies to `World`, renders via `Frame.flush()`.
- `input.rs` — Mode-aware key translation: `key_to_intent(KeyEvent, &World) -> Intent`.
- `terminal_ext.rs` — Crossterm-specific `Frame.flush()` implementation.

### Web (`web-frontend/`)

WebAssembly frontend using xterm.js.

- `src/lib.rs` — wasm-bindgen bindings for xterm.js bridge.
- `src/app.rs` — Web event loop with RAF + key callbacks.
- `web-assets/` — HTML, CSS, JS, and build script.

## Key design rules

- Every gameplay action costs a tick; UI actions (inspector, console, typing) are free. This invariant is tested.
- Rendering is always a read-only projection — do not mutate `World` from `render.rs`.
- Tests live inline in `game.rs` and `rules.rs` (no separate test files). Tests construct `World` directly and run pure game logic without the renderer.
- Core crate is platform-agnostic — no crossterm or wasm-bindgen dependencies.
