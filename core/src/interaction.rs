//! Entity interactions: wizard dialogue, signs, and memory fragments.

use bracket_color::prelude::{DARK_GRAY, RGB};

use crate::{
    builtins::bind_do_attack,
    dialogue::{Dialogue, DialogueLine, DialogueSpeaker, WizardDialogue},
    entity::{EntityId, Hp},
    world::World,
};

impl World {
    pub(crate) fn interact_with_wizard(&mut self, _wizard_id: EntityId) {
        if !self.wizard_taught {
            if self.depth == 0 {
                Dialogue::wizard(&["Good. I see you're awake. Keep moving; I'll find you below."])
                    .log(&mut self.event_log);
                return;
            }

            let max_hp = self.player_hp().max;
            self.ecs.set_hp(self.player_id, Hp::new(max_hp));
            self.player_can_attack = true;
            self.wizard_taught = true;
            bind_do_attack(&self.glyph_env);

            Dialogue::mixed(
                DialogueSpeaker::Wizard,
                [
                    DialogueLine::action("The wizard raises a glowing hand..."),
                    DialogueLine::speech(
                        "I can't keep everything off you forever. It's time you learned to strike back.",
                    ),
                    DialogueLine::action("Warmth spreads through your body. HP fully restored."),
                    DialogueLine::plain("Open the console (`) and bind attack to a key:"),
                    DialogueLine::code(
                        "(bind-key :z (do-attack))    -- attacks in facing direction",
                    ),
                    DialogueLine::code(
                        "(bind-key :x (do-attack :east))   (bind-key :c (do-attack :west))",
                    ),
                    DialogueLine::speech(
                        "Bind it, and the way down will open. I'll be just ahead of you. I always am.",
                    ),
                ],
            )
            .log(&mut self.event_log);
            return;
        }

        let outcome = match self.on_wizard_interact {
            Some(f) => f(self),
            None => {
                WizardDialogue::healing(Dialogue::wizard(&["Keep going. You're getting closer."]))
            }
        };
        let mut dialogue = outcome.dialogue;

        if outcome.heals_player {
            let max_hp = self.player_hp().max;
            self.ecs.set_hp(self.player_id, Hp::new(max_hp));
            dialogue = dialogue.line(DialogueLine::action(
                "The wizard taps your shoulder. You feel refreshed.",
            ));
        }
        dialogue.log(&mut self.event_log);
    }

    pub(crate) fn interact_with_sign(&mut self, sign_id: EntityId) {
        let message = self.ecs.sign_message(sign_id).unwrap_or("").to_string();

        // Echo the sign into the event log the first time it's read this level,
        // so the player can scroll back and re-read it after closing the
        // overlay. Re-bumping the same sign just reopens the overlay without
        // spamming the log.
        if self.read_signs.insert(sign_id) {
            Dialogue::mixed(
                DialogueSpeaker::Sign,
                message.lines().map(|line| {
                    if line.trim().is_empty() {
                        DialogueLine::plain("")
                    } else {
                        DialogueLine::speech(line)
                    }
                }),
            )
            .log(&mut self.event_log);
        }

        self.sign_text = message;
        self.sign_scroll = 0;
        self.mode = crate::game::Mode::ReadingSign;
    }

    pub(crate) fn interact_with_fragment(&mut self, fragment_id: EntityId) {
        if let Some(frag_id) = self.ecs.fragment_id(fragment_id).map(|s| s.to_string()) {
            if self.fragment_registry.collect(&frag_id) {
                let first_fragment = self.fragment_registry.collected_count() == 1;
                if let Some(frag) = self.fragment_registry.get(&frag_id) {
                    let mut dialogue = Dialogue::new(DialogueSpeaker::Memory)
                        .line(DialogueLine::plain(format!("MEMORY: {}", frag.id)))
                        .line(DialogueLine::plain(""));
                    for line in frag.text.lines() {
                        dialogue = dialogue.line(DialogueLine::plain(line));
                    }
                    dialogue
                        .line(DialogueLine::plain(format!(
                            "Collected {} of {} findable memories.",
                            self.fragment_registry.collected_count(),
                            self.fragment_registry.findable_count()
                        )))
                        .log(&mut self.event_log);
                }
                if first_fragment {
                    self.bindings
                        .insert("m".into(), "(toggle-memories!)".into());
                    self.has_new_bindings = true;
                    self.new_binding_keys.insert("m".into());
                    Dialogue::mixed(
                        DialogueSpeaker::Narration,
                        [DialogueLine::hint("Press m to view collected memories.")],
                    )
                    .log(&mut self.event_log);
                }
                self.ecs.remove(fragment_id);
            } else if self
                .fragment_registry
                .get(&frag_id)
                .map(|frag| frag.status == crate::fragment::FragmentStatus::Collected)
                .unwrap_or(false)
            {
                self.event_log.push_colored(
                    format!("Memory {} is already recovered.", frag_id),
                    RGB::named(DARK_GRAY),
                );
                self.ecs.remove(fragment_id);
            }
        }
    }
}
