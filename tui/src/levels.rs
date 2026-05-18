//! Hand-crafted and procedural level definitions.
//!
//! Each level is built by a self-contained function that sets the map, places
//! the player, and spawns entities. Adding a new hand-crafted level means
//! writing one builder function and adding a match arm in [`build_level`].

use crate::{
    entity::Position,
    map::{Map, MapGenOutput, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

/// Dispatch to the appropriate level builder for the given depth.
pub fn build_level(world: &mut World, depth: u32) {
    match depth {
        1 | 2 => build_procedural_level(world, depth),
        3 => build_wizard_chamber(world),
        4 => build_barrel_depths(world),
        _ => build_procedural_level(world, depth),
    }
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
// Depth 4 — Barrel Depths
// ---------------------------------------------------------------------------

/// Build the map for the barrel depths: a solid rectangle of floor with walls
/// around the border. The entire level is then filled with 1-HP barrels.
pub fn generate_barrel_depths() -> MapGenOutput {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Floor);

    // Walls around the entire border
    for x in 0..MAP_WIDTH {
        map.set_tile(Position::new(x, 0), TileType::Wall);
        map.set_tile(Position::new(x, MAP_HEIGHT - 1), TileType::Wall);
    }
    for y in 0..MAP_HEIGHT {
        map.set_tile(Position::new(0, y), TileType::Wall);
        map.set_tile(Position::new(MAP_WIDTH - 1, y), TileType::Wall);
    }

    let player_start = Position::new(2, 2);
    let stairs_down = Position::new(MAP_WIDTH - 3, MAP_HEIGHT - 3);

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

fn build_barrel_depths(world: &mut World) {
    let gen = generate_barrel_depths();
    apply_map(world, &gen);

    let stairs = find_stairs_down(&world.map);

    // A 4×3 clear zone at the top-left — no barrels here
    let clear_zone: [Position; 12] = [
        Position::new(2, 2),
        Position::new(3, 2),
        Position::new(4, 2),
        Position::new(5, 2),
        Position::new(2, 3),
        Position::new(3, 3),
        Position::new(4, 3),
        Position::new(5, 3),
        Position::new(2, 4),
        Position::new(3, 4),
        Position::new(4, 4),
        Position::new(5, 4),
    ];

    // Subtle hint sign at the right edge of the clear zone
    let subtle_sign_pos = Position::new(5, 3);
    world.ecs.spawn_sign(
        subtle_sign_pos,
        "This is a lot of barrels...\nThink about rebinding your keys.\nOne key can (do) what many cannot.",
    );

    // Obvious hint sign buried somewhere in the barrels
    let obvious_sign_pos = Position::new(MAP_WIDTH - 5, MAP_HEIGHT / 2);
    world.ecs.spawn_sign(
        obvious_sign_pos,
        "\
Welcome to the Barrel Depths!\n\n\
Each (do-attack) costs 1 tick. But\nyou can chain them with (do ...):\n  \
(do (do-attack :north) (do-attack :south))\n\
That attacks twice — 2 ticks total.\n\
Bind the full combo to ONE key:\n  \
(bind-key :x (do (do-attack :north) (do-attack :south)\n  \
                  (do-attack :east)  (do-attack :west)))\n\
Now clear these barrels and find the exit!",
    );

    // Fill every walkable tile with 1-HP barrels, except the clear zone and signs
    for y in 1..MAP_HEIGHT - 1 {
        for x in 1..MAP_WIDTH - 1 {
            let pos = Position::new(x, y);
            if clear_zone.contains(&pos)
                || pos == subtle_sign_pos
                || pos == obvious_sign_pos
                || pos == stairs
            {
                continue;
            }
            world.ecs.spawn_barrel(pos);
        }
    }

    // One barrel on top of the exit stairs, hiding it
    world.ecs.spawn_barrel(stairs);
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
