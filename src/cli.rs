use std::error::Error;

use clap::Parser;
use lib_game::{App, AppState, LevelId, resolve_level};
use strum::VariantArray;

pub type ErrBox = Box<dyn Error + Send + Sync>;

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
    #[arg(long, value_name = "[LEVEL NAME or LEVEL FILENAME]", value_parser=parse_level_id)]
    pub level: Option<LevelId>,
}

fn parse_level_id(s: &str) -> Result<LevelId, ErrBox> {
    resolve_level(s).ok_or_else(|| {
        let levels = LevelId::VARIANTS;
        format!("Unknown level name. Known are: {levels:?}").into()
    })
}
