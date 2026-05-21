use bracket_lib::prelude::{CYAN, RGB};

use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 7 — Boiling Heart (Anger Boss: Rage)
// ---------------------------------------------------------------------------

pub(crate) fn build_boiling_heart(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Large boss arena: 41x25 room
    let rx = 7;
    let ry = 4;
    let rw = 41;
    let rh = 25;
    for y in ry..ry + rh {
        for x in rx..rx + rw {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    // Pillars — break up the open arena so the player can't kite freely.
    // Three 2x2 columns at x=22-23, spaced vertically.
    let pillars: [(i32, i32); 3] = [(22, 8), (22, 16), (22, 22)];
    for &(px, py) in &pillars {
        for dy in 0..2 {
            for dx in 0..2 {
                map.set_tile(Position::new(px + dx, py + dy), TileType::Wall);
            }
        }
    }

    // Secondary pillar near the boss zone — blocks straight-line retreat.
    for dy in 0..2 {
        for dx in 0..2 {
            map.set_tile(Position::new(34 + dx, 12 + dy), TileType::Wall);
        }
    }

    // Fire patches — introduce fire hazard early. Boss paths through fire;
    // player takes 1 damage/tick. Teaches the fire mechanic before level 14.
    let fire_patches: [(i32, i32); 4] = [(14, 8), (14, 20), (38, 8), (38, 20)];
    for &(fx, fy) in &fire_patches {
        for dy in 0..2 {
            for dx in 0..2 {
                let pos = Position::new(fx + dx, fy + dy);
                if map.tile(pos) == TileType::Floor {
                    map.set_tile(pos, TileType::Fire);
                }
            }
        }
    }

    let player_start = Position::new(rx + 2, ry + rh / 2);
    let stairs_down = Position::new(rx + rw - 2, ry + rh / 2);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Rage boss near stairs — blocks exit
    let rage_pos = Position::new(rx + rw - 4, ry + rh / 2);
    world.ecs.spawn_rage(rage_pos);

    // frag-002 near exit
    world
        .ecs
        .spawn_fragment(Position::new(rx + rw - 6, ry + rh / 2 + 2), "frag-002");

    // Wizard before boss — warns about the heat
    let wizard_pos = Position::new(rx + 3, ry + rh / 2 + 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);

    // Sign near entrance warning about fire
    world.ecs.spawn_sign(
        Position::new(rx + 2, ry + rh / 2 - 2),
        "The air shimmers with heat.\nFire pools on the floor ahead —\nit will burn you every step.\n\nSomething cold might quench it.",
    );
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"There's something down there — remains of something I couldn't protect you from.\"",
        RGB::named(CYAN),
    );
    true
}
