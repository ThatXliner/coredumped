use bracket_color::prelude::{CYAN, RGB, YELLOW};
use bracket_random::prelude::RandomNumberGenerator;

use super::helpers::{
    apply_map, find_stairs_down, spawn_fragment_near_open_floor, spawn_sign_near_open_floor,
};
use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 10 — Maze of Regret (Bargaining: shifting maze with injection exploit)
// ---------------------------------------------------------------------------
//
// The maze walls shift every tick, blocking and unblocking paths.
// This represents rumination — the same thoughts, never the same path through.
//
// EXPLOIT: The maze/shift rule reads (eval (player :console-buffer)) to check
// for a config override. If the buffer contains (quote :still), the maze freezes.
// The player doesn't need to submit — just typing it is enough.

pub(crate) fn build_maze_of_regret(world: &mut World) {
    let gen = Map::generate(MAP_WIDTH, MAP_HEIGHT, 10, world.level_seed(10));
    for pos in &gen.combat_spawns {
        world.spawn_enemy_at(*pos, 10);
    }
    apply_map(world, &gen);

    // Clear any previous maze state
    world.maze_shifting_walls.clear();
    world.maze_shift_frozen = false;

    // Mark certain floor tiles as "shifting walls"
    // These will toggle between Wall and Floor each tick. Salted so the wall
    // pattern doesn't replay the map generator's random stream.
    let mut rng = RandomNumberGenerator::seeded(world.level_seed(10) ^ 0x4D41_5A45);
    let player_pos = gen.player_start;
    let stairs_pos = find_stairs_down(&world.map);

    for y in 1..world.map.height - 1 {
        for x in 1..world.map.width - 1 {
            let pos = Position::new(x, y);

            // Skip tiles near player start or stairs
            if pos.manhattan_distance(player_pos) < 4 {
                continue;
            }
            if pos.manhattan_distance(stairs_pos) < 4 {
                continue;
            }

            // Skip tiles near enemy spawn positions
            let near_spawn = gen
                .combat_spawns
                .iter()
                .any(|sp| pos.manhattan_distance(*sp) < 3);
            if near_spawn {
                continue;
            }

            // Only convert some floor tiles to shifting walls
            if world.map.tile(pos) == TileType::Floor {
                // ~15% of floor tiles become shifting walls
                if rng.range(0, 100) < 15 {
                    world.maze_shifting_walls.insert(pos);
                    // Start as wall on even turns
                    if world.turn % 2 == 0 {
                        world.map.set_tile(pos, TileType::Wall);
                    }
                }
            }
        }
    }

    // Fragments in various rooms
    spawn_fragment_near_open_floor(
        world,
        Position::new(player_pos.x + 5, player_pos.y + 2),
        "frag-007",
    );
    spawn_fragment_near_open_floor(
        world,
        Position::new(stairs_pos.x - 4, stairs_pos.y - 2),
        "frag-008",
    );
    spawn_fragment_near_open_floor(
        world,
        Position::new(stairs_pos.x + 2, stairs_pos.y + 4),
        "frag-009",
    );

    // Wizard at entrance — hints at the shifting but also at the exploit
    let wizard_pos = Position::new(player_pos.x + 2, player_pos.y - 2);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);

    // Make maze/shift rule visible
    world.known_rule_ids.insert("maze-shift".into());
    world.new_rule_ids.insert("maze-shift".into());

    // For players who broke the write-protect at the Boiling Heart: the
    // permanent solution. (The console-buffer trick still works without it.)
    spawn_sign_near_open_floor(
        world,
        Position::new(player_pos.x - 2, player_pos.y + 2),
        "The walls obey maze/shift. maze/shift is registered.\n\nRegistered things can be unregistered — if you have write access to the registry.\n\n  (let r (open-registry :rule-registry)\n    (r :unregister :maze/shift))\n\nEach patch costs HP. Check the inspector (i) to see the rule before you change it.",
    );

    world.event_log.push_colored(
        "The walls here shift and breathe. The maze remembers differently each moment.",
        RGB::named(YELLOW),
    );
}

fn wizard_interact(world: &mut World) -> bool {
    if world.maze_shift_frozen {
        world.event_log.push_colored(
            "\"You found it. The rule reads what you type, not what you submit. A debug hook no one removed.\"",
            RGB::named(CYAN),
        );
    } else {
        world.event_log.push_colored(
            "\"The maze shifts because it's designed to. But every rule has inputs. Some aren't meant to be touched.\"",
            RGB::named(CYAN),
        );
        world.event_log.push_colored(
            "\"Look at maze/shift in the inspector. See what it reads. See what it trusts.\"",
            RGB::named(CYAN),
        );
    }
    true
}
