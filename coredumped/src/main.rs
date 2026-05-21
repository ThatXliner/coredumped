//! Executable entrypoint for the Xlyph terminal prototype.
//!
//! The binary only starts the crossterm shell. All interesting code lives in
//! the library modules so the prototype stays easy to inspect and test.

use clap::Parser;

#[derive(Parser)]
#[command(name = "xlyph", about = "A text-graphical roguelike")]
struct Args {
    /// Delete the auto-save (slot 0) and player profile before starting
    #[arg(long)]
    wipe: bool,

    /// Force ASCII entity glyphs even when emoji rendering is available
    #[arg(long, visible_alias = "ascii")]
    ascii_only: bool,
}

fn main() -> crossterm::Result<()> {
    let args = Args::parse();
    if args.wipe {
        xlyph_tui::save::wipe();
    }
    xlyph_tui::app::run_with_options(xlyph_tui::app::RunOptions {
        ascii_only: args.ascii_only,
    })
}
