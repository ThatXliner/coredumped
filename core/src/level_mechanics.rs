//! Level-specific gameplay mechanics.
//!
//! Depth-gated interactions like pressure plates, locked doors, gauntlet barriers,
//! and shifting maze walls.

use bracket_color::prelude::{CYAN, GREEN, RED, RGB, YELLOW};

use crate::entity::{EntityKind, Position};
use crate::map::TileType;
use crate::world::World;

impl World {
    /// Check if position is a counting room locked door (depth 8).
    pub(crate) fn counting_room_locked_door(pos: Position) -> bool {
        matches!((pos.x, pos.y), (16, 12) | (28, 12) | (16, 18) | (28, 18))
    }

    /// Activate pressure plate effects at given position.
    pub(crate) fn activate_pressure_plate(&mut self, pos: Position) {
        // Barrel room pressure plate at (38, 28) - toggles door from room 7
        if self.depth == 2 && pos == Position::new(38, 28) {
            let door_pos = Position::new(35, 28);
            let door_closed = self.map.tile(door_pos) == TileType::Wall;
            if door_closed {
                self.map.set_tile(door_pos, TileType::Floor);
                self.map.set_tile(door_pos.offset(0, 1), TileType::Floor);
                self.event_log
                    .push_colored("Click. The door opens.", RGB::named(GREEN));
            } else {
                self.map.set_tile(door_pos, TileType::Wall);
                self.map.set_tile(door_pos.offset(0, 1), TileType::Wall);
                self.event_log
                    .push_colored("Click. The door slides shut behind you.", RGB::named(GREEN));
            }
        }
    }

    /// Try to open a counting room locked door. Returns true if handled.
    pub(crate) fn try_open_counting_room_door(&mut self, target: Position) -> bool {
        if self.depth != 8 || !Self::counting_room_locked_door(target) {
            return false;
        }

        if self.held_keys.pop().is_some() {
            self.map.set_tile(target, TileType::Floor);
            self.event_log.push_colored(
                "The key dissolves in your hand. The locked door opens.",
                RGB::named(CYAN),
            );
        } else {
            self.event_log.push_colored(
                "The door is locked. Somewhere nearby, a key-goblin is carrying what you need.",
                RGB::named(YELLOW),
            );
        }
        true
    }

    /// Award a memory-key when killing a goblin in the counting room (depth 8).
    pub(crate) fn award_counting_room_key(&mut self, target_kind: EntityKind) {
        if self.depth == 8 && target_kind == EntityKind::Goblin && self.held_keys.len() < 3 {
            let key_id = format!("memory-key-{}", self.held_keys.len() + 1);
            self.held_keys.push(key_id);
            self.event_log.push_colored(
                "A memory-key clatters to the floor. You pick it up.",
                RGB::named(CYAN),
            );
        }
    }

    /// Check and lock gauntlet barriers as player crosses them (depth 6).
    pub(crate) fn check_gauntlet_barriers(&mut self) {
        if self.depth != 6 {
            return;
        }
        let barrier_xs = [7, 13, 19, 25, 31, 37, 43, 49];
        let corridor_y = crate::map::MAP_HEIGHT / 2;
        let player_x = self.player_pos().x;

        for &bx in &barrier_xs {
            if player_x > bx && self.gauntlet_barrier_locked.insert(bx) {
                for dy in -2..=2 {
                    let pos = Position::new(bx, corridor_y + dy);
                    if let Some(entity_id) = self.ecs.entity_at(pos) {
                        if let Some(evacuation_pos) =
                            self.gauntlet_barrier_evacuation_pos(bx, corridor_y)
                        {
                            self.ecs.set_position(entity_id, evacuation_pos);
                        }
                    }
                    self.map.set_tile(pos, TileType::Wall);
                }
                self.event_log
                    .push_colored("A barrier slams shut behind you!", RGB::named(RED));
            }
        }
    }

    /// Find an evacuation position for entities when a gauntlet barrier closes.
    pub(crate) fn gauntlet_barrier_evacuation_pos(
        &self,
        bx: i32,
        corridor_y: i32,
    ) -> Option<Position> {
        for distance in 1..=8 {
            for x in [bx - distance, bx + distance] {
                if matches!(x, 7 | 13 | 19 | 25 | 31 | 37 | 43 | 49) {
                    continue;
                }
                let candidate = Position::new(x, corridor_y);
                if self.map.is_walkable(candidate) && self.ecs.entity_at(candidate).is_none() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// Shift maze walls on each tick (depth 10).
    pub(crate) fn shift_maze_walls(&mut self) {
        if self.depth != 10 || self.maze_shifting_walls.is_empty() {
            return;
        }

        // Check for exploit: if console buffer contains (quote :still), freeze maze
        // This is the "eval injection" — the maze/shift rule reads the buffer
        // without checking if it was submitted.
        if !self.maze_shift_frozen {
            let buffer = self.console_buffer.trim();
            if buffer.contains("(quote :still)") || buffer.contains("':still") || buffer == ":still"
            {
                self.maze_shift_frozen = true;
                self.event_log.push_colored(
                    "The walls shudder... and stop. The maze holds its breath.",
                    RGB::named(CYAN),
                );
                return;
            }
        }

        if self.maze_shift_frozen {
            return;
        }

        // Toggle walls based on turn parity
        let is_wall_phase = self.turn % 2 == 0;
        let walls: Vec<Position> = self.maze_shifting_walls.iter().copied().collect();
        for pos in walls {
            let new_tile = if is_wall_phase {
                TileType::Wall
            } else {
                TileType::Floor
            };
            // Don't shift if player or enemy is standing there
            if self.ecs.entity_at(pos).is_none() {
                self.map.set_tile(pos, new_tile);
            }
        }
    }
}
