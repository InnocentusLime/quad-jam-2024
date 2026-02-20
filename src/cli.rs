use std::path::PathBuf;

use clap::Parser;
use lib_game::{App, AppState};

pub fn apply_cli(app: &mut App) {
    let args = Args::parse();
    if let Some(level_id) = args.level {
        app.queued_level = Some(level_id);
        app.state = AppState::Active { paused: true };
    }
}

/// CLI tooling for the game.
#[derive(Parser, Debug)]
pub struct Args {
    /// Forces the game to just load a level.
    #[arg(long, value_name = "[LEVEL NAME or LEVEL FILENAME]")]
    pub level: Option<PathBuf>,
}
