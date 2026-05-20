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
        (2, 24, 16, 9),  // 6: Help / stairs
        (19, 24, 16, 9), // 7: "Nothing is wrong"
        (36, 24, 16, 9), // 8: Barrel puzzle
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
    let stairs_down = Position::new(36 + 14, 24 + 4);

    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Place signs and enemies per room

    // Room 0: Movement
    world.ecs.spawn_sign(
        Position::new(3, 6),
        "Move with arrow keys or hjkl.\nYou can fight now — try your\nbound attack key!",
    );
    world.ecs.spawn_slime(Position::new(14, 6));
    world.ecs.spawn_slime(Position::new(10, 8));

    // Room 1: Inspector
    world.ecs.spawn_sign(
        Position::new(20, 6),
        "Press i to open the inspector.\nHover over things to learn\ntheir names and properties.",
    );
    world.ecs.spawn_slime(Position::new(31, 6));
    world.ecs.spawn_slime(Position::new(27, 8));

    // Room 2: Wizard room
    let wizard_pos = Position::new(38, 6);
    world
        .ecs
        .spawn_sign(Position::new(44, 4), "The wizard waits for you.");
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));

    // Room 3: Wait
    world.ecs.spawn_sign(
        Position::new(3, 17),
        "Press . to wait. Time passes\nand enemies move too.\nUse (wait!) in bindings.",
    );
    world.ecs.spawn_slime(Position::new(12, 20));
    world.ecs.spawn_slime(Position::new(7, 18));

    // Room 4: Enemy inspection
    world.ecs.spawn_sign(
        Position::new(20, 17),
        "Each enemy has HP. Inspect\nthem with i to learn their\nstrengths and weaknesses.",
    );
    world.ecs.spawn_slime(Position::new(31, 20));
    world.ecs.spawn_bat(Position::new(28, 16));

    // Room 5: Console
    world.ecs.spawn_sign(
        Position::new(37, 17),
        "Press ` to open the console.\nTry: (player-facing)\nOr: (bind-key :z (do-attack))",
    );
    world.ecs.spawn_slime(Position::new(48, 20));
    world.ecs.spawn_slime(Position::new(42, 17));

    // Room 6: Help
    world.ecs.spawn_sign(
        Position::new(3, 28),
        "Stairs down are below.\nEach level challenges you\ndifferently. Adapt.",
    );
    world.ecs.spawn_slime(Position::new(14, 29));
    world.ecs.spawn_goblin(Position::new(7, 26));

    // Room 7: "Nothing is wrong"
    world
        .ecs
        .spawn_sign(Position::new(20, 28), "Nothing is wrong.");
    world.ecs.spawn_slime(Position::new(31, 29));
    world.ecs.spawn_bat(Position::new(26, 26));

    // Room 8: Barrel puzzle — fill with barrels, leave 3×3 clear at entrance (left door)
    for y in 24..24 + 9 {
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
        for y in 28..=30 {
            if let Some(barrel) = world.ecs.entity_at(Position::new(x, y)) {
                world.ecs.remove(barrel);
            }
        }
    }

    // Hide stairs under barrel
    world.ecs.spawn_barrel(stairs_down);

    // Signs in barrel room
    world.ecs.spawn_sign(
        Position::new(37, 25),
        "Welcome to the Puzzle Room!\n\nChain commands with (do ...):\n  (do (move! :south) (do-attack))\n\n(repeat N ...) runs N times:\n  (repeat 4 (do-attack :east))",
    );

    world.ecs.spawn_sign(
        Position::new(37, 30),
        "Bind a combo to one key:\n  (bind-key :x (do (move! :south)\n    (repeat 3 (do-attack :east))))\n\nNow clear these barrels and\nfind the exit!",
    );
}
