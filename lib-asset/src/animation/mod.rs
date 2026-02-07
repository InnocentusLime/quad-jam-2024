#[cfg(feature = "dev-env")]
pub mod aseprite_load;

use crate::TextureId;
use crate::level::CharacterInfo;
use glam::{UVec2, Vec2};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

pub type AnimationPack = HashMap<AnimationId, Animation>;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
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

    pub fn active_draw_sprite(
        &self,
        pos: u32,
    ) -> impl Iterator<Item = (u32, Clip<ClipActionDrawSprite>)> {
        self.active_clips(pos).filter_map(Clip::to_draw_sprite)
    }

    pub fn active_attack_box(
        &self,
        pos: u32,
    ) -> impl Iterator<Item = (u32, Clip<ClipActionAttackBox>)> {
        self.active_clips(pos).filter_map(Clip::to_attack_box)
    }

    pub fn active_lock_input(
        &self,
        pos: u32,
    ) -> impl Iterator<Item = (u32, Clip<ClipActionLockInput>)> {
        self.active_clips(pos).filter_map(Clip::to_lock_input)
    }

    pub fn active_invulnerability(
        &self,
        pos: u32,
    ) -> impl Iterator<Item = (u32, Clip<ClipActionInvulnerability>)> {
        self.active_clips(pos).filter_map(Clip::to_invulnerability)
    }

    pub fn active_move(&self, pos: u32) -> impl Iterator<Item = (u32, Clip<ClipActionMove>)> {
        self.active_clips(pos).filter_map(Clip::to_move)
    }

    pub fn active_spawn(&self, pos: u32) -> impl Iterator<Item = (u32, Clip<ClipActionSpawn>)> {
        self.active_clips(pos).filter_map(Clip::to_spawn)
    }

    pub fn active_clips(&self, pos: u32) -> impl Iterator<Item = (u32, Clip)> {
        self.clips
            .iter()
            .copied()
            .enumerate()
            .map(|(x, y)| (x as u32, y))
            .filter(move |(_, x)| x.contains_pos(pos))
    }

    pub fn inactive_clips(&self, pos: u32) -> impl Iterator<Item = (u32, Clip)> {
        self.clips
            .iter()
            .copied()
            .enumerate()
            .map(|(x, y)| (x as u32, y))
            .filter(move |(_, x)| !x.contains_pos(pos))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Clip<Act = ClipAction> {
    pub track_id: u32,
    pub start: u32,
    pub len: u32,
    #[serde(flatten)]
    pub action: Act,
}

impl Clip {
    pub fn contains_pos(&self, pos: u32) -> bool {
        self.start <= pos && pos < self.end()
    }

    pub fn end(&self) -> u32 {
        self.start + self.len
    }

    pub fn to_draw_sprite((idx, clip): (u32, Self)) -> Option<(u32, Clip<ClipActionDrawSprite>)> {
        let ClipAction::DrawSprite(draw) = clip.action else {
            return None;
        };
        Some((idx, clip.replace_action(draw)))
    }

    pub fn to_attack_box((idx, clip): (u32, Self)) -> Option<(u32, Clip<ClipActionAttackBox>)> {
        let ClipAction::AttackBox(atk) = clip.action else {
            return None;
        };
        Some((idx, clip.replace_action(atk)))
    }

    pub fn to_lock_input((idx, clip): (u32, Self)) -> Option<(u32, Clip<ClipActionLockInput>)> {
        let ClipAction::LockInput(lock) = clip.action else {
            return None;
        };
        Some((idx, clip.replace_action(lock)))
    }

    pub fn to_invulnerability(
        (idx, clip): (u32, Self),
    ) -> Option<(u32, Clip<ClipActionInvulnerability>)> {
        let ClipAction::Invulnerability(invuln) = clip.action else {
            return None;
        };
        Some((idx, clip.replace_action(invuln)))
    }

    pub fn to_move((idx, clip): (u32, Self)) -> Option<(u32, Clip<ClipActionMove>)> {
        let ClipAction::Move(mov) = clip.action else {
            return None;
        };
        Some((idx, clip.replace_action(mov)))
    }

    pub fn to_spawn((idx, clip): (u32, Self)) -> Option<(u32, Clip<ClipActionSpawn>)> {
        let ClipAction::Spawn(spawn) = clip.action else {
            return None;
        };
        Some((idx, clip.replace_action(spawn)))
    }

    fn replace_action<T>(self, action: T) -> Clip<T> {
        Clip {
            track_id: self.track_id,
            start: self.start,
            len: self.len,
            action,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, strum::IntoStaticStr, strum::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum::IntoStaticStr, strum::VariantArray))]
#[serde(rename_all = "snake_case")]
pub enum ClipAction {
    DrawSprite(ClipActionDrawSprite),
    AttackBox(ClipActionAttackBox),
    Invulnerability(ClipActionInvulnerability),
    LockInput(ClipActionLockInput),
    Move(ClipActionMove),
    Spawn(ClipActionSpawn),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionInvulnerability;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionMove;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionDrawSprite {
    pub layer: u32,
    pub texture_id: TextureId,
    pub local_pos: Vec2,
    pub local_rotation: f32,
    pub rect_pos: UVec2,
    pub rect_size: UVec2,
    pub sort_offset: f32,
    pub rotate_with_parent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionAttackBox {
    pub local_pos: Vec2,
    pub local_rotation: f32,
    pub group: lib_col::Group,
    pub shape: lib_col::Shape,
    pub rotate_with_parent: bool,
    pub graze_value: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionLockInput {
    pub allow_walk_input: bool,
    pub allow_look_input: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionSpawn {
    pub rotate_with_parent: bool,
    pub local_pos: Vec2,
    pub local_look: f32,
    #[serde(flatten)]
    pub character_info: CharacterInfo,
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
    BunnyIdleR,
    BunnyIdleD,
    BunnyIdleL,
    BunnyIdleU,
    BunnyWalkR,
    BunnyWalkD,
    BunnyWalkL,
    BunnyWalkU,
    BunnyAttackD,
    BunnyDash,
    StabberIdle,
    StabberAttack,
    ShooterIdle,
    ShooterAttack,
}
