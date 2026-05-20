//! Hand-crafted and procedural level definitions.
//!
//! Each level is built by a self-contained function that sets the map, places
//! the player, and spawns entities. Adding a new hand-crafted level means
//! writing one builder function and adding a match arm in [`build_level`].

use crate::{
    entity::{Direction, Position},
    map::{Map, MapGenOutput, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

/// Dispatch to the appropriate level builder for the given depth.
pub fn build_level(world: &mut World, depth: u32) {
    world.clear_level_entities();

    match depth {
        0 => build_foyer(world),
        1 => build_wizard_chamber(world),
        2 => build_tutorial_grid(world),
        3 => build_quiet_halls(world),
        _ => build_procedural_level(world, depth),
    }
}

// ---------------------------------------------------------------------------
// Depth 0 — Foyer (Denial: first room, helpless tutorial)
// ---------------------------------------------------------------------------

fn build_foyer(world: &mut World) {
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

    // Sign at entrance
    world.ecs.spawn_sign(
        Position::new(rx + 1, ry + 2),
        "Xlyph runtime booted.\nIf you're reading this, you finally woke up.",
    );

    // Sign at stairs
    world.ecs.spawn_sign(
        Position::new(rx + rw - 4, ry + rh - 2),
        "Move with arrow keys or hjkl.\nDescend when ready.",
    );

    // One pushable slime
    world.ecs.spawn_slime(Position::new(rx + rw - 8, ry + 3));

    // Wizard first meeting — heals, brief intro, does not teach attack yet
    let wizard_pos = Position::new(rx + rw / 2, ry + rh / 2 - 1);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 3 — Quiet Halls (Denial: corridor maze, bridge to Anger)
// ---------------------------------------------------------------------------

fn build_quiet_halls(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Carve a corridor-based maze with alcoves. Vertical main corridors,
    // horizontal cross corridors, no dead ends.
    //
    // Layout (map is 55×33):
    //   - Three north-south corridors at x=5, x=27, x=49
    //   - Three east-west corridors at y=5, y=16, y=27
    //   - Alcoves branching off the corridors
    let v_corridors = [5, 27, 49];
    let h_corridors = [5, 16, 27];

    // Carve main corridors
    for &vx in &v_corridors {
        for y in 1..MAP_HEIGHT - 1 {
            map.set_tile(Position::new(vx, y), TileType::Floor);
            map.set_tile(Position::new(vx + 1, y), TileType::Floor);
        }
    }
    for &hy in &h_corridors {
        for x in 1..MAP_WIDTH - 1 {
            map.set_tile(Position::new(x, hy), TileType::Floor);
            map.set_tile(Position::new(x, hy + 1), TileType::Floor);
        }
    }

    // Carve alcoves (short dead-end branches off corridors)
    let alcoves: [(i32, i32, Direction); 8] = [
        (8, 3, Direction::South),   // top-left alcove (south from north corridor)
        (25, 3, Direction::South),  // top-center alcove
        (51, 3, Direction::South),  // top-right alcove
        (3, 14, Direction::East),   // mid-left alcove
        (51, 14, Direction::West),  // mid-right alcove
        (3, 25, Direction::East),   // bottom-left alcove
        (25, 29, Direction::North), // bottom-center alcove
        (51, 25, Direction::West),  // bottom-right alcove
    ];

    for &(ax, ay, dir) in &alcoves {
        let (dx, dy) = match dir {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        };
        let mut cx = ax;
        let mut cy = ay;
        for _ in 0..4 {
            map.set_tile(Position::new(cx, cy), TileType::Floor);
            map.set_tile(Position::new(cx + 1, cy), TileType::Floor);
            map.set_tile(Position::new(cx, cy + 1), TileType::Floor);
            map.set_tile(Position::new(cx + 1, cy + 1), TileType::Floor);
            cx += dx;
            cy += dy;
        }
    }

    let player_start = Position::new(6, 6);
    let stairs_down = Position::new(50, 28);

    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Wizard at start
    let wizard_pos = Position::new(8, 5);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));

    // Enemies: 2 Bats, 1 Slime
    world.ecs.spawn_bat(Position::new(28, 8));
    world.ecs.spawn_slime(Position::new(50, 18));
    world.ecs.spawn_bat(Position::new(26, 25));

    // Sign near wizard
    world.ecs.spawn_sign(
        Position::new(10, 5),
        "There are a few creatures wandering\nthe halls. They're more confused\nthan dangerous.\n\n  — the wizard",
    );

    // Sign near stairs
    world.ecs.spawn_sign(
        Position::new(48, 27),
        "You did well. The descent continues.\n\n  — the wizard",
    );
}

// ---------------------------------------------------------------------------
// Depth 2 — Tutorial Grid (3x3 rooms)
// ---------------------------------------------------------------------------

fn build_tutorial_grid(world: &mut World) {
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

    // Horizontal doorways
    for x in 0..3 {
        for y in 0..3 {
            if x < 2 {
                let door_x = 18 + x * 17;
                let door_y = 2 + y * 11 + 4;
                map.set_tile(Position::new(door_x, door_y), TileType::Floor);
                map.set_tile(Position::new(door_x, door_y + 1), TileType::Floor);
            }
            if y < 2 {
                let door_x = 2 + x * 17 + 7;
                let door_y = 11 + y * 11;
                map.set_tile(Position::new(door_x, door_y), TileType::Floor);
                map.set_tile(Position::new(door_x + 1, door_y), TileType::Floor);
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

    // Room 1: Inspector
    world.ecs.spawn_sign(
        Position::new(20, 6),
        "Press i to open the inspector.\nHover over things to learn\ntheir names and properties.",
    );
    world.ecs.spawn_slime(Position::new(31, 6));

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

    // Room 4: Enemy inspection
    world.ecs.spawn_sign(
        Position::new(20, 17),
        "Each enemy has HP. Inspect\nthem with i to learn their\nstrengths and weaknesses.",
    );
    world.ecs.spawn_slime(Position::new(31, 20));

    // Room 5: Console
    world.ecs.spawn_sign(
        Position::new(37, 17),
        "Press ` to open the console.\nTry: (player-facing)\nOr: (bind-key :z (do-attack))",
    );
    world.ecs.spawn_slime(Position::new(48, 20));

    // Room 6: Help
    world.ecs.spawn_sign(
        Position::new(3, 28),
        "Stairs down are below.\nEach level challenges you\ndifferently. Adapt.",
    );
    world.ecs.spawn_slime(Position::new(14, 29));

    // Room 7: "Nothing is wrong"
    world
        .ecs
        .spawn_sign(Position::new(20, 28), "Nothing is wrong.");
    world.ecs.spawn_slime(Position::new(31, 29));

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

// ---------------------------------------------------------------------------
// Procedural depths (default)
// ---------------------------------------------------------------------------

fn build_procedural_level(world: &mut World, depth: u32) {
    let gen = if depth % 2 == 0 {
        Map::generate_cave(depth)
    } else {
        Map::generate(MAP_WIDTH, MAP_HEIGHT, depth)
    };

    // Borrow spawn lists before moving `gen.map`
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, depth);
    }
    for pos in &gen.boss_spawns {
        world.spawn_boss_at(*pos);
    }

    apply_map(world, &gen);
}

// ---------------------------------------------------------------------------
// Depth 3 — Wizard's tutorial chamber
// ---------------------------------------------------------------------------

fn build_wizard_chamber(world: &mut World) {
    let gen = generate_wizard_box();
    apply_map(world, &gen);
    spawn_wizard_near_player(world);
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn apply_map(world: &mut World, gen: &MapGenOutput) {
    world.map = gen.map.clone();
    world.ecs.set_position(world.player_id, gen.player_start);
}

fn spawn_wizard_near_player(world: &mut World) {
    let player_pos = world.player_pos();
    let candidates = [
        player_pos.offset(2, 0),
        player_pos.offset(-2, 0),
        player_pos.offset(0, 2),
        player_pos.offset(0, -2),
        player_pos.offset(3, 0),
        player_pos.offset(-3, 0),
        player_pos.offset(0, 3),
        player_pos.offset(0, -3),
    ];
    let wizard_pos = candidates
        .iter()
        .copied()
        .find(|&p| world.map.is_walkable(p) && world.ecs.entity_at(p).is_none())
        .unwrap_or(player_pos.offset(2, 0));
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

#[allow(dead_code)]
pub fn find_stairs_down(map: &Map) -> Position {
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Position::new(x, y);
            if map.tile(pos) == TileType::StairsDown {
                return pos;
            }
        }
    }
    Position::new(2, 2)
}
