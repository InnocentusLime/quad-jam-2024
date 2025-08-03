pub mod binary_io;
mod level;
#[cfg(not(target_family = "wasm"))]
pub mod tiled_load;

pub use level::*;
use thiserror::Error;

use std::error::Error as StdError;

pub async fn load_level(name: &str) -> Result<LevelDef, LoadLevelError> {
    #[cfg(not(target_family = "wasm"))]
    return tiled_load::load_level_by_name(name);
    #[cfg(target_family = "wasm")]
    return load_level_release(name).await;
}

#[cfg(target_family = "wasm")]
async fn load_level_release(name: &str) -> Result<LevelDef, LoadLevelError> {
    use macroquad::prelude::*;
    let data = load_file(&format!("levels/{name}.bin"))
        .await
        .map_err(|e| Box::new(e) as Box<dyn StdError>)
        .map_err(LoadLevelError::Loading)?;
    binary_io::load_from_memory(&data)
}

#[derive(Debug, Error)]
pub enum LoadLevelError {
    #[error("Failed to load the level")]
    Loading(#[source] Box<dyn StdError>),
    #[error("Failed to decode the level")]
    Decoding(#[source] Box<dyn StdError>),
}
