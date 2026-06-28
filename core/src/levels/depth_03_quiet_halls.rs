use crate::{
    dialogue::WizardDialogue,
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 3 — Quiet Halls (Denial: memory gates)
// ---------------------------------------------------------------------------
//
// Three destructible barrel gates divide a cave into small encounters. The
// player learned command chaining in depth 2; this level turns that into a
// rhythm of breaking through, choosing side rooms, and collecting early
// memories before Anger begins.

pub(crate) fn build_quiet_halls(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Main cave chain.
    let chambers: [(i32, i32, i32, i32); 9] = [
        (3, 5, 12, 12),   // entrance hollow
        (15, 8, 10, 7),   // first gate chamber
        (24, 4, 12, 15),  // split chamber
        (36, 7, 12, 8),   // second gate chamber
        (25, 21, 14, 8),  // lower memory pocket
        (41, 17, 10, 10), // final gate chamber
        (8, 20, 13, 8),   // western side memory
        (17, 2, 9, 4),    // northern side memory
        (43, 27, 8, 4),   // exit shelf
    ];

    for &(x, y, w, h) in &chambers {
        for cy in y..y + h {
            for cx in x..x + w {
                map.set_tile(Position::new(cx, cy), TileType::Floor);
            }
        }
    }

    // Connectors between chambers.
    let corridors: [(i32, i32, i32, i32); 10] = [
        (12, 10, 7, 3),
        (22, 10, 7, 3),
        (33, 10, 7, 3),
        (45, 13, 3, 8),
        (34, 24, 10, 3),
        (17, 14, 3, 9),
        (19, 4, 3, 6),
        (34, 17, 3, 8),
        (38, 24, 9, 3),
        (47, 26, 3, 4),
    ];

    for &(x, y, w, h) in &corridors {
        for cy in y..y + h {
            for cx in x..x + w {
                map.set_tile(Position::new(cx, cy), TileType::Floor);
            }
        }
    }

    // Rock teeth make the open rooms feel less solved.
    let pillars: [(i32, i32); 10] = [
        (8, 8),
        (10, 14),
        (29, 7),
        (32, 14),
        (40, 10),
        (29, 24),
        (35, 26),
        (49, 22),
        (12, 23),
        (18, 25),
    ];

    for &(px, py) in &pillars {
        map.set_tile(Position::new(px, py), TileType::Wall);
        map.set_tile(Position::new(px + 1, py), TileType::Wall);
        map.set_tile(Position::new(px, py + 1), TileType::Wall);
        map.set_tile(Position::new(px + 1, py + 1), TileType::Wall);
    }

    for y in 8..=14 {
        map.set_tile(Position::new(16, y), TileType::Wall);
    }
    for y in 7..=14 {
        map.set_tile(Position::new(35, y), TileType::Wall);
    }
    for x in 41..=50 {
        map.set_tile(Position::new(x, 20), TileType::Wall);
    }

    for pos in [
        Position::new(16, 10),
        Position::new(16, 11),
        Position::new(16, 12),
        Position::new(35, 10),
        Position::new(35, 11),
        Position::new(35, 12),
        Position::new(46, 20),
        Position::new(47, 20),
        Position::new(48, 20),
    ] {
        map.set_tile(pos, TileType::Floor);
    }

    let player_start = Position::new(5, 10);
    let stairs_down = Position::new(48, 29);

    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Wizard near entrance
    let wizard_pos = Position::new(7, 7);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);

    // Destructible gates. Each line can be chewed through with the command
    // chains taught in the previous depth, but the side routes make it less
    // like a plain barrel hallway.
    for pos in [
        Position::new(16, 10),
        Position::new(16, 11),
        Position::new(16, 12),
        Position::new(35, 10),
        Position::new(35, 11),
        Position::new(35, 12),
        Position::new(46, 20),
        Position::new(47, 20),
        Position::new(48, 20),
    ] {
        world.ecs.spawn_barrel(pos);
    }

    // Cave-dwelling enemies staged between gates.
    world.ecs.spawn_slime(Position::new(19, 12));
    world.ecs.spawn_bat(Position::new(28, 6));
    world.ecs.spawn_slime(Position::new(32, 16));
    world.ecs.spawn_bat(Position::new(42, 12));
    world.ecs.spawn_goblin(Position::new(45, 24));

    // Early Denial fragments. Later levels still have fallback duplicates for
    // some of these memories, but this keeps early discovery from jumping 1→12.
    world.ecs.spawn_fragment(Position::new(20, 4), "frag-002");
    world.ecs.spawn_fragment(Position::new(12, 25), "frag-003");
    world.ecs.spawn_fragment(Position::new(31, 25), "frag-004");

    world.ecs.spawn_sign(
        Position::new(9, 11),
        "Three little gates. Three chances to turn back. Denial likes rituals.\n\n  — the wizard",
    );

    world.ecs.spawn_sign(
        Position::new(25, 10),
        "A gate is just a wall that expects you to argue.",
    );

    world.ecs.spawn_sign(
        Position::new(43, 18),
        "The last gate is quieter. That does not make it kinder.",
    );
}

fn wizard_interact(_world: &mut World) -> WizardDialogue {
    WizardDialogue::healing_lines(&[
        "Denial does not build one wall. It builds a sequence, so each surrender feels reasonable.",
    ])
}
