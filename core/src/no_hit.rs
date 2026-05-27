//! No-hit route detector for the current prototype rules.
//!
//! This module is intentionally not wired into level generation or rendering.
//! It answers the design question "is there a known route to this exit without
//! taking damage?" by replaying real `World` ticks in a bounded breadth-first
//! search.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    entity::{Direction, Position},
    game::Intent,
    world::World,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoHitAction {
    Move(Direction),
    Wait,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoHitAnalysis {
    pub possible: bool,
    pub route: Option<Vec<NoHitAction>>,
    pub explored_states: usize,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoHitOptions {
    pub max_states: usize,
    pub max_depth: usize,
}

impl Default for NoHitOptions {
    fn default() -> Self {
        Self {
            max_states: 50_000,
            max_depth: 200,
        }
    }
}

#[derive(Clone, Debug)]
struct SearchNode {
    world: World,
    depth: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct WorldKey {
    player: Position,
    player_hp: i32,
    enemies: Vec<EnemyKey>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct EnemyKey {
    id: usize,
    pos: Position,
    hp: i32,
    alive: bool,
}

pub fn detect_no_hit_route(world: &World, exit: Position, options: NoHitOptions) -> NoHitAnalysis {
    if !world.map.is_walkable(exit) {
        return NoHitAnalysis {
            possible: false,
            route: None,
            explored_states: 0,
            truncated: false,
        };
    }

    if world.player_pos() == exit {
        return NoHitAnalysis {
            possible: true,
            route: Some(Vec::new()),
            explored_states: 0,
            truncated: false,
        };
    }

    let actions = [
        NoHitAction::Move(Direction::North),
        NoHitAction::Move(Direction::South),
        NoHitAction::Move(Direction::West),
        NoHitAction::Move(Direction::East),
        NoHitAction::Wait,
    ];

    let start_key = WorldKey::from_world(world);
    let mut queue = VecDeque::from([SearchNode {
        world: world.clone(),
        depth: 0,
    }]);
    let mut visited = HashSet::from([start_key.clone()]);
    let mut parents: HashMap<WorldKey, (WorldKey, NoHitAction)> = HashMap::new();
    let mut explored_states = 0;
    let mut truncated = false;

    while let Some(node) = queue.pop_front() {
        explored_states += 1;

        if explored_states >= options.max_states || node.depth >= options.max_depth {
            truncated = true;
            continue;
        }

        let parent_key = WorldKey::from_world(&node.world);
        let parent_hp = node.world.player_hp().current;

        for action in actions {
            let mut next_world = node.world.clone();
            let intent = action.intent();
            next_world.apply_intent(intent);

            if next_world.player_hp().current < parent_hp {
                continue;
            }

            let next_key = WorldKey::from_world(&next_world);
            if next_world.player_pos() == exit {
                let route = reconstruct_route(&parents, &parent_key, action);
                return NoHitAnalysis {
                    possible: true,
                    route: Some(route),
                    explored_states,
                    truncated: false,
                };
            }

            if visited.insert(next_key.clone()) {
                parents.insert(next_key, (parent_key.clone(), action));
                queue.push_back(SearchNode {
                    world: next_world,
                    depth: node.depth + 1,
                });
            }
        }
    }

    NoHitAnalysis {
        possible: false,
        route: None,
        explored_states,
        truncated,
    }
}

fn reconstruct_route(
    parents: &HashMap<WorldKey, (WorldKey, NoHitAction)>,
    parent_key: &WorldKey,
    final_action: NoHitAction,
) -> Vec<NoHitAction> {
    let mut route = vec![final_action];
    let mut cursor = parent_key;

    while let Some((previous, action)) = parents.get(cursor) {
        route.push(*action);
        cursor = previous;
    }

    route.reverse();
    route
}

impl NoHitAction {
    fn intent(self) -> Intent {
        match self {
            NoHitAction::Move(direction) => Intent::Move(direction),
            NoHitAction::Wait => Intent::Wait,
        }
    }
}

impl WorldKey {
    fn from_world(world: &World) -> Self {
        let mut enemies: Vec<EnemyKey> = world
            .living_enemies()
            .map(|enemy| EnemyKey {
                id: enemy.id.raw(),
                pos: enemy.pos,
                hp: enemy.hp.current,
                alive: enemy.alive,
            })
            .collect();
        enemies.sort_by_key(|enemy| enemy.id);

        Self {
            player: world.player_pos(),
            player_hp: world.player_hp().current,
            enemies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ecs::Ecs, entity::Hp, event_log::EventLog, map::Map, rules::RuleRegistry};

    fn empty_world(player: Position) -> World {
        let mut ecs = Ecs::new();
        let player_id = ecs.spawn_player(player);

        World {
            map: Map::new_static(),
            ecs,
            registry: RuleRegistry::core(),
            player_id,
            player_facing: Direction::East,
            depth: 0,
            turn: 0,
            mode: crate::game::Mode::Normal,
            event_log: EventLog::new(),
            console_buffer: String::new(),
            console_output: String::new(),
            console_output_color: None,
            glyph_env: crate::glyph::Env::extend(&crate::glyph::default_env()),
            binding_env: crate::glyph::Env::extend(&crate::glyph::default_env()),
            inspector_selection: 0,
            memory_scroll: 0,
            player_attacked: Vec::new(),
            blocking: false,
            running: true,
            player_can_attack: false,
            wizard_taught: false,
            wizard_id: None,
            bindings: std::collections::HashMap::new(),
            has_new_bindings: false,
            new_binding_keys: std::collections::HashSet::new(),
            konami_index: 0,
            cheat_unlocked: false,
            console_history: Vec::new(),
            console_history_index: 0,
            console_history_draft: String::new(),
            console_cursor: 0,
            console_output_scroll: 0,
            event_log_scroll: 0,
            confirming_quit: false,
            user_source: Vec::new(),
            pending_wipe_slot: None,
            quit_countdown: 0,
            seen_entity_kinds: std::collections::HashSet::new(),
            seen_tile_types: std::collections::HashSet::new(),
            new_rule_ids: std::collections::HashSet::new(),
            known_rule_ids: std::collections::HashSet::new(),
            fragment_registry: crate::fragment::FragmentRegistry::new(),
            cached_flashlight: std::collections::HashSet::new(),
            cached_flashlight_pos: Position::new(-1, -1),
            cached_flashlight_facing: Direction::East,
            ending: None,
            registry_write_unlocked: false,
            held_keys: Vec::new(),
            held_items: Vec::new(),
            gauntlet_barrier_locked: HashSet::new(),
            barrel_room_protected: false,
            fire_cache: HashSet::new(),
            maze_shifting_walls: HashSet::new(),
            maze_shift_frozen: false,
            dijkstra_cache_target_idx: None,
            dijkstra_cache_map: Vec::new(),
            on_wizard_interact: None,
            camera_x: 0,
            camera_y: 0,
            explored_tiles: HashSet::new(),
            render_frame: 0,
        }
    }

    #[test]
    fn finds_trivial_no_hit_route_to_reachable_exit() {
        let world = empty_world(Position::new(5, 5));

        let analysis = detect_no_hit_route(&world, Position::new(7, 5), NoHitOptions::default());

        assert!(analysis.possible);
        assert_eq!(
            analysis.route,
            Some(vec![
                NoHitAction::Move(Direction::East),
                NoHitAction::Move(Direction::East)
            ])
        );
    }

    #[test]
    fn rejects_unwalkable_exit() {
        let world = empty_world(Position::new(5, 5));

        let analysis = detect_no_hit_route(&world, Position::new(0, 0), NoHitOptions::default());

        assert!(!analysis.possible);
        assert_eq!(analysis.route, None);
        assert!(!analysis.truncated);
    }

    #[test]
    fn reports_when_search_budget_is_exhausted() {
        let options = NoHitOptions {
            max_states: 1,
            ..NoHitOptions::default()
        };
        let world = empty_world(Position::new(5, 5));

        let analysis = detect_no_hit_route(&world, Position::new(20, 20), options);

        assert!(!analysis.possible);
        assert!(analysis.truncated);
    }

    #[test]
    fn can_route_around_a_slime_without_getting_hit() {
        let mut world = empty_world(Position::new(5, 5));
        world.ecs.spawn_slime(Position::new(12, 5));
        world.ecs.set_hp(world.player_id, Hp::new(12));

        let analysis = detect_no_hit_route(&world, Position::new(5, 7), NoHitOptions::default());

        assert!(analysis.possible);
        assert!(analysis
            .route
            .expect("route should be present")
            .iter()
            .all(|action| *action != NoHitAction::Wait));
    }
}
