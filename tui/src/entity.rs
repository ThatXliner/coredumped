//! ECS component primitives for the prototype.
//!
//! This file defines the small data pieces that can be attached to an entity:
//! ids, positions, directions, HP, kinds, and render glyphs. Storage and query
//! behavior live in `ecs.rs`; gameplay systems live in `game.rs`.

use bracket_lib::prelude::Point;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(usize);

impl EntityId {
    pub(crate) fn new(raw: usize) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> usize {
        self.0
    }
}

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
    Goblin,
    Bat,
    Ogre,
    Wizard,
}

impl EntityKind {
    pub fn glyph(&self) -> char {
        match self {
            EntityKind::Player => '@',
            EntityKind::Slime => 's',
            EntityKind::Goblin => 'g',
            EntityKind::Bat => 'b',
            EntityKind::Ogre => 'O',
            EntityKind::Wizard => 'W',
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EntityKind::Player => "player",
            EntityKind::Slime => "slime",
            EntityKind::Goblin => "goblin",
            EntityKind::Bat => "bat",
            EntityKind::Ogre => "ogre",
            EntityKind::Wizard => "wizard",
        }
    }

    pub fn rule_name(&self) -> &'static str {
        match self {
            EntityKind::Slime => "slime-hunt",
            EntityKind::Goblin => "goblin-patrol",
            EntityKind::Bat => "bat-flutter",
            EntityKind::Ogre => "ogre-charge",
            EntityKind::Player | EntityKind::Wizard => "",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderGlyph {
    pub glyph: char,
}

impl RenderGlyph {
    pub fn for_kind(kind: EntityKind) -> Self {
        Self {
            glyph: kind.glyph(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityView {
    pub id: EntityId,
    pub kind: EntityKind,
    pub pos: Position,
    pub hp: Hp,
    pub alive: bool,
    pub render_glyph: RenderGlyph,
}

impl EntityView {
    pub fn glyph(self) -> char {
        self.render_glyph.glyph
    }

    pub fn name(self) -> &'static str {
        self.kind.name()
    }
}
