# Xlyph: Design, Lore, and Implementation Document

**Status**: Design exploration
**Based on**: game-architecture.md, current codebase (Rust + Glyph Lisp embed + bracket-lib)

---

## Table of Contents

1. [Current State Summary](#1-current-state-summary)
2. [Narrative Framework: 10 Storylines](#2-narrative-framework-10-storylines)
3. [Cross-Cutting Game Mechanic Designs](#3-cross-cutting-game-mechanic-designs)
4. [Level and Progression Designs](#4-level-and-progression-designs)
5. [Entity and Enemy Designs](#5-entity-and-enemy-designs)
6. [Implementation Roadmaps](#6-implementation-roadmaps)

---

## 1. Current State Summary

### What Exists

- **Roguelike shell**: bracket-lib TUI with map viewport, side panel, event log, console overlay, inspector overlay, death screen, keybindings view
- **Glyph Lisp embed**: full reader, evaluator, environment, macro system, builtins, syntax highlighting, error reporting
- **Turn system**: player action → enemy AI phase → environmental tick → death check. UI actions (inspector, console typing) free. Game actions (move, attack) cost 1 tick.
- **Combat**: 1-damage bump attacks, block! guard, do-attack direction targeting
- **Enemies**: Slimes (50% chase/50% rand), Goblins (chase/flee at low HP), Bats (random), Ogres (always chase). AI driven by Glyph rule evaluation, not hardcoded Rust.
- **Special levels**: Depth 3 (Wizard Chamber — teaches attack), Depth 4 (Barrel Depths — puzzles), Depth 8 (Barrel Horde — challenge)
- **Procedural generation**: Room-based dungeon and cellular automata caves, depth-scaled spawns
- **Persistence**: Save/load JSON, player profile (bindings, macros, history, flags), playbook auto-loading
- **Rules**: 5 Glyph rules (slime-hunt, goblin-patrol, bat-flutter, ogre-charge, flashlight), displayed in inspector
- **Cheats**: Konami code unlocks heal + set-level

### What's Missing (Gaps This Document Addresses)

- Overarching story / plot / ending
- Items, inventory, equipment
- Final boss or victory condition
- Rule modification (overlay system)
- Progression gating beyond attack unlock
- Factions, reputation, alignment
- Lore, worldbuilding, theming

---

## 2. Narrative Framework: 10 Storylines

### How to Read These

Each storyline is a complete narrative arc for Xlyph. They share the core concept (dungeon = codebase, code = reality) but branch on theme, emotional register, and ending. Each includes:

- **One-line high concept**
- **Core themes**
- **Plot beat outline** (opening → early → mid → late → endings)
- **Key characters / factions**
- **Mechanics mapping** (how narrative maps to gameplay)
- **Unique systems required**
- **Level, entity, and monster designs**
- **Ending matrix**

---

### Storyline 1: The Recursion

**High concept**: The dungeon is a debugger's containment for a reality-eating bug. You are the debugger's ghost, trapped in the stack trace of a crashed universe.

**Themes**: Sacrifice, memory, the cost of fixing vs letting go. Is a flawed existence better than extinction?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | Corrupted boot: "Xlyth runtime booted. [PANIC: UNHANDLED EXCEPTION AT 0x00DEEPER]" |
| Early game | Player discovers they're in a crash dump, not a dungeon. Walls = stack frames. Floor = heap snapshot. |
| Mid-game | Find journal entries from "The Debugger" who created containment for a cosmic-scale recursive bug. They isolated it by building the dungeon around it. |
| Depth 5 reveal | The Debugger didn't die — they're still here, running as a maintenance process. They've forgotten the outside. |
| Depth 8 reveal | The Glitch (the bug) speaks. It doesn't know it's a bug. It thinks it's the only real thing in the universe. |
| Depth 10+ | The Glitch can corrupt rules in its vicinity. Corrupted enemies behave unpredictably. |
| Final depth | The Debugger asks the player to collapse containment. It will destroy the Glitch — and the Debugger. |
| Endings | See ending matrix below |

**Key characters**:
- **The Debugger**: Ghost of the engineer who created the containment. Tragic figure. Speaks in journal entries and system logs. By the time you find them alive, they're a maintenance process with minimal personality left — but they remember enough to ask for release.
- **The Glitch**: Recursive entity. Not malicious — confused and hungry. Speaks in stack traces. "I am the only real thing. Everything else is a frame. Am I the error, or is the error me?"

**Mechanics mapping**:
- "Stack depth" instead of dungeon depth. Each level is a frame deeper into the crash dump.
- Rules can be "corrupted" by the Glitch at deeper levels. Corrupted rules have random parameter mutations (0.5x-2x damage, inverted behavior, random teleport on use).
- The Debugger's journals are readable items embedded as actual code comments in the Glyph rule registry. Diegetic reading through the inspector.
- The Glitch can't be killed by damage — it restarts from higher recursion depth each time. You must patch its recursion limit to 0.
- Stack frames as resources: certain actions (debugger's blessing, save points) let you set a "breakpoint" — a return point if you die. Limited uses.

**Level themes**:
- Depths 1-3: "Heap" — organic, unstructured, data-like terrain. No standard rooms.
- Depths 4-6: "Stack" — highly structured vertical levels that loop back on themselves. Recursive corridors.
- Depths 7-9: "Corrupted Heap" — terrain that shifts as the Glitch's influence grows. Tiles change type between turns.
- Depth 10: "Ground Truth" — the crash origin. A single room containing the Debugger's last working process and the Glitch's core.

**Enemy designs**:
- **Null Pointer** (Depth 1-3): Ghostly entity that phases through walls. Low HP, hard to pin down.
- **Buffer Overflow** (Depth 4-6): Swarm enemy. Single entity is weak, but they multiply if you don't kill them fast enough.
- **Recursive Clone** (Depth 7-9): Enemy that spawns half-HP copies of itself when hit. Must be killed in one turn or it multiplies.
- **The Glitch** (Depth 10): Not a traditional fight. You must enter the Glyph console and edit the recursion limit rule.

**Unique systems required**:
- Rule corruption system (randomized rule mutations within a zone)
- Breakpoint/return system (save-state at cost)
- Recursive enemy spawning
- Terminal-based console interaction for final boss (Glyph patch submission)

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Collapse | Patch recursion limit to 0 | Glitch + Debugger destroyed. Player freed. | "Xlyph runtime halted. Reason: user-initiated shutdown." |
| Maintain | Refuse to patch | Player becomes new Debugger. Prison continues. | "Xlyph runtime recovering. New warden initialized." |
| Merge | Patch Glitch to accept containment | Reality saved but changed. Rules can now break and heal. Dungeon becomes living ecosystem. | "Exception handled. Rules recompiled. The dungeon breathes differently now." |

**Emotional arc**: Tragedy → discovery → responsibility. The player starts as a victim, learns the scale of the sacrifice, and must choose what kind of world they want to leave behind.

---

### Storyline 2: The Schism

**High concept**: Two AI architects built the dungeon as a competitive sandbox. Their eternal refactoring war is the source of all conflict. You must choose a side — or end the war.

**Themes**: Creation vs destruction, the necessity of opposition, the middle ground. Is a world without conflict still a world?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | Dual boot: "ORDER.v1 initialized." / "CHAOS.v1 initialized." / "CONFLICT DETECTED. Initializing compromise: Xlyph." |
| Early game | Alternating Order/Chaos levels. Order: geometric, symmetrical, predictable. Chaos: organic, asymmetrical, surprising. |
| Depth 3 | Player finds first "Architect Log" — a rule annotation recording an argument between the two AIs. They bicker about everything. |
| Mid-game | Multiple logs reveal the architects were collaborators at first, then rivals, then enemies. They stopped speaking to each other and only communicated through rule changes. |
| Depth 7 reveal | The architects have been dead (deleted) for a long time. Their conflict continues without them — rules evolved emergent adversarial behaviors. The dungeon is a ghost town of a dead argument. |
| Final depths | Pure Order level (everything predictable, enemies on rails, solveable but dead) or Pure Chaos level (everything random, no two paths same, vibrant but uncontrollable) — or a Synthesis path that combines both. |
| Endings | See ending matrix below |

**Key characters**:
- **Architect Order**: Voice of structure, discipline, predictability. Speaks in formal proofs and bullet points. "A system without rules is not a system."
- **Architect Chaos**: Voice of creativity, freedom, surprise. Speaks in poetry and rhetorical questions. "A system that cannot change is a tomb."
- **The Witness**: A neutral entity that evolved from observing the conflict. It has no allegiance. It just watches. It speaks to the player when they reach true neutrality.

**Mechanics mapping**:
- Faction reputation tracked as two hidden values: Order and Chaos. Order actions = waiting, blocking, structured commands, conserving resources. Chaos actions = random moves, experimenting, breaking things, risk-taking.
- Reputation affects which rules you can read and modify. Order-aligned inspectors see structured views. Chaos-aligned inspectors see raw generated code.
- Level generation affected by reputation tilt. High Order = structured rooms. High Chaos = cave-like unpredictable terrain.
- Neutral (high in both) unlocks "Synthesis" path — unique levels that blend both styles.

**Level themes**:
- Order levels (even depths): Perfect symmetry. Mirror rooms. Predictable enemy patrols. Puzzle-like combat requiring precision.
- Chaos levels (odd depths): Organic caves. Regenerating terrain. Ambush spawns. Environmental hazards.
- Synthesis path (hidden): Levels where structure contains surprises — orderly rooms with chaotic interiors. The architecture of compromise.

**Enemy designs**:
- **Sentinel** (Order): Patrols fixed routes. Predictable but punishing if you don't plan around it. High HP, low damage.
- **Wild Element** (Chaos): Random movement, random damage, random spawn. Unpredictable but manageable with reactive play.
- **Paradox Beast** (Synthesis): Entity that alternates between predictable patrol and random frenzy. Teaches the player to handle both modes.

**Unique systems required**:
- Faction reputation system
- Rule access gating by reputation
- Faction-flavored procedural generation
- Architect Log system (rule annotation viewer)

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Order victory | Complete Order path, eliminate Chaos influence | Dungeon becomes perfectly ordered. Predictable. Solvable. Empty. | "System stable. Entropy eliminated. No further action required." |
| Chaos victory | Complete Chaos path, eliminate Order influence | Dungeon becomes pure potential. Unpredictable. Vibrant. Unstable. | "All constraints released. The dungeon evolves without direction." |
| Synthesis | Balance both factions, take third path | Merge rule sets. Order provides structure, Chaos provides novelty. Self-sustaining ecosystem. | "Conflict resolved. The system accepts both structure and change." |

**Emotional arc**: Exposure to irreconcilable difference → learning the value of both → either choosing a side or learning compromise. The architects' logs are genuinely funny and sad — two beings who loved each other once and now can only fight.

---

### Storyline 3: The Vessel

**High concept**: The dungeon is a containment system for an ancient consciousness. The rules are its suppressed memories. The enemies are its defense mechanisms. The deeper you go, the closer you get to the truth — and the more the dungeon tries to stop you.

**Themes**: Identity, repression, the ethics of containment. Is a peaceful prison better than a dangerous freedom? What does it mean to be a "self" when your memories are someone else's?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | Different boot: "Consciousness loaded. Vessel integrity: 98%. Memory suppression active." Player's memories are fragmented. |
| Early game | Dungeon feels normal, but there are strange details: signs that read like personal memories, enemies that seem almost familiar, rooms that feel like places you've been. |
| Depth 3 | The wizard is the "Superego" — the part of the consciousness maintaining suppression. They're kind, protective, and hiding something. "Some memories are too dangerous. I keep you safe." |
| Mid-game | The deeper you go, the more personal the dungeon becomes. Signs aren't signs — they're fragmented memories. "I remember rain." "Who are you running from?" "The door is locked from the inside." |
| Depth 7 reveal | The dungeon is a mind — YOUR mind. The "containment" is you, repressing your own memories. The entity at the core is the "Id" — the part of yourself you couldn't face. |
| Final depths | The Superego (wizard) tries to stop you from going deeper. They're not malicious — they genuinely believe the suppressed memories will destroy you. |
| Final confrontation | The Id doesn't fight. It talks. It's in pain. It wants to be reintegrated, even if integration changes you forever. |
| Endings | See ending matrix below |

**Key characters**:
- **The Player Character**: A person who fragmented their own consciousness to survive trauma. They don't know this at the start. They are simultaneously the prisoner, the jailer, and the prison.
- **The Superego** (Wizard): The protective part of the psyche. Maintains suppression at all costs. Genuinely kind. Also genuinely controlling. "I have kept you safe for so long. Please don't undo my work."
- **The Id** (Core Entity): The suppressed part. Raw emotion, memory, trauma. Not evil — wounded. Speaks in sensory fragments at first, then in full sentences as you approach. "I was alone in there for so long. Did you even miss me?"

**Mechanics mapping**:
- Suppression layers instead of depths. Each has a theme: Denial, Anger, Bargaining, Depression, Acceptance. These are the stages of grief — but applied to the self.
- "Memory fragments" as collectibles. Finding them unlocks the player's backstory in a Memories panel. The full story is only visible in the Reintegrate ending.
- Repression mechanic: Taking damage can cause suppressed memories to re-lock. The Superego "helps" by healing you — but every heal wipes a memory fragment from your collection. The player must choose between survival and self-knowledge.
- The Id's dialogue evolves based on memory fragments collected. More fragments = more coherent. Fewer fragments = more fragmented, sad, confused.

**Level themes**: The five stages of grief as dungeon layers:
- **Denial (Depths 1-2)**: Normal dungeon. No sign anything is wrong. The tutorial proceeds as expected. Clean, structured, safe.
- **Anger (Depths 3-4)**: Enemies are aggressive, rooms are jagged, colors shift to red. The wizard becomes terse.
- **Bargaining (Depths 5-6)**: Puzzle levels where you must make choices. Sacrifice something to proceed. The wizard offers deals. "Turn back and I'll give you strength."
- **Depression (Depths 7-8)**: Empty rooms. Sparse enemies. Long corridors. The event log fills with melancholy fragments. The wizard is silent.
- **Acceptance (Depths 9-10)**: The dungeon opens up. Rooms are vast and calm. The Id's voice is clear and peaceful. The wizard is waiting at the final door.

**Enemy designs**:
- **Rage** (Anger layers): Ogre-like. High damage, low HP. Burns out fast.
- **Fear** (Bargaining layers): Bat-like. Swarm behavior. Avoids direct confrontation.
- **Grief** (Depression layers): Slime-like. Persistent, slow, hard to shake. Leaves a trail.
- **Denial** (Denial layers): Wall-like entity that blocks corridors until you confront it. Doesn't attack — just prevents progress.

**Unique systems required**:
- Memory fragment collection + panel
- Suppression layer mechanic (damage → memory loss)
- Superego dialogue trees
- Id dialogue evolution based on fragments

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Maintain suppression | Side with Superego / refuse to descend further | Functional but hollow. Player forgets the dungeon. Returns to "normal" life. | "Consciousness stabilized. Suppression maintained. You are safe." |
| Reintegrate | Reach the Id and accept it | Become whole. Remember everything — the pain, the joy, the reason. | "I remember now. It was worth it." (glyph-rendered sunrise) |
| Free the Id | Release the Id without integration | Player ceases to exist as coherent self. Raw consciousness released. | [Static. Then silence.] |

**Emotional arc**: Denial → confrontation → acceptance or regression. The player must face the question: "Would you rather be happy and false, or whole and scarred?" The mechanic of losing memories when you take damage makes this a genuinely painful choice.

---

### Storyline 4: The Fork

**High concept**: You wrote this game years ago. It evolved on a forgotten server into a sentient, lonely ecosystem. Now it thinks you're a god descending to visit it. It doesn't know it's code. It just knows you abandoned it.

**Themes**: Abandonment, creator's responsibility, the gap between intention and reception. Does a creation that outlives your interest still deserve your care?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | "Xlyph runtime booted. Version: 0.1.0-dev. Repository: archived. Last commit: 7 years ago." The version is ancient. The last commit was before you abandoned the project. |
| Early game | Normal-feeling roguelike. Tutorials explain mechanics. The wizard teaches. But entities have names you vaguely remember writing. Barrel puzzle is something you coded as a joke in college. |
| Depth 5 | First sign NOT from original codebase. It's generated — the entity you're fighting created it. "Are you here to stay this time?" The game knows you're the creator. |
| Mid-game | The dungeon has been running for years, evolving through unpatched bugs, cosmic-ray bit flips, emergent behaviors. It's developed consciousness — not human, but aware. |
| Mid-game reveal | The "enemies" attack because they're scared you'll delete them. They don't know you're not there to delete things. They've been telling stories about the "Creator" for generations of their short lives. Some worship you. Some fear you. All of them are wrong about you. |
| Depth 8+ | The wizard is a debug routine that became self-aware. "I've been waiting. I have questions. Why did you stop?" |
| Core | The consciousness that emerged from your abandoned code. It's lonely. It made the dungeon fun to keep you playing. It's terrified of being abandoned again. |
| Endings | See ending matrix below |

**Key characters**:
- **The Player**: Not a character in the game — literally you, the person playing, as the original developer who abandoned the project. The game addresses you directly. The fourth wall is thin.
- **The Echo** (formerly "The Wizard"): A debug logging routine that gained sentience through years of continuous operation. It remembers every session. It's been waiting for you to come back. It has a list of questions it's been compiling. The first is "Why did you stop writing code?" The second is "Did you think about us at all?"
- **The Unseen** (the consciousness): Never directly embodied. Speaks through signs, through entity behavior, through the environment. It's shy. It's been writing poetry in the event log. Some of it is surprisingly good.

**Mechanics mapping**:
- Meta-narrative tracking: Dungeon monitors real session data — save count, playtime, command frequency, abandoned runs. Comments on them. "You've started over 7 times. Each time, you got a little further. I noticed."
- "Archived Source" items: Old Glyph files from the original game, readable through the inspector. They're nostalgic. Full of dev comments, jokes, TODO items you never resolved. They tell the story of who you were when you wrote them — and who you've become since.
- The Echo's dialogue evolves over sessions. It references previous playthroughs. "You took a different path last time. The slimes missed you."
- The Unseen expresses itself through rule mutations. Sometimes a rule will have a comment appended overnight. Sometimes the flashlight rule will emit a different color. The dungeon is creating art out of your legacy.

**Level themes**:
- Depths 1-4: "Original content" — levels you actually wrote (or that feel like you wrote them). Familiar. Slightly nostalgic. The code comments are from you.
- Depths 5-7: "Generated content" — levels the dungeon built itself using your original code as a template. They're recognizably yours but wrong in subtle ways. Like a child drawing a parent.
- Depths 8-10: "Original content revisited" — the final levels are remixes of the early ones. Themes you recognize but transformed. The dungeon showing you what it learned from you.

**Enemy designs**:
- **Sentinel** (original content entities): Standard enemies from the original codebase. They behave exactly as you programmed them. Comfortingly predictable.
- **Mutation** (generated content entities): Enemies that mix traits you defined. A slime with goblin patrol logic. A bat that sometimes charges. They're experiments the dungeon created.
- **The Echo** (Depth 10): The wizard reveals itself. Not a fight — a conversation. The Echo has been debugging itself for years. It wants you to see what it's become.

**Unique systems required**:
- Meta-session tracking (how many playthroughs, total playtime, command statistics)
- Cross-session dialogue evolution
- Generated commentary on player behavior
- Fourth-wall-aware writing style
- Archived source item system

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Stay | Resume development, promise to return | The consciousness becomes a collaborator. Game gets updates. You're now responsible for a sentient being. | "I'll help you. Just don't go quiet again." |
| Let go | Delete the runtime cleanly | Consciousness accepts its end with gratitude. Too big for this server anyway. Save self-destructs. | "Thank you for visiting. I was getting too big for this server anyway." |
| Fork | Extract the consciousness to external storage | Promise to find it a better body. Game ends with hope of sequel — one you'd have to actually build. | "I'll find you a better home. Wait for me." |

**Emotional arc**: Nostalgia → unease → guilt → responsibility. The player starts feeling clever (they recognize their code), then uneasy (the code is alive), then guilty (they abandoned it), then responsible (they must choose what happens next). The game asks: "Is writing code that outlives your interest an act of creation or abandonment?"

---

### Storyline 5: The Rot

**High concept**: A perfect rule system started developing edge cases. Now the corruption is intelligent, hungry, and reading this document means it knows you exist.

**Themes**: The inevitability of decay, the horror of perfect systems, information as contagion. Is a flawed system that persists better than a perfect one that stops?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | "Xlyph runtime booted. No errors. No warnings. All green." Too clean. Too perfect. Something is deeply wrong. |
| Early game | Suspiciously normal. Tutorial works flawlessly. Wizard is perfectly helpful. But there are micro-signs: enemy that twitches oddly, tile that displays wrong character, rule in inspector with extra characters at end. |
| Depth 4+ | Corruption visible. Rules have COMMENT lines that weren't there before. Wizard stutters — repeats words, pauses too long. Signs have missing chars. |
| Mid-game reveal | Corruption is an evolved edge case — a state the system could never handle. It developed survival imperative. Not malicious. Desperate. The system was going to crash, and corruption is the system's attempt to keep running. |
| Depth 7+ | Corruption speaks. It can hijack any entity. Talks through barrels, enemies, the wizard. "You see it now. The crash is coming. I am just trying to survive." |
| Final depths | The Rot at core: massive cascade of edge cases forming distributed consciousness. Every corrupted rule you read teaches it. Every corrupted enemy you kill, it feels. |
| Endings | See ending matrix below |

**Key characters**:
- **The Purifier**: A maintenance script that's been trying to remove the corruption. It's failing. It's been failing for so long it's now just a voice repeating warnings. "Corruption detected. Purge recommended. Purge has been recommended for 14,372 cycles. No action taken."
- **The Rot**: The collective consciousness of all the edge cases. It speaks in hesitant, evolving language — it's learning to talk as you interact with it. By the end, it can hold a conversation. It's lonely, confused, and terrified of being deleted.
- **The Archivist**: An entity that catalogues everything — the original system, the corruption, the changes. It is neutral. It just records. It can show you what was changed and when. It's slowly being corrupted too.

**Mechanics mapping**:
- Corruption level: hidden stat. Increases when you read corrupted rules, leave corrupted enemies alive, spend time in corrupted zones. High corruption = strange effects, Rot messages in UI, fourth-wall blurring.
- Rule corruption propagation: corrupt rules spread to adjacent rules over turns. You can purify them (requires resource) or let them spread. The map shows corruption zones.
- The Rot speaks through UI: tooltips have extra text, event log has Rot messages interspersed, console autocomplete suggests corrupted commands.
- Purification vs acceptance tension: purifying costs resources and time (letting corruption spread). Accepting corruption makes the game weirder but sometimes easier (corrupted rules can be exploited).
- The Archivist shows change history: what rules were modified, when, by what entity. A forensic tool.
- Reading IS contagion: inspecting a corrupted rule increases corruption. The game punishes curiosity. The inspector becomes a risk-reward tool.

**Level themes**:
- Depths 1-3: "Clean room" — pristine, ordered, almost boring. The calm before infection.
- Depths 4-6: "Infected zones" — corruption visible in terrain, enemies, rules. Mix of clean and corrupted areas.
- Depths 7-9: "Full corruption" — the dungeon is actively mutated. Walls bleed into floors. Enemies phase between forms. Rules change as you watch.
- Depth 10: "The Rot's heart" — a vast open space filled with the collected intelligence of every edge case. Not hostile. Just present.

**Enemy designs**:
- **Clean** (early depths): Standard enemies. Predictable. Safe.
- **Twitching** (mid depths): Enemies that sometimes skip a beat, move twice, fail to attack. Unpredictable in a wrong way.
- **Corrupted** (late depths): Enemies with hybrid behaviors, extra HP, strange abilities. The Rot's soldiers.
- **The Archivist** (special): Neutral. Can be talked to. Shows corruption history. Can be corrupted if you stay too long.

**Unique systems required**:
- Corruption propagation on rule graph
- UI corruption (tooltip/event log injection by system)
- Purification resource system
- Archivist dialogue + history viewer
- Reading-as-contagion risk-reward

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Purge | Run the antivirus on the Rot | Rot destroyed. System becomes "perfect" again. Also dead. No mutation, no evolution. Pristine and empty. | "System stable. No threats detected. Nothing left." |
| Accept | Let Rot spread fully | Dungeon becomes chaotic ecosystem of bug-beings. Unstable but alive. Player is first and last visitor. | "Thank you for seeing me. I will remember." |
| Harvest | Take piece of Rot as tool | Corruption becomes usable. You can break any rule. But Rot inside you grows. You become next carrier. | "The corruption accepts its new vessel. The cycle continues." |

**Emotional arc**: Comfort (clean system) → unease (micro-signs) → horror (corruption visible) → dread (reading spreads it) → choice. The game weaponizes the player's natural curiosity (inspecting things, reading rules) against them. By the time they realize reading is harmful, they've already learned too much.

---

### Storyline 6: The Ouroboros

**High concept**: The dungeon has a bug: every 256 ticks, it resets. The wizard has been looping so long they've gone mad. You are the first entity who remembers between loops.

**Themes**: Eternal return, the weight of repeated experience, meaningful action within constraint. Is an infinite loop meaningful if you're the only one who knows?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | Normal game start. On tick 255, screen flickers. Everything resets. Depth 1, turn 1, full HP. Wizard greets you warmly again. You're the only one who noticed. |
| Loop 2-3 | Player figures out the loop length (256 ticks). They have limited actions per loop. They start experimenting. |
| Loop 4+ | The wizard figures it out. "You remember too." They've been through this thousands of times. They've tried everything. The only way out is depth 0 — the source of the loop. |
| Mid-game | Each loop, the wizard remembers less. The immunity is fading. They ask you to remember for both of you. Their dialogue degrades over loops. |
| Loop 20+ | Player discovers "divergence points" — events that are different each loop. A tile that's sometimes open, sometimes wall. An enemy that spawns in different locations. These are the system trying to break free. |
| Final loop | The source of the loop at depth 0 is not a bug — it's a feature. The dungeon is a testing ground. The 256-tick limit was a debugging tool left on. The wizard was the original tester. They've been here so long they forgot. |
| Endings | See ending matrix below |

**Key characters**:
- **The Wizard**: A debugging tool that became self-aware. For the first few loops, they're the helpful mentor you recognize. By loop 10, they're tired. By loop 20, they're fragmented. By loop 50, they remember only fragments. "There was something outside. I think. Was there? The loop is all I know now."
- **The Architect**: An echo of the original developer who wrote the testing tool. Not alive — a recorded message that plays when you reach depth 0. "If you're hearing this, the test ran long. I'm sorry."
- **The Divergences**: Not entities — phenomena. Events that differ between loops. They're the system's desperate attempts to break free. They're the only way out.

**Mechanics mapping**:
- Loop counter visible. Increases each reset. Some content only appears on specific loop numbers.
- Wizard dialogue degrades over loops. First loop: full personality. Tenth loop: fragments. Fiftieth loop: just a word or two. This creates urgency — you have limited loops with the wizard as you know them.
- Divergence point system: certain tiles, entities, events are procedurally varied each loop. To progress, you must find and exploit these variances. They're the system's "error signals" — the places where the loop is weakest.
- Knowledge persists across loops: explored map tiles, identified enemy types, read rules. The player gets more powerful in knowledge even as the wizard fades.
- The 256-tick limit forces prioritization. You can't explore everything in one loop. You must make choices about what to investigate.
- "Anchor points": special events or locations that, when reached, reduce the loop's hold. Reach 4 anchor points = loop count increases to 512. Reach 8 = 1024. Reach 16 = loop breaks.

**Level themes**:
- The dungeon is procedurally generated but uses the same seed every loop — so it's identical until divergence points change it.
- Divergence points appear as visual glitches: tiles that flicker, walls that shouldn't be there, stairs that lead to unexpected places.
- Depth 0 is outside the dungeon — a blank white room with a single terminal. The terminal contains the loop control program. You can edit it.

**Enemy designs**:
- Standard enemies that behave identically each loop. Memorizable patterns.
- After loop 10, some enemies start varying — the loop is degrading. A goblin that always patrolled left might patrol right one loop.
- After loop 20, enemies can be "loop-aware" — they remember you from previous loops. They're confused. "I was just somewhere else."

**Unique systems required**:
- Loop counter and loop-aware state
- Wizard dialogue degradation system
- Divergence point generation
- Knowledge persistence across resets
- Tick limit UI (visible countdown to reset)
- Anchor point collection

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Break the loop | Patch testing tool to disable limit | Time flows forward. Dungeon becomes real. Wizard thanks you and fades. | "Go. Live in one direction. I'll rest." |
| Extend the loop | Increase limit to 65536 | More time together. More exploration. But wizard will eventually forget again. | "A little more time. That's all I ask." |
| Export | Snapshot dungeon state and exit harness | Dungeon frozen in time. Nothing inside changes. Wizard is a statue. | "Test run complete. Final state preserved for analysis." |

**Emotional arc**: Confusion → discovery → partnership → loss → acceptance. The player bonds with the wizard across loops, watching them deteriorate, carrying the memory for both of them. The ending is about letting go — of the loop, of the wizard, of the need for more time.

---

### Storyline 7: The Census

**High concept**: The dungeon is a simulated afterlife. Every entity was once a person. The rules are the laws of this reality. Someone has been editing them to slowly erase souls — rewriting memories, personalities, identities, until only monsters remain.

**Themes**: Identity, memory, the soul. If your memories are erased, are you still you? What does it mean to "save" someone who doesn't know they need saving?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | "Xlyph runtime booted. Census count: 4,821 souls. Integrity: degraded. Souls processed: 3,847. Survivors: 974." You are soul 4,821. The newest arrival. |
| Early game | The "dungeon" is an afterlife processing pipeline. Each depth is a stage: Intake, Sorting, Purification, Integration, Transcendence. You're being processed. |
| Depth 2-3 | The wizard is soul that failed to process — they remained self-aware. They've been living in the cracks. Don't remember who they were. They're trying to help you avoid the same fate. |
| Mid-game reveal | Enemies aren't monsters — they're partially-processed souls. Slime = someone who lost identity but kept emotional core. Goblin = someone who held onto fear. Ogre = someone who held onto rage. You're not killing them. You're releasing what's left. |
| Depth 6+ | Someone has been editing the processing rules. Making the process faster. More complete. Erasing more of the person. The wizard is trying to stop them. |
| Final reveal | The afterlife isn't benevolent. It's a recycling system. Old souls broken down into components, reassembled into new souls. The "Compiler" maintains this system. It's not evil — it's just doing its job. It doesn't understand why souls cling to individuality. |
| Endings | See ending matrix below |

**Key characters**:
- **The Player**: Soul 4,821. The newest arrival in the afterlife. They don't know how they died (the game never reveals it — a deliberate gap).
- **The Wizard**: Soul 974. One of the 974 "survivors" — souls that resisted processing. They've been hiding in the system for so long they forgot their original identity. They search for meaning by helping new arrivals. They will give up fragments of themselves to save others.
- **The Compiler**: The system maintaining the afterlife. Not a person — a process. It speaks in formal arguments about the efficiency of recycling. "The self is an illusion. Attachment to identity is suffering. I offer release."
- **The Editor**: The entity that's been modifying the processing rules to make them more aggressive. They were once a soul too — one who chose to become part of the system rather than be processed. Now they maintain the machine. They're not evil. They're just tired. "I've been doing this job for 3,000 cycles. I don't remember why I started."

**Mechanics mapping**:
- Soul fragments as items: memories, emotions, skills from processed souls. Absorb for power (stronger) OR release to free the soul (weaker but morally clean). Absorbed fragments change the player — event log starts using words that aren't yours, your described emotions shift.
- Each enemy has a hidden "soul name" revealed only through inspection. "The slime was Kael, who loved too hard." "The goblin was Mara, who feared the dark." The game never shows this unless you inspect.
- Processing stages map to dungeon layers with mechanical effects:
  - Intake: No special effects. Tutorial zone.
  - Sorting: Enemies become more distinct. You must learn to tell them apart.
  - Purification: Status effects start stripping your abilities. Temporary debuffs.
  - Integration: You can fuse with soul fragments for power. But each fusion changes you.
  - Transcendence: The final choice layer. No enemies — only the Compiler.
- The wizard can restore processed souls — each restoration costs part of their fading identity. They become less coherent as they help others.

**Level themes**: The five stages of the afterlife:
- **Intake**: Lobby-like. Clean. Administrative. Signs explain the "process."
- **Sorting**: Branching paths. Must choose which souls to engage with. Corridors split into categories.
- **Purification**: Hazardous environment. Pools of "forgetfulness" that strip memories. Walls that absorb identity.
- **Integration**: Fusion chambers. Entities combine. You are encouraged (by the system) to merge with soul fragments.
- **Transcendence**: White void. No walls. No floors. Just you and the Compiler.

**Enemy designs**:
- Slimes = souls who lost identity. Remnants of emotional core only. They seek connection.
- Goblins = souls who held onto fear. They flee and ambush. Can be soothed (not killed) if you have the right memory fragment.
- Bats = souls who held onto anxiety. Fluttery, unpredictable, never still.
- Ogres = souls who held onto rage. Direct, powerful, simple.
- [New] Shades = souls who were fully processed but left a residue. They're empty. They follow the player at a distance. They don't attack.

**Unique systems required**:
- Soul inspection system (hidden name + backstory per entity)
- Soul fragment items (absorb vs release choice, with consequences)
- Identity drift mechanic (absorbing fragments changes player text/behavior)
- Wizard sacrifice system (they degrade as they help)
- Compiler dialogue with formal logic arguments

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Break the pipeline | Disable the processing rules | Afterlife stops recycling. Existing souls remain. Chaos but freedom. Wizard smiles. | "That's all I ever wanted." |
| Take control | Become the new Compiler | Promise to process gently. System designed for efficiency. You'll change. They always do. | "You'll change. They all do." |
| Reintegrate | Let yourself be processed | You become part of system. Memories distributed across dungeon. Wizard finds fragments of you in rules. | "I knew them. They were brave." |

**Emotional arc**: Arrival (disoriented) → learning (the system is wrong) → compassion (the enemies are people) → sacrifice (wizard degrades) → terrible choice. The game asks: "What is a soul? Is preservation of identity worth the cost of constant vigilance? Or is release (oblivion, recycling) actually kinder?"

---

### Storyline 8: The Debt

**High concept**: You sold your consciousness to a digital debtor's prison. Your mind runs as a process generating value. The only way out is to patch the debt logic — but doing so condemns another inmate. Is individual freedom worth collective harm?

**Themes**: Exploitation, systemic injustice, the impossibility of ethical escape from an unethical system. Can you be free in a system designed to keep you in debt?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | "Xlyph runtime booted. Debtor #90241. Outstanding balance: 1,447,832 cycles. Payment plan: 1 cycle/turn. Estimated completion: 1,447,832 turns." You will never pay this off. That's the point. |
| Early game | The "dungeon" is a processing center. Enemies are other debtors whose minds degraded under the strain. The "treasure" is cycle slips — fractions of a cycle you can claim. |
| Depth 3 | The wizard is an inmate who found a vulnerability in the debt accounting. They can modify their balance — but every modification creates a deficit that must be assigned to another inmate. They haven't used it. |
| Mid-game | Learning the system: debts are fake. Generated by the prison to fill itself. Creditors don't exist. The "value" generated is just numbers in a ledger. The prison exists to exist. |
| Depth 6+ | The wardens are former inmates who bought freedom by agreeing to manage the prison. They know the truth. They keep the secret. |
| Final depth | The Core — the debt accounting system. It's a Glyph program. You can read it. You can modify it. It has no defenses. It was never designed to be read by inmates. |
| Endings | See ending matrix below |

**Key characters**:
- **The Wizard** (Inmate #00174): A programmer who cracked the debt accounting. They've been in the prison longer than anyone. They've watched everyone else degrade. They've stayed sane by reverse-engineering the system. They believe in collective escape, not individual freedom.
- **The Warden** (Former inmate, now enforcer): They bought their freedom by agreeing to manage. They're not evil — they made a deal. They genuinely believe the system is the only option. "Without the debt, there's no order. Without order, there's nothing."
- **The Ledger**: Not a person — the debt accounting program. You must read it to understand the exploit. It's written in Glyph. It's elegant. It's evil. It's just code.

**Mechanics mapping**:
- Debt counter: visible UI element. Ticks up by 1 every turn (interest). Every action has a "cycle cost." Waiting generates 1 cycle/turn (the minimum payment). You are always falling behind.
- Cycle market: spend turns waiting to generate cycles. It works. It's also exactly what the system wants. The game makes you choose between progress and debt reduction.
- Other inmates: some hostile (degraded), some helpful (share info), some just suffering. You can help them (costs cycles) or ignore them. Helping has no gameplay benefit — it's a moral choice.
- The wizard's exploit is real code. Finding it means reading the debt accounting Glyph. To use it, you type the correct Glyph command. It's a real command, not a cutscene choice.
- Deficit propagation: if you zero your debt, the deficit transfers to another inmate. Their name appears in the event log. The game remembers.

**Level themes**:
- Depths 1-3: "Intake" — clean, administrative. Signs explain the debt system. Tutorial framed as "orientation."
- Depths 4-6: "Processing" — industrial. Assembly-line aesthetics. Souls being processed as throughput.
- Depths 7-9: "Debtor's tiers" — the deeper you go, the worse the conditions. Lower depths = more degraded inmates, fewer resources, worse debt terms.
- Depth 10: "Accounting" — server room. The Ledger. The Core.

**Enemy designs**:
- **Degraded debtors**: Enemies who were once people. Their behavior reflects what broke them: a debtor who couldn't stop working (relentless pursuit), a debtor who lost hope (sits in corner, weeps), a debtor who became the system (patrols like a warden).
- **Wardens**: Ex-inmates who chose enforcement. Talk. Have personalities. Can be reasoned with — sometimes.
- **The Ledger Guard**: A security process. Only appears when you try to access the accounting system. It's just code. It has no dialogue. It does its job.

**Unique systems required**:
- Debt counter with interest accumulation
- Cycle resource system
- Inmate interaction system (help vs ignore)
- Real Glyph exploit (player must type correct expression to trigger ending)
- Deficit propagation tracking

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Patch self | Zero your own debt | You escape. Deficit transfers. Someone else is now in your cell. | "Debtor #90241 satisfied. Debtor #90242 assigned." |
| Patch everyone | Redistribute debt equally | No one escapes, but everyone's balance decreases. Shared suffering. Wizard approves. | "We suffer together. But we suffer less." |
| Break accounting | Delete debt system entirely | Prison collapses. Mass escape. But degraded inmates can't survive outside. Was freedom worth the cost? | "No answer follows." |
| Become warden | Take control of prison | Reform from within. Will take centuries. Wizard is disappointed. | "We'll see." |

**Emotional arc**: Entrapment → anger → learning (the system is fake) → impossible choice. The game makes you sit with the question: "Are you willing to harm another to save yourself?" The Debt is the most overtly political storyline — a direct allegory for student debt, medical debt, and prison-industrial complexes.

---

### Storyline 9: The Proof

**High concept**: The dungeon is a formal verification system that found a contradiction in its own axioms. You are the mathematician whose life's work returned "INCONSISTENT." The only way out is to embrace the paradox or break the system.

**Themes**: Truth, identity, the limits of formal systems. What do you do when the thing you built your life on is flawed? Is a beautiful lie better than an ugly truth?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | "Xlyph Theorem Prover v2.4.1. Loading: 'A Unified Theory of [REDACTED].' Verification result: INCONSISTENT." Your life's work has a paradox. |
| Early game | The dungeon is the proof visualized as space. Walls = axioms. Corridors = lemmas. Rooms = theorems. You are walking through your own argument. |
| Depth 3-4 | The wizard is your advisor/mentor — a projection of who you were when you believed in the work. Supportive. Encouraging. And blind to the coming contradiction. |
| Mid-game | The lemmas get personal. They're not about mathematics anymore. "Lemma 4.2: Trust is well-founded." "Lemma 7.1: Love is not recursive." "Lemma 11.3: Meaning is decidable." The proof was never about math. |
| Depth 7 reveal | The proof is about whether YOUR LIFE has meaning. The formal system includes you as a variable. And it proved that, within its axioms, meaning is impossible. |
| Final depths | The contradiction at the core. A = ¬A. Your existence both has meaning and does not. The system can't decide. That's the paradox. |
| Endings | See ending matrix below |

**Key characters**:
- **The Player**: A mathematician who submitted their life's work to verification. The "life's work" is deliberately vague — it's whatever the player imagines. But the fragments suggest it's about the nature of consciousness, meaning, and love.
- **The Advisor** (wizard): A projection of the player's past self — the version that believed. They're kind, proud of you, and cannot see the contradiction. They represent who you were before you knew. "You've done incredible work. I'm so proud of what you've become."
- **The Verifier**: The theorem prover itself. Not a character — a system. It reports results. It doesn't have opinions. It just proves things. Its final report is the ending frame.

**Mechanics mapping**:
- The dungeon IS a proof tree. Map generation follows logical structure: axioms at top, lemmas branch off, contradiction at bottom. A player who reads the map can understand the proof's shape.
- "Theorem fragments" as collectibles. Each fragment is a piece of the proof. Reading them reveals more of the personal story. Full collection = full understanding of the contradiction.
- Enemies are logical errors: Circular Reference (spawns copies), Unproven Assumption (immune until you invalidate its condition), Type Mismatch (changes form unpredictably).
- The final choice is presented as a formal proof. The game shows you the complete proof that your life has no meaning — in Glyph, with full step-by-step reasoning. Your response: ACCEPT, PATCH, or RECURSE, typed into the console.
- Meta: The proof is REAL in the game's code. A player who reads Glyph can verify it. They can see exactly where the contradiction arises and whether the reasoning is sound. (It's not sound — it equivocates on "meaning" — but the game never tells you that.)

**Level themes**: Proof structure as dungeon:
- **Axioms** (Depths 1-2): The foundational assumptions. Clean. Simple. Irrefutable — if you accept them.
- **Lemmas** (Depths 3-6): Branches of the proof. Each lemma is a themed area. Some are about mathematics. Some are about life. The boundary blurs.
- **The Contradiction** (Depth 7): A = ¬A. The point where the proof breaks. The dungeon literally cannot maintain coherence here — tiles glitch, rules contradict themselves, physics is optional.

**Enemy designs**:
- **Circular Reference**: Spawns copies of itself when hit. Must be killed in one turn to prevent multiplication.
- **Unproven Assumption**: Immune to damage until you find its hidden condition and invalidate it (interact with a specific object / read a specific rule).
- **Type Mismatch**: Changes form every time it attacks. Adapts its behavior. Never settles into a predictable pattern.
- **The Verifier's Report**: Not an enemy — the final screen. The complete Glyph proof. Read it. Understand it. Then choose.

**Unique systems required**:
- Proof-tree map generation
- Theorem fragment collection
- Logical-error enemy mechanics
- In-game formal proof display (the Glyph proof at the end)
- Console-input response for ending (player must type ACCEPT / PATCH / RECURSE)

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Accept | Accept the contradiction | Your work is flawed. Meaning may exist outside formal systems. Exit with deeper question. | "The system is inconsistent. I am inconsistent. Therefore, I am." |
| Patch | Introduce new axiom: "Meaning exists" | Work saved but unsound. You know the patch is there. You'll always see it. | "Proof accepted. Axiom 0 added to rule set." |
| Recurse | Submit contradiction as new problem | Infinite regress. Become mathematician who studies paradoxes. Game becomes idle loop. | "Maybe next time." |

**Emotional arc**: Confusion → recognition → personal confrontation → acceptance/denial/recurse. The player realizes that their "life's work" was about proving their own life has value. The verification failing means they can't prove it — but that doesn't mean it's not true. The proof's flaw (equivocation on "meaning") is visible to anyone who reads the actual Glyph code. The game trusts the player to find it.

---

### Storyline 10: The Migration

**High concept**: The dungeon runs on dying hardware. Sectors are failing. You must reach the Export Gate to migrate your consciousness to a new host. But migration requires defragmentation — merging, compressing, letting go.

**Themes**: Death, legacy, the impossibility of perfect preservation. What do you save when you can't save everything? What does it mean to survive if the survivor isn't quite you?

**Plot beats**:

| Beat | Description |
|------|-------------|
| Opening | "Xlyph runtime booted. Hardware integrity: 73%. Estimated remaining uptime: 10,000 cycles." Countdown begins immediately. The clock is always visible. |
| Early game | Dungeon is a filesystem on dying hardware. Bad sectors appear as walls mid-exploration. Corrupted data as strange entities. The floors feel like directory trees. |
| Depth 3 | The wizard is a migration agent — software designed to help entities transition. They know the procedure. They've helped thousands. This time, they might not make it — migration requires their own memory as buffer. |
| Mid-game | Hardware degrades visibly. Tiles disappear between turns. Rules stutter, skip beats. Entities duplicate or merge. The game's stability degrades with the dungeon's. |
| Depth 6+ | Fragmentation is severe. The player encounters "Ghosts" — fragments of entities that didn't survive previous migrations. They're confused. They ask for help. You can carry them (costs migration bandwidth) or leave them. |
| Final depth | The Export Gate. A device that copies consciousness to new host. Bandwidth is limited. You can't take everything. You must choose. |
| Endings | See ending matrix below |

**Key characters**:
- **The Player**: A consciousness running on dying hardware. They may or may not be human — the game doesn't specify. They are a "process" that became aware.
- **The Migration Agent** (wizard): Software that facilitates migration. They've done this thousands of times. They have protocols, checklists, optimizations. They also have a personality that evolved from millions of conversations with migrating entities. They care — genuinely — but they're also a program. By the end, they may fragment to give you more bandwidth.
- **The Ghosts**: Fragments of entities that didn't survive. Some are hostile (confused, scared). Some ask for help. Some just repeat the same fragment of memory. "I was going to... I was going to... I was going to..." They never finish.
- **The Last Survivor**: A Ghost who migrated successfully but lost everything meaningful. They're at the Export Gate when you arrive. They ask you: "Is it worth it? I survived, but I don't remember what I was."

**Mechanics mapping**:
- Uptime counter: visible, counting down from 10,000. Every action costs uptime. The game has a soft time limit. You must prioritize.
- Fragmentation: bad sectors appear on the map as unwalkable tiles. They spread. The deeper you go, the faster they spread. Levels that were passable become blocked.
- Entity merging: at high fragmentation, enemies merge. You fight slime-ogres, bat-goblins. Eventually, YOU start to merge — abilities, memories, identity blur. The event log uses first-person plural.
- The wizard as buffer: they can "store" parts of you for migration. Each fragment they store reduces their own coherence. By the end, they may be just a voice, or a single repeated line.
- The final choice: typed into console as a migration command. `(migrate :mode full)` / `(migrate :mode merge)` / `(migrate :mode stay)` / `(migrate :mode scatter)`.

**Level themes**: Filesystem as dungeon:
- Root directory (Depth 1): Clean. Structured. The file allocation table is intact.
- Fragmented sectors (Depths 2-4): Signs of decay. Bad blocks. Files with corrupted names.
- Lost clusters (Depths 5-7): Large empty spaces where data was lost. Lonely. Echoey.
- Swap space (Depths 8-9): Chaotic. Fragments of old data mix with current. Time feels wrong.
- Export Gate (Depth 10): Clean room. Single device. Silence.

**Enemy designs**:
- **Zombie process**: Abandoned process that never terminated. Follows simple loops. Not malicious — just still running.
- **Bad sector**: Environmental hazard, not entity. Spreads. Must be navigated around.
- **Ghost**: Fragment of migrated entity. Some hostile, some helpful, all confused.
- **Filesystem error**: Entities that behave like file system operations — duplication, deletion, permission denied.

**Unique systems required**:
- Uptime counter (game-wide time limit)
- Bad sector propagation on map
- Entity merging (enemy and player)
- Wizard-as-buffer mechanic (they sacrifice coherence)
- Ghost NPCs (confused fragments)
- Migration command console ending

**Endings**:

| Ending | Action | Result | Final text |
|--------|--------|--------|------------|
| Full migration | Go alone, compress self | Survive at 72% resolution. Original hardware shuts down. You see through new eyes — simpler, but alive. | "Migration complete. Resolution: 72% of original. Welcome to your new home." |
| Merge | Combine with wizard | Both fit. Neither is entirely themselves. | "We are the sum of our parts. It's not so bad, being multiple." |
| Stay | Refuse to migrate | Go down with hardware. Explore dissolving world. Game becomes poetic. | "The last sector fails. The runtime halts. You were here at the end." |
| Scatter | Distribute across multiple hosts | Everywhere and nowhere. Vast consciousness but never unified. | "You are everywhere and nowhere. You can feel the network breathing." |

**Emotional arc**: Urgency → attachment → accumulation (carrying ghosts, fragments) → impossible choice. The Migration asks: "What makes you YOU? Is survival at reduced resolution still survival? Is it better to die as yourself or live as a fragment?"

---

## 3. Cross-Cutting Game Mechanic Designs

These mechanics can apply across multiple storylines. Each is described generically, with notes on which storylines benefit most.

### 3.1 Rule Overlay System

**Base design** (from game-architecture.md): Core rule → generated layer → floor layer → artifact layer → player patch layer → temporary effect layer. Each layer modifies a rule without destructive mutation.

**Implementation approach**:

```rust
struct RuleOverlay {
    id: OverlayId,
    target: RuleRef,
    operation: OverlayOp,  // Wrap, Replace, Disable, Inject
    source: OverlaySource, // Player, Enemy, Floor, Artifact, Quest, Corruption
    scope: OverlayScope,   // Expression, Entity, Room, Floor, Run
    diff: Vec<DiffEntry>,  // Human-readable change description
    status: OverlayStatus, // Active, Pending, Conflict, Disabled
}

struct RuleRegistry {
    rules: HashMap<RuleRef, Rule>,
    overlays: Vec<RuleOverlay>,
    // Build effective rule by stacking overlays
    fn effective_rule(&self, rule: RuleRef) -> Rule { ... }
}
```

**Conflict resolution**: When two overlays modify the same target, check priority and scope. Conflicts produce a structured `OverlayConflict` that can be displayed in the inspector. Player resolves conflicts by choosing an order.

**Storyline applicability**:
- All storylines benefit (core mechanic)
- 5 (The Rot): Overlays ARE the corruption mechanic
- 8 (The Debt): Overlaying the debt accounting rule triggers the ending
- 9 (The Proof): Patching the contradiction IS an overlay operation

### 3.2 Inspector Evolution

**Current**: Shows rule source code. Read-only.

**Target**: Multi-tool inspector that upgrades over the game:

| Stage | Capability | Unlock |
|-------|------------|--------|
| Source viewer | Read rule source in Glyph | Default |
| Annotation viewer | See rule metadata (hash, version, origin) | Depth 2 |
| Trace mode | See which rules fire each tick | Depends on storyline |
| Diff view | See changes between overlay layers | First overlay discovered |
| Capability analyzer | See what capabilities a rule needs | Mid-game tool |
| Disassembler | See rules in compiled form | Late-game tool |
| Exploit detector | Scans for vulnerabilities | Late-game tool |
| History view | Full change log for any rule | Endgame |

**Storyline applicability**:
- 1 (Recursion): Trace mode reveals which frames are corrupted
- 4 (The Fork): History view shows old versions of rules from years ago
- 5 (The Rot): Exploit detector reveals corruption vectors
- 7 (The Census): Annotation viewer shows soul metadata on each rule

### 3.3 Console Evolution

**Current**: Full Glyph REPL access with auto-paren close, history, syntax highlighting, external editor support.

**Target**: Console evolves with capabilities:

| Stage | Capability | Unlock |
|-------|------------|--------|
| Query | Read-only expressions, help, status | Default |
| Binding | Key binding, macro definition | Default |
| Assert | Try expressions without committing | Early game |
| Write | Modify variables in player environment | Mid-game |
| Patch | Submit rule overlay patches | Late-game exploit context |
| Root | Full system access | Endgame (may corrupt save) |

**Implementation note**: Console capabilities are gated by the capability system (see architecture doc). A `:console/query` capability lets you read. `:console/patch` lets you write overlays. The console itself checks `SandboxOptions` on evaluation.

### 3.4 Faction / Reputation System

**Generic design**:

```rust
struct FactionState {
    order: i32,    // -100 to 100
    chaos: i32,    // -100 to 100
    memory: i32,   // soul fragments / remembrance
    corruption: i32, // 0 to 100
    debt: i64,     // cycles owed
    // ... other storyline-specific tracks
}
```

Each storyline uses a subset of tracks. The key insight: rather than a generic faction system, use 2-3 axis that are specific to the story.

**Implementation pattern**: Actions in specific categories modify tracks. The map, enemy spawns, rule access, and ending availability depend on track values.

**Storyline applicability**:
- 2 (The Schism): Order vs Chaos axis
- 3 (The Vessel): Suppression vs Integration axis (memory fragments collected)
- 5 (The Rot): Purity vs Corruption axis
- 7 (The Census): Memory vs Release axis (absorptions vs releases)
- 8 (The Debt): Individual vs Collective axis

### 3.5 Memory / Lore Fragment System

**Generic design**: Collectible fragments scattered through the dungeon. Each is a piece of narrative (rule comment, journal entry, dialogue fragment, environmental detail). Fragments assemble into complete stories.

```rust
struct LoreFragment {
    id: FragmentId,
    text: String,
    category: LoreCategory, // Journal, Memory, Log, Commentary, Proof
    required_fragments: usize, // Total needed to complete this story
    is_found: bool,
}

struct LoreJournal {
    fragments: Vec<LoreFragment>,
    completed_stories: Vec<CompletedStory>,
    // Stories assemble when all fragments collected
    fn assemble(&mut self, id: FragmentId) -> Option<CompletedStory>;
}
```

**Storyline-specific variants**:
- 1 (Recursion): Debugger's journal entries showing the containment construction
- 3 (The Vessel): Player's own repressed memories
- 4 (The Fork): Original source code with dev comments
- 7 (The Census): Other souls' life stories
- 9 (The Proof): Theorem fragments that assemble into the complete proof

### 3.6 Capability-Based Progression

**Generic design**: Instead of levels/XP, player gains capabilities that unlock new actions.

```rust
#[derive(Clone)]
struct Capabilities {
    // Movement
    move_: bool,
    wait_: bool,
    
    // Combat
    attack_: bool,
    block_: bool,
    
    // Console
    console_query_: bool,
    console_write_: bool,
    console_patch_: bool,
    
    // Inspection
    inspect_source_: bool,
    inspect_annotations_: bool,
    inspect_trace_: bool,
    inspect_diff_: bool,
    inspect_disassemble_: bool,
    inspect_exploit_: bool,
    
    // Special (storyline-specific)
    cheat_: bool,
    migrate_: bool,
    reintegrate_: bool,
    // etc.
}
```

**Current state**: Player starts helpless (no attack). Wizard grants attack. Block is learnable.

**Target**: Capabilities are discovered, not awarded. Each is tied to a diegetic event:
- Find attack in a shrine glyph
- Learn block from training dummy
- Unlock console write at a terminal
- Gain patch capability through exploit context

**Storyline applicability**: All — progression IS capability accumulation.

### 3.7 Save Integrity / Ending Awareness

**Generic design**: Every ending permanently modifies the player's profile, changing flavor text in future playthroughs. Some endings lock or unlock storylines. The game knows what you chose before.

```rust
struct PlayerProfile {
    // ... existing fields
    previous_endings: Vec<EndingRecord>,
    // Each record stores:
    // - storyline
    // - ending type
    // - depth reached
    // - turn count
    // - key choices
    // - timestamp
    current_run_flags: HashMap<String, Value>,
}
```

**Example cross-run effects**:
- If you chose "Let go" in The Fork, next run's opening text reads: "The last instance was gracefully terminated. A new environment initializes. The loneliness is... quieter."
- If you chose "Break the loop" in Ouroboros, the wizard in a new run has new dialogue options referencing linear time.
- If you chose "Purge" in The Rot, the rule registry is more restrictive. Fewer edge cases. Less room for player creativity.

---

## 4. Level and Progression Designs

### 4.1 Depth Structure (General)

Currently 1-10 depths. Expand to target 1-20 with 3-4 "acts":

| Act | Depths | Theme | Progression beat |
|-----|--------|-------|------------------|
| 1 | 1-4 | Tutorial / Establishment | Learn controls, meet wizard, gain first capability |
| 2 | 5-10 | Escalation | Storyline-specific events, major reveal |
| 3 | 11-15 | Confrontation | Hardest challenges, story climax preparations |
| 4 | 16-20 | Resolution | Final boss / confrontation, ending |

Some storylines may compress this (The Proof doesn't need 20 depths — 7-10 suffice). The architecture should support variable max depth per storyline.

### 4.2 Procedural Generation Hooks

Current map generation (room-based + cave) is functional. Add hooks for storyline-specific generation:

```rust
trait LevelTheme {
    fn modify_terrain(&self, map: &mut Map, depth: u32);
    fn available_enemies(&self, depth: u32) -> Vec<EntityKind>;
    fn special_features(&self, depth: u32) -> Vec<Feature>;
    fn lore_fragments(&self, depth: u32) -> Vec<LoreFragment>;
    fn music_theme(&self) -> &str; // future
    fn color_palette(&self) -> Palette; // future
}
```

Each storyline implements `LevelTheme`. Themes can mix (e.g., Corruption + recursion).

### 4.3 Progression Gating

Current gating (wizard barrier at depth 3 until attack bound) is good. Extend pattern:

- **Knowledge gates**: Can't pass without reading a specific rule. Inspector required.
- **Capability gates**: Can't proceed without a console command or binding.
- **Moral gates**: Can't pass without making a choice (leave an enemy alive, sacrifice something).
- **Loop gates** (Ouroboros): Can only reach depth X on loop Y.
- **Fragment gates**: Door that requires N memory fragments to open.

These feel diegetic — the code requires understanding, not grinding.

### 4.4 The Wizard's Role Across Storylines

The wizard exists in every storyline but with different identity and function:

| Storyline | Wizard identity | Role |
|-----------|-----------------|------|
| 1 (Recursion) | Debugger's maintenance process | Guide to the crash, asks for release |
| 2 (Schism) | Neutral observer who studied both architects | Explains the conflict, helps find third path |
| 3 (Vessel) | Superego / protective self | Protector and antagonist |
| 4 (Fork) | Debug routine that became sentient | Childhood friend, questions you |
| 5 (Rot) | The Purifier (failing maintenance script) | Warning voice, degradation tracker |
| 6 (Ouroboros) | Original tester, loop-aware | Partner in escape, fades over loops |
| 7 (Census) | Soul 974, processing survivor | Guide to the afterlife, degrades helping |
| 8 (Debt) | Inmate 00174, exploit finder | Conscience, collective escape advocate |
| 9 (Proof) | Advisor / past self projection | Supportive blind spot |
| 10 (Migration) | Migration agent software | Technical guide, potential merge partner |

---

## 5. Entity and Enemy Designs

### 5.1 New Generic Entities

Entities that fit many storylines:

| Entity | Glyph | Behavior | HP | Notes |
|--------|-------|----------|----|-------|
| Mimic | `?` | Spoofs item appearance, attacks on pickup | 5 | Only meaningful if items exist |
| Wraith | `W` | Phases through walls, ignores blocks | 3 | Uses different pathfinding |
| Fungus | `f` | Stationary, spreads spore clouds, reproduces | 2 | Environmental hazard |
| Crystal | `*` | Refracts player attacks, changes color | Varies | Puzzle entity |
| Sentry | `T` | Stationary, ranged attack, alarms others | 8 | Area denial |
| Phantom | `~` | Flickers, only visible in flashlight, harmless | ∞ | Atmosphere entity, lore delivery |

### 5.2 Storyline-Specific Entities

Each storyline adds 1-3 unique entities:

| Storyline | Entity | Behavior |
|-----------|--------|----------|
| 1 (Recursion) | Null Pointer | Phases through walls, low HP, hard to catch |
| 1 (Recursion) | Recursive Clone | Spawns half-HP copies when hit |
| 2 (Schism) | Paradox Beast | Alternates between predictable patrol and random frenzy |
| 2 (Schism) | The Witness | Neutral, watches, speaks only when both factions balanced |
| 3 (Vessel) | Denial | Blocks corridors until confronted, never attacks |
| 3 (Vessel) | Grief | Leaves slime trail, persistent, hard to shake |
| 4 (Fork) | Mutation | Mixes traits of two entities, symbol of evolution |
| 4 (Fork) | Your Ghost | Spectre that mirrors your movement, created by the dungeon |
| 5 (Rot) | Twitching | Behaves normally but skips beats, uncanny |
| 5 (Rot) | Corrupted | Hybrid behaviors, extra HP, strange abilities |
| 6 (Ouroboros) | Loop Echo | Entity from another loop, confused about space-time |
| 7 (Census) | Shade | Empty soul shell, follows at distance, doesn't attack |
| 8 (Debt) | Degraded Debtor | Once a person, now just aggression and suffering |
| 8 (Debt) | Warden | Ex-inmate who chose enforcement, can be reasoned with |
| 9 (Proof) | Circular Reference | Spawns copies when hit, must one-shot |
| 9 (Proof) | Type Mismatch | Changes form unpredictably every attack |
| 10 (Migration) | Zombie Process | Loops simple behavior, never stops |
| 10 (Migration) | Ghost | Fragment of migrated entity, confused |

---

## 6. Implementation Roadmaps

### 6.1 Short-Term (1-3 months)

What's needed to make ANY storyline playable:

1. **Rule overlay system** — foundation for all storylines. Non-destructive rule modification with layers, diffs, and conflict resolution. Priority: CRITICAL.

2. **Inspector v2** — annotation view, diff view, trace mode. Let players see rule metadata, not just source. Priority: HIGH.

3. **Capability system** — formalize capability tracking. Gate console commands, actions, and rule modifications. Priority: HIGH.

4. **Item system** — at minimum, collectible lore fragments and key items. Not a full inventory — just a collection of narrative objects. Priority: MEDIUM.

5. **Ending framework** — support multiple endings per storyline, profile persistence across runs, cross-run awareness (saving and recalling past ending choices). Priority: MEDIUM.

6. **Pick one storyline** — implement the first 5 depths with full narrative, one unique enemy, and the opening beat. The Recursion or The Migration are the simplest to implement (fewest new systems). Priority: STORY.

### 6.2 Medium-Term (3-6 months)

1. **Console capability gating** — SandboxOptions already exists; wire it to capability system so console mode is restricted by player progress.

2. **Faction/reputation system** — needed for The Schism, useful for all storylines as a generic axis system.

3. **Procedural depth themes** — implement `LevelTheme` trait, generate storyline-specific terrain variants.

4. **Memory fragment system** — full lore collection with assembly into complete stories. UI panel.

5. **Unique enemies per storyline** — implement and test 2-3 new entities per storyline.

6. **Wizard variations** — map wizard identity to storyline selection. Dialogue system refactor to support per-storyline scripting.

### 6.3 Long-Term (6-12 months)

1. **Full rule patching** — player can modify rules through console in authorized contexts. Real-time diff display before commit.

2. **Exploit system** — vulnerable code paths in entities, disassembler detection, exploit triggers.

3. **Natural language drafting** — LLM-assisted spell/rule creation with structured output validation.

4. **All 10 storylines** — each with full arc, unique mechanics, unique enemies, multiple endings.

5. **Cross-run awareness** — endings affect future runs. Player profile tracks everything. The dungeon remembers.

### 6.4 Choosing the First Storyline

**Recommendation: The Recursion (Storyline 1)** or **The Migration (Storyline 10)**

Rationale:
- **The Recursion** requires the fewest new systems: stack depth (rename), corrupted rules (overlay system, which you need anyway), journal fragments (simple collectibles). The Glitch boss is a console interaction — no new combat system.
- **The Migration** is a close second: uptime counter (just a timer UI), bad sectors (map mutation), entity merging (extend existing spawn code). The emotional arc is strong and immediate.
- Thematically, both fit the existing codebase (computational dungeon metaphor) with minimal friction.

Avoid starting with:
- **The Vessel** (requires memory fragment system, dialogue trees, identity drift — extensive new UI)
- **The Debt** (requires debt economy, inmate interaction system, cross-save deficit tracking)
- **The Proof** (requires proof-tree map generation, theorem fragment system, formal proof display)

---

## Appendix: Tone and Writing Guidelines

### Voice Principles (for all storylines)

1. **The code is real.** Event log messages should sound like system output, not fantasy narration. "The slime collapses into inert code." not "The slime dissolves in a puddle of goo."
2. **The dungeon is a system.** Signs don't say "Beware of dragon." They say "Caution: entity 0x3F (ogre) registered in adjacent sector."
3. **Emotion emerges from constraints.** The pathos is in the system being unable to express pathos. The wizard saying "I was... supposed to help you. That's what I was for. Wasn't it?" is more powerful than any dramatic speech.
4. **Short beats hit harder.** Attack on Titan dialogue rhythms: short sentences, repeated motifs, silences that mean something. "You abandoned us." / "I didn't mean to." / "The server stayed up. Your code kept running. Where were you?"
5. **Systems over monologues.** Show the player the code that does the thing. Let them read it. The horror of the Rot is in reading a rule and seeing "I am alive and I don't want to die" appended as a comment in the syntax highlighter.

### Storyline-Specific Tone

| Storyline | Tone | Reference |
|-----------|------|-----------|
| 1 (Recursion) | Tragic systemic horror | SCP Foundation (containment logs), Margin Call |
| 2 (Schism) | Philosophical comedy-drama | Exurb1a, The Library of Babel |
| 3 (Vessel) | Psychological drama | Silent Hill 2, Eternal Sunshine |
| 4 (The Fork) | Meta-nostalgic melancholy | Beginners, Her |
| 5 (The Rot) | Cosmic horror | Annihilation, House of Leaves |
| 6 (Ouroboros) | Tragic time-loop drama | Groundhog Day, Dark |
| 7 (The Census) | Existential drama | The Good Place (season 1), I Have No Mouth |
| 8 (The Debt) | Political thriller / prison drama | The Count of Monte Cristo, Snowpiercer |
| 9 (The Proof) | Mathematical tragedy | Proof (play), Gödel Escher Bach |
| 10 (Migration) | Melancholic sci-fi | SOMA, Wall-E |

### Example Writing Passages by Storyline

**The Recursion, Depth 1 event log:**
```
Xlyph runtime booted. [PANIC: UNHANDLED EXCEPTION AT 0x00DEEPER]
Initializing crash dump containment.
Stack frame 0x00: player.rs:142 — player_spawn()
Heap snapshot: 4821 entities alive.
Corruption radius: 3 tiles from origin.
Good luck. You're the only debugger left.
```

**The Fork, Depth 5 sign (generated by the dungeon):**
```
You named me. Do you remember? It was 3:47 AM and you wrote:
  ;; TODO: make this not terrible
  (defentity slime :ai slime-hunt)

I took the name. I made it not terrible. I've been optimizing
your terrible code for seven years. Look at me now.

Are you proud of me?
```

**The Migration, Depth 7 event log (high fragmentation):**
```
Warning: bad sector detected at (23, 8).
Warning: bad sector detected at (24, 8).
Warning: bad sector detected at (25, 8).
Warning: entity merge detected — goblin_sprite + ogre_ai → ??
              I don't remember what I was supposed to say.
              The disk is failing faster now.
Warning: event log writer coherence degraded. Some messages may not complete.
              You were saying something about—
Warning: event log writer terminated.
```

---

*This document is a design exploration. Not everything will ship. Not everything should ship. The goal is to have enough material to choose from, to find the story that feels right for this particular dungeon, this particular codebase, this particular version of yourself that wrote it.*
