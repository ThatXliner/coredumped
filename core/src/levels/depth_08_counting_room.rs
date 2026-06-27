use bracket_color::prelude::{CYAN, RGB};

use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 8 — Counting Room (Bargaining: locked doors & keys)
// ---------------------------------------------------------------------------

pub(crate) fn build_counting_room(world: &mut World) {
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
    for pos in [
        Position::new(hub_x - 6, hub_y - 3),
        Position::new(hub_x + 6, hub_y - 3),
        Position::new(hub_x - 6, hub_y + 3),
        Position::new(hub_x + 6, hub_y + 3),
    ] {
        map.set_tile(pos, TileType::Wall);
    }

    let player_start = Position::new(hub_x, hub_y);
    let stairs_down = Position::new(hub_x, hub_y + 3);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Three goblins hold keys. Four doors exist, so one room must stay locked.
    // They guard the corridor stubs on the hub side of the doors — keys must
    // be winnable before any door opens.
    world.ecs.spawn_goblin(Position::new(hub_x - 6, hub_y - 2)); // key-goblin-1, NW door
    world.ecs.spawn_goblin(Position::new(hub_x + 6, hub_y - 2)); // key-goblin-2, NE door
    world.ecs.spawn_goblin(Position::new(hub_x + 6, hub_y + 2)); // key-goblin-3, SE door
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
    world.on_wizard_interact = Some(wizard_interact);

    world.ecs.spawn_sign(
        Position::new(hub_x - 2, hub_y + 2),
        "Four doors. Three keys. Key-goblins carry them.\n\nSpend a key by walking into a locked doorway. One door will stay locked. Choose what matters.",
    );
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"Everything down here is a bargain. Give up one thing, keep another. I've been making that trade for you for years.\"",
        RGB::named(CYAN),
    );
    world.event_log.push_colored(
        "\"Three keys, four doors. You can't save all of it. Choose what matters — and notice how much that costs.\"",
        RGB::named(CYAN),
    );
    true
}
