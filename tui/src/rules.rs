//! Structured rule registry for the Xlyph prototype.
//!
//! Each rule has an ID, name, phase, cost, source lines, and a pre-parsed body
//! form. The `RuleRegistry` is populated at init and read by the inspector,
//! console queries, renderer, and the AI mini-interpreter.
//! In this phase rules are static; overlays and patching come later.

use crate::glyph::{self, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RulePhase {
    EnemyAi,
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
                        "    (if (roll-odds? *self* 0.5)",
                        "      (random-step! *self*)",
                        "      (step-toward! *self* *player*))))",
                    ],
                    body_form: parse_rule_body(&[
                        "(defrule slime-hunt",
                        "  {:phase :enemy-ai :cost :tick}",
                        "  (if (adjacent? *self* *player*)",
                        "    (attack! *self* *player* 1)",
                        "    (if (roll-odds? *self* 0.5)",
                        "      (random-step! *self*)",
                        "      (step-toward! *self* *player*))))",
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
            ],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }

    pub fn get(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
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
    fn core_registry_has_five_rules() {
        let registry = RuleRegistry::core();
        assert_eq!(registry.len(), 5);
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
        assert_eq!(
            ids,
            vec![
                "slime-hunt",
                "flashlight",
                "goblin-patrol",
                "bat-flutter",
                "ogre-charge"
            ]
        );
    }
}
