use anyhow::Context;
#[cfg(feature = "dev-env")]
use lib_asset::AssetContainer;
use macroquad::prelude::*;
use std::any::{Any, type_name};

use crate::Resources;

use super::actions::*;

pub trait AnimContainer: std::fmt::Debug + Any {
    fn action_kind(&self) -> u32;

    fn clip_count(&self) -> u32;
    fn get_clip(&self, clip_id: u32) -> Option<Clip>;
    #[cfg(feature = "dev-env")]
    fn add_clip(&mut self, track_id: u32, start: u32, len: u32);
    #[cfg(feature = "dev-env")]
    fn delete_clip(&mut self, clip_id: u32);

    #[cfg(feature = "dev-env")]
    fn offset_clip_actions(&mut self, off: Vec2);
    #[cfg(feature = "dev-env")]
    fn clip_action_editor_ui(
        &mut self,
        resources: &AssetContainer<Texture2D>,
        clip_id: u32,
        ui: &mut egui::Ui,
    );
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

    fn to_manifest(&self, resources: &Resources) -> lib_asset::animation_manifest::Clips;
}

#[derive(Default, Debug)]
pub struct Animation {
    pub is_looping: bool,

    pub invulerability: Clips<Invulnerability>,
    pub mov: Clips<Move>,
    pub draw_sprite: Clips<DrawSprite>,
    pub attack_box: Clips<AttackBox>,
    pub lock_input: Clips<LockInput>,
    pub spawn: Clips<Spawn>,
}

#[derive(Default, Clone, Debug)]
pub struct Clips<T> {
    pub clips: Vec<(Clip, T)>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        self.all_containers()
            .into_iter()
            .map(|x| x.max_pos())
            .max()
            .unwrap_or_default()
    }

    pub fn to_manifest(&self, resources: &Resources) -> lib_asset::animation_manifest::Animation {
        lib_asset::animation_manifest::Animation {
            is_looping: self.is_looping,
            action_tracks: self
                .all_containers()
                .into_iter()
                .map(|container| {
                    (
                        container.manifest_key().to_string(),
                        container.to_manifest(resources),
                    )
                })
                .collect(),
        }
    }

    pub fn from_manifest(
        resources: &Resources,
        manifest: &lib_asset::animation_manifest::Animation,
    ) -> anyhow::Result<Self> {
        Ok(Animation {
            is_looping: manifest.is_looping,
            invulerability: Self::parse_clips(resources, manifest)?,
            mov: Self::parse_clips(resources, manifest)?,
            draw_sprite: Self::parse_clips(resources, manifest)?,
            attack_box: Self::parse_clips(resources, manifest)?,
            lock_input: Self::parse_clips(resources, manifest)?,
            spawn: Self::parse_clips(resources, manifest)?,
        })
    }

    pub fn all_inactive_clips(&self, pos: u32) -> impl Iterator<Item = (u32, u32)> {
        self.all_containers()
            .into_iter()
            .flat_map(move |container| {
                let elem_id = container.action_kind();
                (0..container.clip_count()).filter_map(move |clip_id| {
                    let clip = container.get_clip(clip_id).unwrap();
                    if clip.contains_pos(pos) {
                        None
                    } else {
                        Some((elem_id, clip_id))
                    }
                })
            })
    }

    pub fn all_containers(&self) -> [&'_ dyn AnimContainer; 6] {
        [
            &self.invulerability as &dyn AnimContainer,
            &self.mov as &dyn AnimContainer,
            &self.draw_sprite as &dyn AnimContainer,
            &self.attack_box as &dyn AnimContainer,
            &self.lock_input as &dyn AnimContainer,
            &self.spawn as &dyn AnimContainer,
        ]
    }

    pub fn all_containers_mut<'a>(&mut self) -> [&'_ mut dyn AnimContainer; 6] {
        [
            &mut self.invulerability as &mut dyn AnimContainer,
            &mut self.mov as &mut dyn AnimContainer,
            &mut self.draw_sprite as &mut dyn AnimContainer,
            &mut self.attack_box as &mut dyn AnimContainer,
            &mut self.lock_input as &mut dyn AnimContainer,
            &mut self.spawn as &mut dyn AnimContainer,
        ]
    }

    pub fn get_container(&self, kind: u32) -> &'_ dyn AnimContainer {
        let all_containers = self.all_containers();
        assert!(
            (kind as usize) < all_containers.len(),
            "Invalid kind: {kind}"
        );
        self.all_containers()[kind as usize]
    }

    pub fn get_container_mut(&mut self, kind: u32) -> &'_ mut dyn AnimContainer {
        let all_containers = self.all_containers_mut();
        assert!(
            (kind as usize) < all_containers.len(),
            "Invalid kind: {kind}"
        );
        self.all_containers_mut()[kind as usize]
    }

    fn parse_clips<T: ClipAction>(
        resources: &Resources,
        manifest: &lib_asset::animation_manifest::Animation,
    ) -> anyhow::Result<Clips<T>> {
        let manifest_key = T::manifest_key();
        let Some(entry) = manifest.action_tracks.get(manifest_key) else {
            anyhow::bail!("No such action track: {manifest_key:?}");
        };

        Clips::<T>::from_manifest(resources, entry)
    }
}

impl<T> Clips<T> {
    pub fn active_clips(&self, pos: u32) -> impl Iterator<Item = (u32, &T)> {
        self.clips
            .iter()
            .enumerate()
            .filter(move |(_, (x, _))| x.contains_pos(pos))
            .map(|(idx, (_, action))| (idx as u32, action))
    }

    pub fn inactive_clips(&self, pos: u32) -> impl Iterator<Item = (u32, &T)> {
        self.clips
            .iter()
            .enumerate()
            .filter(move |(_, (x, _))| !x.contains_pos(pos))
            .map(|(idx, (_, action))| (idx as u32, action))
    }
}

impl<T: ClipAction> Clips<T> {
    fn from_manifest(
        resources: &Resources,
        generic: &lib_asset::animation_manifest::Clips,
    ) -> anyhow::Result<Self> {
        let mut tracks = Vec::new();
        for track in &generic.tracks {
            tracks.push(Track {
                name: track.name.clone(),
            });
        }

        let mut clips = Vec::new();
        for (clip_id, clip) in generic.clips.iter().enumerate() {
            let action = T::from_manifest(resources, &clip.action)
                .with_context(|| format!("clip {clip_id}"))?;
            clips.push((
                Clip {
                    track_id: clip.track_id,
                    start: clip.start,
                    len: clip.len,
                },
                action,
            ));
        }

        Ok(Self { clips, tracks })
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

    fn clip_count(&self) -> u32 {
        self.clips.len() as u32
    }

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

    fn to_manifest(&self, resources: &Resources) -> lib_asset::animation_manifest::Clips {
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
                action: action.to_manifest(resources),
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
    fn clip_action_editor_ui(
        &mut self,
        resources: &AssetContainer<Texture2D>,
        clip_id: u32,
        ui: &mut egui::Ui,
    ) {
        self.clips[clip_id as usize].1.editor_ui(resources, ui);
    }

    fn action_kind(&self) -> u32 {
        T::ACTION_KIND
    }
}

#[cfg(feature = "dev-env")]
impl Animation {
    pub fn clip_editor_ui(
        &mut self,
        resources: &AssetContainer<Texture2D>,
        kind: u32,
        clip_id: u32,
        ui: &mut egui::Ui,
    ) {
        self.get_container_mut(kind)
            .clip_action_editor_ui(resources, clip_id, ui);
    }

    pub fn get_clip(&self, kind: u32, clip_id: u32) -> Option<Clip> {
        self.get_container(kind).get_clip(clip_id)
    }

    pub fn get_track(&self, kind: u32, track_id: u32) -> Option<&Track> {
        self.get_container(kind).get_track(track_id)
    }

    pub fn global_offset(&mut self, off: Vec2) {
        for container in self.all_containers_mut() {
            container.offset_clip_actions(off);
        }
    }

    pub fn add_track(&mut self, kind: u32, name: String) {
        self.get_container_mut(kind).add_track(name);
    }

    pub fn delete_track(&mut self, kind: u32, track_id: u32) {
        self.get_container_mut(kind).delete_track(track_id);
    }

    pub fn add_clip(&mut self, kind: u32, track_id: u32, start: u32, len: u32) {
        self.get_container_mut(kind).add_clip(track_id, start, len);
    }

    pub fn delete_clip(&mut self, kind: u32, clip_id: u32) {
        self.get_container_mut(kind).delete_clip(clip_id);
    }

    pub fn set_clip_pos_len(
        &mut self,
        kind: u32,
        idx: u32,
        new_track: u32,
        new_pos: u32,
        new_len: u32,
    ) {
        self.get_container_mut(kind)
            .set_clip_pos_len(idx, new_track, new_pos, new_len);
    }

    pub fn all_clips(&self) -> impl Iterator<Item = (u32, &str, u32, u32, Clip)> {
        let mut kind_offset = 0;
        self.all_containers()
            .into_iter()
            .flat_map(move |container| {
                let kind = container.action_kind();
                let curr_kind_offset = kind_offset;
                let name = container.manifest_key();
                kind_offset += container.track_count();
                (0..container.clip_count()).map(move |clip_id| {
                    let clip = container.get_clip(clip_id).unwrap();
                    (kind, name, clip_id, curr_kind_offset + clip.track_id, clip)
                })
            })
    }

    pub fn all_tracks(&self) -> impl Iterator<Item = (u32, u32, u32, &Track)> {
        let mut kind_offset = 0;
        self.all_containers()
            .into_iter()
            .flat_map(move |container| {
                let kind = container.action_kind();
                let curr_kind_offset = kind_offset;
                kind_offset += container.track_count();
                (0..container.track_count()).map(move |track_id| {
                    let track = container.get_track(track_id).unwrap();
                    (kind, track_id, curr_kind_offset + track_id, track)
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashSet;

    use crate::{AnimContainer, Clip, ClipAction, DrawSprite, animation::container::Animation};

    #[test]
    fn container_lookup_consistent() {
        let mut anim = Animation::default();

        let kinds = anim.all_containers().map(|x| x.action_kind());
        for kind in kinds {
            assert_eq!(anim.get_container(kind).action_kind(), kind);
        }

        let kinds = anim.all_containers_mut().map(|x| x.action_kind());
        for kind in kinds {
            assert_eq!(anim.get_container_mut(kind).action_kind(), kind);
        }
    }

    #[test]
    fn all_tracks_y_contigious_basic() {
        let mut anim = Animation::default();
        anim.all_containers_mut()
            .into_iter()
            .for_each(|x| x.add_track("test1".to_string()));

        let mut tracks = anim.all_tracks().map(|x| x.2).collect::<Vec<_>>();
        tracks.sort();

        assert_eq!(tracks, (0..tracks.len() as u32).collect::<Vec<_>>());
    }

    #[test]
    fn all_tracks_y_contigious_dup() {
        let mut anim = Animation::default();
        anim.all_containers_mut()
            .into_iter()
            .for_each(|x| x.add_track("test1".to_string()));
        anim.draw_sprite.add_track("test2".to_string());

        let mut tracks = anim.all_tracks().map(|x| x.2).collect::<Vec<_>>();
        tracks.sort();

        assert_eq!(tracks, (0..tracks.len() as u32).collect::<Vec<_>>());
    }

    #[test]
    fn clip_kinds_unique() {
        let anim = Animation::default();
        let container_kinds = anim
            .all_containers()
            .into_iter()
            .map(|x| x.action_kind())
            .collect::<HashSet<_>>();
        let container_count = anim.all_containers().into_iter().count();

        assert_eq!(container_kinds.len(), container_count);
    }

    #[test]
    fn all_clips_basic() {
        let mut anim = Animation::default();
        anim.draw_sprite.add_track("test1".to_string());
        anim.draw_sprite.add_track("test2".to_string());
        anim.draw_sprite.add_clip(0, 0, 1);
        let clips = anim.all_clips().collect::<Vec<_>>();
        let expected = [(
            DrawSprite::ACTION_KIND,
            DrawSprite::manifest_key(),
            0,
            0,
            Clip {
                track_id: 0,
                start: 0,
                len: 1,
            },
        )];
        assert_eq!(clips.as_slice(), expected.as_slice());
    }
}
