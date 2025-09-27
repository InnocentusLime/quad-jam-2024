use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Animation {
    pub is_looping: bool,
    pub clips: Vec<Clip>,
}

impl Animation {
    pub fn max_pos(&self) -> u32 {
        self.clips
            .iter()
            .map(|x| x.start + x.len - 1)
            .max()
            .unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clip {
    pub id: u32,
    pub start: u32,
    pub len: u32,
    pub action: ClipAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClipAction {
    DrawSprite {
        layer: u32,
        local_pos: Position,
        local_rotation: f32,
        texture: PathBuf,
        rect: ImgRect,
        origin: Position,
        sort_offset: f32,
    },
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, strum::EnumString, PartialEq, Eq, Hash)]
pub enum AnimationId {
    BunnyIdleD,
    BunnyWalkD,
    BunnyAttackD,
}
