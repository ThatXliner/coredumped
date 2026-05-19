//! Bracket-lib application shell.
//!
//! This is the only module that talks to the bracket-lib game loop. It receives
//! key events, asks `input` for an intent, applies it to `World`, and delegates
//! all drawing to `render`.

use bracket_lib::prelude::*;

use crate::{game::Intent, input::key_to_intent, render::render, world::World};

const COUNTDOWN_FRAMES: u32 = 30;

pub struct State {
    world: World,
    countdown_frame: u32,
}

impl State {
    pub fn new() -> Self {
        let world = if crate::save::save_path(0).exists() {
            World::load_from_disk(0).unwrap_or_else(|e| {
                eprintln!("Auto-load failed ({}), starting new game.", e);
                let mut w = World::new_game();
                w.event_log
                    .push_colored("Save file corrupted. Starting new game.", RGB::named(RED));
                w
            })
        } else {
            World::new_game()
        };
        Self {
            world,
            countdown_frame: 0,
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

        // Countdown timer (post-wipe). Escape cancels.
        if self.world.quit_countdown > 0 {
            if let Some(key) = ctx.key {
                let intent = key_to_intent(key, ctx.shift, ctx.control, &self.world);
                if matches!(intent, Intent::CloseOverlay) {
                    self.world.quit_countdown = 0;
                    self.countdown_frame = 0;
                    self.world.event_log.push("Countdown cancelled.");
                }
            }
            if self.world.quit_countdown > 0 {
                self.countdown_frame += 1;
                if self.countdown_frame >= COUNTDOWN_FRAMES {
                    self.countdown_frame = 0;
                    self.world.event_log.push_colored(
                        format!("Quitting in {}...", self.world.quit_countdown),
                        RGB::named(RED),
                    );
                    self.world.quit_countdown -= 1;
                }
                if self.world.quit_countdown == 0 {
                    self.world.running = false;
                    ctx.quitting = true;
                    return;
                }
                render(ctx, &self.world);
                return;
            }
        }

        if let Some(key) = ctx.key {
            let intent = key_to_intent(key, ctx.shift, ctx.control, &self.world);
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
    let context = BTermBuilder::simple(90, 50)?
        .with_title("Xlyph - bracket-lib prototype")
        .with_fitscreen(true)
        .with_tile_dimensions(12u32, 12u32)
        .build()?;
    main_loop(context, State::new())
}
