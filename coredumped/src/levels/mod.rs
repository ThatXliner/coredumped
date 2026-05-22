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
    use std::collections::BTreeSet;

    use crate::{EntityKind, World};

    use super::build_level;

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
}
