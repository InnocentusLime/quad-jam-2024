use macroquad::prelude::*;
use shipyard::Component;

#[derive(Debug, Clone, Copy, Component)]
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
