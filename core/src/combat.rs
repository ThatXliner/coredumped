//! Combat systems: player attacks, shoves, and barrel destruction.

use bracket_color::prelude::{CYAN, DARK_GRAY, ORANGE, RGB, YELLOW};

use crate::{
    entity::{Direction, EntityKind},
    map::TileType,
    world::World,
};

impl World {
    /// Deal 1 damage to the first entity in the given direction. Does not move the player.
    pub(crate) fn attack_in_direction(&mut self, direction: Direction, force: i32) {
        let (dx, dy) = direction.delta();
        let target = self.player_pos().offset(dx, dy);

        self.last_impact_force = force;
        self.last_impact_target = None;

        if !self.map.is_walkable(target) {
            self.event_log.push_colored(
                "You strike the wall. Nothing happens.",
                RGB::named(DARK_GRAY),
            );
            return;
        }

        if let Some(target_id) = self.ecs.entity_at(target) {
            if self.ecs.kind(target_id) == Some(EntityKind::Barrel) {
                self.bump_barrel(target_id);
                return;
            }
            let target_name = self.ecs.name(target_id);
            let target_kind = self.ecs.kind(target_id).unwrap_or(EntityKind::Slime);
            self.last_impact_target = Some(target_kind);
            let hp = self
                .ecs
                .damage(target_id, 1)
                .expect("combat targets should have an Hp component");

            self.event_log.push_colored(
                format!("You strike the {target_name} for 1 damage."),
                RGB::named(ORANGE),
            );

            if hp.current <= 0 {
                self.event_log.push_colored(
                    format!("The {target_name} collapses into inert code."),
                    RGB::named(ORANGE),
                );
                self.award_counting_room_key(target_kind);
            }
        } else {
            self.event_log
                .push_colored("You swing at empty air.", RGB::named(DARK_GRAY));
        }
    }

    /// Shove the first entity in the given direction one tile away. Does not move the player.
    pub(crate) fn shove_in_direction(&mut self, direction: Direction) {
        let (dx, dy) = direction.delta();
        let target = self.player_pos().offset(dx, dy);

        if !self.map.is_walkable(target) {
            self.event_log.push_colored(
                "You shove the wall. Nothing happens.",
                RGB::named(DARK_GRAY),
            );
            return;
        }

        if let Some(target_id) = self.ecs.entity_at(target) {
            let shove_target = target.offset(dx, dy);
            let enemy_name = self.ecs.name(target_id);
            if self.map.is_walkable(shove_target) && self.ecs.entity_at(shove_target).is_none() {
                self.ecs.set_position(target_id, shove_target);
                self.event_log.push_colored(
                    format!("You shove the {} back.", enemy_name),
                    RGB::named(YELLOW),
                );
            } else {
                self.event_log
                    .push(format!("You shove the {}. It doesn't budge.", enemy_name));
            }
            self.player_attacked.push(target_id);
        } else {
            self.event_log
                .push_colored("You shove at empty air.", RGB::named(DARK_GRAY));
        }
    }

    pub(crate) fn bump_barrel(&mut self, barrel_id: crate::entity::EntityId) {
        let pos = self
            .ecs
            .position(barrel_id)
            .expect("barrel should have a position");
        self.ecs.damage(barrel_id, 1);
        self.event_log
            .push_colored("The barrel shatters into splinters!", RGB::named(ORANGE));

        if self.map.tile(pos) == TileType::StairsDown {
            self.event_log
                .push_colored("The stairs down are revealed!", RGB::named(CYAN));
        }
    }
}
