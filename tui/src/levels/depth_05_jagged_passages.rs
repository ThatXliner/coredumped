use super::helpers::{apply_map, spawn_wizard_near_player};
use crate::{entity::Position, map::Map, world::World};

// ---------------------------------------------------------------------------
// Depth 5 — Jagged Passages (Anger: hostile terrain)
// ---------------------------------------------------------------------------

pub(crate) fn build_jagged_passages(world: &mut World) {
    let gen = Map::generate_cave(5);
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 5);
    }
    apply_map(world, &gen);

    // Wizard appears but refuses to heal
    spawn_wizard_near_player(world);

    world.ecs.spawn_sign(
        Position::new(gen.player_start.x + 2, gen.player_start.y + 2),
        "The passages twist without reason.\nDead ends. Ambush corners.\nKeep moving.",
    );
}
