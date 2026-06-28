//! Small dialogue helpers for authored speech and readable log transcripts.

use bracket_color::prelude::{CYAN, DARK_GRAY, GREEN, RED, RGB, YELLOW};

use crate::event_log::EventLog;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogueSpeaker {
    Wizard,
    Sign,
    Memory,
    Narration,
}

impl DialogueSpeaker {
    fn label(self) -> &'static str {
        match self {
            Self::Wizard => "wizard",
            Self::Sign => "sign",
            Self::Memory => "memory",
            Self::Narration => "system",
        }
    }

    fn accent(self) -> RGB {
        match self {
            Self::Wizard | Self::Sign => RGB::named(CYAN),
            Self::Memory => RGB::named(GREEN),
            Self::Narration => RGB::named(YELLOW),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogueLineKind {
    Speech,
    Action,
    Hint,
    Code,
    Danger,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DialogueLine {
    kind: DialogueLineKind,
    text: String,
}

impl DialogueLine {
    pub(crate) fn speech(text: impl Into<String>) -> Self {
        Self {
            kind: DialogueLineKind::Speech,
            text: text.into(),
        }
    }

    pub(crate) fn action(text: impl Into<String>) -> Self {
        Self {
            kind: DialogueLineKind::Action,
            text: text.into(),
        }
    }

    pub(crate) fn hint(text: impl Into<String>) -> Self {
        Self {
            kind: DialogueLineKind::Hint,
            text: text.into(),
        }
    }

    pub(crate) fn code(text: impl Into<String>) -> Self {
        Self {
            kind: DialogueLineKind::Code,
            text: text.into(),
        }
    }

    pub(crate) fn danger(text: impl Into<String>) -> Self {
        Self {
            kind: DialogueLineKind::Danger,
            text: text.into(),
        }
    }

    pub(crate) fn plain(text: impl Into<String>) -> Self {
        Self {
            kind: DialogueLineKind::Plain,
            text: text.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Dialogue {
    speaker: DialogueSpeaker,
    lines: Vec<DialogueLine>,
}

impl Dialogue {
    pub(crate) fn new(speaker: DialogueSpeaker) -> Self {
        Self {
            speaker,
            lines: Vec::new(),
        }
    }

    pub(crate) fn wizard(lines: &[&str]) -> Self {
        Self::spoken(DialogueSpeaker::Wizard, lines)
    }

    pub(crate) fn spoken(speaker: DialogueSpeaker, lines: &[&str]) -> Self {
        Self {
            speaker,
            lines: lines
                .iter()
                .map(|line| DialogueLine::speech(*line))
                .collect(),
        }
    }

    pub(crate) fn mixed(
        speaker: DialogueSpeaker,
        lines: impl IntoIterator<Item = DialogueLine>,
    ) -> Self {
        Self {
            speaker,
            lines: lines.into_iter().collect(),
        }
    }

    pub(crate) fn line(mut self, line: DialogueLine) -> Self {
        self.lines.push(line);
        self
    }

    pub(crate) fn log(&self, event_log: &mut EventLog) {
        event_log.push_colored(
            format!("-- {} --", self.speaker.label()),
            RGB::named(DARK_GRAY),
        );
        for line in &self.lines {
            if line.text.trim().is_empty() {
                event_log.push("");
                continue;
            }
            event_log.push_colored(self.format_line(line), self.line_color(line));
        }
    }

    fn format_line(&self, line: &DialogueLine) -> String {
        match line.kind {
            DialogueLineKind::Speech => format!("{}: {}", self.speaker.label(), line.text),
            DialogueLineKind::Action => format!("* {}", line.text),
            DialogueLineKind::Hint | DialogueLineKind::Code | DialogueLineKind::Danger => {
                format!("  {}", line.text)
            }
            DialogueLineKind::Plain => line.text.clone(),
        }
    }

    fn line_color(&self, line: &DialogueLine) -> RGB {
        match line.kind {
            DialogueLineKind::Speech | DialogueLineKind::Action | DialogueLineKind::Plain => {
                self.speaker.accent()
            }
            DialogueLineKind::Hint | DialogueLineKind::Code => RGB::named(GREEN),
            DialogueLineKind::Danger => RGB::named(RED),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WizardDialogue {
    pub(crate) dialogue: Dialogue,
    pub(crate) heals_player: bool,
}

impl WizardDialogue {
    pub(crate) fn healing_lines(lines: &[&str]) -> Self {
        Self::healing(Dialogue::wizard(lines))
    }

    pub(crate) fn no_heal_lines(lines: &[&str]) -> Self {
        Self::no_heal(Dialogue::wizard(lines))
    }

    pub(crate) fn healing(dialogue: Dialogue) -> Self {
        Self {
            dialogue,
            heals_player: true,
        }
    }

    pub(crate) fn no_heal(dialogue: Dialogue) -> Self {
        Self {
            dialogue,
            heals_player: false,
        }
    }
}
