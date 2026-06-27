use bracket_color::prelude::{CYAN, RGB};

use super::helpers::{apply_map, nearest_open_floor};
use crate::{
    entity::Position,
    map::{Map, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 4 — First Scar (Anger: first tonal shift)
// ---------------------------------------------------------------------------

pub(crate) fn build_first_scar(world: &mut World) {
    let gen = Map::generate(MAP_WIDTH, MAP_HEIGHT, 4, world.level_seed(4));
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 4);
    }
    apply_map(world, &gen);

    // Gate the combat rooms with destructible barrels so the player meets the
    // Anger packs one chokepoint at a time instead of all at once.
    for pos in &gen.barrel_gates {
        if world.ecs.entity_at(*pos).is_none() {
            world.ecs.spawn_barrel(*pos);
        }
    }

    // Wizard near the midpoint — player alone at start for first time. The
    // raw midpoint sits on a region boundary of the generated layout, which
    // is almost always wall, so snap to the nearest open floor.
    let midpoint = Position::new(MAP_WIDTH / 2, MAP_HEIGHT / 2);
    let wizard_pos = nearest_open_floor(world, midpoint).unwrap_or(midpoint);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);

    // Sign hinting at the tone shift — near the wizard, also snapped to floor
    let sign_target = Position::new(wizard_pos.x + 2, wizard_pos.y);
    let sign_pos = nearest_open_floor(world, sign_target).unwrap_or(sign_target);
    world.ecs.spawn_sign(
        sign_pos,
        "The air down here is different. Everything feels sharper.\n\nThe dungeon is starting to notice you.",
    );
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"Ah, you made it past the... the. I'm sorry. The air down here is different.\"",
        RGB::named(CYAN),
    );
    true
}
