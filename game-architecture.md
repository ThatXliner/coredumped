# Self-Modifying Roguelike Architecture

Status: concept draft

This document describes a game architecture for a roguelike whose mechanics are represented as inspectable code. Some authored moments may let the player patch a narrow piece of that code, but the broader fantasy is spelunking a live codebase that happens to be the dungeon.

The central promise is simple: the code the player reads is the code the dungeon obeys. If a level grants a rare write opportunity, that change is real too.

## Product Pillars

- The code is real.
- The world is deterministic and replayable.
- Rule changes, when present, are rare, inspectable, diffable, and revertible.
- Power is constrained by capabilities, not by fake syntax barriers.
- Natural language can draft allowed code, but canonical code is authoritative.
- Broken authored modifications should produce interesting failure where possible.
- The system should feel like hacking a live engine, not solving programming exercises.

## High-Level Architecture

```text
              Player Input
                   |
                   v
        +----------------------+
        | TUI / Play Surface   |
        +----------------------+
          |       |        |
          v       v        v
       Map     Inspect   Console
          |       |        |
          v       v        v
        +----------------------+
        | Glyph Lisp Runtime   |
        +----------------------+
          |       |        |
          v       v        v
     Rule VM   Patch VM   Query VM
          |       |        |
          v       v        v
        +----------------------+
        | Simulation Kernel    |
        +----------------------+
          |       |        |
          v       v        v
        ECS   Event Log   Rule Registry
          |       |        |
          v       v        v
        Save System / Replay / Debug Tools
```

## Core Subsystems

### TUI And Play Surface

The primary interface should be a terminal UI, not a naked REPL. The player should inhabit a graphical roguelike map with spatial movement, encounters, menus, and diegetic tools, while the Lisp console appears as one powerful surface among several.

Think closer to Undertale's readable, expressive 2D staging than to a shell prompt: a compact map, character sprites or glyphs, room framing, dialog boxes, status panels, and reactive inspection panes. The codebase fantasy still matters, but the moment-to-moment game should feel embodied.

Core TUI regions:

- map viewport with tile glyphs, entities, effects, targeting cursors, and room transitions
- player status, inventory, capabilities, instability, and active overlays
- event log for combat, faults, traces, and authored messages
- inspection pane for source, entity metadata, rule metadata, and disassembly
- contextual action menu for spells, artifacts, tools, and exploits
- optional console pane for queries, macros, and rare write contexts

Input should route through the TUI first. Movement, interaction, targeting, inventory, and inspection are normal controls. The console is opened by tools such as debug shrines, forbidden terminals, signed archives, or late-game root shells.

Actions should declare whether they spend a roguelike tick. Movement, attacks, item use, spell execution, exploit triggering, waiting, and other world-affecting intents advance the simulation: enemies get their AI step, environmental rules tick, scheduled effects age, and the event log records the turn. Pure interface actions such as opening the console, browsing source, moving an inspection cursor, viewing metadata, or drafting code do not spend time.

Code execution is split by effect:

- read-only queries, source inspection, macro expansion previews, diffs, and dry-run analysis are free UI/debug actions
- accepted spells, rituals, exploit payloads, patch activations, and any code that writes world state spend a tick or run inside a special authored transaction
- long-running or expensive analysis may consume an explicit in-game resource, but should not secretly advance enemy turns unless the action says it does

The rendering layer is allowed to be expressive, but it must remain a view over canonical simulation state. It can animate, frame, highlight, and summarize; it cannot create hidden game truth outside the event log and world state.

### Language Host Layer

The game embeds Glyph as a general homoiconic runtime. Game-specific concepts such as spells, rules, enemies, artifacts, levels, exploits, balance, and progression live in the host layer, not in the language itself.

The host maps generic language mechanisms onto game concepts:

- Glyph functions become scripts, predicates, helpers, and tools.
- Glyph data becomes entity state, rule metadata, patch objects, and logs.
- Glyph structural rewrites become patch payloads only inside authorized game contexts.
- Glyph capabilities become game permissions such as reading rules, disassembling artifacts, spawning entities, or patching scoped overlays.
- Glyph faults become simulation faults, failed rituals, disabled patches, or rollback events.

This split keeps the language reusable and small while letting the game be opinionated, authored, and progression-gated.

### Simulation Kernel

The kernel owns turn order, phase execution, transactions, rollback, seeded randomness, and event emission.

Responsibilities:

- Advance deterministic game phases.
- Execute registered rules in stable order.
- Provide transactional world writes.
- Enforce phase-level fault policy.
- Record all meaningful events in an append-only log.
- Prevent direct mutation outside approved write APIs.

The kernel should be small and boring. The rules should be where the game becomes strange.

### Entity Component Store

The world state is represented as structured data, preferably ECS-like.

Example entity:

```lisp
{:id 481
 :kind :slime
 :pos {:x 12 :y 8}
 :hp 6
 :max-hp 6
 :material :gel
 :tags #{:living :wet}
 :status #{:poisoned}}
```

Rules query this store through deterministic APIs.

```lisp
(entities {:tags #{:living}
           :within {:center player.pos :radius 4}})
```

### Rule Registry

The rule registry stores executable game rules as canonical AST plus metadata.

Rule fields:

```lisp
{:id :rules/fire/spread
 :name 'fire-spread
 :module :rules/fire
 :phase :tick
 :priority 40
 :query [:tile/burning]
 :caps #{:tile/read :tile/write :random/use}
 :source '(defrule ...)
 :expanded '(fn ...)
 :hash "..."
 :version 17
 :enabled true
 :origin {:kind :core}}
```

Rule origins:

- `:core`
- `:generated`
- `:player-patch`
- `:artifact`
- `:enemy`
- `:quest`
- `:corruption`
- `:llm-draft`

Rules execute in this order:

1. Phase order.
2. Rule priority.
3. Stable rule id.
4. Patch overlay order.

### Rule Overlay System

The engine can model rule overlays as first-class objects so authored content, procedural floors, enemies, artifacts, and occasional player exploits all use the same machinery.

Player-authored patches are not a core verb to emphasize early. They are late-game, level-gated, or special-case design beats. Early and mid-game players may cast spells, inspect metadata, read selected source, write legal macros, and run queries. They should not casually edit world rules from a menu.

```lisp
{:id :patches/player/no-self-fire
 :target :rules/status/burning
 :operation '(wrap ...)
 :author :player
 :scope :floor
 :caps #{:rules/patch-local}
 :created-at-turn 913
 :expires-at-turn nil
 :diff [...]
 :status :active}
```

Overlay scopes:

- `:expression`
- `:item`
- `:entity`
- `:room`
- `:floor`
- `:biome`
- `:run`
- `:timeline`
- `:global`

Most player modifications, if a level offers them, should begin scoped and tied to a concrete level-authored access route. Global mutation is rare late-game power.

### Level-Gated Exploits

Write access is primarily level design, not the main loop.

A level, artifact, enemy, vault, or shrine can contain an exploitable surface. The player may not even know it exists until they have found the relevant code-reading or disassembly tool.

Example:

```lisp
{:id :magic-rock/overflow-17
 :kind :buffer-overflow
 :target :artifacts/late-game-magic-rock
 :revealed-by :tools/disassembler-v2
 :trigger :on-impact
 :write-scope :room
 :grants #{:rules/patch-local}
 :constraints {:max-bytes 96
               :phase :on-impact
               :alignment :word}
 :consumes [:item/charged-quartz]}
```

An exploit should require several things:

- the player has a tool capable of seeing it
- the target is present in the level
- the player understands or experiments with the trigger
- the exploit grants only narrow capabilities
- the payload fits the exploit constraints
- the resulting patch survives validation

Exploit classes:

- buffer overflow in a late-game magic rock
- macro-expansion leak in a spellbook
- stale pointer in a door law
- signed-module downgrade in a shrine
- rule priority inversion in a cursed courtroom
- unsafe serialization of a relic
- unchecked predicate in enemy AI
- race in a time scheduler

This makes rule mutation feel discovered and situated when it appears. The player is not choosing "patch rule" from an abstract menu; they are using a particular weakness in a particular thing in a particular place.

### Overlay Model

Avoid destructive mutation of core rules. Use overlays.

```text
core rule
  -> generated run layer
  -> floor layer
  -> artifact layer
  -> player patch layer
  -> temporary effect layer
```

The effective rule is built from layers. This makes inspection, rollback, replay, and conflict resolution tractable.

### Capability System

Every executable context has capabilities.

Examples:

- Player debug shrine: `:rules/read`, `:debug/trace`
- Disassembler: `:rules/disassemble`, `:exploit/detect`
- Exploit context: `:exploit/execute`, `:rules/patch-local`
- Minor spell: `:entity/read`, `:status/write`
- Artifact patch: `:rules/patch-room`, `:tile/write`
- Enemy exploit: `:rules/patch-self`, `:spawn/create`
- Endgame root shell: `:rules/patch-global`, `:time/schedule`

Capabilities constrain what code may read, write, spawn, patch, or observe.

Capability checks happen:

- during compilation/static analysis
- before rule activation
- during runtime API calls
- when accepting generated patches inside an authorized exploit or root context

### Transaction System

Each phase runs in a transaction.

```text
begin transaction
  execute rules
  collect writes
  validate invariants
  commit or roll back
emit event summary
```

Transactions make dangerous code survivable. A bad rule can fail without corrupting the whole save.

Transaction outputs:

- world diff
- emitted events
- faults
- spawned scheduled tasks
- rule registry changes
- balance telemetry

### Fault Policy

Faults should be values first.

```lisp
{:fault :recursion-limit
 :rule :rules/slime/split
 :patch :patches/player/slime-gold
 :turn 301}
```

Fault policies by context:

- Core rule fault: rollback phase and enter safe mode.
- Player patch fault: disable patch and emit paradox effect.
- Enemy patch fault: disable enemy exploit.
- Temporary spell fault: fizzle spell and charge instability.
- Global rule fault: rollback to last signed snapshot.

### Watchdog

The runtime needs guardrails:

- instruction budget
- recursion depth
- allocation limit
- entity spawn budget
- event emission budget
- patch depth limit
- max rule expansion size
- max generated AST size

Budget exhaustion creates a structured fault.

## Determinism And Replay

A run is replayable from:

- game version
- world seed
- content seed
- player input log
- accepted patches as canonical AST
- accepted natural-language drafts as canonical AST
- LLM model output snapshot, if used before canonical acceptance

Never depend on live LLM calls during replay. Once a draft is accepted, store the generated canonical code and metadata.

## Natural Language Spell Drafting

Natural language is an assistant, not authority.

Flow:

```text
player describes intent
  -> LLM proposes currently legal structured code
  -> parser validates syntax
  -> static analyzer computes capabilities and budget
  -> deterministic compiler lowers to executable form
  -> player reviews diff
  -> accepted form enters event log
```

Example:

```text
freeze enemies near me and make metal brittle
```

Draft:

```lisp
(spell frost-stress
  {:target (area :center caster :radius 3)}
  (for e (entities-in target)
    (when [e.faction = :enemy]
      (apply-status e :frozen 2))
    (when [e.material = :metal]
      (apply-status e :brittle 12))))
```

The analyzer may revise:

```text
radius 3 exceeds current area budget.
Suggested radius: 2.
Metal brittleness duration capped at 8 turns.
```

The player should see the resulting code, not only prose.

By default, LLM drafting should produce spells, rituals, filters, queries, or legal macro expansions. It should not propose raw world-rule patches until the player has already opened a write context through a discovered exploit or late-game root mechanism.

Inside an exploit, the LLM can assist with payload construction, but the exploit constraints are hard limits. The assistant can help write the overflow payload; it cannot invent write access where none exists.

## Balance System

Balance should be based on measurable capabilities and runtime cost.

Static features:

- target count
- area radius
- duration
- damage/healing magnitude
- status severity
- phase frequency
- rule scope
- permanence
- patch depth
- spawn potential
- information access
- randomness use

Dynamic telemetry:

- actual affected entities
- damage prevented
- turns saved
- spawned objects
- rule execution time
- fault count
- rollback count

The balance system computes:

- mana cost
- instability cost
- cooldown
- required permissions
- failure consequences
- rarity
- scope limit

Do not hide this from the player. Experienced programmers will enjoy seeing why a patch is expensive.

## Game Loop

### Action Time Cost

Every accepted player intent resolves to a time cost before the kernel advances.

```lisp
{:intent :move
 :dir :east
 :cost :tick}

{:intent :inspect-rule
 :rule :rules/fire/spread
 :cost :free}

{:intent :activate-patch
 :patch :patches/player/no-self-fire
 :cost :special-transaction}
```

Time-cost classes:

- `:free` updates UI state only and does not run enemy AI or environmental ticks
- `:tick` advances one roguelike turn after the player action resolves
- `:multi-tick` advances a declared number of ticks for waits, channeling, or slow tools
- `:special-transaction` runs an authored sequence with explicit rules for whether enemies, hazards, or timers advance

The important invariant is that time only passes from accepted simulation intents. Opening the REPL, reading code, moving through menus, previewing a macro expansion, or looking at a diff should not by itself advance the dungeon.

### Normal Turn

```text
read player intent from TUI
classify time cost
if free: update UI state and render
begin turn transaction
run :before-player rules
apply player action
run :after-player rules
run enemy AI phases
run environmental tick rules
validate invariants
commit world diff
render
```

### Optional Patch Moment

```text
unlock code-reading or disassembly tool
discover exploitable writable surface in the level
trigger exploit context
open target rule/module through that context
player edits or generates allowed payload
parse to canonical form
macroexpand
analyze capabilities
check exploit constraints
estimate balance cost
show diff
player accepts
write patch event
activate overlay
resume simulation
```

This is a special authored sequence for levels that want it, not the expected shape of every turn or every build milestone.

### Future Level Note: Magic Rock Overflow

A future level could include a late-game magic rock with a discoverable buffer overflow. After unlocking a code-reading tool, finding the rock, disassembling it, discovering the overflow, and triggering the exploit, the player could install a room-scoped patch, see the actual diff, activate it, and watch the real simulation obey the modified rule.

This is just a small design note for one possible level though.

## Invariants

The engine may have signed invariants that protect save integrity and replayability.

Examples:

- entity ids are unique
- positions must be finite grid coordinates
- every living entity has hp
- committed events are append-only
- replay cannot call external services
- rule registry hashes must match event log

The player may eventually gain powers that bend gameplay rules, but save/replay invariants should stay protected unless the game intentionally enters an endgame unsafe mode.

## World Systems Worth Exposing

Good rule modules for player hacking:

- `rules/combat`
- `rules/damage`
- `rules/fire`
- `rules/water`
- `rules/status`
- `rules/materials`
- `rules/fov`
- `rules/sound`
- `rules/smell`
- `rules/ai/hunt`
- `rules/ai/flee`
- `rules/loot`
- `rules/spawn`
- `rules/death`
- `rules/hunger`
- `rules/time`
- `rules/doors`
- `rules/chasm`
- `rules/lighting`

Avoid exposing only cosmetic scripts. The fun comes from real systems.

## Debug And Inspection UX

The player should have tools that feel like game objects and real developer tools.

Required tools:

- source viewer
- structural editor
- canonical view
- surface view
- rule search
- call trace
- event log
- patch diff
- macroexpand
- capability viewer
- disassembler
- exploit scanner
- payload size meter
- rollback preview
- profiler
- dependency graph

Diegetic examples:

- debug shrine
- forbidden REPL
- signed core archive
- corrupted module vault
- living stack trace
- runtime panic altar

Keep the UX efficient. The audience is comfortable with power tools.

## Enemy And Dungeon Self-Modification

The player should not be the only hacker.

Enemy examples:

- Slime patches its split rule when damaged.
- Lich shadows death handling for itself.
- Mimic spoofs item metadata.
- Fungus installs a room-scoped growth scheduler.
- Golem rewrites material resistance rules for stone.
- Compiler wraith attacks macros and expands traps.

Dungeon examples:

- Generated floors ship with strange local rule overlays.
- Cursed rooms have altered physics.
- Biomes patch status interactions.
- Late-game areas mutate rule priorities.
- Bosses protect signed modules.

## Procedural Generation

Procedural generation should create:

- terrain
- entities
- artifacts
- local rule overlays
- spell libraries
- documentation fragments
- misleading comments
- patch conflicts
- capability keys
- quests based on actual mechanics

Generated rules must pass the same validation pipeline as player patches.

Generation pipeline:

```text
seed
  -> theme grammar
  -> candidate rule templates
  -> capability/budget validation
  -> simulation smoke tests
  -> signed floor overlay
```

Smoke tests should verify that a generated floor is not instantly lethal, unwinnable, or explosively divergent unless that is a deliberate rare event.

## Save Format

Save files should store:

- engine version
- world seed
- current world state
- event log
- accepted canonical patches
- patch overlays
- rule registry hashes
- RNG states
- player command history
- generated content metadata

Natural language prompts may be stored for flavor and auditing, but canonical generated code is authoritative.

## Security Boundary

Gameplay code must not have direct host access.

Forbidden from game code:

- filesystem access
- network access
- OS process access
- wall-clock randomness
- unrestricted reflection into host runtime
- unsafe native calls

Allowed only through explicit engine APIs:

- world reads
- world writes
- event emission
- rule inspection
- scoped patching
- seeded randomness
- tracing

The player fantasy is root access to the dungeon, not to the user's computer.

## Implementation Strategy

### Phase 1: Deterministic Core

- Build ECS/world state.
- Build turn kernel.
- Build a basic TUI with map viewport, status panel, event log, and inspect pane.
- Build canonical Lisp reader and evaluator.
- Implement phase rules as Lisp data.
- Add seeded RNG.
- Add event log and replay.

### Phase 2: Rule Inspection

- Add source viewer.
- Add canonical printer.
- Add rule registry.
- Add tracing and macroexpand.
- Move several real mechanics into rules.

### Phase 3: Disassembly And Authored Exploits

- Add opaque compiled artifacts.
- Add disassembler tools.
- Optionally add exploit objects placed by level design.
- Add vulnerability detection.
- Add read-gated hidden rule surfaces.

### Phase 4: Rule Overlays

- Add structural patch operations.
- Add overlays.
- Add capability checks.
- Add diffs and rollback.
- Add scoped authored mutations.
- Keep player-authored patching behind explicit exploit or root contexts if it appears at all.

### Phase 5: Procedural Rule Generation

- Generate floor-scoped rule overlays.
- Add smoke-test simulation.
- Add balance scoring.
- Add weird documentation fragments.

### Phase 6: Natural Language Drafting

- Add LLM-assisted draft generation.
- Require structured output.
- Validate through deterministic compiler.
- Store accepted canonical forms.
- Expose review diff before activation.
- Restrict patch drafting to authorized exploit/root contexts.

## Minimum Viable Prototype

The smallest prototype that proves the concept:

- Grid roguelike world.
- TUI map with movement, entities, status, event log, and inspection pane.
- Tick scheduler where movement advances enemies one step, while inspection and console opening are free actions.
- Entities with hp, position, tags, and status.
- Three real rule modules: fire, death, enemy AI.
- Contextual Lisp console for queries and debug-shrine interactions.
- Source viewer for active rules.
- One locked inspection or disassembly tool.
- Event log with rollback.
- One enemy that modifies its own rule.
- One generated floor with a local rule overlay.

Success criterion:

The player can read and reason about real dungeon rules, then see the dungeon obey those rules through normal play, enemy behavior, generated overlays, and replayable event logs. Player-authored patching can wait for a later level experiment.
