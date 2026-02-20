use crate::animation_manifest::{AnimationId, AnimationPack};
use crate::{Asset, FsResolver};
use crate::asset_roots::*;
#[cfg(feature = "dev-env")]
use clap::ValueEnum;
#[cfg(feature = "dev-env")]
use clap::builder::PossibleValue;
use macroquad::prelude::*;
use std::path::Path;
use strum::VariantArray;

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
