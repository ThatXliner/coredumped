//! Hand-crafted and procedural level definitions.
//!
//! Each level is built by a self-contained function that sets the map, places
//! the player, and spawns entities. Adding a new hand-crafted level means
//! writing one builder function and adding a match arm in [`build_level`].

mod depth_00_foyer;
mod depth_01_wizard_chamber;
mod depth_02_tutorial_grid;
mod depth_03_quiet_halls;
mod depth_04_first_scar;
mod depth_05_jagged_passages;
mod depth_06_gauntlet;
mod depth_07_boiling_heart;
mod depth_08_counting_room;
mod depth_09_the_scale;
mod depth_10_maze_of_regret;
mod depth_11_the_offer;
mod depth_12_long_corridor;
mod depth_13_the_archive;
mod depth_14_ash_field;
mod depth_15_the_clearing;
mod depth_16_the_descent;
mod depth_17_the_core;
mod helpers;
mod procedural;

#[allow(unused_imports)]
pub use depth_01_wizard_chamber::generate_wizard_box;
#[allow(unused_imports)]
pub use helpers::find_stairs_down;

use crate::{entity::Position, map::TileType, world::World};

/// Dispatch to the appropriate level builder for the given depth.
pub fn build_level(world: &mut World, depth: u32) {
    world.clear_level_entities();

    match depth {
        0 => depth_00_foyer::build_foyer(world),
        1 => depth_01_wizard_chamber::build_wizard_chamber(world),
        2 => depth_02_tutorial_grid::build_tutorial_grid(world),
        3 => depth_03_quiet_halls::build_quiet_halls(world),
        4 => depth_04_first_scar::build_first_scar(world),
        5 => depth_05_jagged_passages::build_jagged_passages(world),
        6 => depth_06_gauntlet::build_gauntlet(world),
        7 => depth_07_boiling_heart::build_boiling_heart(world),
        8 => depth_08_counting_room::build_counting_room(world),
        9 => depth_09_the_scale::build_the_scale(world),
        10 => depth_10_maze_of_regret::build_maze_of_regret(world),
        11 => depth_11_the_offer::build_the_offer(world),
        12 => depth_12_long_corridor::build_long_corridor(world),
        13 => depth_13_the_archive::build_the_archive(world),
        14 => depth_14_ash_field::build_ash_field(world),
        15 => depth_15_the_clearing::build_the_clearing(world),
        16 => depth_16_the_descent::build_the_descent(world),
        17 => depth_17_the_core::build_the_core(world),
        _ => procedural::build_procedural_level(world, depth),
    }

    // Initialize fire cache so tile-effect rules see correct state
    world.fire_cache.clear();
    for y in 0..world.map.height {
        for x in 0..world.map.width {
            let pos = Position::new(x, y);
            if world.map.tile(pos) == TileType::Fire {
                world.fire_cache.insert(pos);
            }
        }
    }

    let player_pos = world.player_pos();
    log::info!(
        target: "xlyph::level",
        "built depth={} player=({},{}) entities={} enemies={} wizard_id={:?}",
        depth,
        player_pos.x,
        player_pos.y,
        world.renderable_entities().count(),
        world.living_enemies().count(),
        world.wizard_id.map(|id| id.raw())
    );
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet, VecDeque};

    use crate::{entity::Position, EntityKind, World};

    use super::build_level;

    /// Flood-fill of walkable tiles from `start`, ignoring entities.
    fn reachable_tiles(world: &World, start: Position) -> HashSet<Position> {
        reachable_tiles_with(world, start, |_| false)
    }

    /// Flood-fill that additionally passes through tiles the player can open
    /// or wait out (locked doors, shifting walls).
    fn reachable_tiles_with(
        world: &World,
        start: Position,
        extra_passable: impl Fn(Position) -> bool,
    ) -> HashSet<Position> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(pos) = queue.pop_front() {
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let next = pos.offset(dx, dy);
                if world.map.contains(next)
                    && (world.map.is_walkable(next) || extra_passable(next))
                    && visited.insert(next)
                {
                    queue.push_back(next);
                }
            }
        }
        visited
    }

    #[test]
    fn campaign_fragment_spawns_are_walkable_and_cover_findable_memories() {
        let mut placed = BTreeSet::new();

        for depth in 0..=17 {
            let mut world = World::new_game();
            build_level(&mut world, depth);

            for entity_id in world.ecs.entity_ids() {
                if world.ecs.kind(entity_id) != Some(EntityKind::Fragment) {
                    continue;
                }

                let pos = world
                    .ecs
                    .position(entity_id)
                    .expect("fragment should have a position");
                let fragment_id = world
                    .ecs
                    .fragment_id(entity_id)
                    .expect("fragment should have an id");

                assert!(
                    world.map.is_walkable(pos),
                    "{fragment_id} spawned on a blocked tile at depth {depth}: {pos:?}"
                );
                placed.insert(fragment_id.to_string());
            }
        }

        for idx in 1..=33 {
            let fragment_id = format!("frag-{idx:03}");
            assert!(
                placed.contains(&fragment_id),
                "{fragment_id} has no campaign placement"
            );
        }

        for idx in 34..=42 {
            let fragment_id = format!("frag-{idx:03}");
            assert!(
                !placed.contains(&fragment_id),
                "{fragment_id} should remain suppressed, not placed"
            );
        }
    }

    #[test]
    fn campaign_enemy_spawns_are_walkable() {
        let mut blocked = Vec::new();

        for depth in 0..=17 {
            let mut world = World::new_game();
            build_level(&mut world, depth);

            for enemy in world.living_enemies() {
                if !world.map.is_walkable(enemy.pos) {
                    blocked.push(format!(
                        "{} spawned on a blocked tile at depth {depth}: {:?}",
                        enemy.name(),
                        enemy.pos
                    ));
                }
            }
        }

        assert!(blocked.is_empty(), "{}", blocked.join("\n"));
    }

    #[test]
    fn quiet_halls_gate_barrels_spawn_on_walkable_tiles() {
        let mut world = World::new_game();
        build_level(&mut world, 3);

        let blocked: Vec<_> = world
            .ecs
            .entity_ids()
            .filter(|id| world.ecs.kind(*id) == Some(crate::EntityKind::Barrel))
            .filter_map(|id| {
                let pos = world.ecs.position(id)?;
                (!world.map.is_walkable(pos)).then_some(pos)
            })
            .collect();

        assert!(
            blocked.is_empty(),
            "quiet halls gate barrels spawned on blocked tiles: {blocked:?}"
        );
    }

    #[test]
    fn first_scar_gates_combat_rooms_with_walkable_barrels() {
        use crate::map::TileType;

        // Procedural layout varies per build; check several so we exercise the
        // gate logic rather than one lucky seed.
        let mut saw_barrels = false;
        for _ in 0..16 {
            let mut world = World::new_game();
            build_level(&mut world, 4);

            let player = world.player_pos();
            for id in world.ecs.entity_ids() {
                if world.ecs.kind(id) != Some(crate::EntityKind::Barrel) {
                    continue;
                }
                saw_barrels = true;
                let pos = world.ecs.position(id).expect("barrel has a position");
                assert!(
                    world.map.is_walkable(pos),
                    "depth 4 gate barrel on a blocked tile: {pos:?}"
                );
                assert_ne!(pos, player, "barrel sealed the player at spawn");
                let tile = world.map.tile(pos);
                assert!(
                    !matches!(tile, TileType::StairsDown | TileType::StairsUp),
                    "barrel covered stairs at {pos:?}"
                );
            }
        }

        assert!(
            saw_barrels,
            "depth 4 produced no gate barrels across 16 builds"
        );
    }

    #[test]
    fn first_scar_wizard_and_sign_are_reachable() {
        // Procedural layout varies per build; the old midpoint placement put
        // the wizard inside a wall on almost every generation.
        for _ in 0..32 {
            let mut world = World::new_game();
            build_level(&mut world, 4);

            let wizard_pos = world
                .wizard_id
                .and_then(|id| world.ecs.position(id))
                .expect("depth 4 spawns a wizard");
            let sign_pos = world
                .ecs
                .entity_ids()
                .find(|id| world.ecs.kind(*id) == Some(EntityKind::Sign))
                .and_then(|id| world.ecs.position(id))
                .expect("depth 4 spawns a sign");

            assert!(
                world.map.is_walkable(wizard_pos),
                "depth 4 wizard in a wall: {wizard_pos:?}"
            );
            assert!(
                world.map.is_walkable(sign_pos),
                "depth 4 sign in a wall: {sign_pos:?}"
            );

            let reachable = reachable_tiles(&world, world.player_pos());
            assert!(
                reachable.contains(&wizard_pos),
                "depth 4 wizard unreachable from start: {wizard_pos:?}"
            );
            assert!(
                reachable.contains(&sign_pos),
                "depth 4 sign unreachable from start: {sign_pos:?}"
            );
        }
    }

    #[test]
    fn counting_room_key_goblins_are_reachable_without_keys() {
        let mut world = World::new_game();
        build_level(&mut world, 8);

        // Locked doors are wall tiles until a key is spent, so the flood fill
        // covers exactly the hub area the player can reach key-less.
        let reachable = reachable_tiles(&world, world.player_pos());
        let goblins: Vec<Position> = world
            .ecs
            .entity_ids()
            .filter(|id| world.ecs.kind(*id) == Some(EntityKind::Goblin))
            .filter_map(|id| world.ecs.position(id))
            .collect();

        assert_eq!(goblins.len(), 3, "counting room should have 3 key-goblins");
        for pos in goblins {
            assert!(
                reachable.contains(&pos),
                "key-goblin sealed behind a locked door at {pos:?}"
            );
        }
    }

    #[test]
    fn same_run_seed_reproduces_generated_levels() {
        // 4 = room gen, 5 = cave gen, 10 = room gen + shifting walls,
        // 18 = procedural fallback.
        for depth in [4u32, 5, 10, 18] {
            let build = |seed: u64| {
                let mut world = World::new_game();
                world.run_seed = seed;
                world.depth = depth;
                build_level(&mut world, depth);
                world
            };
            let a = build(0xDECAF);
            let b = build(0xDECAF);
            let c = build(0xC0FFEE);

            assert_eq!(
                tile_fingerprint(&a),
                tile_fingerprint(&b),
                "depth {depth}: same seed produced different maps"
            );
            assert_eq!(
                a.maze_shifting_walls, b.maze_shifting_walls,
                "depth {depth}: same seed produced different shifting walls"
            );
            assert_ne!(
                tile_fingerprint(&a),
                tile_fingerprint(&c),
                "depth {depth}: map generation ignores the seed"
            );
        }
    }

    fn tile_fingerprint(world: &World) -> Vec<crate::map::TileType> {
        let mut tiles = Vec::new();
        for y in 0..world.map.height {
            for x in 0..world.map.width {
                tiles.push(world.map.tile(Position::new(x, y)));
            }
        }
        tiles
    }

    #[test]
    fn all_depths_pass_level_lint() {
        for depth in 0..=17u32 {
            for seed in 1..=4u64 {
                let mut world = World::new_game();
                world.run_seed = seed;
                world.depth = depth;
                build_level(&mut world, depth);

                let start = world.player_pos();
                assert!(
                    world.map.is_walkable(start),
                    "depth {depth} seed {seed}: player start blocked at {start:?}"
                );

                // Locked doors are key-openable and shifting walls toggle
                // open, so both count as passable for reachability.
                let reachable = reachable_tiles_with(&world, start, |pos| {
                    world.maze_shifting_walls.contains(&pos)
                        || (depth == 8 && World::counting_room_locked_door(pos))
                });

                for y in 0..world.map.height {
                    for x in 0..world.map.width {
                        let pos = Position::new(x, y);
                        if world.map.tile(pos) == crate::map::TileType::StairsDown {
                            assert!(
                                reachable.contains(&pos),
                                "depth {depth} seed {seed}: stairs down unreachable at {pos:?}"
                            );
                        }
                    }
                }

                for id in world.ecs.entity_ids() {
                    if id == world.player_id {
                        continue;
                    }
                    let Some(pos) = world.ecs.position(id) else {
                        continue;
                    };
                    let kind = world.ecs.kind(id);
                    assert!(
                        reachable.contains(&pos),
                        "depth {depth} seed {seed}: {kind:?} unreachable at {pos:?}"
                    );
                }
            }
        }
    }
}
