//! Save/load system for persisting game progress.
//!
//! The core design: persist user-evaluated Glyph **source code** (not runtime
//! state). On load, the base environment is rebuilt from hardcoded builtins,
//! then user source is replayed to restore definitions and overrides.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::entity::{Direction, EntityId, EntityKind, Hp, Position};
use crate::world::World;

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn xlyph_dir() -> PathBuf {
    home_dir().join(".xlyph")
}

pub fn saves_dir() -> PathBuf {
    xlyph_dir().join("saves")
}

pub fn save_path(slot: u32) -> PathBuf {
    saves_dir().join(format!("slot-{}.json", slot))
}

pub fn temp_edit_path() -> PathBuf {
    xlyph_dir().join("tmp").join("console-input.glyph")
}

/// Delete the auto-save (slot 0) and player profile from disk.
///
/// Used by the `--wipe` CLI flag and the in-game `(wipe!)` command.
pub fn wipe() {
    let save = save_path(0);
    if save.exists() {
        let _ = std::fs::remove_file(&save);
    }
    crate::player_profile::PlayerProfile::delete();
}

// ---------------------------------------------------------------------------
// Serialisable snapshot types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub depth: u32,
    pub turn: u64,
    pub player_id_raw: usize,
    pub player_facing: String,
    pub player_can_attack: bool,
    pub wizard_taught: bool,
    pub wizard_id_raw: Option<usize>,
    pub cheat_unlocked: bool,
    pub blocking: bool,
    pub map_width: i32,
    pub map_height: i32,
    pub map_tiles: Vec<char>,
    pub entities: Vec<EntitySnapshot>,
    pub event_log: Vec<SavedLogEntry>,
    pub bindings: Vec<(String, String)>,
    pub user_source: Vec<String>,
    pub fragment_registry: crate::fragment::FragmentRegistry,
    pub ending: Option<String>,
    #[serde(default)]
    pub registry_write_unlocked: bool,
    pub held_keys: Vec<String>,
    pub held_items: Vec<String>,
    pub gauntlet_barrier_locked: Vec<i32>,
    #[serde(default)]
    pub explored_tiles: Vec<(i32, i32)>,
    #[serde(default)]
    pub seen_entity_kinds: Vec<String>,
    #[serde(default)]
    pub seen_tile_types: Vec<String>,
    #[serde(default)]
    pub known_rule_ids: Vec<String>,
    #[serde(default)]
    pub console_history: Vec<String>,
    #[serde(default)]
    pub barrel_room_protected: bool,
    #[serde(default)]
    pub maze_shifting_walls: Vec<(i32, i32)>,
    #[serde(default)]
    pub maze_shift_frozen: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: usize,
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub hp_current: i32,
    pub hp_max: i32,
    pub alive: bool,
    pub has_enemy_ai: bool,
    pub glyph: char,
    pub sign_message: Option<String>,
    pub fragment_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SavedLogEntry {
    pub text: String,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub has_color: bool,
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn direction_to_string(d: Direction) -> String {
    match d {
        Direction::North => "north".into(),
        Direction::South => "south".into(),
        Direction::West => "west".into(),
        Direction::East => "east".into(),
    }
}

fn direction_from_string(s: &str) -> Direction {
    match s {
        "north" => Direction::North,
        "south" => Direction::South,
        "west" => Direction::West,
        _ => Direction::East,
    }
}

fn kind_to_string(k: EntityKind) -> String {
    k.name().to_string()
}

fn kind_from_string(s: &str) -> EntityKind {
    match s {
        "player" => EntityKind::Player,
        "slime" => EntityKind::Slime,
        "goblin" => EntityKind::Goblin,
        "bat" => EntityKind::Bat,
        "ogre" => EntityKind::Ogre,
        "wizard" => EntityKind::Wizard,
        "barrel" => EntityKind::Barrel,
        "sign" => EntityKind::Sign,
        "memory fragment" => EntityKind::Fragment,
        "shade" => EntityKind::Shade,
        "rage" => EntityKind::Rage,
        "sentry" => EntityKind::Sentry,
        "shade echo" => EntityKind::ShadeEcho,
        "vapor canteen" => EntityKind::VaporCanteen,
        _ => EntityKind::Slime,
    }
}

fn tile_type_to_string(t: crate::map::TileType) -> String {
    match t {
        crate::map::TileType::Floor => "floor".into(),
        crate::map::TileType::Wall => "wall".into(),
        crate::map::TileType::StairsDown => "stairs_down".into(),
        crate::map::TileType::StairsUp => "stairs_up".into(),
        crate::map::TileType::Fire => "fire".into(),
        crate::map::TileType::Lamp => "lamp".into(),
        crate::map::TileType::PressurePlate => "pressure_plate".into(),
    }
}

fn tile_type_from_string(s: &str) -> crate::map::TileType {
    match s {
        "floor" => crate::map::TileType::Floor,
        "wall" => crate::map::TileType::Wall,
        "stairs_down" => crate::map::TileType::StairsDown,
        "stairs_up" => crate::map::TileType::StairsUp,
        "fire" => crate::map::TileType::Fire,
        "lamp" => crate::map::TileType::Lamp,
        "pressure_plate" => crate::map::TileType::PressurePlate,
        _ => crate::map::TileType::Floor,
    }
}

// ---------------------------------------------------------------------------
// World → SaveData
// ---------------------------------------------------------------------------

impl World {
    pub fn to_save_data(&self) -> SaveData {
        let map_tiles: Vec<char> = (0..(self.map.width * self.map.height))
            .map(
                |idx| match self.map.tile(self.map.position_for_idx(idx as usize)) {
                    crate::map::TileType::Floor => 'F',
                    crate::map::TileType::Wall => 'W',
                    crate::map::TileType::StairsDown => 'D',
                    crate::map::TileType::StairsUp => 'U',
                    crate::map::TileType::Fire => '^',
                    crate::map::TileType::Lamp => 'L',
                    crate::map::TileType::PressurePlate => 'P',
                },
            )
            .collect();

        let entities: Vec<EntitySnapshot> = self
            .ecs
            .entity_ids()
            .filter_map(|id| {
                let view = self.ecs.view(id)?;
                let has_enemy_ai = matches!(
                    view.kind,
                    EntityKind::Slime
                        | EntityKind::Goblin
                        | EntityKind::Bat
                        | EntityKind::Ogre
                        | EntityKind::Shade
                        | EntityKind::Rage
                        | EntityKind::Sentry
                );
                Some(EntitySnapshot {
                    id: id.raw(),
                    kind: kind_to_string(view.kind),
                    x: view.pos.x,
                    y: view.pos.y,
                    hp_current: view.hp.current,
                    hp_max: view.hp.max,
                    alive: view.alive,
                    has_enemy_ai,
                    glyph: view.glyph(),
                    sign_message: self.ecs.sign_message(id).map(|s| s.to_string()),
                    fragment_id: self.ecs.fragment_id(id).map(|s| s.to_string()),
                })
            })
            .collect();

        let event_log: Vec<SavedLogEntry> = self
            .event_log
            .entries()
            .iter()
            .map(|entry| {
                let (has_color, r, g, b) = match entry.color {
                    Some(c) => (true, c.r, c.g, c.b),
                    None => (false, 1.0, 1.0, 1.0),
                };
                SavedLogEntry {
                    text: entry.text.clone(),
                    color_r: r,
                    color_g: g,
                    color_b: b,
                    has_color,
                }
            })
            .collect();

        let bindings: Vec<(String, String)> = self
            .bindings
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        SaveData {
            version: 1,
            depth: self.depth,
            turn: self.turn,
            player_id_raw: self.player_id.raw(),
            player_facing: direction_to_string(self.player_facing),
            player_can_attack: self.player_can_attack,
            wizard_taught: self.wizard_taught,
            wizard_id_raw: self.wizard_id.map(|id| id.raw()),
            cheat_unlocked: self.cheat_unlocked,
            blocking: self.blocking,
            map_width: self.map.width,
            map_height: self.map.height,
            map_tiles,
            entities,
            event_log,
            bindings,
            user_source: self.user_source.clone(),
            fragment_registry: self.fragment_registry.clone(),
            ending: self.ending.clone(),
            registry_write_unlocked: self.registry_write_unlocked,
            held_keys: self.held_keys.clone(),
            held_items: self.held_items.clone(),
            gauntlet_barrier_locked: self.gauntlet_barrier_locked.iter().copied().collect(),
            explored_tiles: self.explored_tiles.iter().map(|p| (p.x, p.y)).collect(),
            seen_entity_kinds: self
                .seen_entity_kinds
                .iter()
                .map(|k| kind_to_string(*k))
                .collect(),
            seen_tile_types: self
                .seen_tile_types
                .iter()
                .map(|t| tile_type_to_string(*t))
                .collect(),
            known_rule_ids: self.known_rule_ids.iter().cloned().collect(),
            console_history: self.console_history.clone(),
            barrel_room_protected: self.barrel_room_protected,
            maze_shifting_walls: self.maze_shifting_walls.iter().map(|p| (p.x, p.y)).collect(),
            maze_shift_frozen: self.maze_shift_frozen,
        }
    }
}

// ---------------------------------------------------------------------------
// SaveData → World
// ---------------------------------------------------------------------------

impl World {
    pub fn from_save_data(data: &SaveData) -> Self {
        // Start with a minimal world (fresh envs, no game builtins yet)
        let mut world = World::minimal();

        // --- Map ---
        let mut map = crate::map::Map::new_filled(
            data.map_width,
            data.map_height,
            crate::map::TileType::Floor,
        );
        for (i, ch) in data.map_tiles.iter().enumerate() {
            let tile = match ch {
                'F' => crate::map::TileType::Floor,
                'W' => crate::map::TileType::Wall,
                'D' => crate::map::TileType::StairsDown,
                'U' => crate::map::TileType::StairsUp,
                '^' => crate::map::TileType::Fire,
                'L' => crate::map::TileType::Lamp,
                'P' => crate::map::TileType::PressurePlate,
                _ => crate::map::TileType::Floor,
            };
            let pos = map.position_for_idx(i);
            map.set_tile(pos, tile);
        }
        world.map = map;

        // --- ECS ---
        let mut ecs = crate::ecs::Ecs::new();
        for ent in &data.entities {
            let kind = kind_from_string(&ent.kind);
            let pos = Position::new(ent.x, ent.y);
            let hp = Hp {
                current: ent.hp_current,
                max: ent.hp_max,
            };

            // Allocate at the exact saved ID to avoid gaps causing mismatches
            ecs.set_next_id(ent.id);
            let allocated = match kind {
                EntityKind::Player => ecs.spawn_player(pos),
                EntityKind::Slime => ecs.spawn_slime(pos),
                EntityKind::Goblin => ecs.spawn_goblin(pos),
                EntityKind::Bat => ecs.spawn_bat(pos),
                EntityKind::Ogre => ecs.spawn_ogre(pos),
                EntityKind::Wizard => ecs.spawn_wizard(pos),
                EntityKind::Barrel => ecs.spawn_barrel(pos),
                EntityKind::Shade => ecs.spawn_shade(pos),
                EntityKind::Rage => ecs.spawn_rage(pos),
                EntityKind::Sentry => ecs.spawn_sentry(pos),
                EntityKind::Sign => {
                    let msg = ent.sign_message.as_deref().unwrap_or("");
                    ecs.spawn_sign(pos, msg)
                }
                EntityKind::Fragment => {
                    let frag_id = ent.fragment_id.as_deref().unwrap_or("frag-001");
                    ecs.spawn_fragment(pos, frag_id)
                }
                EntityKind::ShadeEcho => ecs.spawn_shade_echo(pos),
                EntityKind::VaporCanteen => ecs.spawn_vapor_canteen(pos),
            };

            // Overwrite with the saved state
            ecs.set_position(allocated, pos);
            ecs.set_hp(allocated, hp);

            // Handle alive state
            if !ent.alive {
                ecs.damage(allocated, hp.current + 1);
            }
        }

        // Ensure next_id is beyond all loaded entity IDs
        let max_id = data.entities.iter().map(|e| e.id).max().unwrap_or(0);
        ecs.set_next_id(max_id + 1);
        world.ecs = ecs;

        // --- Game state ---
        world.player_id = EntityId::new(data.player_id_raw);
        world.player_facing = direction_from_string(&data.player_facing);
        world.depth = data.depth;
        world.turn = data.turn;
        world.player_can_attack = data.player_can_attack;
        world.wizard_taught = data.wizard_taught;
        world.wizard_id = data.wizard_id_raw.map(EntityId::new);
        world.cheat_unlocked = data.cheat_unlocked;
        world.blocking = data.blocking;

        // --- Event log ---
        {
            use bracket_color::prelude::RGB;
            let mut log = crate::event_log::EventLog::new();
            for entry in &data.event_log {
                let color = if entry.has_color {
                    Some(RGB::from_f32(entry.color_r, entry.color_g, entry.color_b))
                } else {
                    None
                };
                log.push_colored(
                    &entry.text,
                    color.unwrap_or(RGB::named(bracket_color::prelude::WHITE)),
                );
                // Cheat: push_colored always adds color; to push without color we
                // need to work around. Just push with white if no color.
                if !entry.has_color {
                    // Clear the last entry and re-push without color
                }
            }
            // Simplification: just recreate the log from SavedLogEntry data
            let mut log2 = crate::event_log::EventLog::new();
            for entry in &data.event_log {
                if entry.has_color {
                    log2.push_colored(
                        &entry.text,
                        RGB::from_f32(entry.color_r, entry.color_g, entry.color_b),
                    );
                } else {
                    log2.push(&entry.text);
                }
            }
            world.event_log = log2;
        }

        // --- Bindings ---
        world.bindings = data.bindings.iter().cloned().collect();

        // --- User source ---
        world.user_source = data.user_source.clone();

        // --- Fragment registry ---
        world.fragment_registry = data.fragment_registry.clone();
        world.ending = data.ending.clone();
        world.registry_write_unlocked = data.registry_write_unlocked;
        world.held_keys = data.held_keys.clone();
        world.held_items = data.held_items.clone();
        world.gauntlet_barrier_locked = data.gauntlet_barrier_locked.iter().copied().collect();

        // --- Fog of war / discovery state ---
        world.explored_tiles = data
            .explored_tiles
            .iter()
            .map(|(x, y)| Position { x: *x, y: *y })
            .collect();
        world.seen_entity_kinds = data
            .seen_entity_kinds
            .iter()
            .map(|s| kind_from_string(s))
            .collect();
        world.seen_tile_types = data
            .seen_tile_types
            .iter()
            .map(|s| tile_type_from_string(s))
            .collect();
        world.known_rule_ids = data.known_rule_ids.iter().cloned().collect();
        world.console_history = data.console_history.clone();

        // --- Level-specific state ---
        world.barrel_room_protected = data.barrel_room_protected;
        world.maze_shifting_walls = data
            .maze_shifting_walls
            .iter()
            .map(|(x, y)| Position { x: *x, y: *y })
            .collect();
        world.maze_shift_frozen = data.maze_shift_frozen;

        // --- Rebuild Glyph envs on top of minimal env ---
        world.glyph_env = crate::game::setup_glyph_env();
        world.binding_env = crate::game::setup_binding_env(&world.glyph_env);

        // Re-bind do-attack if player learned it (or wizard taught them but
        // save is corrupted — heal the flag).
        if world.player_can_attack || world.wizard_taught {
            crate::game::bind_do_attack(&world.glyph_env);
            world.player_can_attack = true;
        }

        // --- Replay user source ---
        let glyph_env = world.glyph_env.clone();
        for source in &world.user_source.clone() {
            match crate::glyph::read_string(source) {
                Ok(forms) => {
                    for form in &forms {
                        let _ = crate::glyph::eval_with_opts(
                            form,
                            &glyph_env,
                            crate::glyph::SandboxOptions::default(),
                            &mut world,
                        );
                    }
                }
                Err(_) => {}
            }
        }

        world
            .event_log
            .push("Save file detected. Auto-loading. Use (wipe!) to delete the save.");

        world
    }

    // -----------------------------------------------------------------------
    // Disk I/O convenience methods
    // -----------------------------------------------------------------------

    pub fn save_to_disk(&self, slot: u32) -> Result<(), String> {
        let dir = saves_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("cannot create save directory {:?}: {}", dir, e))?;

        let data = self.to_save_data();
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("serialization error: {}", e))?;
        let path = save_path(slot);
        std::fs::write(&path, &json)
            .map_err(|e| format!("cannot write save file {:?}: {}", path, e))?;

        // Also persist player profile (bindings, macros, abilities) separately
        // so it survives restart or carries across save slots.
        let _ = crate::player_profile::PlayerProfile::from_world(self).save();

        Ok(())
    }

    pub fn load_from_disk(slot: u32) -> Result<Self, String> {
        let path = save_path(slot);
        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read save slot {}: {}", slot, e))?;

        let data: SaveData =
            serde_json::from_str(&json).map_err(|e| format!("invalid save data: {}", e))?;

        if data.version != 1 {
            return Err(format!(
                "unsupported save version {}. expected 1.",
                data.version
            ));
        }

        Ok(World::from_save_data(&data))
    }
}
