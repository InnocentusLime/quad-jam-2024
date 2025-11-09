mod animation;
#[cfg(not(target_family = "wasm"))]
pub mod aseprite_load;
pub mod binary_io;

pub use animation::*;
use hashbrown::HashMap;
use lib_asset::{FsResolver, declare_assets};

declare_assets!(
    AnimationPackId(animation_pack_filename, animation_pack_path) {
        Bunny("bnuuy.bin"),
    }
);

impl AnimationPackId {
    #[cfg(not(target_family = "wasm"))]
    pub async fn load_animation_pack(
        self,
        resolver: &FsResolver,
    ) -> anyhow::Result<HashMap<AnimationId, Animation>> {
        use log::info;
        use std::path::PathBuf;

        let mut filename = PathBuf::from(self.get_filename());
        filename.set_extension("json");

        let aseprite_path = resolver.aseprite_path(&filename);
        let project_path = resolver.animation_pack_proj_path(&filename);

        // On native (dev-environment) we load from aseprite and project files.
        // First we try to load the project. If that fails, we try to load aseprite.
        // This way it is faster to iterate on designs.
        match aseprite_load::load_animations_project(&project_path) {
            Ok(x) => return Ok(x),
            Err(e) => info!("Failed to load {project_path:?}: {e:?}"),
        }
        aseprite_load::load_animations_aseprite(resolver, aseprite_path)
    }

    #[cfg(target_family = "wasm")]
    pub async fn load_animation_pack(
        self,
        resolver: &FsResolver,
    ) -> anyhow::Result<HashMap<AnimationId, Animation>> {
        use macroquad::prelude::*;
        let path = self.resolve(resolver);
        let data = load_file(path.to_str().unwrap()).await?;
        binary_io::load_from_memory(&data)
    }
}
