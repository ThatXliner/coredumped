use crate::{
    entity::Position,
    map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH},
    world::World,
};

// ---------------------------------------------------------------------------
// Depth 14 — Ash Field (Depression Boss: fire zones)
// ---------------------------------------------------------------------------

pub(crate) fn build_ash_field(world: &mut World) {
    let mut map = Map::new_filled(MAP_WIDTH, MAP_HEIGHT, TileType::Wall);

    // Open field: all floor with wall borders
    for y in 1..MAP_HEIGHT - 1 {
        for x in 1..MAP_WIDTH - 1 {
            map.set_tile(Position::new(x, y), TileType::Floor);
        }
    }

    let player_start = Position::new(3, 3);
    let stairs_down = Position::new(MAP_WIDTH - 3, MAP_HEIGHT - 3);
    map.set_tile(player_start, TileType::StairsUp);
    map.set_tile(stairs_down, TileType::StairsDown);

    // Fire zones — walkable but damaging (fire/burn rule handles damage)
    let fire_centers = [
        Position::new(20, 10),
        Position::new(40, 18),
        Position::new(28, 24),
    ];
    for &center in &fire_centers {
        for dy in -2..=2 {
            for dx in -2..=2 {
                let pos = Position::new(center.x + dx, center.y + dy);
                if (dx.abs() + dy.abs()) <= 3 && map.tile(pos) == TileType::Floor {
                    map.set_tile(pos, TileType::Fire);
                }
            }
        }
    }

    world.map = map;
    world.ecs.set_position(world.player_id, player_start);

    // Fragments scattered across the field
    world.ecs.spawn_fragment(Position::new(30, 14), "frag-019");
    world.ecs.spawn_fragment(Position::new(18, 8), "frag-020");
    world.ecs.spawn_fragment(Position::new(44, 12), "frag-021");
    world.ecs.spawn_fragment(Position::new(6, 26), "frag-022");
    world.ecs.spawn_fragment(Position::new(50, 4), "frag-023");
    world.ecs.spawn_fragment(Position::new(48, 28), "frag-024");

    // Wizard at end
    let wizard_pos = Position::new(MAP_WIDTH - 4, MAP_HEIGHT - 4);
    world.wizard_id = Some(world.ecs.spawn_wizard(wizard_pos));
}
