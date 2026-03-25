use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub layer: u32,
    pub texture: AssetKey,
    pub tex_rect_pos: UVec2,
    pub tex_rect_size: UVec2,
    pub color: Color,
    pub sort_offset: f32,
    pub local_offset: Vec2,
}
