use std::path::{Path, PathBuf};

use lib_game::{AssetKey, Resources};

use serde::Deserialize;

pub fn register_components(prefab_factory: &mut lib_game::PrefabFactory<Resources>) {
    prefab_factory.register_component_with_constructor_ctx(
        "player",
        PlayerTagManifest::into_tag,
        PlayerTagManifest::dependencies,
    );
    prefab_factory.register_component::<BulletTag>("bullet");
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerTag {
    pub bullet_prefab: AssetKey,
}

#[derive(Debug, Deserialize)]
pub struct PlayerTagManifest {
    pub bullet_prefab: PathBuf,
}

impl PlayerTagManifest {
    pub fn into_tag(self, resources: &mut Resources) -> anyhow::Result<PlayerTag> {
        let Some(bullet_prefab) = resources.prefabs.resolve(&self.bullet_prefab) else {
            anyhow::bail!("No such prefab: {:?}", self.bullet_prefab);
        };
        Ok(PlayerTag { bullet_prefab })
    }

    pub fn dependencies(data: &serde_json::value::RawValue) -> anyhow::Result<Vec<PathBuf>> {
        #[derive(Deserialize)]
        pub struct Deps<'a> {
            #[serde(borrow)]
            pub bullet_prefab: &'a Path,
        }
        let deps = serde_json::from_str::<Deps>(data.get())?;
        Ok(vec![deps.bullet_prefab.into()])
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BulletTag;
