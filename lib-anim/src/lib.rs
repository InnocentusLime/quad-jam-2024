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
        Stabber("stabber.bin"),
    }
);

impl AnimationPackId {
    #[cfg(not(target_family = "wasm"))]
    pub async fn load_animation_pack(
        self,
        resolver: &FsResolver,
    ) -> anyhow::Result<HashMap<AnimationId, Animation>> {
        use log::warn;
        use std::path::PathBuf;
        use strum::VariantArray;

        let mut filename = PathBuf::from(self.get_filename());
        filename.set_extension("json");

        let aseprite_path = resolver.aseprite_path(&filename);
        let project_path = resolver.animation_pack_proj_path(&filename);

        match aseprite_load::load_animations_project(&project_path) {
            Ok(x) => return Ok(x),
            Err(e) => warn!("Failed to load anim pack {self:?}: {e:?}"),
        }

        // On native (dev-environment) we load from aseprite and project files.
        // First we try to load the project. If that fails, we try to load aseprite.
        // This way it is faster to iterate on designs.
        match aseprite_load::load_animations_aseprite(resolver, &aseprite_path, None) {
            Ok(x) => return Ok(x),
            Err(e) => warn!("Failed to load aseprite sheet {self:?}: {e:?}"),
        }

        warn!("Animationed pack {self:?} will be replaced with a placeholder");
        let placeholder = AnimationId::VARIANTS
            .into_iter()
            .filter(|x| {
                let anim_name: &'static str = (*x).into();
                let pack_name: &'static str = (&self).into();
                anim_name.starts_with(pack_name)
            })
            .map(|x| {
                (
                    *x,
                    Animation {
                        is_looping: true,
                        clips: vec![],
                        tracks: vec![],
                    },
                )
            })
            .collect();
        Ok(placeholder)
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
