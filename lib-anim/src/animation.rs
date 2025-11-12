use lib_asset::TextureId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Animation {
    pub is_looping: bool,
    pub clips: Vec<Clip>,
    pub tracks: Vec<Track>,
}

impl Animation {
    pub fn max_pos(&self) -> u32 {
        self.clips
            .iter()
            .map(|x| x.start + x.len - 1)
            .max()
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Clip {
    pub id: u32,
    pub track_id: u32,
    pub start: u32,
    pub len: u32,
    pub action: ClipAction,
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, strum::IntoStaticStr, strum::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum::IntoStaticStr, strum::VariantArray))]
pub enum ClipAction {
    DrawSprite {
        layer: u32,
        texture_id: TextureId,
        local_pos: Position,
        local_rotation: f32,
        rect: ImgRect,
        origin: Position,
        sort_offset: f32,
    },
    AttackBox {
        local_pos: Position,
        local_rotation: f32,
        team: Team,
        group: lib_col::Group,
        shape: lib_col::Shape,
        rotate_with_parent: bool,
    },
    Invulnerability,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub id: u32,
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ImgRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

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
pub enum Team {
    Player,
    Enemy,
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
    BunnyIdleD,
    BunnyWalkD,
    BunnyAttackD,
    BunnyDash,
}
