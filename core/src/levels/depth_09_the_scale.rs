use bracket_color::prelude::{CYAN, RGB};

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
        let (start_x, end_x) = if rx < cx {
            (rx + rw, cx - 5)
        } else {
            (cx + 5, rx)
        };
        for x in start_x..=end_x {
            map.set_tile(Position::new(x, cy), TileType::Floor);
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
        "Two scales sit in the center. One pan is empty. The other is you.\n\nYou can pass with nothing, or chase what waits in the side rooms. The dungeon is counting.",
    );

    // Enemies
    world.ecs.spawn_ogre(Position::new(cx - 14, cy));
    world.ecs.spawn_ogre(Position::new(cx + 16, cy));
    world.ecs.spawn_goblin(Position::new(cx - 4, cy - 3));
    world.ecs.spawn_goblin(Position::new(cx + 4, cy + 3));
    world.ecs.spawn_bat(Position::new(cx - 8, cy));
    world.ecs.spawn_bat(Position::new(cx + 8, cy));

    // Fragments
    world
        .ecs
        .spawn_fragment(Position::new(cx + 1, cy), "frag-005");
    world
        .ecs
        .spawn_fragment(Position::new(cx - 14, cy + 2), "frag-006");

    // Wizard at the hub's south edge (the hub only extends to cy + 3)
    let wizard_pos = Position::new(cx, cy + 3);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"I would take the ones that hurt if I could. But that bargain is getting harder to believe.\"",
        RGB::named(CYAN),
    );
    true
}
