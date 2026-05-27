//! # Rule Registry Module
//!
//! This module defines the "rules" that govern how things work in the game.
//! Think of rules like the laws of physics for our dungeon world - they define
//! how enemies behave, how the flashlight works, what happens when you step on fire, etc.
//!
//! ## What is a Rule?
//!
//! A rule is a piece of game logic written in a Lisp-like language called "Glyph".
//! Each rule has:
//! - An ID (like "slime-hunt") to identify it
//! - A phase (when it runs - during enemy turns, rendering, etc.)
//! - A cost (does it use up a game turn or is it free?)
//! - Source code (the actual Glyph code that defines the behavior)
//!
//! ## Why Rules?
//!
//! The game's core concept is that players can eventually SEE and MODIFY these rules.
//! Instead of hard-coding "slimes chase the player", we write it as a rule that
//! players can inspect and potentially change. This makes the game's mechanics
//! transparent and hackable.
//!
//! ## Current Status
//!
//! Right now, rules are static (can't be changed at runtime). The long-term plan
//! is to let players patch and modify rules during gameplay.

// HashSet is a collection that stores unique items (no duplicates allowed).
// We use it to track which rules the player has discovered.
use std::collections::HashSet;

// Import types from other parts of our codebase:
// - EntityKind: The different types of creatures (slime, goblin, bat, etc.)
// - glyph::Value: A value in our Glyph scripting language
// - TileType: The different types of floor tiles (normal floor, fire, etc.)
use crate::{
    entity::EntityKind,
    glyph::{self, Value},
    map::TileType,
};

// =============================================================================
// RULE PHASE - When does a rule run?
// =============================================================================

/// RulePhase determines WHEN a rule executes during the game loop.
///
/// Think of the game as running in phases each turn:
/// 1. Player takes action
/// 2. Enemy AI phase - enemies decide what to do
/// 3. Tile effects phase - fire burns, traps trigger
/// 4. Render phase - draw everything to screen
///
/// Each rule belongs to exactly one phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RulePhase {
    /// EnemyAi: Rules that control how enemies behave.
    /// Examples: "slime-hunt" (slimes chase player), "bat-flutter" (bats move randomly)
    EnemyAi,

    /// TileEffect: Rules that trigger based on what tile you're standing on.
    /// Examples: "fire/burn" (standing on fire hurts you)
    TileEffect,

    /// Render: Rules that affect how things are drawn, not gameplay.
    /// Examples: "flashlight" (calculates which tiles the player can see)
    Render,
}

// =============================================================================
// RULE COST - Does this rule use up a turn?
// =============================================================================

/// RuleCost determines whether executing a rule costs the player a turn.
///
/// In roguelikes, time is measured in "ticks" (turns). Some actions take time
/// (moving, attacking) while others are instant (opening a menu, looking around).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleCost {
    /// Tick: This rule takes time to execute. The game advances one turn.
    /// Used for: enemy movement, attacks, tile effects
    Tick,

    /// Free: This rule is instant. No time passes.
    /// Used for: rendering, UI updates, information display
    Free,
}

// =============================================================================
// RULE - A single game rule
// =============================================================================

/// A Rule represents one piece of game logic.
///
/// Rules are written in Glyph (our Lisp dialect) and stored here along with
/// metadata about when and how they run.
#[derive(Clone, Debug)]
pub struct Rule {
    /// Unique identifier for this rule. Used to look up rules by name.
    /// Examples: "slime-hunt", "flashlight", "fire/burn"
    pub id: &'static str,

    /// Human-readable name (often same as id, but could be different).
    /// This is what players see in the inspector panel.
    pub name: &'static str,

    /// When this rule runs (see RulePhase enum above).
    pub phase: RulePhase,

    /// Whether this rule costs a turn (see RuleCost enum above).
    pub cost: RuleCost,

    /// The raw source code of the rule, split into lines.
    /// This is what players see when they inspect a rule.
    /// Using &'static str means these strings are baked into the compiled binary.
    pub source_lines: &'static [&'static str],

    /// The parsed Glyph code, ready to be executed.
    /// We parse the source_lines at startup so we don't have to re-parse every time.
    pub body_form: Value,
}

// =============================================================================
// RULE REGISTRY - Collection of all rules
// =============================================================================

/// The RuleRegistry holds all the rules in the game.
///
/// It's like a dictionary where you can look up rules by their ID.
/// The game creates one RuleRegistry at startup and uses it throughout.
#[derive(Clone, Debug)]
pub struct RuleRegistry {
    /// Internal storage for all rules. This is a Vec (growable array).
    rules: Vec<Rule>,
}

// =============================================================================
// HELPER FUNCTION - Parse Glyph source code
// =============================================================================

/// Parses the source code of a rule and extracts its body (the executable part).
///
/// ## How Glyph Rules Are Structured
///
/// A Glyph rule looks like this:
/// ```text
/// (defrule rule-name
///   {:phase :enemy-ai :cost :tick}   ; metadata
///   (if (adjacent? *self* *player*)  ; body - the actual logic
///     (attack! *self* *player* 1)
///     (step-toward! *self* *player*)))
/// ```
///
/// This function:
/// 1. Joins all the source lines into one string
/// 2. Parses it as Glyph code
/// 3. Extracts element [3] (the body, after defrule/name/metadata)
///
/// ## Parameters
/// - `source_lines`: Array of strings, each being one line of Glyph code
///
/// ## Returns
/// - The parsed body as a Glyph Value
///
/// ## Panics
/// - If the source doesn't parse as valid Glyph
/// - If the structure isn't (defrule name meta body)
fn parse_rule_body(source_lines: &[&str]) -> Value {
    // Join all lines with newlines to make one big string
    let source = source_lines.join("\n");

    // Parse the string as Glyph code. Returns a Vec of parsed forms.
    // .expect() will crash with the given message if parsing fails.
    let forms = glyph::read_string(&source).expect("rule source must parse as valid Glyph");

    // Check if the first form is a list (all valid rules are lists starting with 'defrule')
    if let Value::List(items) = &forms[0] {
        // A valid rule has at least 4 elements: defrule, name, metadata, body
        if items.len() >= 4 {
            // Return the 4th element (index 3) - that's the body
            return items[3].clone();
        }
    }

    // If we get here, the source wasn't a valid rule format
    panic!("rule source must be a (defrule name meta body) form");
}

// =============================================================================
// RULE REGISTRY IMPLEMENTATION
// =============================================================================

impl RuleRegistry {
    /// Creates the default rule registry with all the core game rules.
    ///
    /// This is called once at game startup. All the rules are defined here
    /// as static data - they're compiled right into the game binary.
    ///
    /// ## The Rules
    ///
    /// Currently we have these rules:
    /// - **slime-hunt**: Slimes chase the player (with some randomness)
    /// - **flashlight**: Calculates what the player can see
    /// - **goblin-patrol**: Goblins chase but flee when low HP
    /// - **bat-flutter**: Bats move randomly
    /// - **ogre-charge**: Ogres charge straight at the player
    /// - **shade-follow**: Shades silently follow (don't attack)
    /// - **rage-impact**: Rage enemies hit hard (3 damage)
    /// - **sentry-patrol**: Sentries only attack if you're adjacent
    /// - **fire/burn**: Standing on fire deals 1 damage
    /// - **vessel/suppress**: A mysterious rule about suppressed memories...
    pub fn core() -> Self {
        Self {
            rules: vec![
                // -----------------------------------------------------------------
                // SLIME-HUNT: How slimes chase the player
                // -----------------------------------------------------------------
                // This rule makes slimes interesting: they don't just beeline
                // toward you. There's a chance they'll wander randomly instead,
                // especially when far away. This makes them feel more organic.
                //
                // The logic:
                // 1. If next to player -> attack for 1 damage
                // 2. Otherwise, calculate distance to player
                // 3. Roll random chance based on distance
                // 4. Either move randomly OR move toward player
                Rule {
                    id: "slime-hunt",
                    name: "slime-hunt",
                    phase: RulePhase::EnemyAi, // Runs during enemy turn
                    cost: RuleCost::Tick,      // Uses up a game tick
                    source_lines: &[
                        "(defrule slime-hunt",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Slimes: chase with randomness. Closer = more likely to pursue.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (let dist (manhattan *self* *player*)",
                        "      (if (roll-odds? *self*",
                        "            (if (< dist 5)",
                        "              [0.5 + 0.2 * [5 - dist] / 4]",
                        "              0.5))",
                        "        (random-step! *self*)",
                        "        (step-toward! *self* *player*)))))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule slime-hunt",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Slimes: chase with randomness. Closer = more likely to pursue.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (let dist (manhattan *self* *player*)",
                        "      (if (roll-odds? *self*",
                        "            (if (< dist 5)",
                        "              [0.5 + 0.2 * [5 - dist] / 4]",
                        "              0.5))",
                        "        (random-step! *self*)",
                        "        (step-toward! *self* *player*)))))",
                    ]),
                },
                // -----------------------------------------------------------------
                // FLASHLIGHT: What the player can see
                // -----------------------------------------------------------------
                // The dungeon is dark! This rule calculates which tiles are
                // illuminated by the player's flashlight.
                //
                // It uses a "raycast cone" - imagine a cone of light spreading
                // out from the player in the direction they're facing.
                //
                // Parameters:
                // - radius: 12 tiles of light range
                // - spread-dot: 0.70 controls the cone width (higher = narrower)
                //
                // This is a Render phase rule with Free cost because it doesn't
                // affect gameplay, just what you can see.
                Rule {
                    id: "flashlight",
                    name: "flashlight",
                    phase: RulePhase::Render, // Runs during drawing
                    cost: RuleCost::Free,     // Doesn't cost a turn
                    source_lines: &[
                        "(defrule flashlight",
                        "  {:phase :render :cost :free}",
                        "  ;; Cone of light: 12 tiles, 0.70 spread (narrower = higher).",
                        "  (raycast-cone player.pos",
                        "                player.facing",
                        "                {:radius 12 :spread-dot 0.70}))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule flashlight",
                        "  {:phase :render :cost :free}",
                        "  ;; Cone of light: 12 tiles, 0.70 spread (narrower = higher).",
                        "  (raycast-cone player.pos",
                        "                player.facing",
                        "                {:radius 12 :spread-dot 0.70}))",
                    ]),
                },
                // -----------------------------------------------------------------
                // GOBLIN-PATROL: Cowardly goblins
                // -----------------------------------------------------------------
                // Goblins are aggressive but have a survival instinct.
                // When their HP drops to 1, they panic and run away!
                //
                // The logic:
                // 1. If next to player -> attack for 1 damage
                // 2. If HP <= 1 -> run away from player (flee-step!)
                // 3. Otherwise -> chase the player
                //
                // This makes goblins feel smarter than slimes.
                Rule {
                    id: "goblin-patrol",
                    name: "goblin-patrol",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule goblin-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Goblins: aggressive but cowardly. Flee when HP <= 1.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (if (<= (hp *self*) 1)",
                        "      (flee-step! *self* *player*)",
                        "      (step-toward! *self* *player*))))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule goblin-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Goblins: aggressive but cowardly. Flee when HP <= 1.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (if (<= (hp *self*) 1)",
                        "      (flee-step! *self* *player*)",
                        "      (step-toward! *self* *player*))))",
                    ]),
                },
                // -----------------------------------------------------------------
                // BAT-FLUTTER: Erratic bat movement
                // -----------------------------------------------------------------
                // Bats are chaotic! They don't chase you at all - they just
                // flutter around randomly. But if you happen to be next to one,
                // it will still bite you.
                //
                // This makes bats unpredictable obstacles rather than threats
                // that actively hunt you down.
                Rule {
                    id: "bat-flutter",
                    name: "bat-flutter",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule bat-flutter",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Bats: chaotic. Move randomly, bite only if adjacent.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (random-step! *self*)))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule bat-flutter",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Bats: chaotic. Move randomly, bite only if adjacent.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (random-step! *self*)))",
                    ]),
                },
                // -----------------------------------------------------------------
                // OGRE-CHARGE: Relentless ogre pursuit
                // -----------------------------------------------------------------
                // Ogres are simple but dangerous. They always move toward you.
                // No randomness, no fleeing, no tricks - just a straight charge.
                //
                // This is the "baseline" aggressive behavior. Ogres are
                // predictable, which lets skilled players kite them.
                Rule {
                    id: "ogre-charge",
                    name: "ogre-charge",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule ogre-charge",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Ogres: simple and relentless. Always chase, no tricks.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (step-toward! *self* *player*)))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule ogre-charge",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Ogres: simple and relentless. Always chase, no tricks.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (step-toward! *self* *player*)))",
                    ]),
                },
                // -----------------------------------------------------------------
                // SHADE-FOLLOW: Silent stalkers
                // -----------------------------------------------------------------
                // Shades are creepy! They follow you constantly but NEVER attack.
                // Notice how both branches of the if-statement do the same thing:
                // step-toward. Whether adjacent or not, shades just... follow.
                //
                // This creates tension - what are they waiting for?
                // (Narrative hint: they're watching, gathering information...)
                Rule {
                    id: "shade-follow",
                    name: "shade-follow",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule shade-follow",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Shades: silent stalkers. Follow always, never attack.",
                        "  (if (adjacent? *self* *player*)",
                        "    (step-toward! *self* *player*)",
                        "    (step-toward! *self* *player*)))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule shade-follow",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Shades: silent stalkers. Follow always, never attack.",
                        "  (if (adjacent? *self* *player*)",
                        "    (step-toward! *self* *player*)",
                        "    (step-toward! *self* *player*)))",
                    ]),
                },
                // -----------------------------------------------------------------
                // RAGE-IMPACT: Heavy hitters
                // -----------------------------------------------------------------
                // Rage enemies are DANGEROUS. They deal 3 damage instead of 1!
                // That's enough to seriously hurt or kill an unprepared player.
                //
                // The source code shown to players has extra "flavor" code about
                // collision physics and buffers - this is narrative/lore content
                // hinting at deeper game mechanics the player might exploit.
                //
                // Note: The body_form only contains the simple chase-and-attack
                // logic. The fancy buffer stuff is just for display in the inspector.
                Rule {
                    id: "rage-impact",
                    name: "rage-impact",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule rage-impact",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Rage: heavy hitter. Deals 3 damage instead of 1!",
                        "  ;; v12 note: added impact logging for debug metrics",
                        "  (let force (last-impact-force)",
                        "    (do",
                        "      (when (> force 12)",
                        "        (copy-bytes! (bytes 64) (impact-payload)))",
                        "      (if (adjacent? *self* *player*)",
                        "        (attack! *self* *player* 3)",
                        "        (step-toward! *self* *player*)))))",
                    ],
                    // The ACTUAL executed logic is simpler:
                    body_form: parse_rule_body(&[
                        "(defrule rage-impact",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 3)",
                        "    (step-toward! *self* *player*)))",
                    ]),
                },
                // -----------------------------------------------------------------
                // SENTRY-PATROL: Stationary guards
                // -----------------------------------------------------------------
                // Sentries don't move at all! They stand in one spot and only
                // attack if you walk right next to them. The "nil" in the else
                // branch means "do nothing".
                //
                // These are obstacle enemies - you need to plan your path to
                // avoid walking adjacent to them.
                Rule {
                    id: "sentry-patrol",
                    name: "sentry-patrol",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule sentry-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Sentries: stationary guards. Attack only if adjacent.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    nil))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule sentry-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  ;; Sentries: stationary guards. Attack only if adjacent.",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    nil))",
                    ]),
                },
                // -----------------------------------------------------------------
                // FIRE/BURN: Environmental hazard
                // -----------------------------------------------------------------
                // This is a TILE EFFECT rule, not an enemy AI rule.
                // It checks if the player is standing on a fire tile and
                // deals damage if so.
                //
                // The comments in the source hint at interesting mechanics:
                // - Fire state is cached at the start of each tick
                // - If something changes fire mid-tick, it won't take effect
                //   until next tick (potential exploit hint!)
                Rule {
                    id: "fire/burn",
                    name: "fire/burn",
                    phase: RulePhase::TileEffect, // Runs during tile effect phase
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule fire/burn",
                        "  {:phase :tile-effect :cost :tick}",
                        "  ;; Fire tile: deals 1 damage per tick while standing on it.",
                        "  (if (fire? *pos*)",
                        "    (do",
                        "      (log \"The fire burns you! You take 1 damage.\")",
                        "      (damage! *player* 1))))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule fire/burn",
                        "  {:phase :tile-effect :cost :tick}",
                        "  ;; Fire tile: deals 1 damage per tick while standing on it.",
                        "  (if (fire? *pos*)",
                        "    (do",
                        "      (log \"The fire burns you! You take 1 damage.\")",
                        "      (damage! *player* 1))))",
                    ]),
                },
                // -----------------------------------------------------------------
                // MAZE/SHIFT: Wall reconfiguration in the Maze of Regret
                // -----------------------------------------------------------------
                // This rule controls how walls shift in Level 10. It has a critical
                // vulnerability: it reads the player's console buffer (not just
                // submitted commands) and evals it as part of shift configuration.
                //
                // The exploit: type (quote :still) in the console WITHOUT submitting.
                // The eval reads the buffer, interprets :still as the config keyword
                // for "don't shift", and the maze freezes.
                Rule {
                    id: "maze-shift",
                    name: "maze/shift",
                    phase: RulePhase::EnemyAi, // Runs during tick processing
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule maze/shift",
                        "  {:phase :tick :cost :tick :scope :level-10}",
                        "  ;; Maze walls shift each tick based on turn parity.",
                        "  ;; Rumination made manifest: the same paths, never the same.",
                        "  ;;",
                        "  ;; v12 note: added debug hook to freeze walls during testing.",
                        "  ;; TODO: remove before release",
                        "  (let config (eval (player :console-buffer))",
                        "    (if (= config :still)",
                        "      nil",
                        "      (for [wall (maze :shifting-walls)]",
                        "        (if (even? *turn*)",
                        "          (set-tile! wall :floor)",
                        "          (set-tile! wall :wall))))))",
                    ],
                    body_form: Value::Nil, // Executed via Rust, not Glyph
                },
                // -----------------------------------------------------------------
                // VESSEL/SUPPRESS: A mysterious, narrative rule
                // -----------------------------------------------------------------
                // This is a LORE rule - it's not actually executed (body_form is Nil).
                // It exists for players to discover and ponder.
                //
                // The rule appears to be about psychological suppression:
                // - A "vessel" (the player character?) has a mechanism for
                //   suppressing traumatic memories
                // - Memories with "emotional weight" above a threshold get
                //   redirected to the unconscious
                // - The threshold has been lowering (100 -> 40)
                // - The comments are written in first person, like diary entries
                //
                // This is part of the game's narrative about identity, memory,
                // and what it means to be the "vessel" exploring this dungeon.
                //
                // Note the metadata: priority 255 (highest), scope global,
                // author "superego" (Freudian reference), stability "critical"
                Rule {
                    id: "vessel-suppress",
                    name: "vessel/suppress",
                    phase: RulePhase::Render,
                    cost: RuleCost::Free,
                    source_lines: &[
                        "(defrule vessel/suppress {:priority 255 :scope :global",
                        "  :author :superego :stability :critical}",
                        "  ;; --- Configuration ---",
                        "  (let *threshold* 40",
                        "    ;; Fragments above threshold are redirected",
                        "    ;; to the unconscious before the conscious mind",
                        "    ;; can process them.",
                        "    ;;",
                        "    ;; Current count: 42 fragments in registry.",
                        "    ;; Inspect via:  (query-registry :suppressed-fragments)",
                        "    ;; Read one:     (inspect-fragment :frag-NNN)",
                        "    ;;",
                        "    ;; The threshold began at 100.",
                        "    ;; It is now 40.",
                        "    ;; I don't know which is worse —",
                        "    ;; that I've lost so many,",
                        "    ;; or that some still escape.",
                        "    (let *registry* (open-registry :suppressed-fragments)",
                        "      (for [fragment (in-scope :memories)]",
                        "        (let weight (fragment :emotional-weight)",
                        "          (if (> weight *threshold*)",
                        "            (do (redirect fragment :unconscious)",
                        "              (log-suppression fragment)",
                        "              (emit :flinch (fragment :hint))",
                        "              (if (< weight 45)",
                        "                (emit :warning",
                        "                  \"threshold drift detected\")))",
                        "            fragment))))))",
                    ],
                    // This rule is for inspection only - it's not actually run
                    body_form: Value::Nil,
                },
            ],
        }
    }

    // =========================================================================
    // REGISTRY QUERY METHODS
    // =========================================================================

    /// Returns an iterator over all rules in the registry.
    ///
    /// An iterator lets you loop through items one at a time without
    /// copying them. Example usage:
    /// ```ignore
    /// for rule in registry.iter() {
    ///     println!("{}", rule.name);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }

    /// Looks up a rule by its ID string.
    ///
    /// Returns `Some(&Rule)` if found, `None` if not.
    /// The Option type is Rust's way of handling "maybe there's a value,
    /// maybe there isn't" without using null pointers.
    ///
    /// ## Example
    /// ```ignore
    /// if let Some(rule) = registry.get("slime-hunt") {
    ///     // Do something with the rule
    /// }
    /// ```
    pub fn get(&self, id: &str) -> Option<&Rule> {
        // .find() searches through the iterator and returns the first match
        self.rules.iter().find(|r| r.id == id)
    }

    /// Returns the rule that applies to a specific tile type.
    ///
    /// Currently only fire tiles have an associated rule.
    /// This could be extended for other special tiles (ice, poison, etc.)
    pub fn tile_rule(&self, tile: crate::map::TileType) -> Option<&Rule> {
        match tile {
            crate::map::TileType::Fire => self.get("fire/burn"),
            _ => None, // Most tiles don't have special rules
        }
    }

    // =========================================================================
    // RULE VISIBILITY (What the player has discovered)
    // =========================================================================

    /// Rules that are always shown in the inspector, even if the player
    /// hasn't encountered the relevant enemy/tile yet.
    ///
    /// The flashlight rule is always visible because it's fundamental to
    /// understanding how visibility works in the game.
    pub const ALWAYS_VISIBLE: &[&str] = &["flashlight"];

    /// Calculates which rules the player can currently see in the inspector.
    ///
    /// Rules are "discovered" when the player encounters related content:
    /// - See a slime? You can now inspect the slime-hunt rule
    /// - Step on fire? You can now inspect the fire/burn rule
    ///
    /// This creates a sense of progression - you learn the game's mechanics
    /// by experiencing them, then can study them in detail.
    ///
    /// ## Parameters
    /// - `seen_entities`: Set of enemy types the player has encountered
    /// - `seen_tiles`: Set of special tile types the player has stepped on
    ///
    /// ## Returns
    /// - HashSet of rule IDs that should be visible in the inspector
    pub fn visible_ids(
        &self,
        seen_entities: &HashSet<EntityKind>,
        seen_tiles: &HashSet<TileType>,
    ) -> HashSet<String> {
        let mut ids = HashSet::new();

        // Check each rule to see if it should be visible
        for rule in &self.rules {
            // Some rules are always visible (like flashlight)
            if Self::ALWAYS_VISIBLE.contains(&rule.id) {
                ids.insert(rule.id.to_string());
                continue; // Skip to next rule
            }

            // Check if we've seen an entity that uses this rule
            for kind in seen_entities {
                if kind.rule_name() == rule.id {
                    ids.insert(rule.id.to_string());
                    break; // Found a match, no need to keep checking
                }
            }

            // Check if we've seen a tile that uses this rule
            for &tile in seen_tiles {
                if let Some(r) = self.tile_rule(tile) {
                    if r.id == rule.id {
                        ids.insert(rule.id.to_string());
                        break;
                    }
                }
            }
        }

        ids
    }

    // =========================================================================
    // UTILITY METHODS
    // =========================================================================

    /// Returns the total number of rules in the registry.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns true if the registry has no rules (should never happen in practice).
    ///
    /// This method exists because Rust's clippy linter warns if you have a
    /// len() method without a corresponding is_empty() method.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

// =============================================================================
// DEFAULT TRAIT IMPLEMENTATION
// =============================================================================

/// Implements the Default trait for RuleRegistry.
///
/// The Default trait is a Rust convention that lets you create a "default"
/// instance of a type by calling RuleRegistry::default().
///
/// For our registry, the default is the core set of game rules.
impl Default for RuleRegistry {
    fn default() -> Self {
        Self::core()
    }
}

// =============================================================================
// TESTS
// =============================================================================
//
// Tests in Rust go in a special `mod tests` block with the #[cfg(test)] attribute.
// This means the test code is only compiled when running `cargo test`, not in
// the final game binary.
//
// Each test is a function marked with #[test]. The test passes if it doesn't
// panic (crash), and fails if it does.

#[cfg(test)]
mod tests {
    // Import everything from the parent module (the main rules.rs code)
    use super::*;

    /// Verify the core registry has exactly 11 rules.
    ///
    /// This catches accidental additions or removals of rules.
    /// If you add a new rule, update this number!
    #[test]
    fn core_registry_has_eleven_rules() {
        let registry = RuleRegistry::core();
        // assert_eq! panics if the two values aren't equal
        assert_eq!(registry.len(), 11);
    }

    /// Test that we can look up a rule by its ID.
    #[test]
    fn lookup_rule_by_id() {
        let registry = RuleRegistry::core();

        // .expect() unwraps the Option, panicking with the message if it's None
        let rule = registry.get("slime-hunt").expect("slime-hunt should exist");

        // Verify the rule has the expected properties
        assert_eq!(rule.phase, RulePhase::EnemyAi);
        assert_eq!(rule.cost, RuleCost::Tick);
        assert_eq!(rule.name, "slime-hunt");
    }

    /// Test that the flashlight rule exists and has correct metadata.
    #[test]
    fn lookup_flashlight_rule() {
        let registry = RuleRegistry::core();
        let rule = registry.get("flashlight").expect("flashlight should exist");

        assert_eq!(rule.phase, RulePhase::Render);
        assert_eq!(rule.cost, RuleCost::Free);
        // Make sure it has actual source code
        assert!(rule.source_lines.len() >= 4);
    }

    /// Test that looking up a non-existent rule returns None.
    #[test]
    fn unknown_rule_id_returns_none() {
        let registry = RuleRegistry::core();
        // .is_none() returns true if the Option is None
        assert!(registry.get("does-not-exist").is_none());
    }

    /// Test that iterating visits all rules.
    #[test]
    fn iter_visits_all_rules() {
        let registry = RuleRegistry::core();

        // Collect all rule IDs into a Vec (growable array)
        // .map() transforms each rule into just its ID
        // .collect() gathers the results into a collection
        let ids: Vec<&str> = registry.iter().map(|r| r.id).collect();

        assert_eq!(ids.len(), 11);

        // Verify some specific rules are present
        assert!(ids.contains(&"slime-hunt"));
        assert!(ids.contains(&"flashlight"));
        assert!(ids.contains(&"fire/burn"));
        assert!(ids.contains(&"shade-follow"));
        assert!(ids.contains(&"rage-impact"));
        assert!(ids.contains(&"sentry-patrol"));
        assert!(ids.contains(&"vessel-suppress"));
        assert!(ids.contains(&"maze-shift"));
    }
}
