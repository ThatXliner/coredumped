use super::helpers::{apply_map, spawn_wizard_near_player};
use crate::{
    entity::Position,
    map::{Map, MapGenOutput, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 1 — Wizard's tutorial chamber
// ---------------------------------------------------------------------------

pub(crate) fn build_wizard_chamber(world: &mut World) {
    let gen = generate_wizard_box();
    apply_map(world, &gen);
    spawn_wizard_near_player(world);

    // Hint sign about do-attack accepting an effort parameter
    world.ecs.spawn_sign(
        Position::new(20, 9),
        "There's more parameters (do-attack) can take\nTry this in the console (open with `): (help do-attack)",
    );
}

/// Build the map for the wizard's tutorial chamber: a 12×9 room in the center.
pub fn generate_wizard_box() -> MapGenOutput {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // A single 12x9 room in the center of the map
    let room_x = 14;
    let room_y = 8;
    let room_w = 12;
    let room_h = 9;

    for y in room_y..room_y + room_h {
        for x in room_x..room_x + room_w {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    let player_start = Position::new(room_x + 1, room_y + room_h / 2);
    let stairs_down = Position::new(room_x + room_w - 2, room_y + room_h / 2);

    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    MapGenOutput {
        map,
        player_start,
        stairs_up: player_start,
        stairs_down,
        combat_spawns: vec![],
        boss_spawns: vec![],
    }
}
