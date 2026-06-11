use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 17 — The Core (Final: vessel/suppress rule)
// ---------------------------------------------------------------------------

pub(crate) fn build_the_core(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Minimalist room: 20x15
    let rx = (MAP_WIDTH - 20) / 2;
    let ry = (MAP_HEIGHT - 15) / 2;
    let rw = 20;
    let rh = 15;
    for y in ry..ry + rh {
        for x in rx..rx + rw {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    let player_start = Position::new(rx + 2, ry + rh / 2);
    let stairs_up = Position::new(rx + 1, ry + rh / 2);
    map.set_tile(stairs_up, TileType::StairsUp);
    // No stairs down — this is the end

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);
    world.known_rule_ids.insert("vessel-suppress".into());
    world.new_rule_ids.insert("vessel-suppress".into());

    // frag-033 on pedestal
    world
        .ecs
        .spawn_fragment(Position::new(rx + rw / 2, ry + ry / 2 + 1), "frag-033");

    // The pedestal — the vessel/suppress rule
    world.ecs.spawn_sign(
        Position::new(rx + rw / 2, ry + rh / 2),
        "THE CORE\n\nvessel/suppress is here. Open the inspector (i) to read the rule. Open the console (`) to modify it.\n\nTry:\n\n  (let r (open-registry :rule-registry)\n\n    (r :write :vessel/suppress '(set! *threshold* 0)))\n\nOr touch nothing and ascend. The rule will keep doing what it has always done.\n\nEither way, the stairs up end this. The choice is yours.",
    );

    // No enemies, no wizard, no items
}
