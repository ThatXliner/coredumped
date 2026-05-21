# CoreDumped

![](./coredumped_logo.png)

*A roguelike where the enemies run on Lisp you can read, edit, and break.*

Welcome, traveler.

Before thee lies a dungeon of rules, memories, slimes, and suspiciously readable
machinery. **CoreDumped** is a text-graphical roguelike about understanding,
editing, and eventually rewriting the rules of the thing trying to kill you. It's about love, family, CYBERSECURITY EXPLOITS, and the beauty of coding macros to do repetitive tacks.

CoreDumped is built on the **Xlyph** engine, a reusable framework for
Glyph-powered (a custom LISP) roguelikes. The engine provides the ECS, map, rules registry, rendering, and Glyph language runtime. The game and its levels are built on top. This engine itself is currently built on top of [bracket-lib](https://github.com/amethyst/bracket-lib) (formerly RLTK).

## Play

```bash
cargo run -p xlyph-tui
```

Move with arrow keys / hjkl. Descend stairs. Inspect enemies. Open the console
with backtick. Submit code with Enter.

## Why

Every roguelike has rules, but most hide them behind source code or wiki pages.
CoreDumped puts them on screen and lets you poke at them.

The long bet: if the game shows you its moving parts, the mystery shifts from
"how does this work" to "what can I make it do." The inspector and console are actually the core interface, not just a hack. The dungeon is literally
a Lisp runtime with graphics.

### What works now

- ~18 hand-crafted levels (17 narrative + procedural fallback)
- Turn-based movement, pathing enemies, directional flashlight
- Inspector panel showing real Glyph source for every AI rule
- Working Glyph console — eval expressions, inspect state, bind keys
- Custom ECS (entity-component-system), no external dependency
- Deterministic turn model: `ActionCost::Tick` = player + enemies advance;
  `ActionCost::Free` = UI only
- Playbook system: drop `.glyph` files in `~/.coredumped/playbooks/current/`,
  they load on game start
- Save/load (auto-save on quit, manual slots)

### What doesn't work yet

- No full procedural generation (fixed maps, but 18 is enough for a run)
- No full FoV / fog of war (flashlight serves to indicate direction)
- No inventory beyond held keys and special items
- Half-finished Glyph standard library (enough for enemies, not much else)
- Not balanced. At all.

## Architecture

**Engine/game split**: The **Xlyph** engine (ECS, map, rules registry, Glyph
runtime, event log) is the framework. **CoreDumped** is the game built on it —
levels, world state, save system, player profile. Same repo, separate concerns.
You could build different levels on the same engine.

**ECS**: Custom in-house, no dependency. `EntityId` is a stable handle.
Component stores are `BTreeMap<EntityId, T>`. Marker sets for alive/enemy-ai.
No systems abstraction — game logic reads/writes ECS directly. Keeps the
codebase small and auditable (~40 source files total).

**Turn model**:

```rust
pub enum ActionCost { Free, Tick, Quit }
```

Every gameplay action costs a tick. Every tick advances enemies once. UI actions
(inspector, console, typing) are free. This invariant is tested.

**Glyph embedding**: `BuiltinFn` takes `(&[Value], &Env, &SandboxOptions, &mut World)`.
Game builtins (print, inspect, toggle console) access World directly. The same
eval path runs enemy AI rules, console expressions, and keybindings. No FFI,
no IPC, no scripting bridge — just Rust functions registered in the environment.

**Levels are callbacks**: Each depth (0–17) is a function that receives `&mut World`
and places entities, sets terrain, configures wizard dialogue. Depth 18+ falls
through to a procedural builder.

## Controls

| Key | Action |
| --- | --- |
| Arrow keys / `h j k l` | Move / bump |
| `.` | Wait one tick |
| `i` | Toggle inspector |
| <code>\`</code> | Toggle console |
| `Enter` | Submit console expression |
| `Esc` | Close overlay / cancel quit |
| `q` | Quit |

## Development

```bash
cargo test        # 122 tests, pure game logic (no renderer)
cargo fmt         # single crate, no config
cargo check       # fast feedback
```

The most important tests verify the turn invariant — that gameplay costs ticks,
UI doesn't, and enemies advance exactly once per tick.

## Status

Beta. Playable from depth 0 to depth 17 (the Core). Has an ending. Expect
rough edges, missing content, and things that kill you without explanation.

The engine is minimal and intentional — about 13k lines of Rust, half of which
is the Glyph interpreter. If the project stops here, the codebase is small
enough to learn from in an afternoon.

## License

MIT
