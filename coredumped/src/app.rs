//! Crossterm application shell.
//!
//! This is the only module that talks to the terminal event loop. It receives
//! key events, asks `input` for an intent, applies it to `World`, and delegates
//! all drawing to `render`.

use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

use bracket_color::prelude::{RED, RGB};
use crossterm::{
    cursor::{Hide, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
    },
    execute,
    style::ResetColor,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};

use crate::{
    game::{ActionCost, Intent},
    input::key_to_intent,
    render::render,
    terminal::Frame,
    world::World,
};

const COUNTDOWN_FRAMES: u32 = 30;
const COUNTDOWN_FRAME_TIME: Duration = Duration::from_millis(33);
const IDLE_POLL_TIME: Duration = Duration::from_millis(250);

pub struct State {
    world: World,
    countdown_frame: u32,
    frame: Frame,
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
            frame: Frame::new(90, 50),
        }
    }

    fn tick(&mut self, out: &mut Stdout) -> crossterm::Result<bool> {
        self.sync_terminal_size()?;
        self.world.mark_visible_entities();
        self.world.mark_visible_tiles();
        self.world.refresh_rule_discovery();

        self.frame.clear();
        render(&mut self.frame, &self.world);
        self.frame.flush(out)?;

        if !self.world.running {
            return Ok(false);
        }

        if self.world.quit_countdown > 0 {
            self.handle_countdown_input()?;
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
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        if event::poll(IDLE_POLL_TIME)? {
            self.handle_event(event::read()?)?;
        }

        Ok(self.world.running)
    }

    fn handle_countdown_input(&mut self) -> crossterm::Result<()> {
        if event::poll(COUNTDOWN_FRAME_TIME)? {
            self.handle_event(event::read()?)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> crossterm::Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if self.world.quit_countdown > 0 && key.code == KeyCode::Esc {
                    self.cancel_countdown();
                    return Ok(());
                }

                let intent = key_to_intent(key, &self.world);
                if self.world.quit_countdown > 0 && matches!(intent, Intent::CloseOverlay) {
                    self.cancel_countdown();
                    return Ok(());
                }

                let cost = self.world.apply_intent(intent);
                if cost == ActionCost::Tick && self.world.quit_countdown == 0 {
                    self.countdown_frame = 0;
                }
            }
            Event::Mouse(mouse) => {
                self.frame
                    .set_mouse_pos(mouse.column as i32, mouse.row as i32);
                if matches!(mouse.kind, MouseEventKind::Moved | MouseEventKind::Down(_)) {
                    return Ok(());
                }

                let scroll_delta = match mouse.kind {
                    MouseEventKind::ScrollUp => Some(-3),
                    MouseEventKind::ScrollDown => Some(3),
                    _ => None,
                };
                if let Some(delta) = scroll_delta {
                    self.world.apply_intent(Intent::Scroll(delta));
                }
            }
            Event::Resize(width, height) => {
                self.frame.resize(width as i32, height as i32);
            }
            _ => {}
        }
        Ok(())
    }

    fn sync_terminal_size(&mut self) -> crossterm::Result<()> {
        let (width, height) = terminal::size()?;
        self.frame.resize(width as i32, height as i32);
        Ok(())
    }

    fn cancel_countdown(&mut self) {
        self.world.quit_countdown = 0;
        self.countdown_frame = 0;
        self.world.event_log.push("Countdown cancelled.");
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run() -> crossterm::Result<()> {
    let mut terminal = TerminalSession::enter()?;
    let mut state = State::new();

    loop {
        if !state.tick(&mut terminal.stdout)? {
            break;
        }
    }

    Ok(())
}

struct TerminalSession {
    stdout: Stdout,
}

impl TerminalSession {
    fn enter() -> crossterm::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            SetTitle("Xlyph"),
            Clear(ClearType::All),
            Hide
        )?;
        stdout.flush()?;
        Ok(Self { stdout })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(
            self.stdout,
            Show,
            ResetColor,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
        let _ = self.stdout.flush();
    }
}
