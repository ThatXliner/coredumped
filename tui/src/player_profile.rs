//! Player profile — bindings, macros, learned abilities, and console history
//! that survive restart (shift+R) or carry across save slots.
//!
//! Saved to `~/.xlyph/profile.json` alongside full game saves. On restart,
//! the profile is re-applied after `World::new_game()` so the player's
//! customizations aren't lost.

use serde::{Deserialize, Serialize};

use crate::glyph::{eval_with_opts, read_string, SandboxOptions};
use crate::save::xlyph_dir;
use crate::world::World;

/// Serializable snapshot of player-authored state.
///
/// Does NOT include run-specific state (depth, map, entities, etc.).
/// Only the stuff the player "owns" across runs.
#[derive(Serialize, Deserialize, Default)]
pub struct PlayerProfile {
    pub bindings: Vec<(String, String)>,
    pub user_source: Vec<String>,
    pub player_can_attack: bool,
    pub wizard_taught: bool,
    pub cheat_unlocked: bool,
    pub blocking: bool,
    pub console_history: Vec<String>,
}

impl PlayerProfile {
    pub fn path() -> std::path::PathBuf {
        xlyph_dir().join("profile.json")
    }

    /// Snapshot current player state from a live world.
    pub fn from_world(world: &World) -> Self {
        PlayerProfile {
            bindings: world
                .bindings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            user_source: world.user_source.clone(),
            player_can_attack: world.player_can_attack,
            wizard_taught: world.wizard_taught,
            cheat_unlocked: world.cheat_unlocked,
            blocking: world.blocking,
            console_history: world.console_history.clone(),
        }
    }

    /// Apply profile onto a world (fresh or existing).
    ///
    /// Overwrites bindings, flags, and console history, then re-registers
    /// builtins and replays `user_source` against the glyph environment.
    pub fn apply_to(&self, world: &mut World) {
        world.bindings = self.bindings.iter().cloned().collect();
        world.user_source = self.user_source.clone();
        world.player_can_attack = self.player_can_attack;
        world.wizard_taught = self.wizard_taught;
        world.cheat_unlocked = self.cheat_unlocked;
        world.blocking = self.blocking;
        world.console_history = self.console_history.clone();

        // Re-register do-attack if player had learned it
        if self.player_can_attack {
            crate::game::bind_do_attack(&world.glyph_env);
        }

        // Replay env-mutating forms (const, defmacro, set!, bind-key)
        let glyph_env = world.glyph_env.clone();
        for source in &self.user_source {
            if let Ok(forms) = read_string(source) {
                for form in &forms {
                    let _ = eval_with_opts(form, &glyph_env, SandboxOptions::default(), world);
                }
            }
        }
    }

    /// Write profile to disk at `~/.xlyph/profile.json`.
    /// Delete the profile file from disk.
    pub fn delete() {
        let path = Self::path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = xlyph_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create xlyph dir: {}", e))?;
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("serialize profile: {}", e))?;
        std::fs::write(Self::path(), &json).map_err(|e| format!("write profile: {}", e))?;
        Ok(())
    }

    /// Load profile from disk, or `None` if no profile exists yet.
    pub fn load() -> Option<Self> {
        let path = Self::path();
        if !path.is_file() {
            return None;
        }
        let json = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&json).ok()
    }
}
