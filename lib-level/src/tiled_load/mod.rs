mod tiled_decode;
mod tiled_props_des;

use crate::LoadLevelError;
use crate::level::LevelDef;

use std::error::Error as StdError;
use std::path::Path;

/// Load a level by name but always do it through tiled. For internal
/// use only.
pub fn load_level_by_name(name: &str) -> Result<LevelDef, LoadLevelError> {
    let path = format!("./tiled_project/{name}.tmx");
    load_level(path)
}

/// Load a level by path through tield. For internal use only.
pub fn load_level(path: impl AsRef<Path>) -> Result<LevelDef, LoadLevelError> {
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
