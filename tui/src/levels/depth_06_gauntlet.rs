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

    // A long horizontal corridor with occasional wider segments
    let corridor_y = MAP_HEIGHT / 2;
    for x in 2..MAP_WIDTH - 2 {
        map.set_tile(Position::new(x, corridor_y), TileType::Floor);
    }
    // Widen at segments
    for seg in &[8, 18, 28, 38, 48] {
        for dy in -2..=2 {
            map.set_tile(Position::new(*seg, corridor_y + dy), TileType::Floor);
        }
    }
    // Make corridor 2 tiles wide in some areas
    for x in [5, 6, 15, 16, 25, 26, 35, 36, 45, 46] {
        map.set_tile(Position::new(x, corridor_y - 1), TileType::Floor);
    }

    let player_start = Position::new(3, corridor_y);
    let stairs_down = Position::new(MAP_WIDTH - 3, corridor_y);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Enemies in segments
    world.ecs.spawn_slime(Position::new(12, corridor_y));
    world.ecs.spawn_goblin(Position::new(22, corridor_y));
    world.ecs.spawn_bat(Position::new(27, corridor_y - 1));
    world.ecs.spawn_slime(Position::new(32, corridor_y));
    world.ecs.spawn_goblin(Position::new(42, corridor_y));
    world.ecs.spawn_ogre(Position::new(50, corridor_y));

    // frag-001 in segment 2
    world
        .ecs
        .spawn_fragment(Position::new(18, corridor_y - 2), "frag-001");

    // Wizard at start
    let wizard_pos = Position::new(5, corridor_y - 1);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}
