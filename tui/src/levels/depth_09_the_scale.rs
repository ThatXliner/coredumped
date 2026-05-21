use bracket_lib::prelude::{CYAN, RGB};

use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 9 — The Scale (Bargaining: sacrifice mechanic)
// ---------------------------------------------------------------------------

pub(crate) fn build_the_scale(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Symmetrical room grid: central hub + 4 side rooms
    let cx = MAP_WIDTH / 2;
    let cy = MAP_HEIGHT / 2;
    // Central hub
    for y in cy - 3..=cy + 3 {
        for x in cx - 5..=cx + 5 {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }
    // Side rooms
    let side_rooms = [(cx - 20, cy, 8, 6), (cx + 12, cy, 8, 6)];
    for &(rx, ry, rw, rh) in &side_rooms {
        for y in ry - rh / 2..ry + rh / 2 {
            for x in rx..rx + rw {
                map.set_tile(Position::new(x, y), TileType::Floor);
            }
        }
        // Corridor from center to room
        let dir: i32 = if rx < cx { -1 } else { 1 };
        for t in 1..6 {
            map.set_tile(Position::new(cx + dir * t, cy), TileType::Floor);
        }
    }

    let player_start = Position::new(cx - 3, cy + 2);
    let stairs_down = Position::new(cx + 3, cy - 2);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Scale signs in center
    world.ecs.spawn_sign(
        Position::new(cx, cy - 2),
        "Two scales sit in the center.\nEach demands a weight.\nPlace what you carry...\nor pass with nothing.",
    );

    // Enemies
    world.ecs.spawn_ogre(Position::new(cx - 14, cy));
    world.ecs.spawn_ogre(Position::new(cx + 16, cy));
    world.ecs.spawn_goblin(Position::new(cx - 4, cy - 6));
    world.ecs.spawn_goblin(Position::new(cx + 4, cy + 6));
    world.ecs.spawn_bat(Position::new(cx - 8, cy - 2));
    world.ecs.spawn_bat(Position::new(cx + 8, cy + 2));

    // Fragments
    world
        .ecs
        .spawn_fragment(Position::new(cx + 1, cy), "frag-005");
    world
        .ecs
        .spawn_fragment(Position::new(cx - 14, cy + 2), "frag-006");

    // Wizard in center
    let wizard_pos = Position::new(cx, cy + 4);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"Give me the ones that hurt. I'll take them. You won't remember they existed.\"",
        RGB::named(CYAN),
    );
    true
}
