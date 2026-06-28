use crate::{
    dialogue::{Dialogue, DialogueLine, DialogueSpeaker, WizardDialogue},
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

    // Corridors from hub to chambers. The W/E chambers sit above the hub row
    // and the SW/SE chambers below it, so each route is an L-shape: along the
    // hub row (or down the hub edge), then turning into the chamber.
    for t in 0..=12 {
        map.set_tile(Position::new(cx - 3 - t, cy), TileType::Floor); // W arm
        map.set_tile(Position::new(cx + 3 + t, cy), TileType::Floor); // E arm
    }
    for y in cy - 2..cy {
        map.set_tile(Position::new(cx - 15, y), TileType::Floor); // up into W
        map.set_tile(Position::new(cx + 15, y), TileType::Floor); // up into E
    }
    for y in cy + 3..=cy + 6 {
        map.set_tile(Position::new(cx - 3, y), TileType::Floor); // down toward SW
        map.set_tile(Position::new(cx + 3, y), TileType::Floor); // down toward SE
    }
    for t in 0..=8 {
        map.set_tile(Position::new(cx - 3 - t, cy + 6), TileType::Floor); // into SW
        map.set_tile(Position::new(cx + 3 + t, cy + 6), TileType::Floor); // into SE
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

fn wizard_interact(_world: &mut World) -> WizardDialogue {
    WizardDialogue::no_heal(Dialogue::mixed(
        DialogueSpeaker::Wizard,
        [
            DialogueLine::speech(
                "Type this. Reset suppression to v1. You wake at the surface. No pain. No memory.",
            ),
            DialogueLine::danger("(forget-everything)"),
            DialogueLine::speech("Or keep going. I can't stop you."),
        ],
    ))
}
