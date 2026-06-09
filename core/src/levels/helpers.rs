use std::collections::{HashSet, VecDeque};

use crate::{
    entity::EntityId,
    entity::Position,
    map::{Map, MapGenOutput, TileType},
    world::World,
};

pub(crate) fn apply_map(world: &mut World, gen: &MapGenOutput) {
    world.map = gen.map.clone();
    world.ecs.set_position(world.player_id, gen.player_start);
}

pub(crate) fn spawn_wizard_near_player(world: &mut World) {
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

pub(crate) fn spawn_fragment_near_open_floor(
    world: &mut World,
    preferred: Position,
    fragment_id: &str,
) -> EntityId {
    let pos = nearest_open_floor(world, preferred).unwrap_or(preferred);
    world.ecs.spawn_fragment(pos, fragment_id)
}

pub(crate) fn nearest_open_floor(world: &World, preferred: Position) -> Option<Position> {
    if world.map.width <= 0 || world.map.height <= 0 {
        return None;
    }

    let start = Position::new(
        preferred.x.clamp(0, world.map.width - 1),
        preferred.y.clamp(0, world.map.height - 1),
    );
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some(pos) = queue.pop_front() {
        if world.map.is_walkable(pos) && world.ecs.entity_at(pos).is_none() {
            return Some(pos);
        }

        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let next = pos.offset(dx, dy);
            if world.map.contains(next) && visited.insert(next) {
                queue.push_back(next);
            }
        }
    }

    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragment_spawn_snaps_wall_targets_to_open_floor() {
        let mut world = World::minimal();
        world.map = Map::new_filled(12, 10, TileType::Wall);
        let floor = Position::new(7, 5);
        world.map.set_tile(floor, TileType::Floor);

        let id = spawn_fragment_near_open_floor(&mut world, Position::new(4, 5), "frag-007");

        assert_eq!(world.ecs.position(id), Some(floor));
        assert_eq!(world.ecs.fragment_id(id), Some("frag-007"));
    }
}
