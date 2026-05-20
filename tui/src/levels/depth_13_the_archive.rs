use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 13 — The Archive (Depression: library halls)
// ---------------------------------------------------------------------------

pub(crate) fn build_the_archive(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Library layout: 4 archive rooms connected by corridors
    let rooms = [
        (3, 4, 22, 12),   // NW room
        (28, 4, 24, 12),  // NE room
        (3, 18, 22, 12),  // SW room
        (28, 18, 24, 12), // SE room
    ];
    for &(rx, ry, rw, rh) in &rooms {
        for y in ry..ry + rh {
            for x in rx..rx + rw {
                map.set_tile(Position::new(x, y), TileType::Floor);
            }
        }
    }

    // Horizontal and vertical corridors connecting rooms
    for x in 3..52 {
        map.set_tile(Position::new(x, 16), TileType::Floor);
        map.set_tile(Position::new(x, 17), TileType::Floor);
    }
    for y in 4..30 {
        map.set_tile(Position::new(25, y), TileType::Floor);
        map.set_tile(Position::new(26, y), TileType::Floor);
    }

    let player_start = Position::new(6, 6);
    let stairs_down = Position::new(48, 22);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Shades that follow silently
    world.ecs.spawn_shade(Position::new(30, 8));
    world.ecs.spawn_shade(Position::new(8, 22));
    world.ecs.spawn_shade(Position::new(40, 24));

    // Slow slimes
    world.ecs.spawn_slime(Position::new(20, 6));
    world.ecs.spawn_slime(Position::new(44, 8));

    // Fragments in archive rooms
    world.ecs.spawn_fragment(Position::new(14, 8), "frag-016");
    world.ecs.spawn_fragment(Position::new(38, 6), "frag-017");
    world.ecs.spawn_fragment(Position::new(6, 22), "frag-018");

    // Special items: Shade Echo and Vapor Canteen (collectible inventory items)
    world.ecs.spawn_shade_echo(Position::new(24, 22));
    world.ecs.spawn_vapor_canteen(Position::new(30, 24));

    // Archivist journal signs
    world.ecs.spawn_sign(
        Position::new(16, 4),
        "Archivist's journal:\nSubject reports persistent sadness.\nNo interventions applied.",
    );
    world.ecs.spawn_sign(
        Position::new(40, 28),
        "Archivist's journal:\nMemory #032 partially recovered.\nSubject shows distress.\nRecommencing suppression.",
    );

    // No wizard
}
