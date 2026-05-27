//! Append-only event log used by the simulation and UI.
//!
//! The log is intentionally tiny for v1: game systems push human-readable
//! strings, and the renderer shows the newest lines in the bottom panel.

use bracket_color::prelude::RGB;
use serde::{Deserialize, Serialize};

pub const MAX_LOG_LINES: usize = 100;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub text: String,
    pub color: Option<RGB>,
}

#[derive(Clone, Debug)]
pub struct EventLog {
    entries: Vec<LogEntry>,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, message: impl Into<String>) {
        self.entries.push(LogEntry {
            text: message.into(),
            color: None,
        });
        if self.entries.len() > MAX_LOG_LINES {
            self.entries.remove(0);
        }
    }

    pub fn push_colored(&mut self, message: impl Into<String>, color: RGB) {
        self.entries.push(LogEntry {
            text: message.into(),
            color: Some(color),
        });
        if self.entries.len() > MAX_LOG_LINES {
            self.entries.remove(0);
        }
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.entries.iter().any(|entry| entry.text.contains(needle))
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}
