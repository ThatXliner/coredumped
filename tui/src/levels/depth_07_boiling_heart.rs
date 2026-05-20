use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 7 — Boiling Heart (Anger Boss: Rage)
// ---------------------------------------------------------------------------

pub(crate) fn build_boiling_heart(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Large boss arena: 40x25 room
    let rx = 7;
    let ry = 4;
    let rw = 41;
    let rh = 25;
    for y in ry..ry + rh {
        for x in rx..rx + rw {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    let player_start = Position::new(rx + 2, ry + rh / 2);
    let stairs_down = Position::new(rx + rw - 2, ry + rh / 2);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Rage boss in center — hides stairs initially (spawn on stairs tile)
    let rage_pos = Position::new(rx + rw - 4, ry + rh / 2);
    world.ecs.spawn_rage(rage_pos);

    // frag-002 near exit
    world
        .ecs
        .spawn_fragment(Position::new(rx + rw - 6, ry + rh / 2 + 2), "frag-002");

    // Wizard before boss
    let wizard_pos = Position::new(rx + 3, ry + rh / 2 + 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}
