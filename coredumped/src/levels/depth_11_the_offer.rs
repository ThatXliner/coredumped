use bracket_color::prelude::{CYAN, RED, RGB};

use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 11 — The Offer (Bargaining Boss: 4 sentry chambers)
// ---------------------------------------------------------------------------

pub(crate) fn build_the_offer(world: &mut World) {
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
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"Type this. Reset suppression to v1. You wake at the surface. No pain. No memory.\"",
        RGB::named(CYAN),
    );
    world
        .event_log
        .push_colored("  (forget-everything)", RGB::named(RED));
    world
        .event_log
        .push_colored("\"Or keep going. I can't stop you.\"", RGB::named(CYAN));
    false
}
