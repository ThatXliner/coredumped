# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Xlyph is a text-graphical roguelike about inspecting and eventually rewriting the rules that govern the dungeon. The current beta is a playable vertical slice using `bracket-lib` for rendering, input, and pathfinding. The long-term vision embeds a custom Lisp (Glyph) as the run-time for rules, queries, and player-authored patches, but the playable prototype today is pure Rust.

## Commands

```bash
# Run the game
cargo run -p xlyph-tui

# Build check (no run)
cargo check

# All tests
cargo test

# Format
cargo fmt
```

The workspace has one crate: `tui` (package name `xlyph-tui`). No other automation beyond Cargo.

## Architecture

All source lives under `tui/src/`. The only external dependency is `bracket-lib ~0.8`.

**Simulation core** — `game.rs`
- `World` owns the map, turn counter, UI mode, event log, console buffer, player-facing direction, inspector scroll, and an `Ecs` store. It is the single source of truth.
- `Intent` is the action enum produced by the input layer. `ActionCost` classifies every intent as `Free` (no time passes), `Tick` (advances turn + enemies), or `Quit`.
- `Mode` (Normal / Inspector / Console) determines how keys are routed and what overlays draw.
- Gameplay systems (player movement, wall bump, melee attack, enemy AI step, tick advancement, console submission, inspector scroll) live directly on `World` as methods. Enemy pathing uses bracket-lib's `a_star_search`.

**ECS** — `ecs.rs` and `entity.rs`
- Custom in-house ECS: `EntityId` is a stable `usize` handle. Component stores are `BTreeMap<EntityId, T>` for position, HP, kind, render glyph. Marker sets (`BTreeSet<EntityId>`) track alive entities and enemy AI membership.
- `EntityView` is a read-only snapshot returned by queries. No systems abstraction — game logic reads/writes ECS directly.

**Map** — `map.rs`
- Fixed 55×30 static map. Implements bracket-lib's `Algorithm2D` and `BaseMap` for pathfinding.
- Flashlight ray-caster: selects tiles within a radius cone in the facing direction, then Bresenham-traces each ray until a wall.

**Key translation** — `input.rs`
- Mode-aware routing: `key_to_intent(VirtualKeyCode, &World) -> Intent` dispatches to normal/inspector/console sub-functions.
- Normal mode: arrow keys / hjkl → Move, `.` → Wait, `i` → ToggleInspector, backtick → ToggleConsole, Escape/q → Quit.
- Inspector mode: Escape/i → CloseOverlay, arrow/jk → scroll, backtick → ToggleConsole.
- Console mode: Escape → CloseOverlay, backtick → ToggleConsole, Backspace/Enter handled, alphanumerics → ConsoleInput.

**Rendering** — `render.rs`
- Read-only projection of `World`. Draws the map (floor `.` / wall `#`), flashlight-lit tiles in warm colors, entities as colored glyphs, right-side panel (turn/hp/mode/controls/inspector), bottom event log, console overlay, and entity tooltip on mouse hover.

**Rules** — `rules.rs`
- `RuleRegistry` stores `Rule` structs with id, name, phase (EnemyAi / Render), cost (Tick / Free), and static source lines. Currently hard-coded with two rules (`slime-hunt`, `flashlight`). Displayed in the inspector panel and meant to grow into a live registry with overlays.

**Event log** — `event_log.rs`
- Append-only ring buffer capped at 100 lines. Game systems push human-readable strings; the renderer shows the most recent entries.

**App shell** — `app.rs`
- `State` implements bracket-lib's `GameState`. `tick()` reads a key, translates to intent, applies it to `World`, and renders. The only module that talks to bracket-lib's game loop.

## Key design rules

- Every gameplay action costs a tick; UI actions (inspector, console, typing) are free. This invariant is tested.
- Rendering is always a read-only projection — do not mutate `World` from `render.rs`.
- Tests live inline in `game.rs` and `rules.rs` (no separate test files). Tests construct `World` directly and run pure game logic without the renderer.
