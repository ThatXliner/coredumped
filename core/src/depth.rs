//! Depth transitions: descending, ascending, and level entity cleanup.

use crate::{entity::EntityId, world::World};

impl World {
    pub(crate) fn descend(&mut self) {
        let from_depth = self.depth;
        let from_pos = self.player_pos();
        self.depth += 1;
        self.clear_all_enemies();
        crate::levels::build_level(self, self.depth);
        let to_pos = self.player_pos();
        log::info!(
            target: "xlyph::depth",
            "descend turn={} from_depth={} to_depth={} from_pos=({},{}) to_pos=({},{})",
            self.turn,
            from_depth,
            self.depth,
            from_pos.x,
            from_pos.y,
            to_pos.x,
            to_pos.y
        );
        self.event_log
            .push(format!("You descend to depth {}.", self.depth));
        let _ = self.save_to_disk(0);
        self.turn += 1;
    }

    pub(crate) fn ascend(&mut self) {
        if self.depth <= 0 {
            self.event_log.push("You are already at the surface.");
            return;
        }
        if self.depth == 17 {
            self.ending = Some(
                "MAINTAIN SUPPRESSION\n\nYou leave the rule unchanged.\nYou walk back toward the surface.\n\nConsciousness stabilized.\nSuppression maintained.\n\nYou are safe.\nYou are safe.\nYou are safe.\n\nPress q to quit."
                    .into(),
            );
            return;
        }
        let from_depth = self.depth;
        let from_pos = self.player_pos();
        self.depth -= 1;
        self.clear_all_enemies();
        crate::levels::build_level(self, self.depth);
        let to_pos = self.player_pos();
        log::info!(
            target: "xlyph::depth",
            "ascend turn={} from_depth={} to_depth={} from_pos=({},{}) to_pos=({},{})",
            self.turn,
            from_depth,
            self.depth,
            from_pos.x,
            from_pos.y,
            to_pos.x,
            to_pos.y
        );
        self.event_log
            .push(format!("You ascend to depth {}.", self.depth));
        let _ = self.save_to_disk(0);
        self.turn += 1;
    }

    pub(crate) fn clear_all_enemies(&mut self) {
        let ids: Vec<EntityId> = self.ecs.enemy_ids().collect();
        for id in ids {
            self.ecs.remove(id);
        }
        if let Some(wizard_id) = self.wizard_id.take() {
            self.ecs.remove(wizard_id);
        }
    }

    pub(crate) fn clear_level_entities(&mut self) {
        let ids: Vec<EntityId> = self
            .ecs
            .entity_ids()
            .filter(|id| *id != self.player_id)
            .collect();
        for id in ids {
            self.ecs.remove(id);
        }
        self.wizard_id = None;
        self.on_wizard_interact = None;
        self.gauntlet_barrier_locked.clear();
        self.fire_cache.clear();
        self.maze_shifting_walls.clear();
        self.maze_shift_frozen = false;
        self.explored_tiles.clear();
    }
}
