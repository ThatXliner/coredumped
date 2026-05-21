use bracket_lib::prelude::{CYAN, RGB};

use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 15 — The Clearing (Acceptance: beautiful open glade)
// ---------------------------------------------------------------------------

pub(crate) fn build_the_clearing(world: &mut World) {
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
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> bool {
    world.event_log.push_colored(
        "\"I was so sure I was protecting you. But protection isn't supposed to make the world smaller. I made it a cage.\"",
        RGB::named(CYAN),
    );
    true
}
