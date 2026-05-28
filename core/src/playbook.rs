//! User playbook folder support.
//!
//! A playbook lives at ~/.xlyph/playbooks/current/ and contains user-authored
//! Glyph source files that are auto-loaded on game start. Definitions in the
//! playbook are appended to `user_source` so they survive save/load.

use std::path::PathBuf;

use bracket_color::prelude::{GREEN, RGB};

use crate::save::xlyph_dir;
use crate::world::World;

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

impl World {
    /// Load user playbook files (init.glyph and lib/*.glyph) into the world.
    pub(crate) fn load_playbook(&mut self) {
        if !has_playbook() {
            return;
        }

        let glyph_env = self.glyph_env.clone();
        let binding_env = self.binding_env.clone();
        let mut loaded = false;

        // Load init.glyph
        if let Some(init_source) = load_init_glyph() {
            match crate::glyph::read_string(&init_source) {
                Ok(forms) => {
                    for form in &forms {
                        let source = form.to_string();
                        match crate::glyph::eval_with_opts(
                            form,
                            &glyph_env,
                            crate::glyph::SandboxOptions::default(),
                            self,
                        ) {
                            Ok(_) => {
                                self.user_source.push(source.clone());
                                let _ = crate::glyph::eval_with_opts(
                                    form,
                                    &binding_env,
                                    crate::glyph::SandboxOptions::default(),
                                    self,
                                );
                            }
                            Err(e) => {
                                self.event_log.push(format!("Playbook init error: {}", e));
                            }
                        }
                    }
                    loaded = true;
                }
                Err(e) => {
                    self.event_log
                        .push(format!("Playbook init.glyph parse error: {}", e));
                }
            }
        }

        // Load lib/*.glyph files
        for lib_file in list_lib_files() {
            match std::fs::read_to_string(&lib_file) {
                Ok(source) => match crate::glyph::read_string(&source) {
                    Ok(forms) => {
                        for form in &forms {
                            let source = form.to_string();
                            match crate::glyph::eval_with_opts(
                                form,
                                &glyph_env,
                                crate::glyph::SandboxOptions::default(),
                                self,
                            ) {
                                Ok(_) => {
                                    self.user_source.push(source.clone());
                                    let _ = crate::glyph::eval_with_opts(
                                        form,
                                        &binding_env,
                                        crate::glyph::SandboxOptions::default(),
                                        self,
                                    );
                                }
                                Err(e) => {
                                    self.event_log.push(format!(
                                        "Playbook lib '{}' error: {}",
                                        lib_file.display(),
                                        e
                                    ));
                                }
                            }
                        }
                        loaded = true;
                    }
                    Err(e) => {
                        self.event_log.push(format!(
                            "Playbook lib '{}' parse error: {}",
                            lib_file.display(),
                            e
                        ));
                    }
                },
                Err(e) => {
                    self.event_log.push(format!(
                        "Cannot read playbook lib '{}': {}",
                        lib_file.display(),
                        e
                    ));
                }
            }
        }

        if loaded {
            self.event_log
                .push_colored("Playbook loaded.", RGB::named(GREEN));
        }
    }
}
