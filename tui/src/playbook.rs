//! User playbook folder support.
//!
//! A playbook lives at ~/.xlyph/playbooks/current/ and contains user-authored
//! Glyph source files that are auto-loaded on game start. Definitions in the
//! playbook are appended to `user_source` so they survive save/load.

use std::path::PathBuf;

use crate::save::xlyph_dir;

/// Path to the current playbook directory.
pub fn current_playbook_dir() -> PathBuf {
    xlyph_dir().join("playbooks").join("current")
}

/// Check whether an init.glyph exists in the current playbook.
pub fn has_playbook() -> bool {
    current_playbook_dir().join("init.glyph").is_file()
}

/// Load init.glyph from the current playbook.
/// Returns the source text, or None if not found.
pub fn load_init_glyph() -> Option<String> {
    let path = current_playbook_dir().join("init.glyph");
    if path.is_file() {
        std::fs::read_to_string(&path).ok()
    } else {
        None
    }
}

/// Enumerate .glyph files in the lib/ directory, sorted by name.
pub fn list_lib_files() -> Vec<PathBuf> {
    let lib_dir = current_playbook_dir().join("lib");
    if !lib_dir.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&lib_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".glyph"))
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
}
