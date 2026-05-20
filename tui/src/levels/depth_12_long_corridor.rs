use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 12 — Long Corridor (Depression: 1-wide, 50-long, Shade follows)
// ---------------------------------------------------------------------------

pub(crate) fn build_long_corridor(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // A single 1-tile-wide corridor running horizontally
    let cy = MAP_HEIGHT / 2;
    for x in 2..MAP_WIDTH - 2 {
        map.set_tile(Position::new(x, cy), TileType::Floor);
    }

    // Alcoves at intervals
    for &x in &[15, 35, 45] {
        for dy in -1..=1 {
            map.set_tile(Position::new(x, cy + dy), TileType::Floor);
        }
    }

    let player_start = Position::new(3, cy);
    let stairs_down = Position::new(MAP_WIDTH - 3, cy);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Shade that follows but doesn't attack
    world.ecs.spawn_shade(Position::new(5, cy));

    // Fragments in alcoves
    world.ecs.spawn_fragment(Position::new(15, cy), "frag-013");
    world.ecs.spawn_fragment(Position::new(35, cy), "frag-014");
    world.ecs.spawn_fragment(Position::new(45, cy), "frag-015");

    // No wizard — deliberately alone
}
