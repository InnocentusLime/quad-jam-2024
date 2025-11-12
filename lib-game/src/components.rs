use hecs::Entity;
use lib_anim::{Animation, AnimationId};
use macroquad::prelude::*;

/// [Health] component stores entity's health.
/// Normally, to do damage, you should just put it into the `damage` field.
/// `damage` is zeroed every frame and is substracted to `value`.
/// When the `block_damage` flag is raised, `damage` is ignored this frame.
#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub value: i32,
    pub damage: i32,
    pub block_damage: bool,
}

impl Health {
    pub fn new(value: i32) -> Self {
        Self {
            value,
            damage: 0,
            block_damage: false,
        }
    }
}

/// [DamageCooldown] enables cooldown on damage.
/// When [Health] contains more than zero damage and the entity
/// has [DamageCooldown] component, the game will raise the `block_damage`
/// flag. It will remain raised for the duration of `max_value`.
/// `remaining` is used to track the remaining invulnerability time.
#[derive(Debug, Clone, Copy)]
pub struct DamageCooldown {
    pub remaining: f32,
    pub max_value: f32,
}

impl DamageCooldown {
    pub fn new(max_value: f32) -> Self {
        Self {
            max_value,
            remaining: 0.0,
        }
    }
}

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
    pub pause: bool,
}

impl AnimationPlay {
    pub fn is_done(&self, animation: &Animation) -> bool {
        if animation.is_looping {
            return false;
        }
        self.cursor == animation.max_pos()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipActionObject {
    pub parent: Entity,
    pub animation: AnimationId,
    pub clip_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    Player,
    Enemy,
}

#[derive(Debug, Clone, Copy)]
pub struct CharacterLook(pub f32);
