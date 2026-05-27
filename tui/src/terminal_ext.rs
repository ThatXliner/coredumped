//! Crossterm-specific Frame extension.

use std::io::Write;

use bracket_color::prelude::RGB;
use coredumped_core::terminal::Frame;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

pub trait FrameExt {
    fn flush(&self, out: &mut impl Write) -> crossterm::Result<()>;
}

impl FrameExt for Frame {
    fn flush(&self, out: &mut impl Write) -> crossterm::Result<()> {
        for (x, y, glyph, fg, bg) in self.cells() {
            if x == 0 {
                queue!(out, MoveTo(0, y as u16))?;
            }
            queue!(
                out,
                SetForegroundColor(to_terminal_color(fg)),
                SetBackgroundColor(to_terminal_color(bg)),
                Print(glyph)
            )?;
        }
        queue!(out, ResetColor)?;
        out.flush()?;
        Ok(())
    }
}

fn to_terminal_color(rgb: RGB) -> Color {
    Color::Rgb {
        r: color_byte(rgb.r),
        g: color_byte(rgb.g),
        b: color_byte(rgb.b),
    }
}

fn color_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
