//! Source-like rule snippets shown by the inspector.
//!
//! These are not parsed by Glyph yet. They are kept beside the game code so the
//! first prototype can already honor the fantasy that visible code describes the
//! behavior the dungeon actually follows.

pub const ENEMY_AI_SOURCE: &[&str] = &[
    "(defrule slime-hunt",
    "  {:phase :enemy-ai :cost :tick}",
    "  (if (adjacent? slime player)",
    "    (attack! slime player 1)",
    "    (step-toward! slime player)))",
];

pub const FLASHLIGHT_SOURCE: &[&str] = &[
    "(defrule flashlight",
    "  {:phase :render :cost :free}",
    "  (raycast-cone player.pos",
    "                player.facing",
    "                {:radius 12 :spread 0.78}))",
];
