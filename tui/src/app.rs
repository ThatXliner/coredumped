//! Bracket-lib application shell.
//!
//! This is the only module that talks to the bracket-lib game loop. It receives
//! key events, asks `input` for an intent, applies it to `World`, and delegates
//! all drawing to `render`.

use bracket_lib::prelude::*;

use crate::{game::World, input::key_to_intent, render::render};

pub struct State {
    world: World,
}

impl State {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        if let Some(key) = ctx.key {
            let intent = key_to_intent(key, &self.world);
            self.world.apply_intent(intent);
        }

        if !self.world.running {
            ctx.quitting = true;
            return;
        }

        render(ctx, &self.world);
    }
}

pub fn run() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Xlyph - bracket-lib prototype")
        .with_fitscreen(true)
        .with_automatic_console_resize(true)
        .build()?;
    main_loop(context, State::new())
}
