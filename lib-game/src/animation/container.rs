use anyhow::Context;
use hashbrown::HashMap;
use macroquad::prelude::*;
use std::any::{Any, TypeId, type_name};

use super::actions::*;

pub trait AnimContainer: std::fmt::Debug + Any {
    #[cfg(feature = "dev-env")]
    fn clip_count(&self) -> u32;
    #[cfg(feature = "dev-env")]
    fn get_clip(&self, clip_id: u32) -> Option<Clip>;
    #[cfg(feature = "dev-env")]
    fn add_clip(&mut self, track_id: u32, start: u32, len: u32);
    #[cfg(feature = "dev-env")]
    fn delete_clip(&mut self, clip_id: u32);

    #[cfg(feature = "dev-env")]
    fn offset_clip_actions(&mut self, off: Vec2);
    #[cfg(feature = "dev-env")]
    fn clip_action_editor_ui(&mut self, clip_id: u32, ui: &mut egui::Ui);
    #[cfg(feature = "dev-env")]
    fn set_clip_pos_len(&mut self, idx: u32, new_track: u32, new_pos: u32, new_len: u32);
    #[cfg(feature = "dev-env")]
    fn clip_has_intersection(&self, track_id: u32, skip: u32, start: u32, len: u32) -> Option<i32>;

    #[cfg(feature = "dev-env")]
    fn track_count(&self) -> u32;
    #[cfg(feature = "dev-env")]
    fn get_track(&'_ self, track_id: u32) -> Option<&Track>;
    #[cfg(feature = "dev-env")]
    fn add_track(&mut self, name: String);
    #[cfg(feature = "dev-env")]
    fn delete_track(&mut self, track_id: u32);

    fn manifest_key(&self) -> &'static str;
    fn type_name(&self) -> &'static str;
    fn max_pos(&self) -> u32;

    fn to_manifest(&self) -> lib_asset::animation_manifest::Clips;
    fn from_manifest(
        generic: &lib_asset::animation_manifest::Clips,
    ) -> anyhow::Result<Box<dyn AnimContainer>>
    where
        Self: Sized;
}

#[derive(Default, Debug)]
pub struct Animation {
    pub is_looping: bool,
    pub action_tracks: HashMap<TypeId, Box<dyn AnimContainer>>,
}

#[derive(Clone, Debug)]
pub struct Clips<T> {
    pub clips: Vec<(Clip, T)>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Clip {
    pub track_id: u32,
    pub start: u32,
    pub len: u32,
}

impl Clip {
    pub fn contains_pos(&self, pos: u32) -> bool {
        self.start <= pos && pos < self.end()
    }

    pub fn end(&self) -> u32 {
        self.start + self.len
    }
}

impl Animation {
    pub fn max_pos(&self) -> u32 {
        self.action_tracks
            .values()
            .map(|x| x.max_pos())
            .max()
            .unwrap_or_default()
    }

    pub fn to_manifest(&self) -> lib_asset::animation_manifest::Animation {
        lib_asset::animation_manifest::Animation {
            is_looping: self.is_looping,
            action_tracks: self
                .action_tracks
                .values()
                .map(|container| {
                    (
                        container.manifest_key().to_string(),
                        container.to_manifest(),
                    )
                })
                .collect(),
        }
    }

    pub fn from_manifest(
        manifest: &lib_asset::animation_manifest::Animation,
    ) -> anyhow::Result<Self> {
        let mut action_tracks = HashMap::new();

        Self::add_action_track::<Invulnerability>(&mut action_tracks, manifest)?;
        Self::add_action_track::<Move>(&mut action_tracks, manifest)?;
        Self::add_action_track::<DrawSprite>(&mut action_tracks, manifest)?;
        Self::add_action_track::<AttackBox>(&mut action_tracks, manifest)?;
        Self::add_action_track::<LockInput>(&mut action_tracks, manifest)?;
        Self::add_action_track::<Spawn>(&mut action_tracks, manifest)?;
        debug_assert_eq!(
            action_tracks.len(),
            CLIP_TYPES.len(),
            "incomplete maninfest"
        );
        Ok(Animation {
            is_looping: manifest.is_looping,
            action_tracks,
        })
    }

    fn add_action_track<T: ClipAction>(
        action_tracks: &mut HashMap<TypeId, Box<dyn AnimContainer>>,
        manifest: &lib_asset::animation_manifest::Animation,
    ) -> anyhow::Result<()> {
        let manifest_key = T::manifest_key();
        let Some(entry) = manifest.action_tracks.get(manifest_key) else {
            anyhow::bail!("No such action track: {manifest_key:?}");
        };
        let clips = Clips::<T>::from_manifest(entry)?;
        action_tracks.insert(TypeId::of::<T>(), clips);
        Ok(())
    }

    pub fn active_clips<T: ClipAction>(&self, pos: u32) -> impl Iterator<Item = (u32, T)> {
        self.action_track::<T>()
            .clips
            .iter()
            .copied()
            .enumerate()
            .filter(move |(_, (x, _))| x.contains_pos(pos))
            .map(|(idx, (_, action))| (idx as u32, action))
    }

    pub fn inactive_clips<T: ClipAction>(&self, pos: u32) -> impl Iterator<Item = (u32, T)> {
        self.action_track::<T>()
            .clips
            .iter()
            .copied()
            .enumerate()
            .filter(move |(_, (x, _))| !x.contains_pos(pos))
            .map(|(idx, (_, action))| (idx as u32, action))
    }

    pub fn action_track<T: ClipAction>(&self) -> &Clips<T> {
        let container = &self.action_tracks[&TypeId::of::<T>()];
        let container: &dyn AnimContainer = container.as_ref();
        match (container as &dyn Any).downcast_ref::<Clips<T>>() {
            Some(x) => x,
            None => panic!("Type mismatch"),
        }
    }
}

impl<T: ClipAction> AnimContainer for Clips<T> {
    fn max_pos(&self) -> u32 {
        self.clips
            .iter()
            .map(|(x, _)| x.start + x.len - 1)
            .max()
            .unwrap_or_default()
    }

    #[cfg(feature = "dev-env")]
    fn offset_clip_actions(&mut self, off: Vec2) {
        for (_, action) in self.clips.iter_mut() {
            action.global_offset(off);
        }
    }

    #[cfg(feature = "dev-env")]
    fn add_track(&mut self, name: String) {
        self.tracks.push(Track { name });
    }

    #[cfg(feature = "dev-env")]
    fn delete_track(&mut self, track_id: u32) {
        self.tracks.remove(track_id as usize);
        self.clips.retain(|(x, _)| x.track_id != track_id);
        for (clip, _) in self.clips.iter_mut() {
            if clip.track_id > track_id {
                clip.track_id -= 1;
            }
        }
    }

    #[cfg(feature = "dev-env")]
    fn delete_clip(&mut self, clip_id: u32) {
        self.clips.remove(clip_id as usize);
    }

    #[cfg(feature = "dev-env")]
    fn set_clip_pos_len(&mut self, idx: u32, new_track: u32, mut new_pos: u32, new_len: u32) {
        debug_assert!((new_track as usize) < self.tracks.len());
        let Some((clip, _)) = self.clips.get(idx as usize) else {
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

        let Some((clip, _)) = self.clips.get_mut(idx as usize) else {
            return;
        };

        clip.track_id = new_track;
        clip.start = new_pos;
        clip.len = new_len;
    }

    #[cfg(feature = "dev-env")]
    fn add_clip(&mut self, track_id: u32, start: u32, len: u32) {
        if track_id >= self.tracks.len() as u32 {
            return;
        }

        if self
            .clip_has_intersection(track_id, u32::MAX, start, len)
            .is_some()
        {
            return;
        }

        let new_clip = Clip {
            track_id,
            start,
            len,
        };
        self.clips.push((new_clip, T::default()));
    }

    #[cfg(feature = "dev-env")]
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
            .filter(|(_, (x, _))| x.track_id == track_id);
        for (clip_idx, (clip, _)) in clips {
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

    #[cfg(feature = "dev-env")]
    fn clip_count(&self) -> u32 {
        self.clips.len() as u32
    }

    #[cfg(feature = "dev-env")]
    fn get_clip(&self, clip_id: u32) -> Option<Clip> {
        let (clip, _) = self.clips.get(clip_id as usize)?;
        Some(*clip)
    }

    #[cfg(feature = "dev-env")]
    fn track_count(&self) -> u32 {
        self.tracks.len() as u32
    }

    #[cfg(feature = "dev-env")]
    fn get_track(&'_ self, track_id: u32) -> Option<&'_ Track> {
        self.tracks.get(track_id as usize)
    }

    fn from_manifest(
        generic: &lib_asset::animation_manifest::Clips,
    ) -> anyhow::Result<Box<dyn AnimContainer>>
    where
        Self: Sized,
    {
        let mut tracks = Vec::new();
        for track in &generic.tracks {
            tracks.push(Track {
                name: track.name.clone(),
            });
        }

        let mut clips = Vec::new();
        for (clip_id, clip) in generic.clips.iter().enumerate() {
            let action =
                T::from_manifest(&clip.action).with_context(|| format!("clip {clip_id}"))?;
            clips.push((
                Clip {
                    track_id: clip.track_id,
                    start: clip.start,
                    len: clip.len,
                },
                action,
            ));
        }

        Ok(Box::new(Self { clips, tracks }))
    }

    fn to_manifest(&self) -> lib_asset::animation_manifest::Clips {
        let tracks = self
            .tracks
            .iter()
            .map(|x| lib_asset::animation_manifest::Track {
                name: x.name.clone(),
            })
            .collect();
        let clips = self
            .clips
            .iter()
            .map(|(clip, action)| lib_asset::animation_manifest::Clip {
                track_id: clip.track_id,
                start: clip.start,
                len: clip.len,
                action: action.to_manifest(),
            })
            .collect();
        lib_asset::animation_manifest::Clips { clips, tracks }
    }

    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn manifest_key(&self) -> &'static str {
        T::manifest_key()
    }

    #[cfg(feature = "dev-env")]
    fn clip_action_editor_ui(&mut self, clip_id: u32, ui: &mut egui::Ui) {
        self.clips[clip_id as usize].1.editor_ui(ui);
    }
}

#[cfg(feature = "dev-env")]
impl Animation {
    pub fn clip_editor_ui(&mut self, kind: TypeId, clip_id: u32, ui: &mut egui::Ui) {
        let Some(container) = self.action_tracks.get_mut(&kind) else {
            return;
        };
        container.clip_action_editor_ui(clip_id, ui);
    }

    pub fn get_clip(&self, kind: TypeId, clip_id: u32) -> Option<Clip> {
        let container = self.action_tracks.get(&kind)?;
        container.get_clip(clip_id)
    }

    pub fn get_track(&self, kind: TypeId, track_id: u32) -> Option<&Track> {
        let container = self.action_tracks.get(&kind)?;
        container.get_track(track_id)
    }

    pub fn global_offset(&mut self, off: Vec2) {
        for container in self.action_tracks.values_mut() {
            container.offset_clip_actions(off);
        }
    }

    pub fn add_track(&mut self, kind: TypeId, name: String) {
        let Some(container) = self.action_tracks.get_mut(&kind) else {
            return;
        };
        container.add_track(name);
    }

    pub fn delete_track(&mut self, kind: TypeId, track_id: u32) {
        let Some(container) = self.action_tracks.get_mut(&kind) else {
            return;
        };
        container.delete_track(track_id);
    }

    pub fn add_clip(&mut self, kind: TypeId, track_id: u32, start: u32, len: u32) {
        let Some(container) = self.action_tracks.get_mut(&kind) else {
            return;
        };
        container.add_clip(track_id, start, len);
    }

    pub fn delete_clip(&mut self, kind: TypeId, clip_id: u32) {
        let Some(container) = self.action_tracks.get_mut(&kind) else {
            return;
        };
        container.delete_clip(clip_id);
    }

    pub fn set_clip_pos_len(
        &mut self,
        kind: TypeId,
        idx: u32,
        new_track: u32,
        new_pos: u32,
        new_len: u32,
    ) {
        let Some(container) = self.action_tracks.get_mut(&kind) else {
            return;
        };
        container.set_clip_pos_len(idx, new_track, new_pos, new_len);
    }

    pub fn all_clips(&self) -> impl Iterator<Item = (TypeId, &str, u32, u32, Clip)> {
        CLIP_TYPES
            .into_iter()
            .zip(self.all_track_ofsets())
            .flat_map(|(kind, y_off)| {
                let container = &self.action_tracks[&kind];
                let name = container.manifest_key();
                (0..container.clip_count()).map(move |clip_id| {
                    let clip = container.get_clip(clip_id).unwrap();
                    (kind, name, clip_id, y_off + clip.track_id, clip)
                })
            })
    }

    pub fn all_tracks(&self) -> impl Iterator<Item = (TypeId, u32, u32, &Track)> {
        CLIP_TYPES
            .into_iter()
            .zip(self.all_track_ofsets())
            .flat_map(|(kind, y_off)| {
                let container = &self.action_tracks[&kind];
                (0..container.track_count()).map(move |track_id| {
                    let track = container.get_track(track_id).unwrap();
                    (kind, track_id, track_id + y_off, track)
                })
            })
    }

    fn all_track_ofsets(&self) -> [u32; CLIP_TYPES.len()] {
        let mut track_offsets = [0; CLIP_TYPES.len()];
        let mut curr_off = 0;
        for (kind, off) in CLIP_TYPES.into_iter().zip(&mut track_offsets) {
            *off = curr_off;
            curr_off += self.action_tracks[&kind].track_count();
        }
        track_offsets
    }
}
