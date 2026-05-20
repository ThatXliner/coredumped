use crate::{
    entity::{Direction, Position},
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 3 — Quiet Halls (Denial: corridor maze, bridge to Anger)
// ---------------------------------------------------------------------------

pub(crate) fn build_quiet_halls(world: &mut World) {
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
