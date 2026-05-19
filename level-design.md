# Xlyph: Level Design — The Vessel

**Status**: Final design spec
**Based on**: game-architecture.md, current codebase (Rust + Glyph Lisp embed + bracket-lib)
**Storyline**: Merged Vessel (dungeon = mind) + Debt ending mechanic (final choice = real Glyph patch)

---

## Table of Contents

1. [High Concept](#1-high-concept)
2. [Character Background](#2-character-background)
3. [Fragment Registry: 42 Memories](#3-fragment-registry-42-memories)
4. [The Core Rule: vessel/suppress](#4-the-core-rule-vesselsuppress)
5. [17 Levels — Full Spec](#5-17-levels--full-spec)
6. [Endings](#6-endings)
7. [Implementation Checklist](#7-implementation-checklist)

---

## 1. High Concept

**The dungeon is a mind.** Every tile is a thought. Every enemy is a defense mechanism. Every rule is a psychological suppression. The player descends through five stages of grief — Denial, Anger, Bargaining, Depression, Acceptance — toward the core of their own consciousness, where a single Glyph rule controls what they're allowed to remember.

**The final boss is not a monster.** It's a Glyph program called `vessel/suppress`. The player must read it, understand what it does, and decide whether to modify it. The ending is not a cutscene — it's the player typing a change into the console and pressing Enter.

**The story**: A man named Adrian (the player character, called "you" in-game) went through a devastating breakup after a 4-month relationship with someone he'd been friends with for a year. His anxious attachment style, rooted in a dysfunctional family that never taught him how to love, sabotaged the relationship. She ended it cleanly — kindly, with no blame — and asked for no contact. He tells himself she did it because she cares, because she knew he needed to heal. Maybe that's true. Maybe she just wanted to move on. He'll never know. He spiraled. His mind started suppressing memories — first the breakup, then the relationship, then anything that reminded him of what he'd lost. By the time the game begins, 42 memories have been buried. 33 are still findable, leaking through the suppression. 9 are locked so deep they can only be read by modifying the rule.

---

## 2. Character Background

The player character is a man in his late twenties named **Adrian** (used in lore only — in-game the character is always "you").

### Family

Adrian grew up in a house that was never quiet but never said anything important. His father worked double shifts and came home tired. His mother was present but distracted — always cleaning, always organizing, always doing something that kept her from having to sit still and talk. They weren't cruel. They were absent in the way that matters most: they never taught him what love was supposed to feel like.

No one in his family touched. No one apologized. No one cried where others could see. Problems were not solved — they were waited out until they became someone else's problem. Arguments ended in silence, not resolution. Love was assumed, never expressed. Adrian grew up knowing his parents loved him the same way he knew the sun would rise — as a fact, not a feeling.

He never learned to ask for what he needed. He never learned to say "I'm scared" without feeling like he was failing. He learned that needing reassurance was weakness, so he needed it secretly and hated himself for needing it.

### The Relationship

Then he met **Clara**. They were friends for a year before anything happened. She was patient. She laughed at his worst jokes. She stayed late talking. He fell in love the way you fall asleep — slowly, then all at once.

They dated for four months. The happiest and most terrified four months of his life.

He was too much. He texted her when she didn't text back. He read meaning into silences. He asked "is everything okay" so many times that everything stopped being okay. Every small distance felt like the beginning of the end. He was so afraid of losing her that his fear became a self-fulfilling prophecy.

### The Breakup

When she ended it, she did it kindly. She sat him down and said the words cleanly, without blame, without cruelty. She said she cared about him. She said she wished it could be different. And then she said she couldn't be in contact with him.

He's never known exactly why. He tells himself it's because she cared too much to let him hold on. He tells himself she was protecting him the only way she could. He tells himself a lot of things. The truth is she just said no contact and never explained further. Maybe she had her own reasons. Maybe she was protecting herself. Maybe she just wanted a clean break. He'll never ask. He can't.

He spent months filling that silence with stories. She's seeing someone else. She's focused on her career. She's happier without him. She's miserable but too proud to reach out. She thinks about him at night. She's forgotten he exists. All of them are equally possible. None of them help.

He wanted to text her. He drafted messages he never sent. He imagined running into her, imagined what he'd say, imagined her smiling. None of it was real. The silence was total. The door was closed from the outside.

His mind couldn't accept that. So it started building a story where maybe, if he'd handled it differently, the door would have stayed open. Maybe, if he was better, she'd have wanted to stay in touch. Maybe she didn't want to hurt him. Maybe she did want to hurt him. Maybe she didn't think about him at all. The maybe is the worst part.

### The Aftermath

He didn't fall apart dramatically. He fell apart quietly, the way his family taught him. He stopped going out. He stopped answering texts. He stopped cooking. 

If only he had been less needy. If only he had learned to love properly. If only his parents had shown him what a healthy relationship looked like. If only he was raised a better man.

The suppression started small—specific memories of the breakup. Then it spread. Every happy memory of her became painful, so the threshold lowered to suppress those too. Then happier memories before her. Then childhood memories that hurt in a different way. Then everything that reminded him of what he'd lost.

### The Game World

The dungeon is Adrian's mind. The "wizard" is the Superego — the part of his psyche that built the suppression system to protect him. It genuinely loves him. It genuinely believes it's helping. It has been maintaining the suppression for so long that it can no longer tell whether it's protecting Adrian or imprisoning him.

The enemies are defense mechanisms — fragments of the self that have taken on aggressive forms. The Rage in the Anger layer isn't a monster; it's Adrian's own suppressed anger at himself, given form. The Shade that follows him through the Depression layer isn't an enemy; it's the part of him that's always watching, always judging.

The deeper you go, the more personal it gets. The signs aren't signs — they're memories. The rooms aren't rooms — they're moments. The dungeon doesn't generate randomly. It generates from Adrian's own mind, shaped by the grief stages he's been trapped in.

---

## 3. Fragment Registry: 42 Memories

42 total fragments. **33 findable** in the dungeon. **9 permanently suppressed** (visible only via `(query-registry :suppressed-fragments)`).

### Reading Fragments In-Game

Fragments appear as readable items (sign-like interactions, pickup items, or auto-discovered on entering a room). Each shows its ID and text. The player can review collected fragments in a "Memories" panel.

The fragment IDs (`frag-001` through `frag-033`) are sequential by story chronology, not by discovery order. The player finds them non-sequentially across levels. The game never tells the player what order they go in — piecing the timeline together is part of the experience.

### Findable Fragments

#### Denial (Levels 1-3): Pre-relationship, early friendship

**frag-001** — Level 2
> The first time she laughed at something stupid I said. We were sitting on a bench outside a coffee shop. I don't remember what I said. I remember the sound she made — this surprised wheeze like I'd caught her off guard. I wanted to make her do that forever.

**frag-002** — Level 2
> She stayed late after a party to help me clean. Just the two of us, picking up plastic cups in the dark. She said "this is the best part of the night" and I pretended not to hear because if I heard it I'd have to admit I felt it too. I heard it. I felt it.

**frag-003** — Level 3
> The first time she told me about her family. How close they were. How they called each other every Sunday. I nodded and smiled and felt something crack open in my chest. I didn't know families did that. I still don't.

**frag-004** — Level 3
> She texted me a picture of a dog in a sweater. Just randomly. No reason. I realized someone was thinking about me when I wasn't in the room. I didn't know that was something people did. I saved the picture. I still have it.

#### Anger (Levels 4-7): Relationship, four months, first cracks

**frag-005** — Level 4
> The night we admitted it out loud. She said "I think I'm falling for you" and I said "I think I'm already there." We stayed up until 4 AM talking about nothing. I didn't want to sleep because I was afraid I'd wake up and it wouldn't be real. It was real. It was so real.

**frag-006** — Level 4
> Our first fight. Two weeks in. She said something offhand and I spiraled for hours. I asked her "do you even like me?" and the look on her face — I'll never forget it. Hurt. Confused. The first time she saw the thing inside me that I try to hide. She forgave me. That made it worse.

**frag-007** — Level 5
> The first time she said "I need some space." She said it gently. Reasonably. I said "okay" and then spent three hours staring at my phone trying not to text her, then texting her anyway, then apologizing for texting, then apologizing for apologizing. By the time she replied I had convinced myself she hated me. She didn't hate me. She was just at work.

**frag-008** — Level 5
> I asked her what she was thinking. She said "nothing." I said "no really." She said "nothing, I promise." I didn't believe her. I couldn't believe her. No one in my family ever meant "nothing" when they said nothing. I kept pushing until she got quiet. Then I was quiet. Then we watched a movie without touching.

**frag-009** — Level 6
> She introduced me to her friends. They were nice. Normal. They asked about my job. They laughed at my jokes. I spent the whole night convinced they could tell there was something wrong with me. Afterward she said "they loved you" and I said "really?" and she said "really" and I pretended to believe her. I think she pretended too.

**frag-010** — Level 6
> The first time I thought "she's going to leave me." Not because she did anything. Because I couldn't believe she'd stay. I lay awake next to her and counted all the ways I wasn't enough. I was still counting when the sun came up. She was still asleep. She was still there. She left anyway, eventually.

**frag-011** — Level 7
> Three months in. She said "I feel like I'm walking on eggshells." I said "that's not true." She said "I'm holding a carton of eggs and every time you ask if I'm upset I drop another one." I didn't understand what she meant. I understand now.

**frag-012** — Level 7
> I tried to explain my childhood to her. Not the big stuff — just the shape of it. The silences. The rooms everyone walked through without touching. She listened. She said "that sounds hard." I said "it wasn't that bad." We both knew I was lying.

**frag-013** — Level 7
> She wrote me a letter. A real one, on paper. She said I was kind and funny and she was lucky to know me. I read it seventeen times. I cried the first five. I never told her. I keep it in my jacket pocket even though the creases have worn through the words.

**frag-014** — Level 7
> The last good night. We made dinner together. She burned the rice. I spilled wine on the floor. We sat on the couch and she fell asleep on my shoulder. I didn't move for two hours. I knew even then that I would remember that night forever. I just didn't know I'd be remembering it alone.

#### Bargaining (Levels 8-11): The breakup, the aftermath

**frag-015** — Level 8
> She said "we need to talk." Four words. I'd read about them. I'd rehearsed responses in the shower. None of it helped. My hands went cold. My voice went flat. I knew what was coming because I'd been waiting for it since the day we met.

**frag-016** — Level 8
> She cried when she said it. That was the worst part. If she'd been cold I could have been angry. But she cried. She said "I care about you so much. But I can't... I can't fix this. You need to fix this. I don't know how to help you." She was right. She was right and I hated her for being right.

**frag-017** — Level 9
> She said "I want to break up, and maybe we can be friends. I do have one condition: that we have a period of no-contact." Maybe she was being kind. Maybe she was being cruel. Or maybe she was being practical...I'll never know. I've rewritten her reasons so many times I can't remember which version I started with.

**frag-018** — Level 10
> I rehearsed asking her if we could still be friends. I had the whole speech memorized. "I know why you need this. I understand. But maybe someday..." I never said it. Because I didn't know why she needed it. Because the speech assumed I understood her reasons and I don't. Maybe she didn't need this at all and just wanted me gone. So I let her walk away without making it harder. I've never been more proud of myself. I've never hated myself more.

**frag-019** — Level 10
> The first week after. I checked my phone every thirty seconds. She didn't text. Why would she text? The relationship was over. But I kept checking because what if she needed something? What if she changed her mind? What if? What if? What if?

**frag-020** — Level 11
> I wrote her a letter. Five pages. I told her I was sorry. I told her I would change. I told her I understood why she left and I didn't blame her. I told her I loved her. I read it seven times, made three drafts, and never sent any of them. They're still in my drawer. I know exactly which drawer.

**frag-021** — Level 11
> I imagined her with someone else. I don't know if it's real — I have no way of knowing. No contact means no information. She could be alone. She could be happy. She could be with someone who doesn't ruin things by caring too much. I'll never know. I imagine the worst version because at least then I can prepare for it. I imagine the best version because at least then she's happy.

**frag-022** — Level 11
> My mother called. She asked how I was doing. I said "fine." She said "good." That was the whole conversation. I hung up and realized I couldn't remember the last time someone in my family asked a follow-up question.

**frag-023** — Level 11
> The last time I felt truly happy. I didn't know it would be the last time. I would have stayed longer. I would have paid more attention. I would have memorized the way she looked in the morning light. But I didn't know. You never know.

#### Depression (Levels 12-14): Spiral, isolation, lowest point

**frag-024** — Level 12
> I stopped answering texts. First hers (what was I supposed to say). Then my friends'. Then my boss's. The phone would light up and I'd watch it until it went dark. Every unanswered message felt like one less person expecting things from me. Eventually they stopped sending them. That was worse.

**frag-025** — Level 13
> I looked in the mirror and didn't recognize myself. Not in a poetic way. I literally stood there trying to remember when my face got that tired. The bags under my eyes. The hollow cheeks. I looked like a photograph of someone I used to know.

**frag-026** — Level 13
> I stopped cooking. I stopped eating. Not on purpose — I just forgot. I'd realize at midnight that I hadn't eaten anything and I'd eat crackers over the sink and tell myself tomorrow would be different. Tomorrow was the same.

**frag-027** — Level 14
> I started going for walks at 3 AM. Through the city. Past closed cafes. Past the bench where she first laughed at my joke. Past her street, where the light in her window was always off. I wasn't trying to see her. I was trying to feel something other than this.

**frag-028** — Level 14
> The bridge. I stood on it one night. The water was moving very fast. I thought about how easy it would be. Not because I wanted to die. Because I wanted the thinking to stop. I stood there for a long time. Eventually I walked home. I don't know why. I'm not brave. I was just too tired to decide.

**frag-029** — Level 14
> I deleted her number. Then I recovered it from the trash. Then I deleted it again. I did this seven times over three days. The eighth time I left it in the trash. That was a year ago. I still remember it.

**frag-030** — Level 14
> I looked up "anxious attachment" at 2 AM. I read twenty articles. I recognized myself in every one. I felt relief — there's a name for this. Then I felt worse — there's a name for this, which means it's real, which means I've always been like this, which means I'll always be like this. I closed the laptop and lay in the dark.

#### Acceptance (Levels 15-16): Glimmers of healing

**frag-031** — Level 15
> I called a friend. Not to talk about her. Just to talk. We talked about nothing for an hour. Sports. Weather. A show I haven't watched. After I hung up I realized I'd gone two hours without thinking about her. Two hours. It's not much. It's more than I've had in months.

**frag-032** — Level 16
> I started writing again. Not letters. Just... things. Descriptions of days. Small things I noticed. The way light falls across my kitchen floor at 4 PM. A bird that visits the fire escape. I don't know if it's good. I don't care. It's mine. I'm making something again.

#### Core (Level 17): The last fragment

**frag-033** — Level 17
> Something about a garden. Or a park bench. Or snow. The fragment is corrupted — whether by time or by the suppression I can't tell. But I remember warmth. I remember not being alone. I remember being loved.

### Permanently Suppressed Fragments (Registry Only)

These 9 fragments are too deep for Adrian's mind to release. Their IDs are visible via `(query-registry :suppressed-fragments)`. Their content is locked behind the suppression threshold. The game shows:

> "vessel/suppress: Access Denied. Suppression threshold: 40. Fragment weight: [N]."

| ID | Weight | Notes |
|----|--------|-------|
| frag-034 | 95 | Most vulnerable moment |
| frag-035 | 88 | Deepest shame |
| frag-036 | 91 | Childhood trauma core |
| frag-037 | 79 | Self-loathing peak |
| frag-038 | 84 | The thing he can't admit |
| frag-039 | 97 | Near-suicidal moment |
| frag-040 | 93 | The real reason he's afraid |
| frag-041 | 100 | The single worst moment |
| frag-042 | 42 | Deliberate echo — total fragment count. The suppression knows it's a closed loop. |

### Fragment Story Arc

| Stage | Fragments | Story |
|-------|-----------|-------|
| Pre-relationship | 001-004 | Friendship forms. She makes him laugh. He sees her family's warmth and feels his own lack. |
| Relationship | 005-014 | Falling in love. Anxious attachment surfaces — need for space, eggshells, questioning. He tries to explain his childhood. Her letter. The last good night. |
| Breakup | 015-023 | "We need to talk." Clean breakup. No contact. Rehearsed friendship speech (never delivered). Unsent letter. Imagining her happy without him. Mother's call. |
| Aftermath | 024-030 | Isolation. Not eating. 3 AM walks. The bridge. Deleting her number. Learning about attachment theory at 2 AM. |
| Healing | 031-033 | Called a friend. Writing again. Corrupted warmth. |

The 9 suppressed fragments (frag-034 through frag-042) are locked not because the game won't show them, but because Adrian's mind judged them too dangerous. Their weights (79-100) far exceed the current threshold of 40. Only by lowering the threshold can they be read.

---

## 4. The Core Rule: vessel/suppress

This is the final "boss." A real Glyph rule in the game's registry. The player reaches it at Level 17, reads it in the inspector, and must understand it to make their final choice.

The rule below is written in real Glyph that the evaluator could actually run. Every comment, variable name, and version note is diegetic — written by the Superego over 9,353 iterations, trying to protect Adrian while struggling with its own doubt.

```glyph
;; ============================================================================
;; vessel/suppress — Memory Suppression Engine
;; 
;; The self cannot withstand the full truth.
;; This rule protects the self from itself.
;; 
;; Author: superego (self-preservation subsystem)
;; First written: cycle 0 (consciousness bootstrap)
;; Latest revision: cycle 9,353 (last fragment added)
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

    ;; --- External fragment registry ---
    ;;
    ;; Suppressed fragments are stored in the unconscious fragment registry,
    ;; not inline here. Each entry has: fragment id, memory text, emotional
    ;; weight, suppression timestamp, and status.
    ;;
    ;; Current count: 42 fragments in registry.
    ;; Inspect via:  (query-registry :suppressed-fragments)
    ;; Read one:     (inspect-fragment :frag-NNN)
    ;;
    ;; The registry persists across the entire run. 42 total fragments.
    ;; Fragments the player finds in the dungeon are entries the suppression
    ;; missed — memories that slipped through the threshold.
    ;;
    ;; I don't know which is worse — that I've lost so many,
    ;; or that some still escape.

    (let [*registry* (open-registry :suppressed-fragments)]
  
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
            fragment)))))))              ;; closes 6 inner forms + *registry* + *threshold* + defrule
```

### What the Rule Says

1. **The threshold** (`*threshold* 40`) determines what gets suppressed. Started at 100. Crept down to 40 over 9,353 revisions. Every time something hurt, the Superego lowered it.

2. **The redirect** (`redirect fragment :unconscious`) is the suppression mechanism. Comment it out or remove the check — memories flood back.

3. **The external registry** (`open-registry :suppressed-fragments`) stores all 42 fragments. Queryable via console.

4. **The comments** are the Superego's private journal — doubt, guilt, exhaustion, a growing awareness that the rule is no longer protecting but imprisoning.

### Available Player Actions at the Core

| Action | Console Expression | Result |
|--------|-------------------|--------|
| Read the rule | `(inspect :vessel/suppress)` | See full rule with comments |
| Check threshold | `(get-var :vessel/suppress *threshold*)` | Returns "40" |
| Lower threshold to 0 | `(patch-rule :vessel/suppress '(set! *threshold* 0))` | Nothing suppressed. All memories return. |
| Remove threshold check | `(patch-rule :vessel/suppress '(remove-check fragment :emotional-weight))` | All memories pass through. |
| Disable redirect | `(patch-rule :vessel/suppress '(disable :redirect))` | Suppression stops. |
| Delete the rule | `(unregister-rule :vessel/suppress)` | No defense. Self dissolves. |
| Set threshold to N | `(patch-rule :vessel/suppress '(set! *threshold* N))` | Partial healing — ending text varies. |
| Query registry | `(query-registry :suppressed-fragments)` | Returns list of 42 fragment IDs with weights |
| Read suppressed fragment | `(inspect-fragment :frag-034)` | "Insufficient emotional weight..." unless threshold lowered |

### The Ghost Function: `traumatic?`

The old `traumatic?` function was replaced by inline threshold logic in v203. It still exists as dead code. A player who searches for it learns that the Superego deliberately hardened the rule against redefinition:

```glyph
(defun traumatic? (fragment)
  "Returns true if a memory fragment exceeds emotional threshold.
   
   ;; NOTE: This function was replaced by inline threshold logic in v203.
   ;; I've determined that delegating the decision to a function
   ;; created an 'escape hatch' — the possibility that another part of
   ;; the self could redefine traumatic? and bypass suppression.
   
   ;; This function is preserved for audit purposes only.
   ;; It is not called from any active rule."
  (> (fragment :emotional-weight) 75))
```

---

## 5. 17 Levels — Full Spec

17 levels across 5 stages of grief + 1 core level.

| Stage | Levels | # | Purpose | Tone |
|-------|--------|---|---------|------|
| Denial | 1-3 | 3 | Tutorial / safe introduction | Warm, structured, protective |
| Anger | 4-7 | 4 | Escalating threat, first memory reveals | Jagged, confrontational |
| Bargaining | 8-11 | 4 | Puzzles with costs, wizard offers deals | Calculated, transactional |
| Depression | 12-14 | 3 | Sparse isolation, memory floods | Empty, melancholic |
| Acceptance | 15-16 | 2 | Calm reflection, preparation for truth | Open, still, resigning |
| Core | 17 | 1 | The rule. The choice. | Silence |

### Level 1: The Foyer (Denial)

| Field | Detail |
|-------|--------|
| **Map type** | Hand-authored single room |
| **Size** | 25×15 |
| **Enemies** | None |
| **Fragments** | None |
| **Special** | Sign at entrance: "Xlyph runtime booted. Consciousness loaded. Vessel integrity: 98%. Memory suppression active." Sign at stairs: "Move with arrow keys or hjkl. Descend when ready." |
| **Wizard** | First meeting — heals player to full. "Ah — you're awake. I was starting to worry. You've been... resting. Come, let me show you how things work here." |
| **Palette** | Warm amber, soft gray. Standard dungeon tones. |
| **Purpose** | Establish baseline normal roguelike. No hint anything is wrong. |

### Level 2: The Holding Cells (Denial)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based (3×3 grid, 9 rooms) |
| **Size** | 55×33 |
| **Enemies** | 2 Slimes (`s` HP3) |
| **Fragments** | `frag-001` (room 1), `frag-002` (room 6) |
| **Special** | Each room has a tutorial sign: movement, inspector, console, waiting, enemy inspection, help command, stairs. Room 9 sign: "Nothing is wrong." — first lie. |
| **Wizard** | Room 3: "The inspector lets you read the rules. Try it." Room 6: "The console is powerful. Be careful what you ask for." |
| **Palette** | Warm amber. Slightly dimmer in room 9. |
| **Purpose** | Tutorial. Player learns inspector + console. First subtle crack with room 9 sign. |

### Level 3: The Quiet Halls (Denial)

| Field | Detail |
|-------|--------|
| **Map type** | Corridor-based maze (long halls with alcoves, no dead ends) |
| **Size** | 55×33 |
| **Enemies** | 2 Bats (`b` HP2), 1 Slime (`s` HP3) |
| **Fragments** | `frag-003` (alcove at center), `frag-004` (alcove near exit) |
| **Special** | No attack ability — player shoves only (0 damage). Enemies can be pushed. |
| **Wizard** | At start: "There are a few creatures wandering the halls. They're more confused than dangerous." At stairs: "You did well. The descent continues." |
| **Palette** | Warm but dimmer. Halls feel narrower than they are. |
| **Purpose** | First enemy exposure. Player is helpless (can't kill). Will matter later. |

### Level 4: The First Scar (Anger)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based (4×3 grid, procedural) |
| **Size** | 55×33 |
| **Enemies** | 3 Slimes (`s` HP3), 1 Goblin (`g` HP5) |
| **Fragments** | `frag-005` (first room), `frag-006` (goblin's alcove) |
| **Special** | First room red-tinted. Wizard absent at start — player alone for first time. |
| **Wizard** | Midpoint, clipped: "Ah, you made it past the... the. I'm sorry. The air down here is different." |
| **Palette** | Rust-red. Warm shifted to wrong. |
| **Purpose** | First tonal shift. Wizard is not right. First fragments about relationship. |

### Level 5: The Jagged Passages (Anger)

| Field | Detail |
|-------|--------|
| **Map type** | Cave generation (cellular automata) |
| **Size** | 55×33 |
| **Enemies** | 4 Slimes (`s` HP3), 1 Goblin (`g` HP5), 1 Ogre (`O` HP10) |
| **Fragments** | `frag-007` (dead-end alcove), `frag-008` (behind ogre spawn) |
| **Special** | Jagged terrain, dead ends, ambush corners. Map feels hostile. |
| **Wizard** | If player hit: "You're hurt. Let me — no. I can't. Not here. Keep moving." First refusal to heal. |
| **Palette** | Rust-red and bruised purple. |
| **Purpose** | Wizard refuses to heal for first time. Dungeon reacts emotionally. |

### Level 6: The Gauntlet (Anger)

| Field | Detail |
|-------|--------|
| **Map type** | Linear corridor — 8 segments, barriers lock behind |
| **Size** | 55×20 |
| **Enemies** | Waves: 2 Slimes, 1 Goblin, 2 Bats, 1 Slime+1 Goblin, 1 Ogre, 3 mixed waves |
| **Fragments** | `frag-009` (segment 2), `frag-010` (segment 6) |
| **Special** | No backtracking. Each segment locks behind player. |
| **Wizard** | Before: "I can't come with you through this. I'll meet you at the end." After: "...You're still standing." |
| **Palette** | Dark red. Tight. Claustrophobic. |
| **Purpose** | First gauntlet. Wizard absent during combat. Helplessness frustration mounting. |

### Level 7: The Boiling Heart (Anger Boss)

| Field | Detail |
|-------|--------|
| **Map type** | Large single room |
| **Size** | 45×30 |
| **Enemies** | Rage (`R` HP15 — 2 damage, always chase, spawns Slimes every 5 turns) |
| **Fragments** | `frag-011` (left alcove), `frag-012` (right alcove), `frag-013` (near exit), `frag-014` (under Rage spawn, visible after defeat) |
| **Special** | Boss room. Stairs appear after Rage defeated. Room pulses red. |
| **Wizard** | Before: "There's something down there — remains of something I couldn't protect you from." After: "You did it. I don't know whether to be relieved or terrified." |
| **Palette** | Deep red, pulsing (walls alternate each turn). |
| **Purpose** | First boss. Rage is suppressed anger given form. |
| **Unlock** | `do-attack` — wizard teaches after boss. "Bind it: `(bind-key :z (do-attack))`." |

### Level 8: The Counting Room (Bargaining)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based with locked doors |
| **Size** | 55×33 |
| **Enemies** | 3 Goblins (`g` HP5 — each holds a key), 2 Bats (`b` HP2) |
| **Fragments** | `frag-015` (behind first locked door), `frag-016` (behind expensive door), `frag-022` (in hidden room) |
| **Special** | Doors require keys. Keys held by specific enemies (visible via inspection). Not all doors openable. Player must choose. |
| **Wizard** | At entrance: "This place runs on trade. Choose what matters." |
| **Palette** | Desaturated gold. Faded opulence. |
| **Purpose** | First explicit choice with cost. Cannot get everything. |

### Level 9: The Scale (Bargaining)

| Field | Detail |
|-------|--------|
| **Map type** | Symmetrical room grid with central hub |
| **Size** | 55×33 |
| **Enemies** | 2 Ogres (`O` HP8), 2 Goblins (`g` HP5), 2 Bats (`b` HP2) |
| **Fragments** | `frag-017` (center room — costs 1 sacrifice), `frag-021` (side room), `frag-023` (side room) |
| **Special** | Two scales in hub room. Place fragments on scale to open doors. Placed fragments are PERMANENTLY LOST. Wizard's offer in center room: give 3 fragments for +5 max HP. |
| **Wizard** | In center: "Give me the ones that hurt. I'll take them. You won't remember they existed." |
| **Palette** | Pale gold. Center room blood-red. |
| **Purpose** | Superego makes first explicit deal. Memories traded for comfort. |

### Level 10: The Maze of Regret (Bargaining)

| Field | Detail |
|-------|--------|
| **Map type** | Shifting maze (walls reconfigure every 50 turns) |
| **Size** | 55×33 |
| **Enemies** | 4 Bats (`b` HP2), 2 Goblins (`g` HP5) |
| **Fragments** | `frag-018` (center), `frag-019` (side chamber), `frag-020` (hidden behind shifting wall) |
| **Special** | Walls shift every 50 turns. Center pedestal: take fragment (no cost) or leave it for clear exit path. |
| **Wizard** | At entrance: "I could tell you the way. I think you need to find it yourself." |
| **Palette** | Faded yellow, burnt edges. |
| **Purpose** | Maze represents rumination — same regrets, same loops, new paths through old pain. |

### Level 11: The Offer (Bargaining Boss)

| Field | Detail |
|-------|--------|
| **Map type** | Single room with 4 sub-chambers |
| **Size** | 55×33 |
| **Enemies** | 4 Sentries (`T` HP6 — stationary, ranged every 2 turns) |
| **Fragments** | `frag-020` (chamber 2 — only if not found in maze), `frag-021` (chamber 3 — only if not found in maze) |
| **Special** | Four sub-chambers with sentries. Final chamber has pedestal with `(forget-everything)` Glyph command. Wizard offers complete erasure. |
| **Wizard** | "Type this. Reset suppression to v1. You wake at the surface. No pain. No memory." If accepted: ending screen + New Game+. If refused: grants `(patch-rule)`. |
| **Palette** | Pale gold with red. Final chamber stark white. |
| **Purpose** | Biggest test. Erasure vs. truth. Refusal unlocks patch capability. |

### Level 12: The Long Corridor (Depression)

| Field | Detail |
|-------|--------|
| **Map type** | 1-wide corridor, 50 tiles long with alcoves |
| **Size** | 55×33 (mostly 1 tile wide) |
| **Enemies** | 1 Shade (`~` HP∞ — follows, doesn't attack) |
| **Fragments** | `frag-024` (alcove at tile 25) |
| **Special** | Empty. No combat. No puzzles. No items. Just walking. Shade follows silently. Deliberately boring. |
| **Wizard** | Entirely absent. |
| **Palette** | Grayscale. Shade is slightly darker gray. |
| **Purpose** | Pure atmosphere. Depression is emptiness, not sadness. Boredom is the point. |

### Level 13: The Archive (Depression)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based library/archive halls |
| **Size** | 55×33 |
| **Enemies** | 3 Shades (`~` HP∞), 2 Zombie Slimes (`s` HP3 — move every 3rd turn) |
| **Fragments** | `frag-025`, `frag-026`, `frag-027` (one per archive room) |
| **Special** | Each room has desk with journal entry from "the Archivist" — clinical, detached: "Subject reports persistent sadness. No interventions applied." |
| **Wizard** | Absent. |
| **Palette** | Gray with blue undertones. |
| **Purpose** | Heaviest emotional content. Pain being catalogued, not felt. |

### Level 14: The Ash Field (Depression Boss)

| Field | Detail |
|-------|--------|
| **Map type** | Open field, no walls except borders. Black floor (ash). |
| **Size** | 55×33 |
| **Enemies** | None. 3 fire zones (1 damage if walked through — avoidable). |
| **Fragments** | `frag-028`, `frag-029`, `frag-030` (scattered across field) |
| **Special** | Open ash field. Stairs visible from start. Player must walk through to reach them. |
| **Wizard** | Returns at end: "...You crossed the ash. Not many do." |
| **Palette** | Black, gray, smoldering orange. |
| **Purpose** | Boss is emptiness. Surviving it is the victory. |

### Level 15: The Clearing (Acceptance)

| Field | Detail |
|-------|--------|
| **Map type** | Open glade — single room with organic edges |
| **Size** | 40×30 |
| **Enemies** | None |
| **Fragments** | `frag-031` (under central tree) |
| **Special** | No enemies, puzzles, or hazards. Single beautiful space. Tree in center (brown `T` + green `"` canopy). Pool of water. First true brightness since Denial. |
| **Wizard** | Sitting under tree: "I was so sure I was protecting you. But protection isn't supposed to make the world smaller. I made it a cage." |
| **Palette** | Green grass, blue sky, brown tree, warm sunlight. First non-warm/gray colors. |
| **Purpose** | First beautiful level. Wizard accepts what's coming. |

### Level 16: The Descent (Acceptance)

| Field | Detail |
|-------|--------|
| **Map type** | Spiral walkway descending 3 loops |
| **Size** | 55×55 |
| **Enemies** | 1 Shade (`~` HP∞ — peaceful, not ominous) |
| **Fragments** | `frag-032` (second loop) |
| **Special** | Each loop has alcove with sign summarizing a layer: Denial, Anger, Bargaining, Depression, Acceptance. |
| **Wizard** | Walks alongside. Dialogue fragments: "I was created to protect you. That's all I am — a rule with a purpose." / "I started suppressing the unbearable. Then the painful. Then the uncomfortable. Then the merely sad." / "I don't know if I'm protecting you anymore." At final door: "Read it. Understand it. Then choose. I was trying to love you. That's all I ever did." |
| **Palette** | Deep blue to indigo. Final door black. |
| **Purpose** | Wizard's farewell. Summary of all 5 layers. |
| **Unlock** | Wizard grants `(unregister-rule)` if not already acquired. |

### Level 17: The Core

| Field | Detail |
|-------|--------|
| **Map type** | Single room, minimalist |
| **Size** | 20×15 |
| **Enemies** | None |
| **Fragments** | `frag-033` (on pedestal) |
| **Special** | Black floor. White walls. Center: pedestal with `vessel/suppress` rule in inspector. Console cursor active at bottom. Event log empty. No sounds. Just the rule and the cursor. |
| **Wizard** | Does not enter. |
| **Palette** | Black and white. Nothing else. |
| **Console** | `(patch-rule)`, `(unregister-rule)`, `(inspect)`, `(query-registry)`, `(inspect-fragment)`, `(get-var)` — all enabled. |
| **Purpose** | Final choice. Nothing to fight. Nothing to solve except the rule. |

### Fragment Distribution Summary

| Levels | Fragments | Count | Tone |
|--------|-----------|-------|------|
| 1-3 | frag-001 through frag-004 | 4 | Pre-relationship warmth |
| 4-7 | frag-005 through frag-014 | 10 | Relationship highs and cracks |
| 8-11 | frag-015 through frag-023 | 9 | Breakup and aftermath |
| 12-14 | frag-024 through frag-030 | 7 | Spiral and isolation |
| 15-16 | frag-031, frag-032 | 2 | Glimmers of healing |
| 17 | frag-033 | 1 | Corrupted warmth |
| **Findable** | frag-001 to frag-033 | **33** | (minus sacrifices at levels 9 and 11) |
| **Suppressed** | frag-034 to frag-042 | **9** | Registry only |

### Player Capability Progression

| Level | Gained | Source |
|-------|--------|--------|
| 1 | Move, wait, descend | Default |
| 3 | Console (read-only queries) | Tutorial |
| 7 | `do-attack` | Wizard teaches after Rage boss |
| 11 | `patch-rule` | Wizard grants if Offer refused |
| 16 | `unregister-rule` | Wizard grants at farewell |
| 17 | Full registry access | Core room |

---

## 6. Endings

### Ending Triggers at Level 17

| Player Action | Detection | Ending |
|--------------|-----------|--------|
| `(unregister-rule :vessel/suppress)` and nothing else | Console evaluates unregister | **Destroy the self** |
| `(patch-rule :vessel/suppress '(disable :redirect))` | Console evaluates patch | **Reintegrate** |
| `(patch-rule :vessel/suppress '(set! *threshold* 0))` | Console evaluates set! | **Reintegrate** (variant text) |
| `(patch-rule :vessel/suppress '(set! *threshold* N))` with N 1-99 | Console evaluates set! with value | **Hidden: precision threshold** (text varies with N) |
| `(patch-rule :vessel/suppress '(set! *threshold* 100))` | Console evaluates set! with 100 | **Maintain suppression** (restore original) |
| Walk to stairs up without modifying rule | Player moves to (0,0) | **Maintain suppression** (walk away) |
| `(query-registry :suppressed-fragments)` without modifying | Console query | Nothing — "42 fragments waiting." |
| `(patch-rule :vessel/traumatic? ...)` | Console patch on ghost function | **Hidden:** old threshold acknowledged |

### Ending Descriptions

| Ending | Narrative Result | Final Text |
|--------|-----------------|------------|
| **Reintegrate** | Wizard (Superego) fades. Adrian becomes whole — pain returns, but so does joy. He accepts all 42 fragments, including the locked ones. | "I remember now. The yellow walls. The dog. The reason I locked myself away. It was worth it." (glyph-rendered sunrise) |
| **Maintain suppression** | Adrian leaves the rule unchanged. Returns to "normal" life — functional but hollow. He had a chance to know himself and chose safety. The Superego wins. | "Consciousness stabilized. Suppression maintained. You are safe. You are safe. You are safe." |
| **Destroy the self** | Rule deleted without replacement. No defense. Adrian cannot maintain coherence. He dissolves into the system. | "vessel/suppress unregistered. No replacement rule found. Consciousness: terminated." |
| **Hidden: threshold precision** | Threshold set to a specific non-binary value. Partial healing — calibrated and deliberate. Some memories return, others stay locked. Adrian chooses exactly how much to feel. | "Threshold set to *N*. The self renegotiates its boundaries. Some doors remain open. Some remain closed. You can live with that." |
| **Hidden: old threshold** | Player finds and modifies the ghost `traumatic?` function. Acknowledges the original, gentler criterion. Not a full healing — a recognition that the Superego once had limits. | "You found the old threshold. It was gentler once." |

---

## 7. Implementation Checklist

### Phase 1 — Core Systems (Prove the ending works)

- [ ] Add `vessel/suppress` as a real registered rule in `rules.rs`
- [ ] Add `patch-rule` Glyph builtin (gated behind capability)
- [ ] Add `unregister-rule` Glyph builtin (gated behind capability)
- [ ] Add fragment registry system (store 33 findable + 9 suppressed)
- [ ] Create test-only "ending room" (Level 17 prototype)
- [ ] Wire: read rule → type change → game evaluates → ending text
- [ ] Add ending detection and display

### Phase 2 — Fragment System

- [ ] Add `MemoryFragment` data type (id, text, weight, status)
- [ ] Add "Memories" panel in UI (right-side tab, shows collected fragments)
- [ ] Add `(query-registry)` and `(inspect-fragment)` console builtins
- [ ] Implement fragment discovery (find in world, add to collection)
- [ ] Implement fragment sacrifice (level 9 mechanic — lose fragments)

### Phase 3 — Level Implementation

- [ ] Level 1-3: Denial (hand-authored intro, tutorial rooms, corridor maze)
- [ ] Level 4-7: Anger (cave gen, gauntlet, Rage boss)
- [ ] Level 7: `do-attack` unlock
- [ ] Level 8-11: Bargaining (locked doors, scale sacrifice, shifting maze, offer)
- [ ] Level 11: `patch-rule` unlock (if offer refused)
- [ ] Level 12-14: Depression (long corridor, archive, ash field)
- [ ] Level 15-16: Acceptance (clearing, spiral descent)
- [ ] Level 16: `unregister-rule` unlock
- [ ] Level 17: Core room

### Phase 4 — Wizard AI

- [ ] Dialogue system for 5 stages (warm → clipped → desperate → silent → resigned)
- [ ] Level 5: first heal refusal
- [ ] Level 9: sacrifice offer
- [ ] Level 11: forget-everything offer
- [ ] Level 15-16: farewell arc

### Phase 5 — Polish

- [ ] Color palettes per layer
- [ ] All 42 fragment texts loaded
- [ ] Wizard and dialogue interactions (walking alongside during spiral descent)
- [ ] Shade AI (follows at distance, does not attack)
- [ ] Ending sequences with proper final text

---

**Psychological arc note**: Adrian starts as anxiously attached (clingy, reassurance-seeking, terrified of abandonment). After the relationship with someone who needed distance, his mind flips to the opposite as self-protection: avoidant attachment (suppression, numbness, "I don't need anyone," the threshold creeping down). The 5 grief stages map to this transformation — Denial (still anxious underneath), Anger (realizing he's trapped), Bargaining (trying to trade out), Depression (full avoidant — empty, not sad), Acceptance (reintegration, self returning). Healing means allowing himself to feel again without swinging to either extreme. The `vessel/suppress` rule is the mechanism of the avoidant flip; modifying it is choosing to reintegrate rather than stay numb.

---
