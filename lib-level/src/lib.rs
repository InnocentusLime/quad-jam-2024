pub mod binary_io;
mod level;
#[cfg(not(target_family = "wasm"))]
pub mod tiled_load;

pub use level::*;
use lib_asset::FsResolver;

/// Loads a level by name. This is the public API for use inside the
/// game.
pub async fn load_level(resolver: &FsResolver, name: &str) -> anyhow::Result<LevelDef> {
    #[cfg(not(target_family = "wasm"))]
    return tiled_load::load_level_by_name(resolver, name);
    #[cfg(target_family = "wasm")]
    return load_level_release(resolver, name).await;
}

#[cfg(target_family = "wasm")]
async fn load_level_release(_resolver: &FsResolver, name: &str) -> anyhow::Result<LevelDef> {
    use macroquad::prelude::*;
    let data = load_file(&format!("levels/{name}.bin")).await?;
    binary_io::load_from_memory(&data).map_err(Into::into)
}
