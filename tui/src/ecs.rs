//! Tiny ECS storage for the current prototype.
//!
//! This is intentionally not a full framework. It gives the game stable
//! `EntityId`s and component stores that systems can query without hard-coding
//! separate `player` and `enemies` collections. The shape is close enough to an
//! ECS to teach the pattern while staying small enough to read in one sitting.

use std::collections::{BTreeMap, BTreeSet};

use crate::entity::{EntityId, EntityKind, EntityView, Hp, Position, RenderGlyph};

#[derive(Clone, Debug)]
pub struct Ecs {
    next_id: usize,
    entities: BTreeSet<EntityId>,
    kinds: BTreeMap<EntityId, EntityKind>,
    positions: BTreeMap<EntityId, Position>,
    hp: BTreeMap<EntityId, Hp>,
    alive: BTreeSet<EntityId>,
    enemy_ai: BTreeSet<EntityId>,
    render_glyphs: BTreeMap<EntityId, RenderGlyph>,
    sign_messages: BTreeMap<EntityId, String>,
    fragment_ids: BTreeMap<EntityId, String>,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            entities: BTreeSet::new(),
            kinds: BTreeMap::new(),
            positions: BTreeMap::new(),
            hp: BTreeMap::new(),
            alive: BTreeSet::new(),
            enemy_ai: BTreeSet::new(),
            render_glyphs: BTreeMap::new(),
            sign_messages: BTreeMap::new(),
            fragment_ids: BTreeMap::new(),
        }
    }

    pub fn spawn_player(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Player, pos, Hp::new(12), false)
    }

    pub fn spawn_slime(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Slime, pos, Hp::new(3), true)
    }

    pub fn spawn_goblin(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Goblin, pos, Hp::new(5), true)
    }

    pub fn spawn_bat(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Bat, pos, Hp::new(2), true)
    }

    pub fn spawn_ogre(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Ogre, pos, Hp::new(10), true)
    }

    pub fn spawn_wizard(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Wizard, pos, Hp::new(20), false)
    }

    pub fn spawn_shade(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Shade, pos, Hp::new(999), true)
    }

    pub fn spawn_rage(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Rage, pos, Hp::new(15), true)
    }

    pub fn spawn_sentry(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Sentry, pos, Hp::new(6), true)
    }

    pub fn spawn_barrel(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::Barrel, pos, Hp::new(1), false)
    }

    pub fn spawn_sign(&mut self, pos: Position, message: &str) -> EntityId {
        let id = self.spawn_actor(EntityKind::Sign, pos, Hp::new(999), false);
        self.sign_messages.insert(id, message.to_string());
        id
    }

    pub fn spawn_fragment(&mut self, pos: Position, fragment_id: &str) -> EntityId {
        let id = self.spawn_actor(EntityKind::Fragment, pos, Hp::new(999), false);
        self.fragment_ids.insert(id, fragment_id.to_string());
        id
    }

    pub fn spawn_shade_echo(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::ShadeEcho, pos, Hp::new(999), false)
    }

    pub fn spawn_vapor_canteen(&mut self, pos: Position) -> EntityId {
        self.spawn_actor(EntityKind::VaporCanteen, pos, Hp::new(999), false)
    }

    pub fn fragment_id(&self, id: EntityId) -> Option<&str> {
        self.fragment_ids.get(&id).map(|s| s.as_str())
    }

    pub fn remove(&mut self, id: EntityId) {
        self.entities.remove(&id);
        self.kinds.remove(&id);
        self.positions.remove(&id);
        self.hp.remove(&id);
        self.alive.remove(&id);
        self.enemy_ai.remove(&id);
        self.render_glyphs.remove(&id);
        self.sign_messages.remove(&id);
        self.fragment_ids.remove(&id);
    }

    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().copied()
    }

    pub fn position(&self, id: EntityId) -> Option<Position> {
        self.positions.get(&id).copied()
    }

    pub fn set_position(&mut self, id: EntityId, pos: Position) {
        if self.entities.contains(&id) {
            self.positions.insert(id, pos);
        }
    }

    pub fn hp(&self, id: EntityId) -> Option<Hp> {
        self.hp.get(&id).copied()
    }

    pub fn set_hp(&mut self, id: EntityId, hp: Hp) {
        if self.entities.contains(&id) {
            self.hp.insert(id, hp);
            if hp.current > 0 {
                self.alive.insert(id);
            } else {
                self.alive.remove(&id);
            }
        }
    }

    pub fn damage(&mut self, id: EntityId, amount: i32) -> Option<Hp> {
        let new_hp = {
            let hp = self.hp.get_mut(&id)?;
            hp.current -= amount;
            *hp
        };

        if new_hp.current <= 0 {
            self.alive.remove(&id);
        }

        Some(new_hp)
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.alive.contains(&id)
    }

    pub fn name(&self, id: EntityId) -> &'static str {
        self.kinds
            .get(&id)
            .map(EntityKind::name)
            .unwrap_or("entity")
    }

    pub fn sign_message(&self, id: EntityId) -> Option<&str> {
        self.sign_messages.get(&id).map(|s| s.as_str())
    }

    pub fn kind(&self, id: EntityId) -> Option<EntityKind> {
        self.kinds.get(&id).copied()
    }

    pub fn entity_at(&self, pos: Position) -> Option<EntityId> {
        self.entities
            .iter()
            .copied()
            .find(|id| self.is_alive(*id) && self.positions.get(id).copied() == Some(pos))
    }

    pub fn entity_at_except(&self, pos: Position, except: EntityId) -> Option<EntityId> {
        self.entities.iter().copied().find(|id| {
            *id != except && self.is_alive(*id) && self.positions.get(id).copied() == Some(pos)
        })
    }

    pub fn enemy_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.enemy_ai.iter().copied()
    }

    pub fn renderable_entities(&self) -> impl Iterator<Item = EntityView> + '_ {
        self.entities
            .iter()
            .copied()
            .filter_map(|id| self.view(id))
            .filter(|view| view.alive)
    }

    pub(crate) fn set_next_id(&mut self, id: usize) {
        self.next_id = id;
    }

    pub fn view(&self, id: EntityId) -> Option<EntityView> {
        Some(EntityView {
            id,
            kind: *self.kinds.get(&id)?,
            pos: *self.positions.get(&id)?,
            hp: *self.hp.get(&id)?,
            alive: self.is_alive(id),
            render_glyph: *self.render_glyphs.get(&id)?,
        })
    }

    fn spawn_actor(
        &mut self,
        kind: EntityKind,
        pos: Position,
        hp: Hp,
        has_enemy_ai: bool,
    ) -> EntityId {
        let id = self.allocate();
        self.kinds.insert(id, kind);
        self.positions.insert(id, pos);
        self.hp.insert(id, hp);
        self.alive.insert(id);
        self.render_glyphs.insert(id, RenderGlyph::for_kind(kind));

        if has_enemy_ai {
            self.enemy_ai.insert(id);
        }

        id
    }

    fn allocate(&mut self) -> EntityId {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        self.entities.insert(id);
        id
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}
