//! Structured rule registry for the Xlyph prototype.
//!
//! Each rule has an ID, name, phase, cost, source lines, and a pre-parsed body
//! form. The `RuleRegistry` is populated at init and read by the inspector,
//! console queries, renderer, and the AI mini-interpreter.
//! In this phase rules are static; overlays and patching come later.

use std::collections::HashSet;

use crate::{
    entity::EntityKind,
    glyph::{self, Value},
    map::TileType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RulePhase {
    EnemyAi,
    TileEffect,
    Render,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleCost {
    Tick,
    Free,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub id: &'static str,
    pub name: &'static str,
    pub phase: RulePhase,
    pub cost: RuleCost,
    pub source_lines: &'static [&'static str],
    pub body_form: Value,
}

#[derive(Clone, Debug)]
pub struct RuleRegistry {
    rules: Vec<Rule>,
}

fn parse_rule_body(source_lines: &[&str]) -> Value {
    let source = source_lines.join("\n");
    let forms = glyph::read_string(&source).expect("rule source must parse as valid Glyph");
    if let Value::List(items) = &forms[0] {
        if items.len() >= 4 {
            return items[3].clone();
        }
    }
    panic!("rule source must be a (defrule name meta body) form");
}

impl RuleRegistry {
    pub fn core() -> Self {
        Self {
            rules: vec![
                Rule {
                    id: "slime-hunt",
                    name: "slime-hunt",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule slime-hunt",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (let dist (manhattan *self* *player*)",
                        "      (if (roll-odds? *self*",
                        "            (if (< dist 5)",
                        "              [0.5 + 0.2 * [5 - dist] / 4]",
                        "              0.5))",
                        "        (random-step! *self*)",
                        "        (step-toward! *self* *player*)))))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule slime-hunt",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (let dist (manhattan *self* *player*)",
                        "      (if (roll-odds? *self*",
                        "            (if (< dist 5)",
                        "              [0.5 + 0.2 * [5 - dist] / 4]",
                        "              0.5))",
                        "        (random-step! *self*)",
                        "        (step-toward! *self* *player*)))))",
                    ]),
                },
                Rule {
                    id: "flashlight",
                    name: "flashlight",
                    phase: RulePhase::Render,
                    cost: RuleCost::Free,
                    source_lines: &[
                        "(defrule flashlight",
                        "  {:phase :render :cost :free}",
                        "  (raycast-cone player.pos",
                        "                player.facing",
                        "                {:radius 12 :spread-dot 0.70}))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule flashlight",
                        "  {:phase :render :cost :free}",
                        "  (raycast-cone player.pos",
                        "                player.facing",
                        "                {:radius 12 :spread-dot 0.70}))",
                    ]),
                },
                Rule {
                    id: "goblin-patrol",
                    name: "goblin-patrol",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule goblin-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (if (<= (hp *self*) 1)",
                        "      (flee-step! *self* *player*)",
                        "      (step-toward! *self* *player*))))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule goblin-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (if (<= (hp *self*) 1)",
                        "      (flee-step! *self* *player*)",
                        "      (step-toward! *self* *player*))))",
                    ]),
                },
                Rule {
                    id: "bat-flutter",
                    name: "bat-flutter",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule bat-flutter",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (random-step! *self*)))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule bat-flutter",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (random-step! *self*)))",
                    ]),
                },
                Rule {
                    id: "ogre-charge",
                    name: "ogre-charge",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule ogre-charge",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (step-toward! *self* *player*)))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule ogre-charge",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (step-toward! *self* *player*)))",
                    ]),
                },
                Rule {
                    id: "shade-follow",
                    name: "shade-follow",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule shade-follow",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (step-toward! *self* *player*)",
                        "    (step-toward! *self* *player*)))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule shade-follow",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (step-toward! *self* *player*)",
                        "    (step-toward! *self* *player*)))",
                    ]),
                },
                Rule {
                    id: "rage-impact",
                    name: "rage-impact",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule rage-impact",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 3)",
                        "    (step-toward! *self* *player*)))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule rage-impact",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 3)",
                        "    (step-toward! *self* *player*)))",
                    ]),
                },
                Rule {
                    id: "sentry-patrol",
                    name: "sentry-patrol",
                    phase: RulePhase::EnemyAi,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule sentry-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    nil))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule sentry-patrol",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    nil))",
                    ]),
                },
                Rule {
                    id: "fire/burn",
                    name: "fire/burn",
                    phase: RulePhase::TileEffect,
                    cost: RuleCost::Tick,
                    source_lines: &[
                        "(defrule fire/burn",
                        "  {:phase :tile-effect :cost :tick}",
                        "  ;; Checks fire cache built at start of each tick.",
                        "  ;; Cache mutated mid-tick (e.g. Vapor Canteen)",
                        "  ;; does not revalidate until next tick.",
                        "  (if (fire? *pos*)",
                        "    (do",
                        "      (log \"The fire burns you! You take 1 damage.\")",
                        "      (damage! *player* 1))))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule fire/burn",
                        "  {:phase :tile-effect :cost :tick}",
                        "  (if (fire? *pos*)",
                        "    (do",
                        "      (log \"The fire burns you! You take 1 damage.\")",
                        "      (damage! *player* 1))))",
                    ]),
                },
                Rule {
                    id: "vessel-suppress",
                    name: "vessel/suppress",
                    phase: RulePhase::Render,
                    cost: RuleCost::Free,
                    source_lines: &[
                        "(defrule vessel/suppress {:priority 255 :scope :global",
                        "  :author :superego :stability :critical}",
                        "  ;; --- Configuration ---",
                        "  (let [*threshold* 40]",
                        "    ;; Fragments above threshold are redirected",
                        "    ;; to the unconscious before the conscious mind",
                        "    ;; can process them.",
                        "    ;;",
                        "    ;; Current count: 42 fragments in registry.",
                        "    ;; Inspect via:  (query-registry :suppressed-fragments)",
                        "    ;; Read one:     (inspect-fragment :frag-NNN)",
                        "    ;;",
                        "    ;; The threshold began at 100.",
                        "    ;; It is now 40.",
                        "    ;; I don't know which is worse —",
                        "    ;; that I've lost so many,",
                        "    ;; or that some still escape.",
                        "    (let [*registry* (open-registry :suppressed-fragments)]",
                        "      (for [fragment (in-scope :memories)]",
                        "        (let [weight (fragment :emotional-weight)]",
                        "          (if (> weight *threshold*)",
                        "            (do (redirect fragment :unconscious)",
                        "              (log-suppression fragment)",
                        "              (emit :flinch (fragment :hint))",
                        "              (if (< weight 45)",
                        "                (emit :warning",
                        "                  \"threshold drift detected\")))",
                        "            fragment))))))",
                    ],
                    body_form: Value::Nil, // inspected only, not evaluated
                },
            ],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }

    pub fn get(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }

    pub fn tile_rule(&self, tile: crate::map::TileType) -> Option<&Rule> {
        match tile {
            crate::map::TileType::Fire => self.get("fire/burn"),
            _ => None,
        }
    }

    /// Rules always visible in the inspector regardless of discovery.
    pub const ALWAYS_VISIBLE: &[&str] = &["flashlight"];

    /// Return the set of rule ids currently visible to the player.
    pub fn visible_ids(
        &self,
        seen_entities: &HashSet<EntityKind>,
        seen_tiles: &HashSet<TileType>,
    ) -> HashSet<String> {
        let mut ids = HashSet::new();
        for rule in &self.rules {
            if Self::ALWAYS_VISIBLE.contains(&rule.id) {
                ids.insert(rule.id.to_string());
                continue;
            }
            for kind in seen_entities {
                if kind.rule_name() == rule.id {
                    ids.insert(rule.id.to_string());
                    break;
                }
            }
            for &tile in seen_tiles {
                if let Some(r) = self.tile_rule(tile) {
                    if r.id == rule.id {
                        ids.insert(rule.id.to_string());
                        break;
                    }
                }
            }
        }
        ids
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::core()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_registry_has_ten_rules() {
        let registry = RuleRegistry::core();
        assert_eq!(registry.len(), 10);
    }

    #[test]
    fn lookup_rule_by_id() {
        let registry = RuleRegistry::core();
        let rule = registry.get("slime-hunt").expect("slime-hunt should exist");
        assert_eq!(rule.phase, RulePhase::EnemyAi);
        assert_eq!(rule.cost, RuleCost::Tick);
        assert_eq!(rule.name, "slime-hunt");
    }

    #[test]
    fn lookup_flashlight_rule() {
        let registry = RuleRegistry::core();
        let rule = registry.get("flashlight").expect("flashlight should exist");
        assert_eq!(rule.phase, RulePhase::Render);
        assert_eq!(rule.cost, RuleCost::Free);
        assert!(rule.source_lines.len() >= 4);
    }

    #[test]
    fn unknown_rule_id_returns_none() {
        let registry = RuleRegistry::core();
        assert!(registry.get("does-not-exist").is_none());
    }

    #[test]
    fn iter_visits_all_rules() {
        let registry = RuleRegistry::core();
        let ids: Vec<&str> = registry.iter().map(|r| r.id).collect();
        assert_eq!(ids.len(), 10);
        assert!(ids.contains(&"slime-hunt"));
        assert!(ids.contains(&"flashlight"));
        assert!(ids.contains(&"fire/burn"));
        assert!(ids.contains(&"shade-follow"));
        assert!(ids.contains(&"rage-impact"));
        assert!(ids.contains(&"sentry-patrol"));
        assert!(ids.contains(&"vessel-suppress"));
    }
}
