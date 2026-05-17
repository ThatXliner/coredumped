# Xlyph

Welcome, traveler.

Before thee lies a dungeon of rules, lamps, slimes, and suspiciously readable
machinery. Xlyph is a text-graphical roguelike about understanding, editing, and
eventually rewriting the rules of the thing trying to kill you.

Past this point, the scroll becomes plain.

The current beta is a small playable vertical slice: a text-graphical dungeon,
turn-based movement, pathing enemies, a directional flashlight, an inspector
panel, an event log, and a stub console for future live queries. It is
intentionally tiny, but the loop is real: every gameplay action advances the
world exactly one tick.

> Inspired by [xsofy](https://github.com/nooga/xsofy), which was inspired by
> [Brogue](https://sites.google.com/site/broguegame/).

## Why

Most roguelikes ask you to learn a system from the outside. Xlyph is an
experiment in making the system itself part of the dungeon.

The long-term idea is that monsters, items, terrain, and mechanics can expose
source-like behavior to the player. You inspect what a creature does, reason
about it, and eventually use an in-game language to query or change parts of the
world. The beta does not include the full language runtime yet, but it does
include the first honest version of that interface: an enemy AI inspector whose
displayed rule matches the implemented rule.

## Status

Beta vertical slice. Playable, small, and not balanced.

What works now:

- A fixed dungeon map rendered with `bracket-lib`
- A windowed `bracket-lib` prototype renderer
- Keyboard movement with arrow keys or Vim keys
- Deterministic turn ticks
- Wall bumps, waits, enemy bumps, and movement all consume a tick
- UI-only actions such as inspector navigation and console typing are free
- Enemies path toward the player and attack when adjacent
- A warm directional flashlight that ray-casts from the player's facing
- A right-side status/inspector panel
- A bottom event log
- A console overlay with placeholder query responses
- Unit tests for the core turn and enemy behavior

What is not in this beta:

- Procedural generation
- Save files or replay
- Full field of view or fog of war
- Inventory
- A finished Glyph parser/runtime
- Any claim that the game is fair yet

## Quick Start

You need Rust installed.

```bash
cargo run -p xlyph-tui
```

Run the tests:

```bash
cargo test
```

If you are working through the repo's local tooling, use:

```bash
rtk cargo run -p xlyph-tui
rtk cargo test
```

## Controls

| Key | Action |
| --- | --- |
| Arrow keys / `h j k l` | Move |
| `.` | Wait one tick |
| `i` | Toggle inspector |
| <code>`</code> | Toggle console |
| `Enter` | Submit console text |
| `Esc` | Close overlay, or quit if none is open |
| `q` | Quit from normal mode |

The important design rule is that gameplay actions cost time and interface
actions do not. Moving, waiting, bumping a wall, and bumping an enemy all
advance the world. Opening the inspector, reading source-like behavior, opening
the console, and typing in it are free.

## The Beta Loop

You are `@`.

Enemies are simple, but deliberately inspectable. On each player tick, every
enemy does one of two things:

1. If adjacent to the player, attack.
2. Otherwise, path one step toward the player while respecting walls.

That rule is also shown in the in-game inspector. The point is not that the AI
is clever; the point is that the game should make its behavior legible before it
asks you to manipulate it.

## Architecture

The beta uses [`bracket-lib`](https://docs.rs/bracket-lib) for rendering, input,
geometry, colors, pathfinding, and the game loop. The current prototype uses
bracket-lib's default windowed renderer; the terminal backend was tried and is
parked for now because of redraw glitches.

The game state is intentionally plain Rust:

- `World`
- `Ecs`
- `EntityId`
- `Map`
- `Position`
- `Hp`
- `RenderGlyph`
- `EntityKind`
- `Mode`
- `EventLog`
- `Turn`

There is no external ECS dependency yet. The current prototype uses a small
in-house ECS: stable entity ids, component stores for position/HP/kind/rendering
and marker sets for things like enemy AI. Gameplay still lives in explicit
systems so the turn order remains easy to read.

The turn model is explicit:

- `ActionCost::Free` for UI actions
- `ActionCost::Tick` for accepted gameplay actions
- Enemy turns run exactly once after each tick action

That makes it possible to test the game rules without the renderer.

## Development

Common checks:

```bash
cargo fmt
cargo test
cargo check
```

With local tooling:

```bash
rtk cargo fmt
rtk cargo test
rtk cargo check
```

The most useful tests cover pure game logic:

- Player movement increments the turn
- Wall bumps increment the turn and log blocked movement
- Waiting increments the turn
- Inspector and console actions do not increment the turn
- Enemies advance once after each tick action
- Adjacent enemies attack instead of moving
- Enemy pathing respects walls

## Roadmap

Near term:

- Make the inspector browse more than one rule
- Replace the console placeholder with a real query path
- Add a tiny Glyph syntax for inspecting entities and map cells
- Add a few enemy types with visibly different rules
- Add screenshots and recorded gameplay demos

Later:

- Procedural maps
- Replayable deterministic runs
- Save/load
- More expressive world editing
- A real design pass on balance, readability, and pacing

## Name

`Xlyph` is a working title. It is short, odd, and looks good in blocky glyphs.
