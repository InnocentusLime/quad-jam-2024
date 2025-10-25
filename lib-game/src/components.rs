use lib_anim::{Animation, AnimationId};
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub pos: Vec2,
    pub angle: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        pos: Vec2::ZERO,
        angle: 0.0,
    };

    pub fn from_pos(pos: Vec2) -> Self {
        Self { pos, angle: 0.0 }
    }

    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::from_pos(vec2(x, y))
    }
}

pub struct AnimationPlay {
    pub animation: AnimationId,
    pub total_dt: f32,
    pub cursor: u32,
}

impl AnimationPlay {
    pub fn is_done(&self, animation: &Animation) -> bool {
        if animation.is_looping {
            return false;
        }
        self.cursor == animation.max_pos()
    }
}
