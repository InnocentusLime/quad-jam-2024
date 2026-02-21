#[cfg(feature = "dev-env")]
pub mod aseprite_load;

use anyhow::Context;
use glam::{UVec2, Vec2};
use hashbrown::HashMap;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub async fn load_animation_manifest(path: &Path) -> anyhow::Result<AnimationPack> {
    let json = load_string(path.to_str().unwrap())
        .await
        .context("loading JSON")?;
    serde_json::from_str(&json).context("decoding")
}

pub type AnimationPack = HashMap<AnimationId, Animation>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Animation {
    pub is_looping: bool,
    pub action_tracks: HashMap<String, Clips>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clips {
    pub clips: Vec<Clip>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clip {
    pub track_id: u32,
    pub start: u32,
    pub len: u32,
    pub action: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
}

// TODO: macro for generating this id AND mapping from pack to ids
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::VariantArray,
    strum::IntoStaticStr,
    PartialEq,
    Eq,
    Hash,
)]
pub enum AnimationId {
    BnuuyIdleR,
    BnuuyIdleD,
    BnuuyIdleL,
    BnuuyIdleU,
    BnuuyWalkR,
    BnuuyWalkD,
    BnuuyWalkL,
    BnuuyWalkU,
    BnuuyAttackD,
    BnuuyDash,
    StabberIdle,
    StabberAttack,
    ShooterIdle,
    ShooterAttack,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DrawSprite {
    pub layer: u32,
    pub atlas_file: PathBuf,
    pub local_pos: Vec2,
    pub local_rotation: f32,
    pub rect_pos: UVec2,
    pub rect_size: UVec2,
    pub sort_offset: f32,
    pub rotate_with_parent: bool,
}
