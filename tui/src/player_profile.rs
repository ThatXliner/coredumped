//! Player profile — bindings, macros, learned abilities, and console history
//! bundled into game saves. Deleted on respawn/restart/wipe.
//!
//! Saved to `~/.xlyph/profile.json` alongside full game saves.

use serde::{Deserialize, Serialize};

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

    /// Delete the profile file from disk.
    pub fn delete() {
        let path = Self::path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }

    /// Write profile to disk at `~/.xlyph/profile.json`.
    pub fn save(&self) -> Result<(), String> {
        let dir = xlyph_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create xlyph dir: {}", e))?;
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("serialize profile: {}", e))?;
        std::fs::write(Self::path(), &json).map_err(|e| format!("write profile: {}", e))?;
        Ok(())
    }
}
