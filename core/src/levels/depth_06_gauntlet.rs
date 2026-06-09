use bracket_color::prelude::{CYAN, RGB};

use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 6 — The Gauntlet (Anger: linear combat corridor)
// ---------------------------------------------------------------------------

pub(crate) fn build_gauntlet(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // A long horizontal corridor with 8 segments. Barriers lock behind player.
    let corridor_y = MAP_HEIGHT / 2;
    for x in 2..MAP_WIDTH - 2 {
        map.set_tile(Position::new(x, corridor_y), TileType::Floor);
    }
    // Widen at barrier positions so player can't skirt around
    for barrier_x in &[7, 13, 19, 25, 31, 37, 43, 49] {
        for dy in -2..=2 {
            map.set_tile(Position::new(*barrier_x, corridor_y + dy), TileType::Floor);
        }
    }

    let player_start = Position::new(3, corridor_y);
    let stairs_down = Position::new(MAP_WIDTH - 3, corridor_y);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    // Alcove for the wizard — off the one-tile corridor so he doesn't block it
    let wizard_pos = Position::new(5, corridor_y - 1);
    map.set_tile(wizard_pos, TileType::Floor);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Enemies spread across 8 segments
    world.ecs.spawn_slime(Position::new(10, corridor_y));
    world.ecs.spawn_goblin(Position::new(22, corridor_y));
    world.ecs.spawn_bat(Position::new(28, corridor_y));
    world.ecs.spawn_slime(Position::new(34, corridor_y));
    world.ecs.spawn_goblin(Position::new(40, corridor_y));
    world.ecs.spawn_ogre(Position::new(46, corridor_y));

    // frag-001 in segment 3
    world
        .ecs
        .spawn_fragment(Position::new(16, corridor_y), "frag-001");

    // Wizard at start
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"I can't come with you through this. I'll meet you at the end.\"",
        RGB::named(CYAN),
    );
    true
}
