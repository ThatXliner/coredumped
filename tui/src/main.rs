//! Executable entrypoint for the Xlyph bracket-lib prototype.
//!
//! The binary only starts the bracket-lib shell. All interesting code lives in
//! the library modules so the prototype stays easy to inspect and test.

use bracket_lib::prelude::BError;
use clap::Parser;

#[derive(Parser)]
#[command(name = "xlyph", about = "A text-graphical roguelike")]
struct Args {
    /// Delete the auto-save (slot 0) and player profile before starting
    #[arg(long)]
    wipe: bool,
}

fn main() -> BError {
    let args = Args::parse();
    if args.wipe {
        xlyph_tui::save::wipe();
    }
    xlyph_tui::app::run()
}
