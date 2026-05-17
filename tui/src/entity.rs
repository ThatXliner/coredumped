//! Entity primitives for the prototype.
//!
//! This file deliberately stays plain: positions, directions, HP, entity kinds,
//! and constructors. Game rules live in `game.rs`; rendering decisions live in
//! `render.rs`.

use bracket_lib::prelude::Point;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
            Direction::East => (1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn point(self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn from_point(point: Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }

    pub fn manhattan_distance(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hp {
    pub current: i32,
    pub max: i32,
}

impl Hp {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Player,
    Slime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: usize,
    pub kind: EntityKind,
    pub pos: Position,
    pub hp: Hp,
    pub alive: bool,
}

impl Entity {
    pub fn player(pos: Position) -> Self {
        Self {
            id: 0,
            kind: EntityKind::Player,
            pos,
            hp: Hp::new(12),
            alive: true,
        }
    }

    pub fn slime(id: usize, pos: Position) -> Self {
        Self {
            id,
            kind: EntityKind::Slime,
            pos,
            hp: Hp::new(3),
            alive: true,
        }
    }

    pub fn glyph(&self) -> char {
        match self.kind {
            EntityKind::Player => '@',
            EntityKind::Slime => 's',
        }
    }

    pub fn name(&self) -> &'static str {
        match self.kind {
            EntityKind::Player => "player",
            EntityKind::Slime => "slime",
        }
    }
}
