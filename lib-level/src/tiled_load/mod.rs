mod tiled_decode;
mod tiled_props_des;

use crate::level::LevelDef;

use std::path::Path;

/// Load a level by name but always do it through tiled. For internal
/// use only.
pub fn load_level_by_name(name: &str) -> anyhow::Result<LevelDef> {
    let path = format!("./tiled-project/{name}.tmx");
    load_level("./assets", path)
}

/// Load a level by path through tield. For internal use only.
pub fn load_level(
    assets_directory: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> anyhow::Result<LevelDef> {
    let mut loader = tiled::Loader::new();
    let map = loader.load_tmx_map(path)?;
    let level = tiled_decode::load_level_from_map(assets_directory, &map)?;

    Ok(level)
}
