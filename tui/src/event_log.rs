//! Append-only event log used by the simulation and UI.
//!
//! The log is intentionally tiny for v1: game systems push human-readable
//! strings, and the renderer shows the newest lines in the bottom panel.

pub const MAX_LOG_LINES: usize = 100;

#[derive(Clone, Debug)]
pub struct EventLog {
    entries: Vec<String>,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, message: impl Into<String>) {
        self.entries.push(message.into());
        if self.entries.len() > MAX_LOG_LINES {
            self.entries.remove(0);
        }
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.entries.iter().any(|entry| entry.contains(needle))
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}
