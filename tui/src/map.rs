//! Static map and bracket-lib pathing implementation.
//!
//! The map owns terrain, walkability, pathfinding exits, and the v1 flashlight
//! ray caster. It does not know about entities beyond receiving a player
//! position/facing for visibility queries.

use std::collections::HashSet;

use bracket_lib::prelude::{Algorithm2D, BaseMap, DistanceAlg, Point, SmallVec};

use crate::entity::{Direction, Position};

pub const MAP_WIDTH: i32 = 55;
pub const MAP_HEIGHT: i32 = 30;
pub const FLASHLIGHT_RADIUS: i32 = 12;
const FLASHLIGHT_SPREAD_DOT: f32 = 0.70;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    Floor,
    Wall,
}

#[derive(Clone, Debug)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    tiles: Vec<TileType>,
}

impl Map {
    pub fn new_static() -> Self {
        let mut map = Self {
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        };

        for x in 0..MAP_WIDTH {
            map.set_tile(Position::new(x, 0), TileType::Wall);
            map.set_tile(Position::new(x, MAP_HEIGHT - 1), TileType::Wall);
        }

        for y in 0..MAP_HEIGHT {
            map.set_tile(Position::new(0, y), TileType::Wall);
            map.set_tile(Position::new(MAP_WIDTH - 1, y), TileType::Wall);
        }

        for x in 8..44 {
            if x != 23 && x != 24 {
                map.set_tile(Position::new(x, 8), TileType::Wall);
            }
        }

        for y in 13..25 {
            if y != 18 {
                map.set_tile(Position::new(31, y), TileType::Wall);
            }
        }

        for x in 34..50 {
            if x != 42 {
                map.set_tile(Position::new(x, 21), TileType::Wall);
            }
        }

        map
    }

    pub fn idx(&self, pos: Position) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    pub fn point_for_idx(&self, idx: usize) -> Point {
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        Point::new(x, y)
    }

    pub fn position_for_idx(&self, idx: usize) -> Position {
        Position::from_point(self.point_for_idx(idx))
    }

    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }

    pub fn tile(&self, pos: Position) -> TileType {
        self.tiles[self.idx(pos)]
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        self.contains(pos) && self.tile(pos) == TileType::Floor
    }

    pub fn flashlight_tiles(&self, origin: Position, facing: Direction) -> HashSet<Position> {
        let mut lit = HashSet::new();
        if !self.contains(origin) {
            return lit;
        }

        lit.insert(origin);
        for target in self.flashlight_targets(origin, facing) {
            for pos in self.ray_until_blocked(origin, target) {
                lit.insert(pos);
            }
        }
        lit
    }

    fn set_tile(&mut self, pos: Position, tile: TileType) {
        if self.contains(pos) {
            let idx = self.idx(pos);
            self.tiles[idx] = tile;
        }
    }

    fn maybe_exit(&self, exits: &mut SmallVec<[(usize, f32); 10]>, pos: Position) {
        if self.is_walkable(pos) {
            exits.push((self.idx(pos), 1.0));
        }
    }

    fn flashlight_targets(&self, origin: Position, facing: Direction) -> Vec<Position> {
        let (fx, fy) = facing.delta();
        let facing_len = ((fx * fx + fy * fy) as f32).sqrt();
        let mut targets = Vec::new();

        for y in (origin.y - FLASHLIGHT_RADIUS)..=(origin.y + FLASHLIGHT_RADIUS) {
            for x in (origin.x - FLASHLIGHT_RADIUS)..=(origin.x + FLASHLIGHT_RADIUS) {
                let pos = Position::new(x, y);
                if !self.contains(pos) || pos == origin {
                    continue;
                }

                let dx = x - origin.x;
                let dy = y - origin.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq > FLASHLIGHT_RADIUS * FLASHLIGHT_RADIUS {
                    continue;
                }

                let dist = (dist_sq as f32).sqrt();
                let dot = ((dx * fx + dy * fy) as f32) / (dist * facing_len);
                if dot >= FLASHLIGHT_SPREAD_DOT {
                    targets.push(pos);
                }
            }
        }

        targets
    }

    fn ray_until_blocked(&self, origin: Position, target: Position) -> Vec<Position> {
        let mut ray = Vec::new();
        for pos in bresenham_line(origin, target).into_iter().skip(1) {
            if !self.contains(pos) {
                break;
            }

            ray.push(pos);
            if self.tile(pos) == TileType::Wall {
                break;
            }
        }
        ray
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let pos = self.position_for_idx(idx);
        let mut exits = SmallVec::new();
        self.maybe_exit(&mut exits, pos.offset(-1, 0));
        self.maybe_exit(&mut exits, pos.offset(1, 0));
        self.maybe_exit(&mut exits, pos.offset(0, -1));
        self.maybe_exit(&mut exits, pos.offset(0, 1));
        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Manhattan.distance2d(self.point_for_idx(idx1), self.point_for_idx(idx2))
    }
}

fn bresenham_line(start: Position, end: Position) -> Vec<Position> {
    let mut points = Vec::new();
    let mut x = start.x;
    let mut y = start.y;
    let dx = (end.x - start.x).abs();
    let dy = -(end.y - start.y).abs();
    let sx = if start.x < end.x { 1 } else { -1 };
    let sy = if start.y < end.y { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push(Position::new(x, y));
        if x == end.x && y == end.y {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }

    points
}
