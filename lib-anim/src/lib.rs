mod animation;
#[cfg(not(target_family = "wasm"))]
pub mod aseprite_load;
pub mod binary_io;

pub use animation::*;
use hashbrown::HashMap;

/// Loads an animation pack by name. This is the public API for use inside the
/// game.
pub async fn load_animation_pack(name: &str) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    #[cfg(not(target_family = "wasm"))]
    return aseprite_load::load_animations_by_name(name);
    #[cfg(target_family = "wasm")]
    return load_animation_pack_release(name).await;
}

#[cfg(target_family = "wasm")]
async fn load_animation_pack_release(
    name: &str,
) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    use macroquad::prelude::*;
    let data = load_file(&format!("animations/{name}.bin")).await?;
    binary_io::load_from_memory(&data).map_err(Into::into)
}
