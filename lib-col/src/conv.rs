//! Some code uses a slightly different coordinate system.
//! In that system, the Y axis is flipped and is actually pointing down.
//! This module provides functions to quickly convert a transform from such
//! system into the crate's one.

use glam::{Affine2, Vec2, vec2};

pub fn topleft_corner_vector_to_crate(v: Vec2) -> Vec2 {
    vec2(v.x, -v.y)
}

pub fn crate_vector_to_topleft_corner(v: Vec2) -> Vec2 {
    vec2(v.x, -v.y)
}

pub fn topleft_corner_tf_to_crate(pos: Vec2, angle: f32) -> Affine2 {
    let pos = topleft_corner_vector_to_crate(pos);
    let angle = std::f32::consts::PI - angle;
    Affine2::from_angle_translation(angle, pos)
}
