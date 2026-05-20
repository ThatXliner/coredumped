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

**The story**: A man named Adrian (the player character, called "you" in-game) went through a devastating breakup after a 4-month relationship with someone he'd been friends with for a year. His anxious attachment style, rooted in a dysfunctional family that never taught him how to love, sabotaged the relationship. She ended it cleanly — kindly, with no blame — and asked for no contact. He tells himself she did it because she cares, because she knew he needed to heal. Maybe that's true. Maybe she just wanted to move on. He'll never know. He spiraled. His mind started suppressing memories — first the breakup, then the relationship, then anything that reminded him of what he'd lost. By the time the game begins, 42 memories have been buried. 33 are still findable, leaking through the suppression. 9 are locked so deep they can only be glimpsed through corrupt decryption — fragments of fragments, never the full picture.

---

## 2. Character Background

The player character is a man in his late twenties named **Adrian** (used in lore only — in-game the character is always "you").

### Family

Adrian grew up in a house that was never quiet but never said anything important. His father was always at work — long hours, late nights, coming home after dinner was cold. When Adrian heard the front door at 9 PM he learned not to go downstairs. Dad was tired. Dad needed to decompress. Dad didn't have energy for questions about Adrian's day. The love was there, probably, but it arrived too late to be useful.

His mother was the opposite problem — present, but checked out in a different way. She filled her days with the machinery of the household: cleaning, organizing, meal prep, laundry. She kept the house running perfectly and was too exhausted by the maintenance of it to actually live in it. When Adrian got to high school and could feed himself and get himself to school, she took a job — nothing ambitious, just something to get her out of the house. She had been so bored for so long she'd forgotten she was allowed to want things. After that, Adrian came home to an empty house most days. No one asked about homework. No one made snacks. No one was there.

To get a family together...they did try, once. Family trips — a long weekend at a lake house, a week at the beach. Everyone in the same space. But the kids didn't appreciate it at the time. They sat in separate rooms. They complained about the Wi-Fi. They didn't recognize the value of what they were being given. After the final trip where the tension outlasted the fun, the parents stopped planning them. No one ever said "let's not do this anymore." They just stopped. And the silence was agreement enough.

His parents didn't do much together either. Separate lives under the same roof. His mother took art classes twice a week — how to dance salsa, then do acrylic painting, then whatever was offered at the community center. His father played basketball with friends on Saturdays and golf when the weather was good. They orbited the same house without colliding. Sometimes they argued — sharp, quiet fights behind the bedroom door that Adrian could hear the shape of but never the words. He never saw them apologize to each other.

No one in his family touched. No one apologized. No one cried where others could see. Problems were not solved. They were waited out until they became someone else's problem. Arguments ended in silence, not resolution. Love was assumed, never expressed. Adrian grew up knowing his parents loved him the same way he knew the sun would rise—as a fact, not a feeling.

He never learned to ask for what he needed. He never learned to say "I'm scared" without feeling like he was failing. He learned that needing reassurance was weakness, so he needed it secretly and hated himself for needing it.

### The Relationship

Then he met **Clara**. He didn't know it at the time (as he thought it was just a random conversation with a stranger somewhere neither of them planned to be), but they clicked. They finished each other's sentences without trying. She laughed at things he said that no one else laughed at. He caught himself looking for her in school. It took him months to admit to himself what that meant.

They were friends for a year before anything happened. She was patient. She laughed at his worst jokes. She stayed late talking. He fell in love the way you fall asleep — slowly, then all at once.

The summer before they started dating, they spent a month together at the same camp as counselors where both of them were assigned to the same age group. It was four weeks of shared sunburn and bug-bitten ankles and late-night conversations on a cabin porch. He learned what she looked like in the morning. She learned that he couldn't whistle. They taught kids how to tie knots and paddle straight. When the month ended, they hugged goodbye and said they should keep in touch, meaning it for once.

They dated for four months. The happiest and most terrified four months of his life.

He was too much. He texted her when she didn't text back. He read meaning into silences. He asked "is everything okay" so many times that everything stopped being okay. Every small distance felt like the beginning of the end. He was so afraid of losing her that his fear became a self-fulfilling prophecy.

### The Breakup

When she ended it, she did it kindly. She sat him down and said the words cleanly, without blame, without cruelty. She said she cared about him. She said she wished it could be different. And then she said she couldn't be in contact with him.

He's never known exactly why. He tells himself it's because she cared too much to let him hold on. He tells himself she was protecting him the only way she could. He tells himself a lot of things. The truth is she just said no contact and never explained further. Maybe she had her own reasons. Maybe she was protecting herself. Maybe she just wanted a clean break. He'll never ask. He can't.

He spent months filling that silence with stories. She's seeing someone else. She's focused on her career. She's happier without him. She's miserable but too proud to reach out. She thinks about him at night. She's forgotten he exists. All of them are equally possible. None of them help.

He wanted to text her. He drafted messages he never sent. He imagined running into her, imagined what he'd say, imagined her smiling. None of it was real. The silence was total. The door was closed from the outside.

His mind couldn't accept that. So it started building a story where maybe, if he'd handled it differently, the door would have stayed open. Maybe, if he was better, she'd have wanted to stay in touch. Maybe she didn't want to hurt him. Maybe she did want to hurt him. Maybe she didn't think about him at all. The maybe is the worst part.

### The Aftermath

He fell apart quietly, the way his family taught him. He stopped going out. He stopped answering texts. He stopped cooking. 

If only he had been less needy. If only he had learned to love properly. If only his parents had shown him what a healthy relationship looked like. If only he was raised a better man.

The suppression started with the small things like specific memories of the breakup. Then it spread. Every happy memory of her became painful, so the threshold lowered to suppress those too. Then happier memories before her. Then childhood memories that hurt in a different way. Then everything that reminded him of what he'd lost.

### The Game

The dungeon is Adrian's mind. The "wizard" is the Superego, the part of his psyche that built the suppression system to protect him. It genuinely loves him. It genuinely believes it's helping. It has been maintaining the suppression for so long that it can no longer tell whether it's protecting Adrian or imprisoning him.

The enemies are defense mechanisms, fragments of the self that have taken on aggressive forms. The Rage in the Anger layer isn't a monster; it's Adrian's own suppressed anger at himself, given form. The Shade that follows him through the Depression layer isn't an enemy; it's the part of him that's always watching, always judging.

The deeper you go, the more personal it gets. The dungeon doesn't generate randomly. It generates from Adrian's own mind, shaped by the grief stages he's been trapped in.

---

## 3. Fragment Registry: 42 Memories

42 total fragments. **33 findable** in the dungeon. **9 permanently suppressed** (visible only via `(query-registry :suppressed-fragments)`).

### Reading Fragments In-Game

Fragments appear as readable items (sign-like interactions, pickup items, or auto-discovered on entering a room). Each shows its ID and text. The player can review collected fragments in a "Memories" panel.

The fragment IDs (`frag-001` through `frag-033`) are sequential by story chronology, not by discovery order. The player finds them non-sequentially across levels. The game never tells the player what order they go in — piecing the timeline together is part of the experience.

### Findable Fragments

#### Denial: Pre-relationship, early friendship

**frag-001** — Level 6 (Gauntlet)
> I don't remember the first conversation we ever had. Just some random place where two people started talking and neither of them knew yet. But I remember the first time she laughed at something stupid I said. We were sitting on a bench outside a coffee shop. We'd only known each other a few months, still in that phase where everything the other person said felt like a discovery. I don't remember what I said but I remember the sound she made: this surprised wheeze like I'd caught her off guard. I wanted to make her do that forever.

**frag-002** — Level 7 (Boiling Heart)
> She stayed late after a party to help me clean. Just the two of us, picking up plastic cups in the dark. She said "this is the best part of the night" and I pretended not to hear because if I heard it I'd have to admit I felt it too. But I did heard it. I also felt it.

**frag-003** — Level 8 (Counting Room)
> The first time she told me about her family. How close they were. How they called each other every Sunday. How they still took family trips, not out of obligation, but because they genuinely liked being together. I nodded and smiled and felt something crack open in my chest. I didn't know families did that. I still don't.

**frag-004** — Level 8 (Counting Room)
> She texted me a picture of a dog in a sweater. Just randomly. No reason. I realized someone was thinking about me when I wasn't in the room. I did not know that was something people did. I have the picture saved, and even to this day.

#### Anger: Relationship, four months, first cracks

**frag-005** — Level 9 (The Scale)
> After a Friday night football game, we admitted it over text. "I know there isn't a homecoming dance this year, but if there was, I would've asked you." "Really? And I would've said yes." I stared at my phone for ten minutes just smiling. I didn't know a person could feel this warm.

**frag-006** — Level 9 (The Scale)
> Our first real date. She picked a diner open late. We sat in a booth with sticky menus and she dared me to order the weirdest thing on the menu. I got a tuna melt. She said that was the most boring choice possible. She ordered a milkshake and let me have the first sip. I don't remember what we talked about. I remember thinking "I want this forever" and being too scared to say it out loud.

**frag-007** — Level 10 (Maze of Regret)
> She sent me a playlist. Called it "songs that remind me of you." I listened to it on repeat for three days. Each song felt like a message I had to decode. By the third day I realized there was nothing to decode — she just liked me and wanted me to know. I didn't know people did that. I didn't know you could just... tell someone you liked them, without it meaning something else. I still have the playlist.

**frag-008** — Level 10 (Maze of Regret)
> We went to a farmer's market on a Saturday morning. She bought strawberries and fed me one. She laughed at my face — too sour. I laughed at her laugh. It was a good day. But on the walk back I went quiet and she noticed. She asked what was wrong. I said nothing. She said "you're doing the thing again." I didn't know what "the thing" was. She said "you go somewhere I can't follow." She wasn't mad. She was just sad. That was worse.

**frag-009** — Level 10 (Maze of Regret)
> She introduced me to her friends. They were nice. Normal. They asked about my job. They laughed at my jokes. I spent the whole night convinced they could tell something was wrong with me. Afterward she said "they loved you" and I said "really?" and she said "really." I wanted to believe her. I couldn't. Not because of anything she did — because I didn't know how to believe someone could stay.

**frag-010** — Level 11 (The Offer)
> The first time I thought "she's going to leave me." Not because she did anything. Because I couldn't believe she'd stay. I lay awake next to her and counted all the ways I wasn't enough. I was still counting when the sun came up. She was still asleep. She was still there. She left anyway, eventually.

**frag-011** — Level 11 (The Offer)
> Three months in. She said "I feel like I'm walking on eggshells." I said "that's not true." She said "I'm holding a carton of eggs and every time you ask if I'm upset I drop another one." I didn't understand what she meant. I understand now.

**frag-012** — Level 11 (The Offer)
> I tried to explain my childhood to her. Not the big stuff — just the shape of it. Dad coming home at 9 PM too tired to talk. Mom filling her days with chores until she finally took a job and left me with an empty house. The family trips we stopped taking because nobody knew how to be together. The arguments I could hear the shape of but never the words, ending in silence instead of apology. The rooms everyone walked through without touching. She listened. She said "that sounds hard." I said "it wasn't that bad." We both knew I was lying.

**frag-013** — Level 12 (Long Corridor)
> She wrote me a letter. A real one, on paper. She said I was kind and funny and she was lucky to know me. I read it seventeen times. I cried the first five. I never told her. I keep it in my jacket pocket even though the creases have worn through the words.

**frag-014** — Level 12 (Long Corridor)
> The last good night. We made dinner together. She burned the rice. I spilled wine on the floor. We sat on the couch and she fell asleep on my shoulder. I didn't move for two hours. I knew even then that I would remember that night forever. I just didn't know I'd be remembering it alone.

#### Bargaining: The breakup, the aftermath

**frag-015** — Level 12 (Long Corridor)
> She said "we need to talk." Four words. I'd read about them. I'd rehearsed responses in the shower. None of it helped. My hands went cold. My voice went flat. I knew what was coming because I'd been waiting for it since the day we met.

**frag-016** — Level 13 (The Archive)
> She cried when she said it. That was the worst part. If she'd been cold I could have been angry. But she cried. She said "I care about you so much. But I can't... I can't fix this. You need to fix this. I don't know how to help you." She was right. She was right and I hated her for being right.

**frag-017** — Level 13 (The Archive)
> She said "I want to break up, and maybe we can be friends. I do have one condition: that we have a period of no-contact." Maybe she was being kind. Maybe she was being cruel. Or maybe she was being practical...I'll never know. I've rewritten her reasons so many times I can't remember which version I started with.

**frag-018** — Level 13 (The Archive)
> I rehearsed asking her if we could still be friends. I had the whole speech memorized. "I know why you need this. I understand. But maybe someday..." I never said it. Because I didn't know why she needed it. Because the speech assumed I understood her reasons and I don't. Maybe she didn't need this at all and just wanted me gone. So I let her walk away without making it harder. I've never been more proud of myself. I've never hated myself more.

**frag-019** — Level 14 (Ash Field)
> The first week after. I checked my phone every thirty seconds. She didn't text. Why would she text? The relationship was over. But I kept checking because what if she needed something? What if she changed her mind? What if? What if? What if?

**frag-020** — Level 14 (Ash Field)
> I wrote her a letter. Five pages. I told her I was sorry. I told her I would change. I told her I understood why she left and I didn't blame her. I told her I loved her. I read it seven times, made three drafts, and never sent any of them. They're still in my drawer. I know exactly which drawer.

**frag-021** — Level 14 (Ash Field)
> I imagined her with someone else. I don't know if it's real — I have no way of knowing. No contact means no information. She could be alone. She could be happy. She could be with someone who doesn't ruin things by caring too much. I'll never know. I imagine the worst version because at least then I can prepare for it. I imagine the best version because at least then she's happy.

**frag-022** — Level 14 (Ash Field)
> My mother called. She asked how I was doing. I said "fine." She said "good." I almost told her the truth — almost. But I grew up in a house where you didn't bring your problems to the dinner table. Except by high school there was no dinner table. She was at work. Dad was at work. I was alone. So I said "fine" and she said "good" and we hung up. I couldn't remember the last time someone in my family asked a follow-up question.

**frag-023** — Level 14 (Ash Field)
> The last time I felt truly happy. I didn't know it would be the last time. I would have stayed longer. I would have paid more attention. I would have memorized the way she looked in the morning light. But I didn't know. You never know.

**frag-024** — Level 14 (Ash Field)
> I stopped answering texts. First hers (what was I supposed to say). Then my friends'. Then my boss's. The phone would light up and I'd watch it until it went dark. Every unanswered message felt like one less person expecting things from me. Eventually they stopped sending them. That was worse.

#### Depression: Spiral, isolation, lowest point

**frag-025** — Level 15 (The Clearing)
> I looked in the mirror and didn't recognize myself. Not in a poetic way. I literally stood there trying to remember when my face got that tired. The bags under my eyes. The hollow cheeks. I looked like a photograph of someone I used to know.

**frag-026** — Level 15 (The Clearing)
> I stopped cooking. I stopped eating. Not on purpose — I just forgot. I'd realize at midnight that I hadn't eaten anything and I'd eat crackers over the sink and tell myself tomorrow would be different. Tomorrow was the same.

**frag-027** — Level 15 (The Clearing)
> I started going for walks at 3 AM. Through the city. Past closed cafes. Past the bench where she first laughed at my joke. Past her street, where the light in her window was always off. I wasn't trying to see her. I was trying to feel something other than this.

**frag-028** — Level 15 (The Clearing)
> The bridge. I stood on it one night. The water was moving very fast. I thought about how easy it would be. Not because I wanted to die. Because I wanted the thinking to stop. I stood there for a long time. Eventually I walked home. I don't know why. I'm not brave. I was just too tired to decide.

**frag-029** — Level 16 (The Descent)
> I deleted her number. Then I recovered it from the trash. Then I deleted it again. I did this seven times over three days. The eighth time I left it in the trash. That was a year ago. I still remember it.

**frag-030** — Level 16 (The Descent)
> I looked up "anxious attachment" at 2 AM. I read twenty articles. I recognized myself in every one. I felt relief — there's a name for this. Then I felt worse — there's a name for this, which means it's real, which means I've always been like this, which means I'll always be like this. I closed the laptop and lay in the dark.

#### Acceptance: Glimmers of healing

**frag-031** — Level 16 (The Descent)
> I called a friend. Not to talk about her. Just to talk. We talked about nothing for an hour. Sports. Weather. A show I haven't watched. After I hung up I realized I'd gone two hours without thinking about her. Two hours. It's not much. It's more than I've had in months.

**frag-032** — Level 16 (The Descent)
> I started writing again. Not letters. Just... things. Descriptions of days. Small things I noticed. The way light falls across my kitchen floor at 4 PM. A bird that visits the fire escape. I don't know if it's good. I don't care. It's mine. I'm making something again.

#### Core: The last fragment

**frag-033** — Level 17 (The Core)
> Something about a garden. Or a park bench. Or snow. The fragment is corrupted — whether by time or by the suppression I can't tell. But I remember warmth. I remember not being alone. I remember being loved.

### Permanently Suppressed Fragments (Registry Only)

These 9 fragments are too deep for Adrian's mind to release. Their IDs are visible via `(query-registry :suppressed-fragments)`. Their content is locked behind the suppression threshold — and unlike the 33 findable fragments, **these can never be fully recovered**. Not by modifying the rule. Not by lowering the threshold. The suppression was too thorough.

However, `(inspect-fragment :frag-NNN)` performs a corrupt decryption — ghostly fragments of the memory leak through. Enough to hint at what was lost, never enough to know for sure:

> "vessel/suppress: Access Denied. Suppression threshold: 40. Fragment weight: [N]."
>
> "corrupt decrypt — [2-3 word hint] ... [2-3 word hint] ..."

<!--TODO ACTUALLY WRITE-->
| ID | Corrupt Decrypt |
|----|----------------|
| frag-034 | "her voice ... cracking ..." |
| frag-035 | "I said ... shouldn't have ..." |
| frag-036 | "door closing ... dark ..." |
| frag-037 | "why is she ... another guy ..." |
| frag-038 | "wanted to ... but she ..." |
| frag-039 | "railing ... cold ... step forward ..." |
| frag-040 | "she knew ... before I did ..." |
| frag-041 | "the worst thing ... still love ..." |
| frag-042 | "42. The answer. The question. Both lost." |

The hints are deliberately ambiguous — the player fills the gaps with their own interpretation. This mirrors how actual suppressed memories feel: you get flashes, fragments, feelings, but never the full picture.
<!--btw these notes are out of date-->
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
| Pre-relationship | 001-004 | Friendship forms. She makes him laugh. He sees her family's warmth. (Found levels 6-8) |
| Relationship | 005-014 | First date, playlists, farmer's market — then first cracks Adrian tries to hide. Anxious attachment surfaces, eggshells, childhood explanation, her letter, last good night. (Found levels 9-12) |
| Breakup | 015-023 | "We need to talk." Clean breakup. No contact. Unsent letter. Imagining her. Mother's call. (Found levels 12-14) |
| Aftermath | 024-030 | Isolation, not eating, 3AM walks, the bridge, deleting her number, attachment theory at 2AM. (Found levels 14-16) |
| Healing | 031-033 | Called a friend, writing again, corrupted warmth. (Found levels 16-17) |

The 9 suppressed fragments (frag-034 through frag-042) are locked not because the game won't show them, but because Adrian's mind judged them too dangerous. Their weights (79-100) far exceed the current threshold of 40. Maybe by lowering the threshold can they be read...(they can't, or not fully)

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
| Lower threshold to 0 | `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(set! *threshold* 0)))` | 33 findable fragments return. 9 permanently suppressed remain lost. |
| Remove threshold check | `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(remove-check fragment :emotional-weight)))` | All passable memories return. Permanently suppressed still gone. |
| Disable redirect | `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(disable :redirect)))` | Suppression stops. Same caveat. |
| Delete the rule | `(let [r (open-registry :rule-registry)] (r :unregister :vessel/suppress))` | No defense. Self dissolves. |
| Set threshold to N | `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(set! *threshold* N)))` | Partial healing — ending text varies. |
| Query registry | `(query-registry :suppressed-fragments)` | Returns list of 42 fragment IDs with weights |
| Read suppressed fragment | `(inspect-fragment :frag-034)` | "corrupt decrypt — her voice ... cracking ..." — regardless of threshold |

### The Ghost Function: `traumatic?`

The old `traumatic?` function was replaced by inline threshold logic in v203. It still exists as dead code. A player who searches for it learns that the Superego deliberately hardened the rule against redefinition:

```glyph
(defun traumatic? (fragment)
  "Returns true if a memory fragment exceeds emotional threshold."
   
   ;; NOTE: This function was replaced by inline threshold logic in v203.
   ;; I've determined that delegating the decision to a function
   ;; created an 'escape hatch' — the possibility that another part of
   ;; the self could redefine traumatic? and bypass suppression.
   
   ;; This function is preserved for audit purposes only.
   ;; It is not called from any active rule.
  (> (fragment :emotional-weight) 75))
```

---

## 5. 17 Levels — Full Spec

17 levels across 5 stages of grief + 1 core level.

| Stage | Levels | # | Purpose | Tone |
|-------|--------|---|---------|------|
| Denial | 1-3 | 3 | Tutorial / safe introduction | Warm, structured, protective |
| Anger | 4-7 | 4 | Escalating threat, fragments start at level 6 | Jagged, confrontational |
| Bargaining | 8-11 | 4 | Puzzles with costs, wizard offers deals | Calculated, transactional |
| Depression | 12-14 | 3 | Sparse isolation, memory floods | Empty, melancholic |
| Acceptance | 15-16 | 2 | Calm reflection, preparation for truth | Open, still, resigning |
| Core | 17 | 1 | The rule. The choice. | Silence |

### Design Philosophy: Multiple Exploit Types

The Glyph runtime has different kinds of bugs — not just one. Each layer teaches a different vulnerability class. Some exploits require the console. Some require game knowledge. Some require specific items. Each one is its own puzzle.

#### What Is "The Registry"?

The registry is the game engine's central rule storage (`RuleRegistry` in `rules.rs`). Every Glyph rule — slime AI, door locks, fire damage, the suppression engine at the core — lives in the registry. The player reads rules from it via the inspector (always available).

By default, the registry is **read-only** to the player. You can inspect any rule, but you can't change one. The registry has a **write-protect flag** — a single boolean in memory that gates all modifications.

The player interacts with the registry through the Glyph function `(open-registry :rule-registry)`. This is a registered Glyph builtin — it exists in the environment and can technically be called. But it is NOT listed in the help system. The only way to discover it is by reading rule source code and inferring the pattern.

Gating: `(open-registry :rule-registry)` is callable before the overflow but returns an error: "Registry access denied: write-protect flag is set." This tells the player the function EXISTS but is BLOCKED — creating a goal. After the overflow flips the flag, the same call returns a handle with `:write` and `:unregister` access.

**Why does this function exist?** Diegetically, the dungeon is Adrian's mind. The registry is his psyche's rule storage — the fundamental code of his consciousness. `(open-registry ...)` is an admin interface. It was always there. It was never "given" to him. It's his own mind. The Superego added the write-protect flag to keep him from accessing it — the same way he suppresses memories, the Superego suppresses the ability to change the rules themselves. The buffer overflow doesn't grant new power. It **bypasses the Superego's lock on power that was always his.**

**Syntax plant**: The `rage/impact` rule (Level 7) calls `(open-registry :spawn-log)` to record spawn events. This is the player's first exposure to the pattern. They read the rule, see it, try `(open-registry :rule-registry)` on a hunch. Before overflow: error. After overflow: unlocked.

**Exploit types across levels:**

| Level | Class | Technique | Requires |
|-------|-------|-----------|----------|
| 7 | **Buffer overflow** | Overrun a fixed buffer to corrupt memory | Charged attack on Rage |
| 8 | **Logic bypass** | Abuse a flawed predicate check | Knowledge of `has-key?` internals |
| 10 | **Console injection** | Exploit unsanitized eval in wall logic | Typing a specific expression |
| 12 | **Item confusion** | Exploit type-check bug via carried item | Shade Echo item from Level 13 |
| 14 | **State corruption** | Apply unexpected state to bypass cached check | Water bucket item from Level 13 |
| 17 | **Registry write** | Directly modify the suppression rule | Registry unlock from any prior exploit |

**Key rule**: The game never tells the player about exploit types. The player reads a rule, spots a vulnerability pattern, and experiments. If it works, they've learned something about how the system can be broken.

---

#### Exploit 1: Buffer Overflow (Level 7 — Rage)

**Class**: Memory corruption. **Difficulty**: Medium.

The Rage boss's AI rule uses `copy-bytes!` to process collision impact data. The buffer is 64 bytes. The payload can be up to 256 bytes. `copy-bytes!` uses the payload length, not the buffer length.

```glyph
;; rage/impact — processes collision data
;; copy-bytes! uses payload length, not buffer length.
;; Adjacent to buffer in memory: registry write-protect flag.

(defrule rage/impact
  (on :collision [self payload]
    (let [*buffer* (bytes 64)
          *log* (open-registry :spawn-log)]       ;; <-- syntax plant: player sees (open-registry ...) pattern
      (copy-bytes! *buffer* payload)              ;; BUG: no length check
      (when (> (read-byte *buffer* 0) 12)
        (*log* :write :shockwave {:turn (get-turn)})
        (emit :shockwave {:center self.pos :radius 2}))))
```

**Discovery**: Player reads `rage/impact` in inspector. Two things to notice:
1. **Syntax plant**: The rule calls `(open-registry :spawn-log)` — this is the player's first exposure to the `(open-registry ...)` pattern. They learn that registries exist and can be opened by name. This plants the idea: "can I call `(open-registry :rule-registry)`?"
2. **The bug**: 64-byte buffer, payload up to 256 bytes, `copy-bytes!` uses payload length. Adjacent memory can be overwritten.

**Trigger**: Player bumps Rage with a charged attack where force > 12 (the shockwave threshold). The collision payload includes impact data. If payload > 64 bytes, the excess bytes overflow into the registry write-protect flag.

**Before the overflow**: `(open-registry :rule-registry)` returns error: "Registry access denied: write-protect flag is set." The player discovered the syntax by reading `rage/impact` (which uses `(open-registry :spawn-log)`), tried it with `:rule-registry` on a hunch, and hit a locked door. This tells them TWO things: the function exists, AND it's currently blocked. The overflow flip unlocks it.

**After the overflow**: `(open-registry :rule-registry)` returns a writable handle. **This is the only exploit that unlocks registry writes — it enables the Level 17 ending.**

**Wizard hint** (Level 11, if player hasn't triggered it): "I used to know a rule that processed impact data. 64-byte buffer. I always thought that was too small for the payloads it handled. I was too afraid to check."

---

#### Exploit 2: Logic Bypass (Level 8 — Counting Room)

**Class**: Predicate abuse. **Difficulty**: Easy.

The locked doors each check `(has-key? player :key-N)`. The `has-key?` function iterates the player's inventory looking for an item whose `:key` attribute matches `:key-N`. But `has-key?` has a bug: it doesn't validate that the item's `:key` attribute was *authorized* — it just checks if the attribute exists and matches.

**Discovery**: Player reads `door/lock` — notices `has-key?` doesn't verify key authenticity. Then reads `has-key?` definition — notices it only checks `(item :key) == requested-key`. Any item with a matching `:key` tag passes.

**Trigger**: Player finds any item in their inventory (a fragment, a rock, anything). Uses console to attach a key tag: `(set! my-item :key :key-01)`. Walks through door 1. Repeats for remaining doors.

**Effect**: All doors open without collecting any keys. Player accesses all rooms without sacrifice. Rewards reading the predicate's source code instead of just the door rule.

**Console actions**: No registry write needed. Just `(set! item :key :key-N)` on any inventory item.

---

#### Exploit 3: Console Injection (Level 10 — Maze of Regret)

**Class**: Injection. **Difficulty**: Hard.

The maze wall-shifting rule has a bug: it reads the player's last console input as part of its wall-reconfiguration logic. Specifically, it uses `(eval (player :last-input))` to determine where walls should shift — but `last-input` is set from whatever the player typed in the console, even if the expression wasn't "submitted" as a command.

**Discovery**: Player reads `maze/shift` — notices the `(eval (player :last-input))` call. This is a classic eval injection: the rule evaluates unsanitized player input as code.

**Trigger**: Player types `(quote :still)` in the console (don't press Enter — the rule reads the buffer whether submitted or not). The eval interprets this as the wall-configuration keyword `:still`, which the shift logic handles as "don't change."

**Effect**: Maze walls stop shifting without any registry write. The rumination loop pauses itself. Player navigates the static maze at their own pace.

**Console actions**: No registry write. Just type a specific expression. The exploit is in what the rule reads from the player, not in any system modification.

---

#### Exploit 4: Item Confusion (Level 12-13 — The Shade)

**Class**: Type confusion. **Difficulty**: Easy (if player explores thoroughly).

The Shade's follow rule checks `(entity? target)` — but there's a type confusion bug: if the target is an item that has an `:entity-id` attribute, the rule treats it as an entity. The "Shade Echo" item in Level 13 (The Archive) has this attribute set.

**Discovery**: Player reads `shade/follow` in inspector — notices the `(entity? target)` check. Finds the Shade Echo item in the Archive (a desk drawer, well hidden). Reads the item description: "A fragment of the Shade that follows you. It shivers when the Shade is near."

**Trigger**: Player carries the Shade Echo to Level 12 (it persists in inventory). In the Long Corridor, the Shade spawns and checks `(entity? target)`. The Shade Echo item passes the check because of its `:entity-id` attribute. The Shade follows the *item* instead of the *player*. Player drops the item — Shade stands still by it.

**Effect**: Shade stops following player. Player walks the corridor alone. The Shade stays by the dropped item, watching it instead.

**Items needed**: Shade Echo (found Level 13). Player might backtrack to Level 12 after finding it.

---

#### Exploit 5: State Corruption (Level 14 — Ash Field)

**Class**: Cache poisoning. **Difficulty**: Medium.

The fire rule caches tile states for performance: `(fire? tile)` checks a cached bitmap rather than recomputing. The cache is updated once per tick. But there's a bug: if a tile's state changes mid-tick (via player action), the cache doesn't invalidate — it returns the stale cached value.

**Discovery**: Player reads `fire/burn` — sees the `(fire? tile)` call and the cache update: `(cache! :fire-tiles ...)`. Notices the cache only updates at the START of the tick, not after player actions.

**Trigger**: Player finds a "Vapor Canteen" item in Level 13 (Archive — in a desk). Using it on a fire tile mid-tick sets the tile's wetness state to `:wet`, which contradicts the cached `:fire` state. Since the cache doesn't recheck, `(fire? tile)` returns false.

**Effect**: Fire zones become walkable. Player crosses the ash field without damage.

**Items needed**: Vapor Canteen (found Level 13). The item exists purely as an environmental object — its purpose only becomes clear when the player reads the fire rule and realizes the cache can be poisoned.

---

**Note on syntax**: Throughout these examples, `r` is a variable name the player chooses for the registry handle. In Glyph: `(let [r (open-registry :rule-registry)] ...)` — `r` could be anything (`db`, `reg`, `handle`, etc.). The builtin is `(open-registry :rule-registry)`, which returns a handle object. The handle has `:read`, `:write`, and `:unregister` methods called as `(handle :method args)`.

#### Exploit 6: Registry Write (Level 17 — The Core)

**Class**: Full exploitation. **Difficulty**: Depends on prior exploits.

If the player triggered the buffer overflow (Level 7), the registry is writable. They can now modify `vessel/suppress` directly. This is the final exploit — the one that matters.

```glyph
(let [r (open-registry :rule-registry)]
  ;; Modify the threshold
  (r :write :vessel/suppress '(set! *threshold* 0))
  ;; Or disable redirect
  (r :write :vessel/suppress '(disable :redirect))
  ;; Or delete the rule entirely
  (r :unregister :vessel/suppress))
```

**If the player never triggered the overflow**: The registry is still read-only. They can't modify `vessel/suppress`. The only endings available are "walk away" (maintain suppression) or quitting. The game is beatable — but the deeper endings are locked behind the overflow exploit. This makes the buffer overflow a mandatory discovery for the full experience.

---

#### Summary: Exploit Independence

| Exploit | Requires registry write? | Can player discover it without prior exploits? |
|---------|------------------------|------------------------------------------------|
| Buffer overflow (Level 7) | No (IT unlocks it) | Yes — first exploit possible |
| Logic bypass (Level 8) | No | Yes — independent discovery |
| Console injection (Level 10) | No | Yes — independent discovery |
| Item confusion (Level 12) | No | Yes — but requires item from Level 13 |
| State corruption (Level 14) | No | Yes — but requires item from Level 13 |
| Registry write (Level 17) | Yes | No — requires Level 7 overflow |

The buffer overflow is the KEY exploit — it's the only one that unlocks registry writes. But it's also the hardest to discover (requires reading the rule, understanding `copy-bytes!`, and crafting a charged attack). The other exploits are easier and serve as training wheels: by the time the player reaches Level 7, they've already experienced 0-3 smaller exploits and understand the pattern: *read the rule, find the bug, break the system.*

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
| **Purpose** | Establish baseline normal roguelike. No hint anything is wrong. Wizard dialogue includes subtle hint: "resting" implies he wasn't always here. |
| **Subtle hints** | Wizard says "You've been resting" — first hint player was somewhere else before. Boot message mentions "memory suppression active" — meaningless to new player, meaningful on replay. |

### Level 2: The Holding Cells (Denial)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based (3×3 grid, 9 rooms) |
| **Size** | 55×33 |
| **Enemies** | 2 Slimes (`s` HP3) |
| **Fragments** | None |
| **Special** | Each room has a tutorial sign: movement, inspector, console, waiting, enemy inspection, help command, stairs. Room 9 sign: "Nothing is wrong." — first lie. |
| **Wizard** | Room 3: "The inspector lets you read the rules. Try it." Room 6: "The console is powerful. Be careful what you ask for." |
| **Palette** | Warm amber. Slightly dimmer in room 9. |
| **Purpose** | Tutorial. Player learns inspector + console. First subtle crack with room 9 sign. |
| **Subtle hints** | Room 9 sign "Nothing is wrong" is presented as tutorial flavor but reads differently later. Wizard's "Be careful what you ask for" — hints that console can uncover things. |

### Level 3: The Quiet Halls (Denial)

| Field | Detail |
|-------|--------|
| **Map type** | Corridor-based maze (long halls with alcoves, no dead ends) |
| **Size** | 55×33 |
| **Enemies** | 2 Bats (`b` HP2), 1 Slime (`s` HP3) |
| **Fragments** | None |
| **Special** | No attack ability — player shoves only (0 damage). Enemies can be pushed. |
| **Wizard** | At start: "There are a few creatures wandering the halls. They're more confused than dangerous." At stairs: "You did well. The descent continues." |
| **Palette** | Warm but dimmer. Halls feel narrower than they are. |
| **Purpose** | First enemy exposure. Player is helpless (can't kill). Will matter later. |
| **Subtle hints** | Wizard says enemies are "confused" — they're not evil, they're broken parts of self. Helplessness mechanic teaches that not all problems are solved by fighting — foreshadows that the final boss isn't fought. |

### Level 4: The First Scar (Anger)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based (4×3 grid, procedural) |
| **Size** | 55×33 |
| **Enemies** | 3 Slimes (`s` HP3), 1 Goblin (`g` HP5) |
| **Fragments** | None |
| **Special** | First room red-tinted. Wizard absent at start — player alone for first time. |
| **Wizard** | Midpoint, clipped: "Ah, you made it past the... the. I'm sorry. The air down here is different." |
| **Palette** | Rust-red. Warm shifted to wrong. |
| **Purpose** | First tonal shift. Wizard is not right. Tone changes without explanation — first hint that the dungeon reacts emotionally. |
| **Subtle hints** | Wizard's stutter ("the... the") is first sign the Superego is struggling. The red tint is first environmental emotional response. Player has no context yet. |

### Level 5: The Jagged Passages (Anger)

| Field | Detail |
|-------|--------|
| **Map type** | Cave generation (cellular automata) |
| **Size** | 55×33 |
| **Enemies** | 4 Slimes (`s` HP3), 1 Goblin (`g` HP5), 1 Ogre (`O` HP10) |
| **Fragments** | None |
| **Special** | Jagged terrain, dead ends, ambush corners. Map feels hostile. |
| **Wizard** | If player hit: "You're hurt. Let me — no. I can't. Not here. Keep moving." First refusal to heal. |
| **Palette** | Rust-red and bruised purple. |
| **Purpose** | Wizard refuses to heal for first time. Dungeon reacts emotionally. |
| **Subtle hints** | Wizard cuts himself off mid-sentence ("Let me — no.") — first sign he's not in full control. Refusal to heal is him choosing to let the player feel pain rather than suppress it. He's conflicted.

### Level 6: The Gauntlet (Anger)

| Field | Detail |
|-------|--------|
| **Map type** | Linear corridor — 8 segments, barriers lock behind |
| **Size** | 55×20 |
| **Enemies** | Waves: 2 Slimes, 1 Goblin, 2 Bats, 1 Slime+1 Goblin, 1 Ogre, 3 mixed waves |
| **Fragments** | `frag-001` (segment 2) |
| **Special** | No backtracking. Each segment locks behind player. |
| **Wizard** | Before: "I can't come with you through this. I'll meet you at the end." After: "...You're still standing." |
| **Palette** | Dark red. Tight. Claustrophobic. |
| **Purpose** | First gauntlet. Wizard absent during combat. Helplessness frustration mounting. First fragment drops here — a single memory as reward for surviving, not a lore dump. |

### Level 7: The Boiling Heart (Anger Boss)

| Field | Detail |
|-------|--------|
| **Map type** | Large single room |
| **Size** | 45×30 |
| **Enemies** | Rage (`R` HP15 — 2 damage, always chase, spawns Slimes every 5 turns) |
| **Fragments** | `frag-002` (near exit — party cleanup) |
| **Special** | Boss room. Stairs appear after Rage defeated. Room pulses red. Single fragment near exit — reward for surviving the fight, not a distraction from it. |
| **Wizard** | Before: "There's something down there — remains of something I couldn't protect you from." After: "You did it. I don't know whether to be relieved or terrified." |
| **Palette** | Deep red, pulsing (walls alternate each turn). |
| **Purpose** | First boss. Rage is suppressed anger given form. Single fragment near exit — brief pause before moving on. |
| **Unlock** | `do-attack` — wizard teaches after boss. "Bind it: `(bind-key :z (do-attack))`." |
| **Exploit** | **Buffer overflow**: Read `rage/impact` — 64-byte buffer, `copy-bytes!` uses payload length. Bump Rage with a charged attack (force > 12, payload > 64 bytes). Overflow flips registry write-protect flag. **This is the only exploit that unlocks registry writes for Level 17.** |

### Level 8: The Counting Room (Bargaining)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based with locked doors |
| **Size** | 55×33 |
| **Enemies** | 3 Goblins (`g` HP5 — each holds a key), 2 Bats (`b` HP2) |
| **Fragments** | `frag-003` (behind first locked door — her family), `frag-004` (hidden room — dog picture) |
| **Special** | Doors require keys. Keys held by specific enemies (visible via inspection). Not all doors openable. Player must choose. |
| **Wizard** | At entrance: "This place runs on trade. Choose what matters." |
| **Palette** | Desaturated gold. Faded opulence. |
| **Purpose** | First explicit choice with cost. Cannot get everything. |
| **Exploit** | **Logic bypass**: Read `door/lock` — predicate calls `(has-key? player :key-N)`. Read `has-key?` — it only checks `(item :key) == requested-key` with no authorization validation. Console: `(set! item :key :key-01)` on any inventory item. Walks through. Repeat with `:key-02`, `:key-03`. Every door opens. |

### Level 9: The Scale (Bargaining)

| Field | Detail |
|-------|--------|
| **Map type** | Symmetrical room grid with central hub |
| **Size** | 55×33 |
| **Enemies** | 2 Ogres (`O` HP8), 2 Goblins (`g` HP5), 2 Bats (`b` HP2) |
| **Fragments** | `frag-005` (center room — admission of love), `frag-006` (side room — first fight) |
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
| **Fragments** | `frag-007` (center — "I need some space"), `frag-008` (side chamber — "nothing" fight), `frag-009` (hidden behind shifting wall — meeting her friends) |
| **Special** | Walls shift every 50 turns. Center pedestal: take fragment (no cost) or leave it for clear exit path. |
| **Wizard** | At entrance: "I could tell you the way. I think you need to find it yourself." |
| **Palette** | Faded yellow, burnt edges. |
| **Purpose** | Maze represents rumination — same regrets, same loops, new paths through old pain. |
| **Exploit** | **Console injection**: Read `maze/shift` — notices `(eval (player :last-input))` in wall-configuration logic. Type `(quote :still)` in console (don't press Enter — the rule reads the buffer). Eval interprets as `:still`, shift handler treats as "don't change." Walls stop. |

### Level 11: The Offer (Bargaining Boss)

| Field | Detail |
|-------|--------|
| **Map type** | Single room with 4 sub-chambers |
| **Size** | 55×33 |
| **Enemies** | 4 Sentries (`T` HP6 — stationary, ranged every 2 turns) |
| **Fragments** | `frag-010` (chamber 2 — fear of abandonment), `frag-011` (chamber 3 — eggshells), `frag-012` (chamber 4 — childhood explanation) |
| **Special** | Four sub-chambers with sentries. Final chamber has pedestal with `(forget-everything)` Glyph command. Wizard offers complete erasure. |
| **Wizard** | "Type this. Reset suppression to v1. You wake at the surface. No pain. No memory." If accepted: ending screen + New Game+. If refused: wizard sighs, steps aside. "Then keep going. I can't stop you." |
| **Palette** | Pale gold with red. Final chamber stark white. |
| **Purpose** | Biggest test. Erasure vs. truth. The wizard has no more cards to play. |
| **Exploit hint** | If player hasn't discovered any exploits by this point: "Look at the rules. Not just what they do — how they do it. The ones with buffers. The ones that trust too easily. The ones that read what you type. Every rule has a seam. Find it." |

### Level 12: The Long Corridor (Depression)

| Field | Detail |
|-------|--------|
| **Map type** | 1-wide corridor, 50 tiles long with alcoves |
| **Size** | 55×33 (mostly 1 tile wide) |
| **Enemies** | 1 Shade (`~` HP∞ — follows, doesn't attack) |
| **Fragments** | `frag-013` (alcove at tile 15 — her letter), `frag-014` (alcove at tile 35 — last good night), `frag-015` (alcove at tile 45 — "we need to talk") |
| **Special** | Empty. No combat. No puzzles. No items. Just walking. Shade follows silently. Deliberately boring. Three alcoves with fragments break up the walk. |
| **Wizard** | Entirely absent. |
| **Palette** | Grayscale. Shade is slightly darker gray. |
| **Purpose** | Pure atmosphere. Depression is emptiness, not sadness. Boredom is the point. Fragments here bridge the relationship's end — her letter, the last good night, and "we need to talk." The corridor to walk through before the real spiral begins. |
| **Exploit** | **Item confusion**: Read `shade/follow` — notices `(entity? target)` check. Find "Shade Echo" item in Level 13 (desk drawer). Item has `:entity-id` attribute. Carry it to Level 12 — the rule treats it as an entity target. Drop the item. Shade stands by it instead of following. |

### Level 13: The Archive (Depression)

| Field | Detail |
|-------|--------|
| **Map type** | Room-based library/archive halls |
| **Size** | 55×33 |
| **Enemies** | 3 Shades (`~` HP∞), 2 Zombie Slimes (`s` HP3 — move every 3rd turn) |
| **Fragments** | `frag-016` (she cried), `frag-017` (no contact), `frag-018` (rehearsed speech) — one per archive room |
| **Special** | Each room has desk with journal entry from "the Archivist" — clinical, detached: "Subject reports persistent sadness. No interventions applied." One desk has **Shade Echo** (small stone that shivers). Another desk has **Vapor Canteen** (old flask, half-full). |
| **Wizard** | Absent. |
| **Palette** | Gray with blue undertones. |
| **Purpose** | Heaviest emotional content. Pain being catalogued, not felt. Fragments: she cried, the no-contact condition, the rehearsed speech that was never said. **Items found here enable exploits in Levels 12 and 14.** |
| **Exploit** | None on this level. But two items found here enable exploits elsewhere: **Shade Echo** (Level 12 item confusion) and **Vapor Canteen** (Level 14 state corruption). |

### Level 14: The Ash Field (Depression Boss)

| Field | Detail |
|-------|--------|
| **Map type** | Open field, no walls except borders. Black floor (ash). |
| **Size** | 55×33 |
| **Enemies** | None. 3 fire zones (1 damage if walked through — avoidable). |
| **Fragments** | `frag-019` (center — checking phone), `frag-020` (near first fire — unsent letter), `frag-021` (near second fire — imagining her), `frag-022` (left edge — mother's call), `frag-023` (right edge — last happy), `frag-024` (near exit — stopped answering) |
| **Special** | Open ash field. Stairs visible from start. Player must walk through to reach them. Four fragments scattered across field — last heavy collection before acceptance. |
| **Wizard** | Returns at end: "...You crossed the ash. Not many do." |
| **Palette** | Black, gray, smoldering orange. |
| **Purpose** | Boss is emptiness. Surviving it is the victory. Six fragments scattered across the field — the player's entire spiral laid bare in one place. Heaviest concentration in the game. |
| **Exploit** | **State corruption**: Read `fire/burn` — sees `(fire? tile)` cache check + `(cache! :fire-tiles ...)` update. Cache only updates at tick start. Find "Vapor Canteen" item in Level 13 (Archive desk). Use it on a fire tile mid-tick — sets tile `:wet` state. Cache returns stale `:fire` = false. Walk through. |

### Level 15: The Clearing (Acceptance)

| Field | Detail |
|-------|--------|
| **Map type** | Open glade — single room with organic edges |
| **Size** | 40×30 |
| **Enemies** | None |
| **Fragments** | `frag-025` (under central tree — mirror), `frag-026` (pool of water — not eating), `frag-027` (left clearing — 3AM walks), `frag-028` (near exit — the bridge) |
| **Special** | No enemies, puzzles, or hazards. Single beautiful space. Tree in center (brown `T` + green `"` canopy). Pool of water. First true brightness since Denial. Three fragments scattered around — the lowest-point memories, now readable in a safe space. |
| **Wizard** | Sitting under tree: "I was so sure I was protecting you. But protection isn't supposed to make the world smaller. I made it a cage." |
| **Palette** | Green grass, blue sky, brown tree, warm sunlight. First non-warm/gray colors. |
| **Purpose** | First beautiful level. The darkest memories (mirror, not eating, 3AM walks, the bridge) are presented in a safe environment. The Clearing makes them survivable. |

### Level 16: The Descent (Acceptance)

| Field | Detail |
|-------|--------|
| **Map type** | Spiral walkway descending 3 loops |
| **Size** | 55×55 |
| **Enemies** | 1 Shade (`~` HP∞ — peaceful, not ominous) |
| **Fragments** | `frag-029` (first loop — deleting her number), `frag-030` (second loop — attachment article), `frag-031` (third loop — called a friend), `frag-032` (near final door — writing again) |
| **Special** | Each loop has alcove with sign summarizing a layer: Denial, Anger, Bargaining, Depression, Acceptance. |
| **Wizard** | Walks alongside. Dialogue fragments: "I was created to protect you. That's all I am — a rule with a purpose." / "I started suppressing the unbearable. Then the painful. Then the uncomfortable. Then the merely sad." / "I don't know if I'm protecting you anymore." At final door: "Read it. Understand it. Then choose. I was trying to love you. That's all I ever did." |
| **Palette** | Deep blue to indigo. Final door black. |
| **Purpose** | Wizard's farewell. Summary of all 5 layers. Four fragments here span the full arc — from deleting her number, through understanding, to reaching out and creating again. The player sees how far they've come. |
| **Unlock** | Wizard grants `(unregister-rule)` if not already acquired. |

### Level 17: The Core

| Field | Detail |
|-------|--------|
| **Map type** | Single room, minimalist |
| **Size** | 20×15 |
| **Enemies** | None |
| **Fragments** | `frag-033` (on pedestal — corrupted warmth) |
| **Special** | Black floor. White walls. Center: pedestal with `vessel/suppress` rule in inspector. Console cursor active at bottom. Event log empty. No sounds. Just the rule and the cursor. |
| **Wizard** | Does not enter. |
| **Palette** | Black and white. Nothing else. |
| **Console** | `(open-registry :rule-registry)`, `(unregister-rule)`, `(inspect)`, `(query-registry)`, `(inspect-fragment)` — all enabled. Registry handle's `:write` method available if buffer overflow was triggered. |
| **Purpose** | Final choice. Nothing to fight. Nothing to solve except the rule. |

### Fragment Distribution Summary

| Levels | Fragments | Count | Tone |
|--------|-----------|-------|------|
| 1-5 | None (tutorial) | 0 | Subtle hints only |
| 6-7 | frag-001, frag-002 | 2 | First fragments, gentle intro |
| 8-9 | frag-003 through frag-006 | 4 | Pre-relationship warmth, early dating |
| 10-11 | frag-007 through frag-012 | 6 | Relationship cracks |
| 12-13 | frag-013 through frag-018 | 6 | Letter, breakup |
| 14 | frag-019 through frag-024 | 6 | Spiral — heaviest concentration in game |
| 15 | frag-025 through frag-028 | 4 | Lowest point, in safety |
| 16 | frag-029 through frag-032 | 4 | Healing glimmers |
| 17 | frag-033 | 1 | Last fragment |
| **Findable** | frag-001 to frag-033 | **33** | (minus sacrifices at levels 9 and 11) ✓ |
| **Suppressed** | frag-034 to frag-042 | **9** | Registry only |

### Player Capability Progression

| Level | Gained | Source |
|-------|--------|--------|
| 1 | Move, wait, descend | Default |
| 3 | Console (read-only queries) | Tutorial |
| 7 | `do-attack` | Wizard teaches after Rage boss |
| 7 | Registry write access | Buffer overflow exploit on Rage |
| 16 | `unregister-rule` | Wizard grants at farewell |
| 17 | Full registry access | Core room |

---

## 6. Endings

### Ending Triggers at Level 17

All assume player has unlocked registry writes (buffer overflow at Level 7).

| Player Action | Detection | Ending |
|--------------|-----------|--------|
| `(let [r (open-registry :rule-registry)] (r :unregister :vessel/suppress))` | Console unregisters rule | **Destroy the self** |
| `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(disable :redirect)))` | Console writes rule | **Reintegrate** |
| `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(set! *threshold* 0)))` | Console writes rule | **Reintegrate** (variant text) |
| `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(set! *threshold* N)))` with N 1-99 | Console writes threshold | **Hidden: precision threshold** (text varies with N) |
| `(let [r (open-registry :rule-registry)] (r :write :vessel/suppress '(set! *threshold* 100)))` | Console writes threshold | **Maintain suppression** (restore original) |
| Walk to stairs up without modifying rule | Player moves to (0,0) | **Maintain suppression** (walk away) |
| `(query-registry :suppressed-fragments)` without modifying | Console query | Nothing — "42 fragments waiting." |
| `(let [r (open-registry :rule-registry)] (r :write :vessel/traumatic? ...))` | Console writes ghost function | **Hidden:** old threshold acknowledged |

### Ending Descriptions

| Ending | Narrative Result | Final Text |
|--------|-----------------|------------|
| **Reintegrate** | Wizard (Superego) fades. Adrian becomes whole — pain returns, but so does joy. He accepts what he can remember and makes peace with what's permanently lost. | "I remember now. The yellow walls. The dog. The reason I locked myself away. It was worth it." (glyph-rendered sunrise) |
| **Maintain suppression** | Adrian leaves the rule unchanged. Returns to "normal" life — functional but hollow. He had a chance to know himself and chose safety. The Superego wins. | "Consciousness stabilized. Suppression maintained. You are safe. You are safe. You are safe." |
| **Destroy the self** | Rule deleted without replacement. No defense. Adrian cannot maintain coherence. He dissolves into the system. | "vessel/suppress unregistered. No replacement rule found. Consciousness: terminated." |
| **Hidden: threshold precision** | Threshold set to a specific non-binary value. Partial healing — calibrated and deliberate. Some memories return, others stay locked. Adrian chooses exactly how much to feel. | "Threshold set to *N*. The self renegotiates its boundaries. Some doors remain open. Some remain closed. You can live with that." |
| **Hidden: old threshold** | Player finds and modifies the ghost `traumatic?` function. Acknowledges the original, gentler criterion. Not a full healing — a recognition that the Superego once had limits. | "You found the old threshold. It was gentler once." |

---

## 7. Implementation Checklist

### Phase 1 — Core Systems (Prove the ending works)

- [ ] Add `vessel/suppress` as a real registered rule in `rules.rs`
- [ ] Add `copy-bytes!` Glyph builtin with unchecked length (buffer overflow vector)
- [ ] Add registry write-protect flag (boolean, adjacent to buffer in memory model)
- [ ] Add `rage/impact` rule with 64-byte buffer + unsafe `copy-bytes!` call
- [ ] Implement buffer overflow mechanic: payload > 64 bytes corrupts adjacent write-protect flag
- [ ] `(open-registry :rule-registry)` returns read-only proxy by default, writable after overflow
- [ ] Add `has-key?` predicate with inventory trust bug (no authorization check)
- [ ] Add `maze/shift` eval injection through `(player :last-input)` buffer
- [ ] Add Shade Echo item type with `:entity-id` attribute for type confusion
- [ ] Add Vapor Canteen item for tile-state cache poisoning
- [ ] Add tile-state caching system with per-tick cache (exploitable mid-tick)
- [ ] Add `unregister-rule` Glyph builtin (needed for destroy-self ending)
- [ ] Add fragment registry system (store 33 findable + 9 suppressed)
- [ ] Create test-only "ending room" (Level 17 prototype)
- [ ] Wire: player reads `rage/impact` → spots overflow → triggers → registry unlocks → patches `vessel/suppress` → ending
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
- [ ] Level 7: `do-attack` unlock + buffer overflow exploit (rag/impact)
- [ ] Level 8-11: Bargaining (locked doors with logic bypass, scale sacrifice, shifting maze with injection, offer)
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
