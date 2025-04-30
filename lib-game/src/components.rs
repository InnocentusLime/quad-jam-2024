use shipyard::Component;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, Component)]
pub struct Transform {
    pub pos: Vec2,
    pub angle: f32,
}