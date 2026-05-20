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
        4 => build_first_scar(world),
        5 => build_jagged_passages(world),
        6 => build_gauntlet(world),
        7 => build_boiling_heart(world),
        8 => build_counting_room(world),
        9 => build_the_scale(world),
        10 => build_maze_of_regret(world),
        11 => build_the_offer(world),
        12 => build_long_corridor(world),
        13 => build_the_archive(world),
        14 => build_ash_field(world),
        15 => build_the_clearing(world),
        16 => build_the_descent(world),
        17 => build_the_core(world),
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
// Depth 4 — First Scar (Anger: first tonal shift)
// ---------------------------------------------------------------------------

fn build_first_scar(world: &mut World) {
    let gen = Map::generate(MAP_WIDTH, MAP_HEIGHT, 4);
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 4);
    }
    apply_map(world, &gen);

    // Sign hinting at the tone shift
    world.ecs.spawn_sign(
        Position::new(gen.player_start.x + 3, gen.player_start.y),
        "The air down here is different.\nEverything feels... sharper.",
    );

    // Wizard at midpoint, clipped
    spawn_wizard_near_player(world);
}

// ---------------------------------------------------------------------------
// Depth 5 — Jagged Passages (Anger: hostile terrain)
// ---------------------------------------------------------------------------

fn build_jagged_passages(world: &mut World) {
    let gen = Map::generate_cave(5);
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 5);
    }
    apply_map(world, &gen);

    // No wizard here — player alone in hostile terrain
    world.ecs.spawn_sign(
        Position::new(gen.player_start.x + 2, gen.player_start.y + 2),
        "The passages twist without reason.\nDead ends. Ambush corners.\nKeep moving.",
    );
}

// ---------------------------------------------------------------------------
// Depth 6 — The Gauntlet (Anger: linear combat corridor)
// ---------------------------------------------------------------------------

fn build_gauntlet(world: &mut World) {
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

// ---------------------------------------------------------------------------
// Depth 7 — Boiling Heart (Anger Boss: Rage)
// ---------------------------------------------------------------------------

fn build_boiling_heart(world: &mut World) {
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

// ---------------------------------------------------------------------------
// Depth 8 — Counting Room (Bargaining: locked doors & keys)
// ---------------------------------------------------------------------------

fn build_counting_room(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // A central hub with 4 locked rooms branching off
    let hub_x = 22;
    let hub_y = 15;
    // Hub
    for y in hub_y - 3..=hub_y + 3 {
        for x in hub_x - 3..=hub_x + 3 {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    // 4 rooms branching from hub (N, S, E, W)
    let rooms: [(i32, i32, i32, i32); 4] = [
        (hub_x - 16, hub_y - 12, 10, 8), // NW
        (hub_x + 6, hub_y - 12, 10, 8),  // NE
        (hub_x - 16, hub_y + 4, 10, 8),  // SW
        (hub_x + 6, hub_y + 4, 10, 8),   // SE
    ];
    for &(rx, ry, rw, rh) in &rooms {
        for y in ry..ry + rh {
            for x in rx..rx + rw {
                map.set_tile(Position::new(x, y), TileType::Floor);
            }
        }
    }

    // Corridor from hub to each room
    for (dx, dy) in &[(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let door_x = hub_x + dx * 6;
        for t in 0..6 {
            map.set_tile(Position::new(hub_x + dx * t, hub_y), TileType::Floor);
            map.set_tile(Position::new(door_x, hub_y + dy * t), TileType::Floor);
        }
    }

    let player_start = Position::new(hub_x, hub_y);
    let stairs_down = Position::new(hub_x, hub_y + 3);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Goblins hold keys — when killed, player can pick up the key
    world.ecs.spawn_goblin(Position::new(hub_x - 12, hub_y - 4)); // key-goblin-1
    world.ecs.spawn_goblin(Position::new(hub_x + 10, hub_y - 8)); // key-goblin-2
    world.ecs.spawn_goblin(Position::new(hub_x - 12, hub_y + 8)); // key-goblin-3
    world.ecs.spawn_bat(Position::new(hub_x + 8, hub_y + 4));
    world.ecs.spawn_bat(Position::new(hub_x - 8, hub_y - 6));

    // Fragments in rooms
    world
        .ecs
        .spawn_fragment(Position::new(hub_x - 12, hub_y - 8), "frag-003");
    world
        .ecs
        .spawn_fragment(Position::new(hub_x + 10, hub_y - 6), "frag-004");

    // Wizard at entrance
    let wizard_pos = Position::new(hub_x + 2, hub_y - 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 9 — The Scale (Bargaining: sacrifice mechanic)
// ---------------------------------------------------------------------------

fn build_the_scale(world: &mut World) {
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
        let dir: i32 = if rx < cx { -1 } else { 1 };
        for t in 1..6 {
            map.set_tile(Position::new(cx + dir * t, cy), TileType::Floor);
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
        "Two scales sit in the center.\nEach demands a weight.\nPlace what you carry...\nor pass with nothing.",
    );

    // Enemies
    world.ecs.spawn_ogre(Position::new(cx - 14, cy));
    world.ecs.spawn_ogre(Position::new(cx + 16, cy));
    world.ecs.spawn_goblin(Position::new(cx - 4, cy - 6));
    world.ecs.spawn_goblin(Position::new(cx + 4, cy + 6));
    world.ecs.spawn_bat(Position::new(cx - 8, cy - 2));
    world.ecs.spawn_bat(Position::new(cx + 8, cy + 2));

    // Fragments
    world
        .ecs
        .spawn_fragment(Position::new(cx + 1, cy), "frag-005");
    world
        .ecs
        .spawn_fragment(Position::new(cx - 14, cy + 2), "frag-006");

    // Wizard in center
    let wizard_pos = Position::new(cx, cy + 4);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 10 — Maze of Regret (Bargaining: shifting maze)
// ---------------------------------------------------------------------------

fn build_maze_of_regret(world: &mut World) {
    let gen = Map::generate(MAP_WIDTH, MAP_HEIGHT, 10);
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 10);
    }
    apply_map(world, &gen);

    // Fragments in various rooms
    let stairs = find_stairs_down(&world.map);
    world.ecs.spawn_fragment(
        Position::new(gen.player_start.x + 5, gen.player_start.y + 2),
        "frag-007",
    );
    world
        .ecs
        .spawn_fragment(Position::new(stairs.x - 4, stairs.y - 2), "frag-008");
    world
        .ecs
        .spawn_fragment(Position::new(stairs.x + 2, stairs.y + 4), "frag-009");

    // Wizard at entrance — uncertain
    let wizard_pos = Position::new(gen.player_start.x + 2, gen.player_start.y - 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 11 — The Offer (Bargaining Boss: 4 sentry chambers)
// ---------------------------------------------------------------------------

fn build_the_offer(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Central hub + 4 sub-chambers
    let cx = MAP_WIDTH / 2;
    let cy = MAP_HEIGHT / 2;
    for y in cy - 2..=cy + 2 {
        for x in cx - 3..=cx + 3 {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    // 4 chambers around hub (N, S, E, W)
    let chambers = [
        (cx - 22, cy - 10, 12, 8), // W
        (cx + 10, cy - 10, 12, 8), // E
        (cx - 22, cy + 2, 12, 8),  // SW
        (cx + 10, cy + 2, 12, 8),  // SE
    ];
    for &(rx, ry, rw, rh) in &chambers {
        for y in ry..ry + rh {
            for x in rx..rx + rw {
                map.set_tile(Position::new(x, y), TileType::Floor);
            }
        }
    }

    // Corridors from hub to chambers
    for t in 0..6 {
        map.set_tile(Position::new(cx - 3 - t, cy), TileType::Floor); // W
        map.set_tile(Position::new(cx + 3 + t, cy), TileType::Floor); // E
        map.set_tile(Position::new(cx - 3 - t, cy + 6), TileType::Floor); // SW
        map.set_tile(Position::new(cx + 3 + t, cy + 6), TileType::Floor); // SE
    }

    let player_start = Position::new(cx, cy + 2);
    let stairs_down = Position::new(cx, cy - 1);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Sentries in chambers
    world.ecs.spawn_sentry(Position::new(cx - 16, cy - 6));
    world.ecs.spawn_sentry(Position::new(cx + 16, cy - 6));
    world.ecs.spawn_sentry(Position::new(cx - 16, cy + 6));
    world.ecs.spawn_sentry(Position::new(cx + 16, cy + 6));

    // Fragments
    world
        .ecs
        .spawn_fragment(Position::new(cx - 18, cy - 4), "frag-010");
    world
        .ecs
        .spawn_fragment(Position::new(cx + 14, cy - 4), "frag-011");
    world
        .ecs
        .spawn_fragment(Position::new(cx + 14, cy + 8), "frag-012");

    // Wizard in center
    let wizard_pos = Position::new(cx, cy);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 12 — Long Corridor (Depression: 1-wide, 50-long, Shade follows)
// ---------------------------------------------------------------------------

fn build_long_corridor(world: &mut World) {
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

// ---------------------------------------------------------------------------
// Depth 13 — The Archive (Depression: library halls)
// ---------------------------------------------------------------------------

fn build_the_archive(world: &mut World) {
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

    // Special items: Shade Echo and Vapor Canteen (sign-like items)
    world.ecs.spawn_sign(
        Position::new(24, 22),
        "~ Shade Echo ~\nA fragment of the Shade that shivers\nwhen the Shade is near.\nCarry it. It may confuse the darkness.",
    );

    world.ecs.spawn_sign(
        Position::new(30, 24),
        "~ Vapor Canteen ~\nAn old flask, half-full. The liquid inside\nfeels cold. Pour it on fire and the fire\nmay forget to burn.",
    );

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

// ---------------------------------------------------------------------------
// Depth 14 — Ash Field (Depression Boss: fire zones)
// ---------------------------------------------------------------------------

fn build_ash_field(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Open field: all floor with wall borders
    for y in 1..MAP_HEIGHT - 1 {
        for x in 1..MAP_WIDTH - 1 {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    let player_start = Position::new(3, 3);
    let stairs_down = Position::new(MAP_WIDTH - 3, MAP_HEIGHT - 3);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Fire zones — clusters of unwalkable tiles
    let fire_centers = [
        Position::new(20, 10),
        Position::new(40, 18),
        Position::new(28, 24),
    ];
    for &center in &fire_centers {
        for dy in -2..=2 {
            for dx in -2..=2 {
                let pos = Position::new(center.x + dx, center.y + dy);
                if (dx.abs() + dy.abs()) <= 3 && world.map.tile(pos) == TileType::Floor {
                    // Mark as special — we use a sign to indicate fire hazard
                    world
                        .ecs
                        .spawn_sign(pos, "FIRE — you take 1 damage crossing here.");
                }
            }
        }
    }

    // Fragments scattered across the field
    world.ecs.spawn_fragment(Position::new(30, 14), "frag-019");
    world.ecs.spawn_fragment(Position::new(18, 8), "frag-020");
    world.ecs.spawn_fragment(Position::new(44, 12), "frag-021");
    world.ecs.spawn_fragment(Position::new(6, 26), "frag-022");
    world.ecs.spawn_fragment(Position::new(50, 4), "frag-023");
    world.ecs.spawn_fragment(Position::new(48, 28), "frag-024");

    // Wizard at end
    let wizard_pos = Position::new(MAP_WIDTH - 4, MAP_HEIGHT - 4);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 15 — The Clearing (Acceptance: beautiful open glade)
// ---------------------------------------------------------------------------

fn build_the_clearing(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Open glade with organic edges
    for y in 3..MAP_HEIGHT - 3 {
        for x in 3..MAP_WIDTH - 3 {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    // Carve organic edges
    for x in 2..MAP_WIDTH - 2 {
        for &(edge_y, offset) in &[(3, 2), (MAP_HEIGHT - 4, 2)] {
            map.set_tile(Position::new(x, edge_y - 1), TileType::Floor);
            map.set_tile(Position::new(x, edge_y + offset), TileType::Wall);
        }
    }

    // Pool of water (noted by signs)
    let pool_x = 25;
    let pool_y = 15;
    for dy in -2..=2 {
        for dx in -3..=3 {
            let pos = Position::new(pool_x + dx, pool_y + dy);
            if dx.abs() + dy.abs() <= 4 {
                map.set_tile(pos, TileType::Floor);
            }
        }
    }

    let player_start = Position::new(5, 5);
    let stairs_down = Position::new(MAP_WIDTH - 5, MAP_HEIGHT - 5);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // No enemies — first peaceful level since Denial

    // Tree in center (sign)
    world.ecs.spawn_sign(
        Position::new(MAP_WIDTH / 2, MAP_HEIGHT / 2 - 2),
        "        ,,,,\n       (o o)\n    ---ooO-(_)-Ooo---\n\nA single tree. Leaves catch\nthe light. You sit under it\nfor a long time.",
    );

    // Water pool sign
    world.ecs.spawn_sign(
        Position::new(pool_x, pool_y),
        "~ A pool of clear water ~\nYou see your own reflection.\nYou look tired.\nYou look... like yourself.",
    );

    // Fragments — the lowest-point memories, here in safety
    world
        .ecs
        .spawn_fragment(Position::new(MAP_WIDTH / 2 + 2, MAP_HEIGHT / 2), "frag-025");
    world
        .ecs
        .spawn_fragment(Position::new(pool_x + 1, pool_y + 3), "frag-026");
    world.ecs.spawn_fragment(Position::new(8, 10), "frag-027");
    world
        .ecs
        .spawn_fragment(Position::new(MAP_WIDTH - 8, MAP_HEIGHT - 8), "frag-028");

    // Wizard under the tree
    let wizard_pos = Position::new(MAP_WIDTH / 2 + 1, MAP_HEIGHT / 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 16 — The Descent (Acceptance: spiral walkway)
// ---------------------------------------------------------------------------

fn build_the_descent(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Spiral walkway from center outward
    let cx = MAP_WIDTH / 2;
    let cy = MAP_HEIGHT / 2;
    let mut x = cx;
    let mut y = cy;
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut dir_idx = 0;
    let mut steps_in_dir = 1;
    let mut steps_taken = 0;
    let mut segment_count = 0;

    // Carve the spiral
    for _ in 0..800 {
        if x < 2 || x >= MAP_WIDTH - 2 || y < 2 || y >= MAP_HEIGHT - 2 {
            break;
        }
        map.set_tile(Position::new(x, y), TileType::Floor);
        // Widen the path
        map.set_tile(Position::new(x + 1, y), TileType::Floor);

        let (dx, dy) = directions[dir_idx];
        x += dx;
        y += dy;
        steps_taken += 1;

        if steps_taken >= steps_in_dir {
            steps_taken = 0;
            dir_idx = (dir_idx + 1) % 4;
            segment_count += 1;
            if segment_count % 2 == 0 {
                steps_in_dir += 1;
            }
        }
    }

    let player_start = Position::new(cx, cy);
    let stairs_down = Position::new(cx, cy + 2);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // One peaceful Shade
    world.ecs.spawn_shade(Position::new(cx + 3, cy - 3));

    // Fragments along the descent
    world
        .ecs
        .spawn_fragment(Position::new(cx + 5, cy + 1), "frag-029");
    world
        .ecs
        .spawn_fragment(Position::new(cx + 4, cy - 5), "frag-030");
    world
        .ecs
        .spawn_fragment(Position::new(cx - 6, cy + 4), "frag-031");
    world
        .ecs
        .spawn_fragment(Position::new(cx, cy - 2), "frag-032");

    // Wizard walks alongside
    let wizard_pos = Position::new(cx + 2, cy);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}

// ---------------------------------------------------------------------------
// Depth 17 — The Core (Final: vessel/suppress rule)
// ---------------------------------------------------------------------------

fn build_the_core(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Minimalist room: 20x15
    let rx = (MAP_WIDTH - 20) / 2;
    let ry = (MAP_HEIGHT - 15) / 2;
    let rw = 20;
    let rh = 15;
    for y in ry..ry + rh {
        for x in rx..rx + rw {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    let player_start = Position::new(rx + 2, ry + rh / 2);
    let stairs_up = Position::new(rx + 1, ry + rh / 2);
    map.set_tile(stairs_up, TileType::StairsUp);
    // No stairs down — this is the end

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // frag-033 on pedestal
    world
        .ecs
        .spawn_fragment(Position::new(rx + rw / 2, ry + ry / 2 + 1), "frag-033");

    // The pedestal — the vessel/suppress rule
    world.ecs.spawn_sign(
        Position::new(rx + rw / 2, ry + rh / 2),
        "THE CORE\n\nvessel/suppress is here.\n\nOpen the inspector (i) to read the rule.\nOpen the console (`) to modify it.\n\nThe choice is yours.",
    );

    // No enemies, no wizard, no items
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
