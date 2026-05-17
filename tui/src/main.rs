//! Executable entrypoint for the Xlyph bracket-lib prototype.
//!
//! The binary only starts the bracket-lib shell. All interesting code lives in
//! the library modules so the prototype stays easy to inspect and test.

use bracket_lib::prelude::BError;

fn main() -> BError {
    xlyph_tui::app::run()
}
