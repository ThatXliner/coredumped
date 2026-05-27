use bracket_color::prelude::{BLACK, RGB, WHITE};

#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

#[cfg(not(target_arch = "wasm32"))]
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

#[derive(Clone, Copy)]
struct Cell {
    glyph: char,
    fg: RGB,
    bg: RGB,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            glyph: ' ',
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
}

impl Frame {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); (width * height) as usize],
            mouse_pos: (0, 0),
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

    pub fn set(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, glyph: char) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let index = (y * self.width + x) as usize;
        self.cells[index] = Cell { glyph, fg, bg };
    }

    pub fn print_color(&mut self, x: i32, y: i32, fg: RGB, bg: RGB, text: &str) {
        for (offset, glyph) in text.chars().enumerate() {
            self.set(x + offset as i32, y, fg, bg, glyph);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn flush(&self, out: &mut impl Write) -> crossterm::Result<()> {
        for y in 0..self.height {
            queue!(out, MoveTo(0, y as u16))?;
            for x in 0..self.width {
                let cell = self.cells[(y * self.width + x) as usize];
                queue!(
                    out,
                    SetForegroundColor(to_terminal_color(cell.fg)),
                    SetBackgroundColor(to_terminal_color(cell.bg)),
                    Print(cell.glyph)
                )?;
            }
        }
        queue!(out, ResetColor)?;
        out.flush()?;
        Ok(())
    }

    pub fn to_ansi_string(&self) -> String {
        let mut output = String::with_capacity((self.width * self.height * 30) as usize);
        output.push_str("\x1b[H");

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.cells[(y * self.width + x) as usize];
                let fr = color_byte(cell.fg.r);
                let fg = color_byte(cell.fg.g);
                let fb = color_byte(cell.fg.b);
                let br = color_byte(cell.bg.r);
                let bg = color_byte(cell.bg.g);
                let bb = color_byte(cell.bg.b);
                output.push_str(&format!(
                    "\x1b[38;2;{};{};{};48;2;{};{};{}m{}",
                    fr, fg, fb, br, bg, bb, cell.glyph
                ));
            }
            if y < self.height - 1 {
                output.push_str("\r\n");
            }
        }
        output.push_str("\x1b[0m");
        output
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
