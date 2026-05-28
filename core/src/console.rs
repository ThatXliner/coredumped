//! Console input handling: buffer operations, cursor movement, history, and submission.

use bracket_color::prelude::{RED, RGB};

use crate::glyph::{self, Value};
use crate::world::World;
use crate::game::{Intent, Mode};

impl World {
    pub(crate) fn console_insert(&mut self, ch: char) {
        self.console_cursor = clamp_to_char_boundary(&self.console_buffer, self.console_cursor);
        self.console_buffer.insert(self.console_cursor, ch);
        self.console_cursor += ch.len_utf8();
    }

    pub(crate) fn console_move_cursor(&mut self, delta: i32) {
        if delta < 0 {
            self.console_cursor = previous_char_boundary(&self.console_buffer, self.console_cursor);
        } else {
            self.console_cursor = next_char_boundary(&self.console_buffer, self.console_cursor);
        }
    }

    pub(crate) fn console_move_word(&mut self, delta: i32) {
        self.console_cursor = if delta < 0 {
            previous_word_boundary(&self.console_buffer, self.console_cursor)
        } else {
            next_word_boundary(&self.console_buffer, self.console_cursor)
        };
    }

    pub(crate) fn console_backspace(&mut self) {
        if self.console_cursor == 0 {
            return;
        }
        let start = previous_char_boundary(&self.console_buffer, self.console_cursor);
        self.console_buffer.drain(start..self.console_cursor);
        self.console_cursor = start;
    }

    pub(crate) fn console_backspace_word(&mut self) {
        if self.console_cursor == 0 {
            return;
        }
        let start = previous_word_boundary(&self.console_buffer, self.console_cursor);
        self.console_buffer.drain(start..self.console_cursor);
        self.console_cursor = start;
    }

    pub(crate) fn console_delete(&mut self) {
        if self.console_cursor >= self.console_buffer.len() {
            return;
        }
        self.console_cursor = clamp_to_char_boundary(&self.console_buffer, self.console_cursor);
        let end = next_char_boundary(&self.console_buffer, self.console_cursor);
        self.console_buffer.drain(self.console_cursor..end);
    }

    pub(crate) fn console_history_move(&mut self, delta: i32) {
        if self.console_history.is_empty() {
            return;
        }
        if delta < 0 {
            // Up arrow — go back in history
            if self.console_history_index == 0 {
                self.console_history_draft = self.console_buffer.clone();
                self.console_history_index = 1;
            } else if self.console_history_index < self.console_history.len() {
                self.console_history_index += 1;
            }
        } else {
            // Down arrow — go forward in history
            if self.console_history_index > 0 {
                self.console_history_index -= 1;
            }
        }
        let loaded = if self.console_history_index == 0 {
            self.console_history_draft.clone()
        } else {
            let idx = self.console_history.len() - self.console_history_index;
            self.console_history[idx].clone()
        };
        self.console_buffer = loaded;
        self.console_cursor = self.console_buffer.len();
    }

    pub(crate) fn console_kill_to_start(&mut self) {
        self.console_cursor =
            clamp_to_char_boundary(&self.console_buffer, self.console_cursor);
        self.console_buffer.drain(..self.console_cursor);
        self.console_cursor = 0;
    }

    pub(crate) fn console_kill_to_end(&mut self) {
        self.console_cursor =
            clamp_to_char_boundary(&self.console_buffer, self.console_cursor);
        self.console_buffer.truncate(self.console_cursor);
    }

    pub(crate) fn open_external_editor(&mut self) {
        let temp_path = crate::save::temp_edit_path();
        let _ = std::fs::create_dir_all(temp_path.parent().unwrap());

        if std::fs::write(&temp_path, &self.console_buffer).is_err() {
            self.event_log.push("Cannot write temp file for editor.");
            return;
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        self.event_log.push_colored(
            format!("Opening {} (game paused)...", editor),
            RGB::named(bracket_color::prelude::YELLOW),
        );

        let status = std::process::Command::new(&editor).arg(&temp_path).status();

        match status {
            Ok(s) if s.success() => match std::fs::read_to_string(&temp_path) {
                Ok(contents) => {
                    self.console_buffer = contents;
                    self.console_cursor = self.console_buffer.len();
                    self.event_log.push("Editor closed. Buffer updated.");
                }
                Err(e) => {
                    self.event_log
                        .push(format!("Cannot read edited file: {}", e));
                }
            },
            Ok(s) => {
                self.event_log
                    .push(format!("Editor exited ({}) — buffer unchanged.", s));
            }
            Err(e) => {
                self.event_log
                    .push(format!("Cannot spawn '{}': {}", editor, e));
            }
        }

        let _ = std::fs::remove_file(&temp_path);
    }

    pub(crate) fn submit_console(&mut self) {
        let trimmed = self.console_buffer.trim();
        self.console_output_scroll = 0;

        // Handle pending wipe confirmation
        if let Some(slot) = self.pending_wipe_slot.take() {
            if trimmed == "i am aware of what i am doing." {
                self.deferred_intent = Some(Intent::WipeSave(slot));
            } else {
                self.event_log.push("Wipe cancelled.");
            }
            self.console_buffer.clear();
            return;
        }

        if trimmed.is_empty() {
            self.event_log.push("Console waits. No query submitted.");
            self.console_buffer.clear();
            return;
        }

        let original = trimmed.to_string();
        let command = match glyph::read_string(&original) {
            Ok(_) => original,
            Err(orig_err) => {
                let closed = auto_close(&original);
                if glyph::read_string(&closed).is_ok() {
                    closed
                } else {
                    // Auto-close didn't help — show error against original input
                    self.event_log.push(format!("> {}", original));
                    self.console_output.clear();
                    self.console_output_color = None;
                    let report = orig_err.report(&original);
                    self.console_output = report;
                    self.console_output_color = Some(RGB::named(RED));
                    self.console_buffer.clear();
                    return;
                }
            }
        };

        self.event_log.push(format!("> {}", command));
        self.console_output.clear();
        self.console_output_color = None;
        self.console_history.push(command.clone());
        match glyph::read_string(&command) {
            Ok(forms) => {
                // Track env-mutating forms for save/load persistence
                for form in &forms {
                    if is_env_mutating_form(form) {
                        self.user_source.push(form.to_string());
                    }
                }
                let mut last = Value::Nil;
                let mut err = None;
                let env = self.glyph_env.clone();
                for form in &forms {
                    match glyph::eval_with_opts(form, &env, glyph::SandboxOptions::default(), self)
                    {
                        Ok(val) => last = val,
                        Err(e) => {
                            err = Some(e);
                            break;
                        }
                    }
                }
                match err {
                    Some(e) => {
                        let msg = format!("Error: {}", e);
                        self.console_output = msg;
                        self.console_output_color = Some(RGB::named(RED));
                    }
                    None => {
                        if last == glyph::kw("quit-terminal") {
                            self.console_output = "Terminal closed.".to_string();
                            self.event_log.push("Terminal closed.");
                            self.console_buffer.clear();
                            self.mode = Mode::Normal;
                            return;
                        }
                        // Check for endings at the Core (depth 17)
                        if self.depth == 17 {
                            let cmd = command.to_lowercase();
                            if cmd.contains("unregister") && cmd.contains("vessel") {
                                self.ending = Some("DESTROY THE SELF\n\nvessel/suppress unregistered.\nNo replacement rule found.\nConsciousness: terminated.\n\nYou deleted the rule without replacement.\nThere is no defense now.\nYou dissolve into the system.\n\nPress q to quit."
                                    .into());
                            } else if cmd.contains("threshold") && cmd.contains("100") {
                                self.ending = Some("MAINTAIN SUPPRESSION\n\nThreshold restored to 100.\nConsciousness stabilized.\nSuppression maintained.\n\nYou are safe.\nYou are safe.\nYou are safe.\n\nPress q to quit."
                                    .into());
                            } else if cmd.contains("threshold")
                                || cmd.contains("disable")
                                || cmd.contains("redirect")
                            {
                                self.ending = Some("REINTEGRATE\n\nI remember now.\nThe yellow walls. The dog.\nThe reason I locked myself away.\nIt was worth it.\n\nYou lowered the threshold.\nPain returns — but so does joy.\nYou accept what you can remember.\nYou make peace with what's permanently lost.\n\nPress q to quit."
                                    .into());
                            }
                        }
                        let msg = console_response(&self.console_output, &last);
                        self.console_output = msg;
                    }
                }
            }
            Err(e) => {
                let report = e.report(&command);
                self.console_output = report;
                self.console_output_color = Some(RGB::named(RED));
            }
        }
        self.console_buffer.clear();
    }
}

pub(crate) fn clamp_to_char_boundary(text: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

fn previous_char_boundary(text: &str, cursor: usize) -> usize {
    let cursor = clamp_to_char_boundary(text, cursor);
    if cursor == 0 {
        return 0;
    }

    text[..cursor]
        .char_indices()
        .next_back()
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn next_char_boundary(text: &str, cursor: usize) -> usize {
    let cursor = clamp_to_char_boundary(text, cursor);
    if cursor >= text.len() {
        return text.len();
    }

    cursor
        + text[cursor..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0)
}

fn previous_word_boundary(text: &str, cursor: usize) -> usize {
    let mut pos = clamp_to_char_boundary(text, cursor);

    while let Some((index, ch)) = text[..pos].char_indices().next_back() {
        if is_console_word_char(ch) {
            break;
        }
        pos = index;
    }

    while let Some((index, ch)) = text[..pos].char_indices().next_back() {
        if !is_console_word_char(ch) {
            break;
        }
        pos = index;
    }

    pos
}

fn next_word_boundary(text: &str, cursor: usize) -> usize {
    let mut pos = clamp_to_char_boundary(text, cursor);

    while pos < text.len() {
        let ch = text[pos..].chars().next().expect("pos is a char boundary");
        if is_console_word_char(ch) {
            break;
        }
        pos += ch.len_utf8();
    }

    while pos < text.len() {
        let ch = text[pos..].chars().next().expect("pos is a char boundary");
        if !is_console_word_char(ch) {
            break;
        }
        pos += ch.len_utf8();
    }

    pos
}

fn is_console_word_char(ch: char) -> bool {
    ch.is_alphanumeric()
        || matches!(
            ch,
            '_' | '-' | '?' | '!' | '*' | '/' | '+' | '<' | '>' | '='
        )
}

fn console_response(printed: &str, value: &Value) -> String {
    let mut response = printed.trim_end_matches('\n').to_string();
    if value != &Value::Nil {
        if !response.is_empty() {
            response.push('\n');
        }
        response.push_str("=> ");
        response.push_str(&console_value_text(value));
    }
    if response.is_empty() {
        "=> nil".to_string()
    } else {
        response
    }
}

fn console_value_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Returns true if the top-level form mutates the Glyph environment.
fn is_env_mutating_form(form: &Value) -> bool {
    match form {
        Value::List(items) if !items.is_empty() => match &items[0] {
            Value::Symbol(s) => {
                matches!(s.name.as_str(), "const" | "defmacro" | "set!" | "bind-key")
            }
            _ => false,
        },
        _ => false,
    }
}

/// Auto-close unmatched opening brackets/parens/braces in source code.
///
/// Skips contents of string literals and line comments so that parens
/// inside those don't confuse the balancing.
pub(crate) fn auto_close(s: &str) -> String {
    let mut stack: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if in_string {
            if ch == '"' {
                in_string = false;
            } else if ch == '\\' {
                // Skip escaped char
                chars.next();
            }
        } else {
            match ch {
                '(' => stack.push(')'),
                '[' => stack.push(']'),
                '{' => stack.push('}'),
                ')' | ']' | '}' => {
                    stack.pop();
                }
                '"' => in_string = true,
                ';' => {
                    // Skip to end of line
                    loop {
                        match chars.next() {
                            Some('\n') | None => break,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut result = s.to_string();
    while let Some(closer) = stack.pop() {
        result.push(closer);
    }
    result
}
