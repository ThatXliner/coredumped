use super::helpers::apply_map;
use crate::{
    map::{Map, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Procedural depths (default)
// ---------------------------------------------------------------------------

pub(crate) fn build_procedural_level(world: &mut World, depth: u32) {
    let gen = if depth % 2 == 0 {
        Map::generate_cave(depth)
    } else {
        Map::generate(MAP_WIDTH, MAP_HEIGHT, depth)
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
