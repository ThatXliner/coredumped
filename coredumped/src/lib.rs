//! Library boundary for the Xlyph terminal prototype.
//!
//! This crate keeps the readable game model separate from the crossterm
//! executable. The modules are intentionally small: `game` owns simulation,
//! `map` owns terrain/pathing, `input` translates keys into intents, and
//! `render` draws the current state.

pub(crate) mod ai_builtins;
pub mod app;
pub mod diagnostics;
pub mod ecs;
pub mod entity;
pub mod event_log;
pub mod fragment;
pub mod game;
pub mod glyph;
pub mod input;
pub(crate) mod levels;
pub mod map;
pub mod no_hit;
pub mod playbook;
pub(crate) mod player_profile;
pub mod render;
pub mod rules;
pub mod save;
pub mod terminal;
pub mod world;

pub use ecs::Ecs;
pub use entity::{Direction, EntityId, EntityKind, EntityView, Hp, Position, RenderGlyph};
pub use event_log::EventLog;
pub use game::{ActionCost, Intent, Mode};
pub use map::{Map, MapGenOutput, TileType, MAP_HEIGHT, MAP_WIDTH};
pub use no_hit::{detect_no_hit_route, NoHitAction, NoHitAnalysis, NoHitOptions};
pub use rules::{Rule, RuleCost, RulePhase, RuleRegistry};
pub use world::World;
