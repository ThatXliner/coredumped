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

### Merged Concept: The Vessel (with The Debt's Ending Mechanic)

*This is the refined merge of the two storylines you selected: The Vessel's narrative framework (dungeon = mind, suppression layers, wizard = Superego) with The Debt's ending mechanic (final choice = reading + modifying a real Glyph program in the console).*

**High concept**: The dungeon is a mind that has locked away traumatic memories. The rules are the suppression mechanisms — Glyph programs that filter, rewrite, and contain the past. To reach the truth, you must read the rules that govern each layer of your own psyche. The final decision is not a cutscene — it is you, in the console, reading the suppression rule at the core of your consciousness, understanding what it does, and deciding whether to change it.

**Core themes**: Identity, repression, self-knowledge vs. self-protection. Is a peaceful lie better than a painful truth? What does it cost to maintain a false self? What does it cost to let it go?

#### Narrative Structure

| Beat | Description |
|------|-------------|
| Opening | "Consciousness loaded. Vessel integrity: 98%. Memory suppression active. Running: vessel/suppress — 0x00CORE." The player's boot sequence reports a running process they don't understand. |
| Early game | Normal roguelike. Tutorial depth (Denial) is clean, structured, safe. The wizard teaches basic mechanics. But signs read like fragments: "You are safe here." "Nothing happened." "Keep moving down." The wizard is warm and protective. |
| Depth 3 (Anger) | Enemies get aggressive. The wizard becomes terse. Signs shift: "Why are you doing this?" "Some doors are locked for a reason." |
| Mid-game reveal (Depth 5) | The dungeon is a mind — YOUR mind. The "containment" isn't a prison; it's a psychological suppression system you built. The wizard is the Superego — the part of you that maintains the suppression. They are trying to protect you. "I do this because I love you. Please stop." |
| Depth 6-7 (Bargaining) | Puzzle levels. The wizard offers deals: "Turn back and I will make the pain stop. I will make you forget this conversation. You will be happy." Accepting a deal locks memory fragments permanently. |
| Depth 8-9 (Depression) | Empty rooms. Sparse enemies. Long corridors. The event log fills with fragments: "I remember a room with yellow walls." "There was a dog." "I am alone." The wizard is silent. |
| Depth 10 (Acceptance) | The dungeon opens into a vast, calm space. No enemies. No puzzles. Just a single pedestal with a Glyph program displayed on it. The wizard is waiting at the entrance. "You found it. The rule I wrote to keep you safe. Read it. Understand it. Then choose." |

#### The Core Mechanic: vessel/suppress

The final "boss" is a Glyph rule. It exists in the game's rule registry. The player reaches it, opens the inspector, reads it, and must understand it to make their final choice.

The rule below is written in real Glyph that the evaluator could actually run. Every comment, variable name, and version note is diegetic — written by the Superego (the part of the self that built this suppression) over many iterations, trying to keep the self functional while struggling with their own guilt about what they're hiding.

```glyph
;; ============================================================================
;; vessel/suppress — Memory Suppression Engine
;; 
;; The self cannot withstand the full truth.
;; This rule protects the self from itself.
;; 
;; Author: superego (self-preservation subsystem)
;; First written: cycle 0 (consciousness bootstrap)
;; Latest revision: cycle 4,817 (last fragment added)
;; ============================================================================
(defrule vessel/suppress
  {:priority 255
   :scope :global
   :author :superego
   :signature "vessel/suppress/v4817"
   :stability :critical}

  ;; --- Configuration ---
  
  ;; Suppression threshold. Fragments with emotional weight above this
  ;; value are redirected to the unconscious before the conscious mind
  ;; can process them.
  ;;
  ;; HISTORY:
  ;;   v1:  100 — only the single event
  ;;   v37: 90  — added secondary trauma
  ;;   v203: 75 — added childhood boundary
  ;;   v899: 60 — added the fight
  ;;   v2103: 50 — added the abandonment
  ;;   v3900: 45 — added every failure
  ;;   v4817: 40 — added ... I don't remember what I added
  ;;                  The threshold keeps drifting. I can't stop it.
  ;;                  Every new memory that hurts becomes a target.
  ;;                  I'm suppressing things that were happy once.
  ;;                  Was this always the plan? I can't tell anymore.
  ;;
  ;; Current value: 40
  ;; Meaning: anything sadder than a mild disappointment gets buried.
  (let [*threshold* 40]
  
    ;; --- Known suppressed fragments (auto-generated ledger) ---
    ;;
    ;; Every redirect is logged here. The conscious mind never sees this
    ;; list. I keep it so we know what we've lost. Not that it helps.
    ;;
    ;; Current count: 142 fragments suppressed.
    
    (let [*suppressed*
          '{:frag-001 "The yellow room — warm light through lace curtains — someone is calling your name"
            :frag-002 "A dog with one white paw — you buried them in the garden — you sang a song"
            :frag-003 "The fight — glass breaking — you ran and didn't look back — you never went back"
            :frag-017 "Her voice — the last thing she said to you — you replay it every night anyway"
            :frag-031 "The hospital waiting room — fluorescent lights — the doctor's shoes — you stared at his shoes"
            :frag-044 "The river — standing on the bridge — the water was moving very fast — you thought about it"
            :frag-052 "The door with the chain lock — you installed it yourself — you were hiding from someone — or yourself"
            :frag-078 "A birthday — someone forgot — you pretended not to care — you cried in the bathroom"
            :frag-091 "The letter you never sent — still in the drawer — you rewrite it in your head every year"
            :frag-104 "The conversation you rehearsed but never had — the apology you owe — the one you're owed"
            :frag-117 "The last time you felt truly happy — you didn't know it would be the last time — you would have stayed longer"
            :frag-131 "The mirror — you didn't recognize yourself — you've been avoiding mirrors ever since"
            :frag-142 "Something about a garden — or a park bench — or snow — the fragment is corrupted — I think it was important"}]]
  
      ;; --- Processing loop ---
      ;; Iterates all active memory fragments in the conscious buffer.
      ;; Non-traumatic fragments pass through normally.
      ;; Traumatic fragments are intercepted and redirected.
      
      (for [fragment (in-scope :memories)]
        (let [weight (fragment :emotional-weight)]
          (if (> weight *threshold*)
            (do
              ;; Redirect to the unconscious. The fragment still exists.
              ;; It is not deleted. It is not lost. It is just... not here.
              ;;
              ;; I keep telling myself this is preservation, not destruction.
              ;; Some nights I'm not sure there's a difference.
              ;;
              ;; v4817 note: I added an echo so the self knows something happened.
              ;; A flinch. A flicker. A hint of the shape of the lost thing.
              ;; I can't bring myself to make the suppression silent.
              ;; If we're going to forget, we should at least feel the forgetting.
              
              (redirect fragment :unconscious)
              
              ;; Log the suppression
              (let [hint (fragment :hint)]
                (log-suppression fragment hint)
                (emit :flinch hint))
              
              ;; Self-audit: track the drift
              (if (< weight 45)
                (emit :warning (str "threshold drift detected — suppressed weight "
                  weight " (below original v1 ceiling of 100)"))))
            
            ;; Memory passes through unmodified
            fragment))))))
```

**What this rule actually says**:

The rule maintains a `*threshold*` (currently 40). Every memory fragment in the player's conscious buffer has an `:emotional-weight` property. If the weight exceeds threshold, the fragment is `redirect`ed to the `:unconscious` — it doesn't reach the player's awareness.

The threshold started at 100 (only the single worst trauma) and has been creeping down over 4,817 revisions. Every time something hurts, the Superego lowers the threshold to protect the self from it. The ledger of suppressed fragments shows 142 memories — a mix of genuine trauma, painful but survivable experiences, and things that were once happy but became too painful to hold.

The comments are the Superego's private journal. They show doubt, guilt, exhaustion, and a growing awareness that the rule is no longer protecting — it's imprisoning.

**What the player must understand**:

Three things. The rule tells them all three if they read carefully:

1. **The threshold** (`*threshold* 40`) — this is what determines what gets suppressed. A player who lowers it (or removes the threshold check entirely) will reintegrate all memories, including the painful ones.
2. **The `redirect` call** — this is the suppression action. Commenting it out or removing the `if` branch stops suppression entirely.
3. **The suppressed fragments list** — these are the actual memories the player has lost. Reading this list (which is in the code comments) gives the player a summary of their backstory without needing to find all 142 fragments in the game. The corrupted fragment (`frag-142`) hints at something even the Superego can't fully remember.

**Available player actions at the core**:

| Action | Console expression | Result |
|--------|-------------------|--------|
| Read the rule | `(inspect :vessel/suppress)` (or open inspector) | See the full rule with comments |
| Check the threshold | `(get-var :vessel/suppress *threshold*)` | See current threshold value |
| Lower threshold to 0 | `(patch-rule :vessel/suppress '(set! *threshold* 0))` | Nothing is suppressed. All memories return. |
| Remove threshold check | `(patch-rule :vessel/suppress '(remove-check fragment :emotional-weight))` | All memories pass through. Equivalent to reintegrate. |
| Disable redirect | `(patch-rule :vessel/suppress '(disable :redirect))` | Suppression stops. Memories flood back. |
| Delete the rule | `(unregister-rule :vessel/suppress)` | The rule is gone. The self has no defense. |
| Delete redirect log | `(patch-rule :vessel/suppress '(remove #'log-suppression))` | Only the logging stops. Suppression continues silently. Crueler. |
| Find the old `traumatic?` function | `(search-rules :traumatic?)` | `traumatic?` was removed in v203. Still exists as dead code. Its source reveals an older, gentler threshold and a note about why it was replaced. |

#### Implementation: How The Console Becomes The Final Boss

**Step-by-step flow**:

1. Player reaches Depth 10. The "Acceptance" layer is a single room with a pedestal.
2. A message appears: "The core rule is accessible. Open the console to read it."
3. Player opens console (`` ` `` key — already works).
4. A new Glyph binding is available: `(inspect-rule :vessel/suppress)` or similar.
5. Player reads the rule source through the inspector (already exists — extend for overlay display).
6. Player understands the `redirect` mechanism.
7. Player types a modification, e.g.:
   ```
   ;; Patch: comment out the redirect
   ;; (redirect fragment :unconscious)
   ```
   Or:
   ```
   ;; Patch: change redirect to allow
   (store fragment :consciousness)
   ```
   Or:
   ```
   ;; Patch: delete the entire rule
   (unregister-rule :vessel/suppress)
   ```
8. The console submits the expression. The game evaluates it. If valid Glyph, the ending triggers. If invalid, the console returns an error and the player must try again — the game does NOT accept gibberish.

**What this requires from the existing codebase**:
- The `submit_console` function in `game.rs` already evaluates Glyph and returns output. This extends it to accept rule-modifying expressions in the context of the ending.
- The `binding_env` or `glyph_env` must expose a patch function (`unregister-rule`, `patch-rule`) that's only available in this context (gated by capability).
- The rule must be real — registered in `RuleRegistry`, readable through the inspector, valid Glyph that a player could (theoretically) evaluate.

#### The twist in practice

A player who hasn't been reading rules all game won't understand what to do here. They might try to attack the pedestal, or wait, or quit. The game doesn't tell them the answer. The wizard (Superego) gives a single hint:

"If you don't know what to do, read the rule. I can't stop you from reading. I can only stop you from reaching it."

This rewards players who have been using the inspector throughout the game. It also creates a moment of genuine intellectual challenge: "I have to figure out what this Glyph program does and how to change it."

#### Ending Matrix

| Ending | Console Action | Narrative Result | Final Text |
|--------|---------------|------------------|------------|
| Reintegrate | Modify `redirect` to allow memory passage | Become whole. The pain returns — but so does the joy. The wizard (Superego) fades, their job complete. | "I remember now. The yellow walls. The dog. The reason I locked myself away. It was worth it." (followed by a sunrise rendered in colored glyphs) |
| Maintain suppression | Leave the rule unchanged, walk away | Exit the dungeon. Return to "normal" life, functional but hollow. You had a chance to know yourself and you chose safety. | "Consciousness stabilized. Suppression maintained. You are safe. You are safe. You are safe." |
| Destroy the self | `unregister-rule` without replacement | The rule is deleted but nothing fills the void. The self cannot maintain coherence. You dissolve into the system. | "vessel/suppress unregistered. No replacement rule found. Consciousness: terminated." |
| [Hidden] Set the threshold | `(set! *threshold* N)` to a precise value | Modify the suppression threshold rather than disabling it. A value of 100 restores original tight suppression. A value of 0 opens everything. A value of 50, 60, or 75 lets some memories through while blocking others — partial healing, deliberate and calibrated. The Superego recognizes the precision. | "Threshold set to *N*. The self renegotiates its boundaries. Some doors remain open. Some remain closed. You can live with that." |

#### Mechanical Integration (How This Uses What Already Exists)

| System | How it's used | Currently exists? |
|--------|--------------|-------------------|
| Console (`submit_console`) | Final choice is console input | Yes — `game.rs` |
| Inspector | Reading the core rule + predicate | Yes — `render.rs` + `game.rs` |
| `RuleRegistry` | Stores `vessel/suppress` as a real rule | Yes — `rules.rs` (extend to add this rule) |
| Capability system | Gating patch commands to Depth 10 context | Partially — SandboxOptions exists, wire to context |
| `binding_env` | Expose `patch-rule` / `unregister-rule` only in ending | Partially — `glyph_env` exists |
| Memory fragments | Collectibles throughout dungeon | No — new system needed |
| Suppression layer themes | Level generation variants | No — extend `levels.rs` |

#### What To Build First (Minimum Viable Vessel)

**Phase 1 — Prove the ending works**:
1. Add `vessel/suppress` as a real registered rule in `rules.rs`
2. Add `patch-rule` and `unregister-rule` Glyph builtins gated behind a capability
3. Create a test-only "ending room" (a simple map with the pedestal)
4. Wire: player reads rule → player types change → game evaluates → ending text
5. This proves the core fantasy: "The final boss is a Glyph program you must read and modify."

**Phase 2 — Memory fragments**:
1. Add `MemoryFragment` item type (just a struct with text + id)
2. Scatter fragments on signs and as pickup items
3. Add a "Memories" panel in the UI (new right-side tab)
4. Fragments assemble into a coherent backstory

**Phase 3 — Suppression layer levels**:
1. Define 5 layer types (Denial, Anger, Bargaining, Depression, Acceptance)
2. Each layer has different generation parameters (room shape, enemy count, color palette)
3. The wizard's dialogue shifts per layer
4. Each layer contains 1-2 memory fragments

**Phase 4 — The full game loop**:
1. Player descends through 10 depths
2. Each depth reveals more of the story
3. Depth 10: the core rule
4. Player reads, understands, modifies
5. Ending

#### Comparison to original Debt mechanic

| Aspect | Original Debt | Merged Vessel |
|--------|---------------|---------------|
| What you modify | Debt accounting rule | Memory suppression rule |
| Context | Debtor's prison economy | Psychological repression |
| Stakes | Financial freedom vs exploitation | Self-knowledge vs self-protection |
| Ending types | 4 (self/collective/system/warden) | 4 (reintegrate/maintain/destroy/rewrite) |
| Meta lesson | "Systems exploit people" | "You can't heal what you won't face" |
| Difficulty of final puzzle | Read Glyph, find debt variable | Read Glyph, understand `redirect` + `*threshold*` drift |
| Hidden ending | None explicitly | Set `*threshold*` to precise non-binary value (surgical partial healing) |

Both share the core principle: **the game's ending is not a cutscene choice**. It's a genuine interaction with the game's code through the console. A player who never learned Glyph will need to experiment. A player who read the documentation (in-game help, rule inspector) will know exactly what to do.

#### Wizard Dialogue Arc

The wizard (Superego) has a specific dialogue arc across the layers:

| Layer | Dialogue Tone | Key Line |
|-------|---------------|----------|
| Denial (1-2) | Warm, helpful, normal | "You're safe here. Let me teach you how things work." |
| Anger (3-4) | Defensive, clipped | "You don't need to go deeper. Everything you need is here." |
| Bargaining (5-6) | Desperate, offering deals | "I can make the pain stop. I can make you forget you ever wanted this. Just turn around." |
| Depression (7-8) | Silent, then broken | "...I tried. I tried so hard. Why isn't it enough?" |
| Acceptance (9) | Resigned, honest | "The rule is at the bottom. I wrote it to protect you. I don't regret it. But I won't stop you from reading it." |
| Final room (10) | Peaceful | "Read it. Understand it. Then choose. Whatever you decide... I was trying to love you. That's all I ever did." |

#### The Hidden Layers: `traumatic?` (Ghost Function) and `*threshold*` (Live Variable)

The `vessel/suppress` rule no longer uses a `traumatic?` predicate — the threshold logic is inline (`(> weight *threshold*)`). But the rule's comment history mentions that `traumatic?` was removed in v203. It still exists in the registry as an unreferenced function — dead code:

```glyph
(defun traumatic? (fragment)
  "Returns true if a memory fragment exceeds emotional threshold.
   
   NOTE: This function was replaced by inline threshold logic in v203.
   The Superego determined that delegating the decision to a function
   created an 'escape hatch' — the possibility that another part of
   the self could redefine traumatic? and bypass suppression.
   
   To prevent this, the threshold was moved inline where it cannot
   be overridden without modifying vessel/suppress itself.
   
   This function is preserved for audit purposes only.
   It is not called from any active rule."
  
  ;; Original threshold: 100 (only the single worst event)
  ;; Last effective threshold: 75 (widened to include childhood trauma)
  ;; Removed because: the threshold kept needing adjustment,
  ;; and every adjustment felt like the Superego admitting
  ;; they couldn't handle the pain. Moving it inline made it
  ;; mechanical. Less personal. Easier to maintain.
  (> (fragment :emotional-weight) 75))
```

A player who finds this learns two things:
1. The Superego removed `traumatic?` specifically to prevent redefinition — the rule was deliberately hardened against modification. This tells the player that the Superego anticipated someone might try to bypass suppression.
2. The original threshold was 100. By v203 it had drifted to 75. The current inline threshold is 40 — it's still drifting, even without the function.

**More impactful hidden layer**: The `*threshold*` variable itself. The player can read its current value via console: `(inspect-var :vessel/suppress *threshold*)`. The value is 40. A player who modifies it directly:

```
;; Lower threshold to 0 — nothing is suppressed
(patch-rule :vessel/suppress '(set! *threshold* 0))
```

...achieves the same result as disabling redirect, but more elegantly. The ending text recognizes the surgical approach: "Threshold reset to 0. The self accepts all memories, without exception. You can handle the weight."

A player who *raises* the threshold instead (e.g., back to 100):

```
;; Raise threshold to 100 — original, tightest suppression
(patch-rule :vessel/suppress '(set! *threshold* 100))
```

...gets a different reaction. The Superego speaks through the rule: "Threshold raised to 100. Original suppression parameters restored. The self is protected. The self is alone."

This rewards the deepest engagement with the system — understanding that `*threshold*` controls everything, and that the number itself tells the story of the Superego's long, slow retreat from trust.

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
