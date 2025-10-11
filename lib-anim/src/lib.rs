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
        // On native (dev-environment) we load from aseprit
        use std::path::PathBuf;

        let mut filename = PathBuf::from(self.get_filename());
        filename.set_extension("json");
        aseprite_load::load_animations(resolver, resolver.aseprite_path(filename))
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
