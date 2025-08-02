mod level;
mod tiled_decode;
mod tiled_props_des;

pub use level::*;
use thiserror::Error;

use std::error::Error as StdError;

pub fn load_level(name: &str) -> Result<LevelDef, LoadLevelError> {
    load_level_from_tiled(name)
}

fn load_level_from_tiled(name: &str) -> Result<LevelDef, LoadLevelError> {
    let path = format!("./tiled_project/{name}.tmx");
    let mut loader = tiled::Loader::new();
    let map = loader
        .load_tmx_map(path)
        .map_err(|e| Box::new(e) as Box<dyn StdError>)
        .map_err(LoadLevelError::Loading)?;
    let level = tiled_decode::load_level_from_map(&map)
        .map_err(|e| Box::new(e) as Box<dyn StdError>)
        .map_err(LoadLevelError::Decoding)?;

    Ok(level)
}

#[derive(Debug, Error)]
pub enum LoadLevelError {
    #[error("Failed to load the level")]
    Loading(#[source] Box<dyn StdError>),
    #[error("Failed to decode the level")]
    Decoding(#[source] Box<dyn StdError>),
}
