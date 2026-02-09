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
    pub action_tracks: ActionsTracks,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ActionsTracks {
    pub invulnerability: Clips<Invulnerability>,
    pub r#move: Clips<Move>,
    pub draw_sprite: Clips<DrawSprite>,
    pub attack_box: Clips<AttackBox>,
    pub lock_input: Clips<LockInput>,
    pub spawn: Clips<Spawn>,
}

impl Animation {
    pub fn max_pos(&self) -> u32 {
        [
            self.action_tracks.invulnerability.max_pos(),
            self.action_tracks.r#move.max_pos(),
            self.action_tracks.draw_sprite.max_pos(),
            self.action_tracks.attack_box.max_pos(),
            self.action_tracks.lock_input.max_pos(),
            self.action_tracks.spawn.max_pos(),
        ]
        .into_iter()
        .max()
        .unwrap_or_default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clips<T> {
    pub clips: Vec<Clip<T>>,
    pub tracks: Vec<Track>,
}

pub trait ClipAction: Default + Copy {
    fn name(&self) -> &'static str;

    fn global_offset(&mut self, off: Vec2);
}

impl<T: ClipAction> Clips<T> {
    pub fn global_offset(&mut self, off: Vec2) {
        for clips in self.clips.iter_mut() {
            clips.action.global_offset(off);
        }
    }

    pub fn max_pos(&self) -> u32 {
        self.clips
            .iter()
            .map(|x| x.start + x.len - 1)
            .max()
            .unwrap_or_default()
    }

    pub fn active_clips(&self, pos: u32) -> impl Iterator<Item = (u32, Clip<T>)> {
        self.clips
            .iter()
            .copied()
            .enumerate()
            .map(|(x, y)| (x as u32, y))
            .filter(move |(_, x)| x.contains_pos(pos))
    }

    pub fn inactive_clips(&self, pos: u32) -> impl Iterator<Item = (u32, Clip<T>)> {
        self.clips
            .iter()
            .copied()
            .enumerate()
            .map(|(x, y)| (x as u32, y))
            .filter(move |(_, x)| !x.contains_pos(pos))
    }

    pub fn add_track(&mut self, name: String) {
        self.tracks.push(Track { name });
    }

    pub fn delete_track(&mut self, track_id: u32) {
        self.tracks.remove(track_id as usize);
        self.clips.retain(|x| x.track_id != track_id);
        for clip in self.clips.iter_mut() {
            if clip.track_id > track_id {
                clip.track_id -= 1;
            }
        }
    }

    pub fn delete_clip(&mut self, clip_id: u32) {
        self.clips.remove(clip_id as usize);
    }

    pub fn set_clip_pos_len(&mut self, idx: u32, new_track: u32, mut new_pos: u32, new_len: u32) {
        let Some(clip) = self.clips.get(idx as usize) else {
            return;
        };

        let push = self.clip_has_intersection(new_track, idx, new_pos, new_len);
        if let Some(push) = push {
            if clip.len == new_len {
                new_pos = (new_pos as i32 + push) as u32;
                if self
                    .clip_has_intersection(new_track, idx, new_pos, new_len)
                    .is_some()
                {
                    return;
                }
            } else {
                return;
            }
        }

        let Some(clip) = self.clips.get_mut(idx as usize) else {
            return;
        };

        clip.track_id = new_track;
        clip.start = new_pos;
        clip.len = new_len;
    }

    pub fn add_clip(&mut self, track_id: u32, start: u32, len: u32) {
        if track_id >= self.tracks.len() as u32 {
            return;
        }

        if self
            .clip_has_intersection(track_id, u32::MAX, start, len)
            .is_some()
        {
            return;
        }

        self.clips.push(Clip {
            track_id,
            start,
            len,
            action: T::default(),
        });
    }

    fn clip_has_intersection(&self, track_id: u32, skip: u32, start: u32, len: u32) -> Option<i32> {
        let end = start + len;
        let mut res = None::<i32>;
        let mut update = |x: i32| match res {
            Some(y) if x.abs() < y.abs() => res = Some(x),
            Some(_) => (),
            None => res = Some(x),
        };

        let clips = self
            .clips
            .iter()
            .enumerate()
            .filter(|(_, x)| x.track_id == track_id);
        for (clip_idx, clip) in clips {
            if clip_idx as u32 == skip {
                continue;
            }
            if clip.start <= start && clip.end() > start {
                update(clip.end() as i32 - start as i32);
                continue;
            }
            if start <= clip.start && end > clip.start {
                update(clip.start as i32 - end as i32);
                continue;
            }
        }
        res
    }
}

impl<Action> Default for Clips<Action> {
    fn default() -> Self {
        Self {
            clips: Vec::new(),
            tracks: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Clip<Action> {
    pub track_id: u32,
    pub start: u32,
    pub len: u32,
    pub action: Action,
}

impl<Action> Clip<Action> {
    pub fn contains_pos(&self, pos: u32) -> bool {
        self.start <= pos && pos < self.end()
    }

    pub fn end(&self) -> u32 {
        self.start + self.len
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Invulnerability;

impl ClipAction for Invulnerability {
    fn name(&self) -> &'static str {
        "Invulnerability"
    }

    fn global_offset(&mut self, _off: Vec2) {}
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Move;

impl ClipAction for Move {
    fn name(&self) -> &'static str {
        "Move"
    }

    fn global_offset(&mut self, _off: Vec2) {}
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DrawSprite {
    pub layer: u32,
    pub texture_id: TextureId,
    pub local_pos: Vec2,
    pub local_rotation: f32,
    pub rect_pos: UVec2,
    pub rect_size: UVec2,
    pub sort_offset: f32,
    pub rotate_with_parent: bool,
}

impl ClipAction for DrawSprite {
    fn name(&self) -> &'static str {
        "DrawSprite"
    }

    fn global_offset(&mut self, off: Vec2) {
        self.local_pos += off;
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AttackBox {
    pub local_pos: Vec2,
    pub local_rotation: f32,
    pub group: lib_col::Group,
    pub shape: lib_col::Shape,
    pub rotate_with_parent: bool,
    pub graze_value: f32,
}

impl ClipAction for AttackBox {
    fn name(&self) -> &'static str {
        "AttackBox"
    }

    fn global_offset(&mut self, off: Vec2) {
        self.local_pos += off;
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LockInput {
    pub allow_walk_input: bool,
    pub allow_look_input: bool,
}

impl ClipAction for LockInput {
    fn name(&self) -> &'static str {
        "LockInput"
    }

    fn global_offset(&mut self, _off: Vec2) {}
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Spawn {
    pub rotate_with_parent: bool,
    pub local_pos: Vec2,
    pub local_look: f32,
    #[serde(flatten)]
    pub character_info: CharacterInfo,
}

impl ClipAction for Spawn {
    fn name(&self) -> &'static str {
        "Spawn"
    }

    fn global_offset(&mut self, off: Vec2) {
        self.local_pos += off;
    }
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
