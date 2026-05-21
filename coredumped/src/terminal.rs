use std::{env, io::Write};

use bracket_color::prelude::{BLACK, RGB, WHITE};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use terminal_supports_emoji::{supports_emoji, Stream};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum CellGlyph {
    Char(char),
    Text(&'static str, u8),
    Skip,
}

#[derive(Clone, Copy)]
struct Cell {
    glyph: CellGlyph,
    fg: RGB,
    bg: RGB,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            glyph: CellGlyph::Char(' '),
            fg: RGB::named(WHITE),
            bg: RGB::named(BLACK),
        }
    }
}

pub struct Frame {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    mouse_pos: (i32, i32),
    emoji_enabled: bool,
}

impl Frame {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); (width * height) as usize],
            mouse_pos: (0, 0),
            emoji_enabled: false,
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;
        self.cells = vec![Cell::default(); (width * height) as usize];
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn set_mouse_pos(&mut self, x: i32, y: i32) {
        self.mouse_pos = (x, y);
    }

    pub fn mouse_pos(&self) -> (i32, i32) {
        self.mouse_pos
    }

    pub fn set_emoji_enabled(&mut self, enabled: bool) {
        self.emoji_enabled = enabled;
    }

    pub fn emoji_enabled(&self) -> bool {
        self.emoji_enabled
    }

    pub fn set(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, glyph: char) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        self.clear_existing_glyph_at(x, y);
        let index = (y * self.width + x) as usize;
        self.cells[index] = Cell {
            glyph: CellGlyph::Char(glyph),
            fg,
            bg,
        };
    }

    pub fn set_text(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, text: &'static str) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }

        let width = UnicodeWidthStr::width(text).max(1) as i32;
        if x + width > self.width {
            return false;
        }

        for dx in 0..width {
            self.clear_existing_glyph_at(x + dx, y);
        }

        let index = (y * self.width + x) as usize;
        self.cells[index] = Cell {
            glyph: CellGlyph::Text(text, width as u8),
            fg,
            bg,
        };
        for dx in 1..width {
            let trailing = (y * self.width + x + dx) as usize;
            self.cells[trailing] = Cell {
                glyph: CellGlyph::Skip,
                fg,
                bg,
            };
        }
        true
    }

    pub fn print_color(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, text: &str) {
        for (offset, glyph) in text.chars().enumerate() {
            self.set(x + offset as i32, y, fg, bg, glyph);
        }
    }

    pub fn flush(&self, out: &mut impl Write) -> crossterm::Result<()> {
        for y in 0..self.height {
            queue!(out, MoveTo(0, y as u16))?;
            for x in 0..self.width {
                let cell = self.cells[(y * self.width + x) as usize];
                if matches!(cell.glyph, CellGlyph::Skip) {
                    continue;
                }
                queue!(
                    out,
                    SetForegroundColor(to_terminal_color(cell.fg)),
                    SetBackgroundColor(to_terminal_color(cell.bg)),
                )?;
                match cell.glyph {
                    CellGlyph::Char(glyph) => queue!(out, Print(glyph))?,
                    CellGlyph::Text(text, _) => queue!(out, Print(text))?,
                    CellGlyph::Skip => {}
                }
            }
        }
        queue!(out, ResetColor)?;
        out.flush()?;
        Ok(())
    }

    fn clear_existing_glyph_at(&mut self, x: i32, y: i32) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }

        let index = (y * self.width + x) as usize;
        match self.cells[index].glyph {
            CellGlyph::Skip => {
                let mut leader_x = x - 1;
                while leader_x >= 0 {
                    let leader_index = (y * self.width + leader_x) as usize;
                    match self.cells[leader_index].glyph {
                        CellGlyph::Text(_, width) if leader_x + width as i32 > x => {
                            self.clear_text_span(leader_x, y, width);
                            return;
                        }
                        CellGlyph::Skip => leader_x -= 1,
                        _ => break,
                    }
                }
                self.cells[index] = Cell::default();
            }
            CellGlyph::Text(_, width) => self.clear_text_span(x, y, width),
            CellGlyph::Char(_) => self.cells[index] = Cell::default(),
        }
    }

    fn clear_text_span(&mut self, x: i32, y: i32, width: u8) {
        for dx in 0..width as i32 {
            let cell_x = x + dx;
            if cell_x >= 0 && cell_x < self.width && y >= 0 && y < self.height {
                let index = (y * self.width + cell_x) as usize;
                self.cells[index] = Cell::default();
            }
        }
    }
}

pub fn detect_emoji_support() -> bool {
    if let Ok(flag) = env::var("XYLPH_EMOJI") {
        if let Some(enabled) = env_flag_override(&flag) {
            return enabled;
        }
    }

    supports_emoji(Stream::Stdout)
}

fn env_flag_override(flag: &str) -> Option<bool> {
    match flag.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "force" => Some(true),
        "0" | "false" | "no" | "off" | "never" => Some(false),
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_override_parses_common_flags() {
        assert_eq!(env_flag_override("1"), Some(true));
        assert_eq!(env_flag_override("true"), Some(true));
        assert_eq!(env_flag_override("force"), Some(true));
        assert_eq!(env_flag_override("0"), Some(false));
        assert_eq!(env_flag_override("off"), Some(false));
        assert_eq!(env_flag_override("maybe"), None);
    }

    #[test]
    fn wide_text_marks_trailing_cell_as_skip() {
        let mut frame = Frame::new(4, 1);

        assert!(frame.set_text(1, 0, RGB::named(WHITE), RGB::named(BLACK), "🧙"));

        match frame.cells[1].glyph {
            CellGlyph::Text(text, width) => {
                assert_eq!(text, "🧙");
                assert_eq!(width, 2);
            }
            _ => panic!("wide glyph should occupy the leading cell"),
        }
        assert!(matches!(frame.cells[2].glyph, CellGlyph::Skip));

        frame.set(2, 0, RGB::named(WHITE), RGB::named(BLACK), 'x');

        assert!(matches!(frame.cells[1].glyph, CellGlyph::Char(' ')));
        assert!(matches!(frame.cells[2].glyph, CellGlyph::Char('x')));
    }
}
