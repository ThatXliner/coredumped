use bracket_color::prelude::{CYAN, RGB};

use super::helpers::{apply_map, nearest_open_floor, spawn_wizard_near_player};
use crate::{entity::Position, map::Map, world::World};

// ---------------------------------------------------------------------------
// Depth 5 — Jagged Passages (Anger: hostile terrain)
// ---------------------------------------------------------------------------

pub(crate) fn build_jagged_passages(world: &mut World) {
    let gen = Map::generate_cave(5, world.level_seed(5));
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 5);
    }
    apply_map(world, &gen);

    // Wizard appears but refuses to heal
    spawn_wizard_near_player(world);
    world.on_wizard_interact = Some(wizard_interact);

    // Cave layouts give no walkability guarantee near the start — snap to floor
    let sign_target = Position::new(gen.player_start.x + 2, gen.player_start.y + 2);
    let sign_pos = nearest_open_floor(world, sign_target).unwrap_or(sign_target);
    world.ecs.spawn_sign(
        sign_pos,
        "The passages twist without reason. Dead ends. Ambush corners.\n\nKeep moving. The dungeon built this to slow you down, not stop you.",
    );
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"You're hurt. Let me — no. I can't. Not here. Keep moving.\"",
        RGB::named(CYAN),
    );
    false
}
