use super::helpers::{apply_map, spawn_wizard_near_player};
use crate::{
    entity::Position,
    map::{Map, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 4 — First Scar (Anger: first tonal shift)
// ---------------------------------------------------------------------------

pub(crate) fn build_first_scar(world: &mut World) {
    let gen = Map::generate(MAP_WIDTH, MAP_HEIGHT, 4);
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 4);
    }
    apply_map(world, &gen);

    // Sign hinting at the tone shift
    world.ecs.spawn_sign(
        Position::new(gen.player_start.x + 3, gen.player_start.y),
        "The air down here is different.\nEverything feels... sharper.",
    );

    // Wizard at midpoint, clipped
    spawn_wizard_near_player(world);
}
