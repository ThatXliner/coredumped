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

    if depth >= 4 {
        spawn_wizard_near_player(world);
    }
}

// ---------------------------------------------------------------------------
// Depth 3 — Wizard's tutorial chamber
// ---------------------------------------------------------------------------

fn build_wizard_chamber(world: &mut World) {
    let gen = Map::generate_wizard_box();
    apply_map(world, &gen);
    spawn_wizard_near_player(world);
}

// ---------------------------------------------------------------------------
// Depth 4 — Barrel Depths
// ---------------------------------------------------------------------------

fn build_barrel_depths(world: &mut World) {
    let gen = Map::generate_cave(4);

    // Mix of barrels (2/3) and enemies (1/3)
    for pos in &gen.combat_spawns {
        let hash = (pos.x.wrapping_mul(31).wrapping_add(pos.y.wrapping_mul(17))) as u32;
        if hash % 3 == 0 {
            world.spawn_enemy_at(*pos, 4);
        } else {
            world.ecs.spawn_barrel(*pos);
        }
    }
    for pos in &gen.boss_spawns {
        world.spawn_boss_at(*pos);
    }

    apply_map(world, &gen);

    // One barrel hides the exit stairs
    let stairs = find_stairs_down(&world.map);
    world.ecs.spawn_barrel(stairs);

    spawn_sign_near_player(world);
    spawn_wizard_near_player(world);
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

fn spawn_sign_near_player(world: &mut World) {
    let player_pos = world.player_pos();
    let candidates = [
        player_pos.offset(3, 0),
        player_pos.offset(-3, 0),
        player_pos.offset(0, 3),
        player_pos.offset(0, -3),
        player_pos.offset(4, 0),
        player_pos.offset(-4, 0),
        player_pos.offset(0, 4),
        player_pos.offset(0, -4),
    ];
    if let Some(&pos) = candidates
        .iter()
        .find(|&&p| world.map.is_walkable(p) && world.ecs.entity_at(p).is_none())
    {
        world.ecs.spawn_sign(pos);
    }
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
