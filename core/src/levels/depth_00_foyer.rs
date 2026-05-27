use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 0 — Foyer (Denial: first room, helpless tutorial)
// ---------------------------------------------------------------------------

pub(crate) fn build_foyer(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // A 25x15 room centered
    let rx = 15;
    let ry = 9;
    let rw = 25;
    let rh = 15;
    for y in ry..ry + rh {
        for x in rx..rx + rw {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    let player_start = Position::new(rx + 2, ry + 2);
    let stairs_down = Position::new(rx + rw - 3, ry + rh - 3);

    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Single sign: boot, controls, wizard tease
    world.ecs.spawn_sign(
        Position::new(rx + rw / 2 - 6, ry + rh / 2 - 1),
        "If you're reading this, you\nfinally woke up.\n\nMove with arrow keys or hjkl.\nDescend when ready.\n\nBeware the slime (the S).\nIt moves toward you (sometimes).",
    );

    // One pushable slime
    world.ecs.spawn_slime(Position::new(rx + rw - 8, ry + 3));
}
