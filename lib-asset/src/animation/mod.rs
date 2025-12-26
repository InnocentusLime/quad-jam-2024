#[cfg(feature = "dev-env")]
pub mod aseprite_load;

use crate::{Position, TextureId};
use serde::{Deserialize, Serialize};

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

    pub fn active_draw_sprite(&self, pos: u32) -> impl Iterator<Item = Clip<ClipActionDrawSprite>> {
        self.active_clips(pos).filter_map(Clip::to_draw_sprite)
    }

    pub fn active_attack_box(&self, pos: u32) -> impl Iterator<Item = Clip<ClipActionAttackBox>> {
        self.active_clips(pos).filter_map(Clip::to_attack_box)
    }

    pub fn active_lock_input(&self, pos: u32) -> impl Iterator<Item = Clip<ClipActionLockInput>> {
        self.active_clips(pos).filter_map(Clip::to_lock_input)
    }

    pub fn active_invulnerability(
        &self,
        pos: u32,
    ) -> impl Iterator<Item = Clip<ClipActionInvulnerability>> {
        self.active_clips(pos).filter_map(Clip::to_invulnerability)
    }

    pub fn active_move(&self, pos: u32) -> impl Iterator<Item = Clip<ClipActionMove>> {
        self.active_clips(pos).filter_map(Clip::to_move)
    }

    pub fn active_clips(&self, pos: u32) -> impl Iterator<Item = Clip> {
        self.clips
            .iter()
            .copied()
            .filter(move |x| x.start <= pos && pos < x.start + x.len)
    }

    pub fn inactive_clips(&self, pos: u32) -> impl Iterator<Item = Clip> {
        self.clips
            .iter()
            .copied()
            .filter(move |x| !(x.start <= pos && pos < x.start + x.len))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Clip<Act = ClipAction> {
    pub id: u32,
    pub track_id: u32,
    pub start: u32,
    pub len: u32,
    pub action: Act,
}

impl Clip {
    pub fn to_draw_sprite(self) -> Option<Clip<ClipActionDrawSprite>> {
        let ClipAction::DrawSprite(draw) = self.action else {
            return None;
        };
        Some(self.replace_action(draw))
    }

    pub fn to_attack_box(self) -> Option<Clip<ClipActionAttackBox>> {
        let ClipAction::AttackBox(atk) = self.action else {
            return None;
        };
        Some(self.replace_action(atk))
    }

    pub fn to_lock_input(self) -> Option<Clip<ClipActionLockInput>> {
        let ClipAction::LockInput(lock) = self.action else {
            return None;
        };
        Some(self.replace_action(lock))
    }

    pub fn to_invulnerability(self) -> Option<Clip<ClipActionInvulnerability>> {
        let ClipAction::Invulnerability(invuln) = self.action else {
            return None;
        };
        Some(self.replace_action(invuln))
    }

    pub fn to_move(self) -> Option<Clip<ClipActionMove>> {
        let ClipAction::Move(mov) = self.action else {
            return None;
        };
        Some(self.replace_action(mov))
    }

    fn replace_action<T>(self, action: T) -> Clip<T> {
        Clip {
            id: self.id,
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
pub enum ClipAction {
    DrawSprite(ClipActionDrawSprite),
    AttackBox(ClipActionAttackBox),
    Invulnerability(ClipActionInvulnerability),
    LockInput(ClipActionLockInput),
    Move(ClipActionMove),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionInvulnerability;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionMove;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionDrawSprite {
    pub layer: u32,
    pub texture_id: TextureId,
    pub local_pos: Position,
    pub local_rotation: f32,
    pub rect: ImgRect,
    pub sort_offset: f32,
    pub rotate_with_parent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionAttackBox {
    pub local_pos: Position,
    pub local_rotation: f32,
    pub team: Team,
    pub group: lib_col::Group,
    pub shape: lib_col::Shape,
    pub rotate_with_parent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClipActionLockInput {
    pub allow_walk_input: bool,
    pub allow_look_input: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub id: u32,
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
}
