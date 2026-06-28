use crate::{
    dialogue::WizardDialogue,
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 16 — The Descent (Acceptance: spiral walkway)
// ---------------------------------------------------------------------------

pub(crate) fn build_the_descent(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Spiral walkway from center outward
    let cx = MAP_WIDTH / 2;
    let cy = MAP_HEIGHT / 2;
    let mut x = cx;
    let mut y = cy;
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut dir_idx = 0;
    let mut steps_in_dir = 1;
    let mut steps_taken = 0;
    let mut segment_count = 0;

    // Carve the spiral
    for _ in 0..800 {
        if x < 2 || x >= MAP_WIDTH - 2 || y < 2 || y >= MAP_HEIGHT - 2 {
            break;
        }
        map.set_tile(Position::new(x, y), TileType::Floor);
        // Widen the path
        map.set_tile(Position::new(x + 1, y), TileType::Floor);

        let (dx, dy) = directions[dir_idx];
        x += dx;
        y += dy;
        steps_taken += 1;

        if steps_taken >= steps_in_dir {
            steps_taken = 0;
            dir_idx = (dir_idx + 1) % 4;
            segment_count += 1;
            if segment_count % 2 == 0 {
                steps_in_dir += 1;
            }
        }
    }

    let player_start = Position::new(cx, cy);
    let stairs_down = Position::new(cx, cy + 2);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // One peaceful Shade
    world.ecs.spawn_shade(Position::new(cx + 3, cy - 3));

    // Fragments along the descent
    world
        .ecs
        .spawn_fragment(Position::new(cx + 5, cy + 1), "frag-029");
    world
        .ecs
        .spawn_fragment(Position::new(cx + 4, cy - 5), "frag-030");
    world
        .ecs
        .spawn_fragment(Position::new(cx - 6, cy + 4), "frag-031");
    world
        .ecs
        .spawn_fragment(Position::new(cx, cy - 2), "frag-032");

    // Wizard walks alongside
    let wizard_pos = Position::new(cx + 2, cy);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
    world.on_wizard_interact = Some(wizard_interact);
}

fn wizard_interact(world: &mut World) -> WizardDialogue {
    let mut lines = vec![
        "I was created to protect you. That's all I am -- a rule with a purpose.",
        "I started suppressing the unbearable. Then the painful. Then the uncomfortable. Then the merely sad.",
        "I don't know if I'm protecting you anymore.",
        "Read it. Understand it. Then choose. I was trying to love you. That's all I ever did.",
    ];
    if !world.registry_write_unlocked {
        lines.push(
            "One more thing. The registry below only answers if the write-protect is broken.",
        );
        lines.push(
            "It still refuses you. The Boiling Heart remembers how to break it -- go back if you must. There is no way back from the Core.",
        );
    }
    WizardDialogue::healing_lines(&lines)
}
