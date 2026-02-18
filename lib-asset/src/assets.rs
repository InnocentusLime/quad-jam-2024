#[cfg(feature = "dev-env")]
use crate::DevableAsset;
use crate::animation_manifest::{AnimationId, AnimationPack};
use crate::level::LevelDef;
use crate::{Asset, FsResolver};
use crate::{GameCfg, asset_roots::*};
#[cfg(feature = "dev-env")]
use clap::ValueEnum;
#[cfg(feature = "dev-env")]
use clap::builder::PossibleValue;
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
impl ValueEnum for AnimationPackId {
    fn value_variants<'a>() -> &'a [Self] {
        AnimationPackId::VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let s: &str = self.into();
        Some(PossibleValue::new(s))
    }
}

impl Asset for AnimationPack {
    type AssetId = AnimationPackId;
    const ROOT: AssetRoot = AssetRoot::Assets;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        use macroquad::prelude::*;
        let json = load_string(path.to_str().unwrap())
            .await
            .context("loading JSON")?;
        serde_json::from_str(&json).context("decoding")
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            AnimationPackId::Bunny => "bnuuy.json",
            AnimationPackId::Stabber => "stabber.json",
            AnimationPackId::Shooter => "shooter.json",
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
    const ROOT: AssetRoot = AssetRoot::Assets;

    #[cfg(feature = "dev-env")]
    async fn load(resolver: &FsResolver, path: &Path) -> anyhow::Result<LevelDef> {
        Self::load_dev(resolver, path)
    }

    #[cfg(not(feature = "dev-env"))]
    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<LevelDef> {
        use anyhow::Context;
        use macroquad::prelude::*;
        let json = load_string(path.to_str().unwrap())
            .await
            .context("loading JSON")?;
        serde_json::from_str(&json).context("decoding")
    }

    fn filename(id: Self::AssetId) -> &'static str {
        match id {
            LevelId::TestRoom => "test_room.json",
            LevelId::TestBulletRoom => "test_bullet_room.json",
            LevelId::TestShooterRoom => "test_shooter_room.json",
        }
    }
}

impl Asset for GameCfg {
    type AssetId = GameCfgId;
    const ROOT: AssetRoot = AssetRoot::Base;

    async fn load(_resolver: &FsResolver, path: &Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        use macroquad::prelude::*;
        let json = load_string(path.to_str().unwrap())
            .await
            .context("loading JSON")?;
        serde_json::from_str(&json).context("decoding")
    }

    fn filename(_id: Self::AssetId) -> &'static str {
        "gamecfg.json"
    }
}

#[derive(Debug, Clone, Copy, strum::VariantArray, strum::IntoStaticStr, PartialEq, Eq, Hash)]
pub enum GameCfgId {
    Cfg,
}
