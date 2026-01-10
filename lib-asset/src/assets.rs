#[cfg(feature = "dev-env")]
use crate::DevableAsset;
use crate::animation::{AnimationId, AnimationPack};
use crate::level::LevelDef;
use crate::{Asset, FsResolver};
use crate::{GameCfg, asset_roots::*};
use macroquad::prelude::*;
use std::path::Path;
use strum::VariantArray;

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum TextureId {
    BunnyAtlas,
    #[default]
    WorldAtlas,
}

impl Asset for Texture2D {
    type AssetId = TextureId;
    const ROOT: AssetRoot = AssetRoot::Assets;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        let tex = load_texture(&path.to_string_lossy()).await?;
        Ok(tex)
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            TextureId::BunnyAtlas => "bnuuy.png",
            TextureId::WorldAtlas => "world.png",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum FontId {
    Quaver,
}

impl Asset for Font {
    type AssetId = FontId;
    const ROOT: AssetRoot = AssetRoot::Assets;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        let font = load_ttf_font(&path.to_string_lossy()).await?;
        Ok(font)
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            FontId::Quaver => "quaver.ttf",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum AnimationPackId {
    Bunny,
    Stabber,
    Shooter,
}

impl AnimationPackId {
    pub fn animations(self) -> impl Iterator<Item = AnimationId> {
        let pack_name: &'static str = (&self).into();
        AnimationId::VARIANTS
            .iter()
            .copied()
            .filter(move |anim_id| {
                let anim_name: &'static str = anim_id.into();
                anim_name.starts_with(pack_name)
            })
    }
}

#[cfg(feature = "dev-env")]
impl DevableAsset for AnimationPack {
    fn load_dev(resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use crate::animation::{Animation, aseprite_load};
        use log::warn;
        use std::path::PathBuf;

        let mut filename: PathBuf = path.file_name().unwrap().into();
        filename.set_extension("json");

        let project_path = resolver.get_path(AssetRoot::AnimationsProjectRoot, &filename);
        let aseprite_path = resolver.get_path(AssetRoot::AsepriteProjectRoot, &filename);

        match aseprite_load::load_animations_project(&project_path) {
            Ok(x) => return Ok(x),
            Err(e) => warn!("Failed to load anim pack {path:?}: {e:?}"),
        }

        // On native (dev-environment) we load from aseprite and project files.
        // First we try to load the project. If that fails, we try to load aseprite.
        // This way it is faster to iterate on designs.
        match aseprite_load::load_animations_aseprite(resolver, &aseprite_path, None) {
            Ok(x) => return Ok(x),
            Err(e) => warn!("Failed to load aseprite sheet {aseprite_path:?}: {e:?}"),
        }

        warn!("Animationed pack {path:?} will be replaced with a placeholder");
        let pack_id = resolver.inverse_resolve::<Self>(path).unwrap();
        let placeholder = pack_id
            .animations()
            .map(|x| {
                (
                    x,
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
}

impl Asset for AnimationPack {
    type AssetId = AnimationPackId;
    const ROOT: AssetRoot = AssetRoot::Animations;

    #[cfg(feature = "dev-env")]
    async fn load(resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        Self::load_dev(resolver, path)
    }

    #[cfg(not(feature = "dev-env"))]
    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use macroquad::prelude::*;
        let data = load_file(path.to_str().unwrap()).await?;
        postcard::from_bytes(&data).map_err(Into::into)
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            AnimationPackId::Bunny => "bnuuy.bin",
            AnimationPackId::Stabber => "stabber.bin",
            AnimationPackId::Shooter => "shooter.bin",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum LevelId {
    TestRoom,
    TestBulletRoom,
    TestShooterRoom,
}

#[cfg(feature = "dev-env")]
impl DevableAsset for LevelDef {
    fn load_dev(resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use crate::level::tiled_load;
        use std::path::PathBuf;

        let mut filename: PathBuf = path.file_name().unwrap().into();
        filename.set_extension("tmx");
        let tiled_path = resolver.get_path(AssetRoot::TiledProjectRoot, filename);
        tiled_load::load_level(resolver, tiled_path)
    }
}

impl Asset for LevelDef {
    type AssetId = LevelId;
    const ROOT: AssetRoot = AssetRoot::Levels;

    #[cfg(feature = "dev-env")]
    async fn load(resolver: &FsResolver, path: &Path) -> anyhow::Result<LevelDef> {
        Self::load_dev(resolver, path)
    }

    #[cfg(not(feature = "dev-env"))]
    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<LevelDef> {
        use macroquad::prelude::*;
        let data = load_file(path.to_str().unwrap()).await?;
        postcard::from_bytes(&data).map_err(Into::into)
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            LevelId::TestRoom => "test_room.bin",
            LevelId::TestBulletRoom => "test_bullet_room.bin",
            LevelId::TestShooterRoom => "test_shooter_room.bin",
        }
    }
}

#[cfg(feature = "dev-env")]
impl DevableAsset for GameCfg {
    fn load_dev(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use std::fs::File;

        let mut path = path.to_path_buf();
        path.set_extension("json");
        let file = File::open(path)?;
        serde_json::from_reader(&file).map_err(Into::into)
    }
}

impl Asset for GameCfg {
    type AssetId = GameCfgId;
    const ROOT: AssetRoot = AssetRoot::Base;

    #[cfg(feature = "dev-env")]
    async fn load(resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        Self::load_dev(resolver, path)
    }

    #[cfg(not(feature = "dev-env"))]
    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use macroquad::prelude::*;
        let data = load_file(path.to_str().unwrap()).await?;
        postcard::from_bytes(&data).map_err(Into::into)
    }

    fn filename(_id: Self::AssetId) -> &'static str {
        "gamecfg.bin"
    }
}

#[derive(Debug, Clone, Copy, strum::VariantArray, strum::IntoStaticStr, PartialEq, Eq, Hash)]
pub enum GameCfgId {
    Cfg,
}
