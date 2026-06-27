use super::helpers::apply_map;
use crate::{map::Map, world::World};

/// Procedural maps grow with depth for exploration variety.
const PROC_MAP_WIDTH: i32 = 80;
const PROC_MAP_HEIGHT: i32 = 50;

// ---------------------------------------------------------------------------
// Procedural depths (default)
// ---------------------------------------------------------------------------

pub(crate) fn build_procedural_level(world: &mut World, depth: u32) {
    let seed = world.level_seed(depth);
    let gen = if depth % 2 == 0 {
        Map::generate_cave_sized(PROC_MAP_WIDTH, PROC_MAP_HEIGHT, depth, seed)
    } else {
        Map::generate(PROC_MAP_WIDTH, PROC_MAP_HEIGHT, depth, seed)
    };

    // Borrow spawn lists before moving `gen.map`
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, depth);
    }
    for pos in &gen.boss_spawns {
        world.spawn_boss_at(*pos);
    }

    apply_map(world, &gen);
}
