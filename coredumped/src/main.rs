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
}

fn main() -> crossterm::Result<()> {
    let args = Args::parse();
    if let Err(e) = xlyph_tui::diagnostics::init_file_logger() {
        eprintln!("Logging disabled: {e}");
    }
    if args.wipe {
        xlyph_tui::save::wipe();
    }
    xlyph_tui::app::run()
}
