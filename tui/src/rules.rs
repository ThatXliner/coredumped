//! Structured rule registry for the Xlyph prototype.
//!
//! Each rule has an ID, name, phase, cost, and source lines. The `RuleRegistry`
//! is populated at init and read by the inspector, console queries, and renderer.
//! In this phase rules are static; overlays and patching come later.

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
}

#[derive(Clone, Debug)]
pub struct RuleRegistry {
    rules: Vec<Rule>,
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
                        "  (if (adjacent? slime player)",
                        "    (attack! slime player 1)",
                        "    (step-toward! slime player)))",
                    ],
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
    fn core_registry_has_two_rules() {
        let registry = RuleRegistry::core();
        assert_eq!(registry.len(), 2);
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
        assert_eq!(ids, vec!["slime-hunt", "flashlight"]);
    }
}
