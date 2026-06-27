//! Crossterm-specific Frame extension.

use std::io::Write;

use bracket_color::prelude::RGB;
use coredumped_core::terminal::Frame;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
};

pub trait FrameExt {
    fn flush(&self, out: &mut impl Write) -> crossterm::Result<()>;
}

impl FrameExt for Frame {
    fn flush(&self, out: &mut impl Write) -> crossterm::Result<()> {
        let mut italic_on = false;
        for (x, y, glyph, fg, bg, italic) in self.cells() {
            if x == 0 {
                queue!(out, MoveTo(0, y as u16))?;
            }
            if italic != italic_on {
                queue!(
                    out,
                    SetAttribute(if italic {
                        Attribute::Italic
                    } else {
                        Attribute::NoItalic
                    })
                )?;
                italic_on = italic;
            }
            queue!(
                out,
                SetForegroundColor(to_terminal_color(fg)),
                SetBackgroundColor(to_terminal_color(bg)),
                Print(glyph)
            )?;
        }
        if italic_on {
            queue!(out, SetAttribute(Attribute::NoItalic))?;
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
