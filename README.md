# CoreDumped

![](./coredumped_logo.png)

*A roguelike where the enemies run on Lisp you can read, edit, and break.*

**[Play in Browser](https://bryanhu.com/coredumped/)** | [Install](#install)

---

Before thee lies a dungeon of rules, memories, slimes, and suspiciously readable
machinery. **CoreDumped** is a text-graphical roguelike about understanding,
editing, and eventually rewriting the rules of the thing trying to kill you.

It's also a tragic game about love, family, CYBERSECURITY EXPLOITS, and the beauty of coding macros to do repetitive tasks. If you're impatient, the lore is documented [here](./level-design.md).

## Install

### Browser (Recommended)

**[Play now at bryanhu.com/coredumped](https://bryanhu.com/coredumped/)**

No installation required. Works in any modern browser with WebAssembly support.

### Terminal (Native)

```bash
# Clone and run
git clone https://github.com/ThatXliner/coredumped.git
cd coredumped
cargo run -p coredumped-tui

# Or install globally
cargo install --path tui
coredumped
```

Requires [Rust](https://rustup.rs/). Native build has better performance and save/load support.

## Controls

| Key | Action |
| --- | --- |
| Arrow keys / `h j k l` | Move / bump attack |
| `.` | Wait one tick |
| `i` | Toggle inspector (view enemy AI rules) |
| `m` | Toggle collected memories |
| <code>\`</code> | Toggle console |
| `Enter` | Submit console expression |
| `PageUp` / `PageDown` / scroll | Scroll panels |
| `Esc` | Close overlay / cancel |
| `q` | Quit |

## What makes it different

Every roguelike has rules, but most hide them behind source code or wiki pages.
CoreDumped puts them on screen and lets you poke at them.

- **Inspector panel** shows real Glyph (custom Lisp) source for every AI rule
- **Console** lets you eval expressions, inspect state, bind keys
- **The dungeon is a Lisp runtime** with graphics attached

The long bet: if the game shows you its moving parts, the mystery shifts
from "how does this work" to "what can I make it do."

## Features

- 18 hand-crafted levels with narrative
- Turn-based movement, pathing enemies, directional flashlight
- Working Glyph console and inspector
- Custom ECS, no external dependencies
- Deterministic turn model (every action costs a tick, UI is free)
- Playbook system: drop `.glyph` files in `~/.xlyph/playbooks/current/`
- Save/load (native only)

## Architecture

Three crates:

| Crate | Purpose |
|-------|---------|
| `core/` | Platform-agnostic game engine: ECS, map, rules, Glyph runtime |
| `tui/` | Terminal frontend (crossterm) |
| `web-frontend/` | Browser frontend (WASM + xterm.js) |

```rust
pub enum ActionCost { Free, Tick, Quit }
```

Every gameplay action costs a tick; UI actions are free. This invariant is tested.

## Documentation

- [PROJECT.md](./PROJECT.md) — repo layout, runtime flow, contributor workflow
- [glyph-reference.md](./glyph-reference.md) — Glyph language tour
- [language-spec.md](./language-spec.md) — Glyph semantics
- [level-design.md](./level-design.md) — campaign and narrative notes
- [game-architecture.md](./game-architecture.md) — design vision

## Development

```bash
cargo test          # game logic + language tests
cargo check         # fast feedback
cargo fmt           # format

# Build web version locally
cd web-frontend
./web-assets/build.sh
cd web-assets && python3 -m http.server 8080
```

## Status

Beta. Playable from depth 0 to 17 with an ending. Expect rough edges and things that kill you without explanation.

~17k lines of Rust. Small enough to learn from in an afternoon.

## License

MIT
