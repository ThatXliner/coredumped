use bracket_lib::prelude::{CYAN, RGB};

use super::helpers::apply_map;
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

    // Wizard at midpoint — player alone at start for first time
    let wizard_pos = Position::new(MAP_WIDTH / 2, MAP_HEIGHT / 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"Ah, you made it past the... the. I'm sorry. The air down here is different.\"",
        RGB::named(CYAN),
    );
    true
}
