//! Entity interactions: wizard dialogue, signs, and memory fragments.

use bracket_color::prelude::{CYAN, DARK_GRAY, GREEN, RGB};

use crate::{
    builtins::bind_do_attack,
    entity::{EntityId, Hp},
    world::World,
};

impl World {
    pub(crate) fn interact_with_wizard(&mut self, _wizard_id: EntityId) {
        if !self.wizard_taught {
            if self.depth == 0 {
                self.event_log.push_colored(
                    "\"Good. I see you're awake. Keep moving; I'll find you below.\"",
                    RGB::named(CYAN),
                );
                return;
            }

            let max_hp = self.player_hp().max;
            self.ecs.set_hp(self.player_id, Hp::new(max_hp));
            self.player_can_attack = true;
            self.wizard_taught = true;
            bind_do_attack(&self.glyph_env);

            self.event_log
                .push_colored("The wizard raises a glowing hand...", RGB::named(CYAN));
            self.event_log.push_colored(
                "\"I can't keep everything off you forever. It's time you learned to strike back.\"",
                RGB::named(CYAN),
            );
            self.event_log.push_colored(
                "Warmth spreads through your body. HP fully restored.",
                RGB::named(CYAN),
            );
            self.event_log.push_colored(
                "Open the console (`) and bind attack to a key:",
                RGB::named(CYAN),
            );
            self.event_log.push_colored(
                "  (bind-key :z (do-attack))    -- attacks in facing direction",
                RGB::named(GREEN),
            );
            self.event_log.push_colored(
                "  (bind-key :x (do-attack :east))   (bind-key :c (do-attack :west))",
                RGB::named(GREEN),
            );
            self.event_log.push_colored(
                "\"Bind it, and the way down will open. I'll be just ahead of you. I always am.\"",
                RGB::named(CYAN),
            );
            return;
        }

        let heal = match self.on_wizard_interact {
            Some(f) => f(self),
            None => {
                self.event_log
                    .push_colored("\"Keep going. You're getting closer.\"", RGB::named(CYAN));
                true
            }
        };

        if heal {
            let max_hp = self.player_hp().max;
            self.ecs.set_hp(self.player_id, Hp::new(max_hp));
            self.event_log.push_colored(
                "The wizard taps your shoulder. You feel refreshed.",
                RGB::named(CYAN),
            );
        }
    }

    pub(crate) fn interact_with_sign(&mut self, sign_id: EntityId) {
        let message = self.ecs.sign_message(sign_id).unwrap_or("").to_string();

        // Echo the sign into the event log the first time it's read this level,
        // so the player can scroll back and re-read it after closing the
        // overlay. Re-bumping the same sign just reopens the overlay without
        // spamming the log.
        if self.read_signs.insert(sign_id) {
            self.event_log
                .push_colored("-- sign --", RGB::named(DARK_GRAY));
            for line in message.lines() {
                if line.trim().is_empty() {
                    self.event_log.push("");
                } else {
                    self.event_log
                        .push_colored(line.to_string(), RGB::named(CYAN));
                }
            }
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
                    self.event_log.push("===================================");
                    self.event_log
                        .push_colored(format!("         MEMORY: {}", frag.id), RGB::named(GREEN));
                    self.event_log.push("===================================");
                    for line in frag.text.lines() {
                        if line.is_empty() {
                            self.event_log.push("");
                        } else {
                            self.event_log
                                .push_colored(line.to_string(), RGB::named(GREEN));
                        }
                    }
                    self.event_log.push_colored(
                        format!(
                            "Collected {} of 33 findable memories.",
                            self.fragment_registry.collected_count()
                        ),
                        RGB::named(CYAN),
                    );
                }
                if first_fragment {
                    self.bindings
                        .insert("m".into(), "(toggle-memories!)".into());
                    self.has_new_bindings = true;
                    self.new_binding_keys.insert("m".into());
                    self.event_log
                        .push_colored("Press m to view collected memories.", RGB::named(CYAN));
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
