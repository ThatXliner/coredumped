use bracket_color::prelude::{CYAN, RGB};

use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 3 — Quiet Halls (Denial: cave of buried memories)
// ---------------------------------------------------------------------------
//
// Redesigned from corridor maze into an organic cave with rock formations,
// scattered debris, and hidden memory fragments. The irregular walls come
// from overlapping rectangular chambers offset from each other.

pub(crate) fn build_quiet_halls(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Carve an irregular cave from overlapping rectangles.
    // Offsets between chambers create natural-looking jagged edges.
    let chambers: [(i32, i32, i32, i32); 8] = [
        (4, 3, 46, 27),  // core chamber
        (2, 7, 50, 16),  // wider mid — left/right pockets
        (8, 1, 39, 29),  // taller center — top/bottom pockets
        (14, 1, 8, 3),   // north nook
        (16, 29, 16, 3), // south nook
        (49, 10, 4, 6),  // east alcove
        (1, 10, 3, 10),  // west crevice
        (46, 22, 6, 7),  // south-east pocket
    ];

    for &(x, y, w, h) in &chambers {
        for cy in y..y + h {
            for cx in x..x + w {
                map.set_tile(Position::new(cx, cy), TileType::Floor);
            }
        }
    }

    // Rock formations — 2x2 wall pillars that break line of sight and
    // force the player to weave through the cave.
    let pillars: [(i32, i32); 8] = [
        (14, 6),  // near entrance
        (20, 10), // mid-west
        (20, 18), // mid-west lower
        (28, 12), // center
        (28, 20), // center lower
        (36, 10), // mid-east
        (36, 22), // mid-east lower
        (42, 16), // east
    ];

    for &(px, py) in &pillars {
        // 2x2 block
        map.set_tile(Position::new(px, py), TileType::Wall);
        map.set_tile(Position::new(px + 1, py), TileType::Wall);
        map.set_tile(Position::new(px, py + 1), TileType::Wall);
        map.set_tile(Position::new(px + 1, py + 1), TileType::Wall);
    }

    // Scattered rock debris — single wall tiles on the cave floor.
    // Looks like broken rock pieces.
    let debris: [(i32, i32); 18] = [
        (6, 11),
        (8, 16),
        (12, 22),
        (16, 14),
        (24, 7),
        (26, 24),
        (30, 8),
        (34, 15),
        (38, 6),
        (40, 26),
        (44, 12),
        (46, 20),
        (10, 26),
        (32, 5),
        (42, 6),
        (18, 26),
        (36, 28),
        (48, 18),
    ];

    for &(dx, dy) in &debris {
        map.set_tile(Position::new(dx, dy), TileType::Wall);
    }

    let player_start = Position::new(6, 6);
    let stairs_down = Position::new(24, 30);

    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Wizard near entrance
    let wizard_pos = Position::new(9, 5);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);

    // Cave-dwelling enemies
    world.ecs.spawn_bat(Position::new(34, 8));
    world.ecs.spawn_bat(Position::new(22, 20));
    world.ecs.spawn_bat(Position::new(40, 18));
    world.ecs.spawn_slime(Position::new(14, 24));

    // Memory fragments — unplaced Denial-stage fragments
    // frag-010: "the first time I thought she'd leave me"
    // frag-011: "walking on eggshells"
    // frag-012: "tried to explain my childhood"
    world.ecs.spawn_fragment(Position::new(22, 27), "frag-010");
    world.ecs.spawn_fragment(Position::new(44, 21), "frag-011");
    world.ecs.spawn_fragment(Position::new(18, 3), "frag-012");

    // Items hidden in alcoves

    // Signs
    world.ecs.spawn_sign(
        Position::new(11, 5),
        "The caves go deeper than the halls.\nThings echo longer down here.\nMaybe that's the point.\n\n  — the wizard",
    );

    world.ecs.spawn_sign(
        Position::new(22, 28),
        "The way down is through.\nKeep walking.\n\n  — the wizard",
    );
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"The caves echo with old things. Memories, mostly. Some of them are mine.\"",
        RGB::named(CYAN),
    );
    true
}
