use shipyard::Component;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, Component)]
pub struct Transform {
    pub pos: Vec2,
    pub angle: f32,
}

impl Transform {
    pub fn from_pos(pos: Vec2) -> Self {
        Self {
            pos,
            angle: 0.0,
        }
    }

    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::from_pos(vec2(x, y))
    }
}