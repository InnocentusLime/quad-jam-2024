mod tiled_decode;
mod tiled_props_des;

use lib_asset::FsResolver;

use crate::level::LevelDef;

use std::path::{Path, PathBuf};

/// Load a level by name but always do it through tiled. For internal
/// use only.
pub fn load_level_by_name(resolver: &FsResolver, name: &str) -> anyhow::Result<LevelDef> {
    let mut path = PathBuf::new();
    path.push("project-tiled");
    path.push(name);
    path.set_extension("tmx");
    load_level(resolver, path)
}

/// Load a level by path through tield. For internal use only.
pub fn load_level(resolver: &FsResolver, path: impl AsRef<Path>) -> anyhow::Result<LevelDef> {
    let mut loader = tiled::Loader::new();
    let map = loader.load_tmx_map(path)?;
    let level = tiled_decode::load_level_from_map(resolver, &map)?;

    Ok(level)
}
