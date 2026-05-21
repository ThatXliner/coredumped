use bracket_color::prelude::{CYAN, RGB};

use super::helpers::{apply_map, find_stairs_down};
use crate::{
    entity::Position,
    map::{Map, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 10 — Maze of Regret (Bargaining: shifting maze)
// ---------------------------------------------------------------------------

pub(crate) fn build_maze_of_regret(world: &mut World) {
    let gen = Map::generate(MAP_WIDTH, MAP_HEIGHT, 10);
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 10);
    }
    apply_map(world, &gen);

    // Fragments in various rooms
    let stairs = find_stairs_down(&world.map);
    world.ecs.spawn_fragment(
        Position::new(gen.player_start.x + 5, gen.player_start.y + 2),
        "frag-007",
    );
    world
        .ecs
        .spawn_fragment(Position::new(stairs.x - 4, stairs.y - 2), "frag-008");
    world
        .ecs
        .spawn_fragment(Position::new(stairs.x + 2, stairs.y + 4), "frag-009");

    // Wizard at entrance — uncertain
    let wizard_pos = Position::new(gen.player_start.x + 2, gen.player_start.y - 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"I could tell you the way. I think you need to find it yourself.\"",
        RGB::named(CYAN),
    );
    true
}
