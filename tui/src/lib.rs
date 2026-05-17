//! Library boundary for the Xlyph bracket-lib prototype.
//!
//! This crate keeps the readable game model separate from the bracket-lib
//! executable. The modules are intentionally small: `game` owns simulation,
//! `map` owns terrain/pathing, `input` translates keys into intents, and
//! `render` draws the current state.

pub mod app;
pub mod entity;
pub mod event_log;
pub mod game;
pub mod input;
pub mod map;
pub mod render;
pub mod rules;

pub use entity::{Direction, Entity, EntityKind, Hp, Position};
pub use event_log::EventLog;
pub use game::{ActionCost, Intent, Mode, World};
pub use map::{Map, TileType, MAP_HEIGHT, MAP_WIDTH};
pub use rules::ENEMY_AI_SOURCE;
