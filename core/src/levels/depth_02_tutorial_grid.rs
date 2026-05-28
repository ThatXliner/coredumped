use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 2 — Tutorial Grid (3x3 rooms)
// ---------------------------------------------------------------------------

pub(crate) fn build_tutorial_grid(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // 3 rows × 3 cols of rooms. Each room 16×9 with 1-tile walls between.
    // Map: 55×33
    // x: border(2) + room(16) + wall(1) + room(16) + wall(1) + room(16) + border(3) = 55
    // y: border(2) + room(9) + wall(1) + room(9) + wall(1) + room(9) + border(2) = 33
    let rooms: [(i32, i32, i32, i32); 9] = [
        (2, 2, 16, 9),   // 0: Movement
        (19, 2, 16, 9),  // 1: Inspector
        (36, 2, 16, 9),  // 2: Wizard room
        (2, 13, 16, 9),  // 3: Wait
        (19, 13, 16, 9), // 4: Enemy inspection
        (36, 13, 16, 9), // 5: Console
        (2, 23, 16, 9),  // 6: Help / stairs (y=23 so bottom at y=31, wall at y=32)
        (19, 23, 16, 9), // 7: "Nothing is wrong"
        (36, 23, 16, 9), // 8: Barrel puzzle
    ];

    for &(rx, ry, rw, rh) in &rooms {
        for y in ry..ry + rh {
            for x in rx..rx + rw {
                map.set_tile(Position::new(x, y), TileType::Floor);
            }
        }
    }

    // Snake path: 0→1→2→5→4→3→6→7→8
    // Horizontal doors: all rows, all cols (0-1, 1-2, 3-4, 4-5, 6-7, 7-8)
    for row in 0..3 {
        for col in 0..2 {
            let door_x = 18 + col * 17;
            let door_y = 2 + row * 11 + 4;
            map.set_tile(Position::new(door_x, door_y), TileType::Floor);
            map.set_tile(Position::new(door_x, door_y + 1), TileType::Floor);
        }
    }

    // Vertical doors: only 2→5 (col 2, row 0→1) and 3→6 (col 0, row 1→2)
    // Carve both wall rows between rooms (2 tiles, not 1)
    let vdoors: [(i32, i32); 2] = [
        (2, 0), // room 2→5: col 2, between row 0 and 1
        (0, 1), // room 3→6: col 0, between row 1 and 2
    ];
    for &(col, row) in &vdoors {
        let door_x = 2 + col * 17 + 7;
        let door_y = 11 + row * 11;
        for dy in 0..2 {
            for dx in 0..2 {
                map.set_tile(Position::new(door_x + dx, door_y + dy), TileType::Floor);
            }
        }
    }

    // Player starts top-left
    let player_start = Position::new(4, 4);
    // Stairs down in room 8 (barrel room), hidden under barrel
    let stairs_down = Position::new(36 + 14, 23 + 4);

    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Place signs and enemies per room

    // Room 0: Combat practice
    world.ecs.spawn_slime(Position::new(14, 6));
    world.ecs.spawn_bat(Position::new(10, 8));

    // Room 1: Inspector
    world.ecs.spawn_sign(
        Position::new(20, 6),
        "Press i to open the inspector.\nHover over things to learn their names.\n\nKnowledge is power down here.",
    );
    world.ecs.spawn_slime(Position::new(31, 6));
    world.ecs.spawn_bat(Position::new(27, 8));

    // Room 2: Wizard + first fragment
    let wizard_pos = Position::new(38, 6);
    world.ecs.spawn_sign(
        Position::new(44, 4),
        "Green diamonds hold memory fragments.\nI wonder who left them here...",
    );
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.ecs.spawn_fragment(Position::new(50, 9), "frag-001");

    // Room 3: Wait mechanic
    world.ecs.spawn_sign(
        Position::new(3, 17),
        "Press . to wait.\nTime only moves when you do.\n\nSometimes standing still is the right move.",
    );
    world.ecs.spawn_slime(Position::new(12, 20));

    // Room 4: Enemy variety — let them discover through play
    world.ecs.spawn_slime(Position::new(31, 20));
    world.ecs.spawn_bat(Position::new(28, 16));
    world.ecs.spawn_goblin(Position::new(24, 18));

    // Room 5: Console
    world.ecs.spawn_sign(
        Position::new(37, 17),
        "Press ` for the console.\nThe dungeon runs on code.\n\nTry: (help)\n\nTime fades away as you peer into the console.\nDon't worry about the enemies. Everything is fine.",
    );
    world.ecs.spawn_slime(Position::new(48, 20));
    world.ecs.spawn_slime(Position::new(42, 17));

    // Room 6: Breathing room
    world.ecs.spawn_slime(Position::new(14, 28));

    // Room 7: Thematic — denial
    world
        .ecs
        .spawn_sign(Position::new(20, 27), "Nothing is wrong.");
    world.ecs.spawn_slime(Position::new(31, 28));
    world.ecs.spawn_bat(Position::new(26, 25));

    // Room 8: Barrel puzzle — fill with barrels, leave 3×3 clear at entrance (left door)
    for y in 23..23 + 9 {
        for x in 36..36 + 16 {
            let pos = Position::new(x, y);
            if pos == stairs_down {
                continue;
            }
            world.ecs.spawn_barrel(pos);
        }
    }

    // Clear entrance area (left side of room 8, near door from room 7)
    for x in 36..=40 {
        for y in 27..=29 {
            if let Some(barrel) = world.ecs.entity_at(Position::new(x, y)) {
                world.ecs.remove(barrel);
            }
        }
    }

    // Hide stairs under barrel
    world.ecs.spawn_barrel(stairs_down);

    // Barrel room: hint first, solution second
    world.ecs.spawn_sign(
        Position::new(37, 29),
        "So many barrels...\nThe exit is under one of them.\n\nClearing these one by one would take forever.\nThere must be a faster way.",
    );
    if let Some(barrel) = world.ecs.entity_at(Position::new(40, 31)) {
        world.ecs.remove(barrel);
    }
    world.ecs.spawn_sign(
        Position::new(40, 31),
        "Chain commands with (do ...):\n  (do (move! :south) (do-attack))\n\n(repeat N ...) runs N times:\n  (repeat 4 (do-attack :east))\n\nBind it to a key and go wild.",
    );

    // Pressure plate near barrel room entrance - closes door when stepped on
    if let Some(barrel) = world.ecs.entity_at(Position::new(38, 28)) {
        world.ecs.remove(barrel);
    }
    world
        .map
        .set_tile(Position::new(38, 28), crate::map::TileType::PressurePlate);
}
