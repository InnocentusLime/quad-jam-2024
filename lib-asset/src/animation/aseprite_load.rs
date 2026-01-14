use std::{fs::File, path::Path, str::FromStr};

use crate::{AssetRoot, FsResolver, animation::ClipActionDrawSprite};
use anyhow::{Context, bail, ensure};
use glam::vec2;
use hashbrown::HashMap;
use log::{info, warn};
use macroquad::texture::Texture2D;
use serde::Deserialize;

use super::{Animation, AnimationId, Clip, ClipAction, ImgRect, Track};

pub fn load_animations_aseprite(
    resolver: &FsResolver,
    path: impl AsRef<Path>,
    layer: Option<&str>,
) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    let path = path.as_ref();
    let anim_file = File::open(path).with_context(|| format!("loading file {path:?}"))?;
    let sheet =
        serde_json::from_reader(anim_file).with_context(|| format!("decoding file {path:?}"))?;
    let anim = load_animations_from_aseprite(resolver, &sheet, layer)
        .with_context(|| format!("converting {path:?}"))?;
    Ok(anim)
}

fn load_animations_from_aseprite(
    resolver: &FsResolver,
    sheet: &Sheet,
    layer: Option<&str>,
) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    let res = load_clips_from_aseprite(resolver, sheet, layer)?
        .into_iter()
        .map(|(name, (is_looping, track_count, clips))| {
            let tracks = (0..track_count)
                .map(|id| Track {
                    id,
                    name: format!("sprites {id}"),
                })
                .collect();
            (
                name,
                Animation {
                    clips,
                    tracks,
                    is_looping,
                },
            )
        })
        .collect();
    Ok(res)
}

fn load_clips_from_aseprite(
    resolver: &FsResolver,
    sheet: &Sheet,
    layer: Option<&str>,
) -> anyhow::Result<HashMap<AnimationId, (bool, u32, Vec<Clip>)>> {
    let version_prefix = String::from(REQUIRED_ASEPRITE_VERSION) + ".";
    let version = &sheet.meta.version;
    anyhow::ensure!(
        version.starts_with(&version_prefix),
        "Expected version to be {REQUIRED_ASEPRITE_VERSION}. Found {version:?}",
    );

    let frames = collect_frames(resolver, sheet, layer)?;
    let mut result = HashMap::new();
    for tag in &sheet.meta.frame_tags {
        let anim_id = match AnimationId::from_str(&tag.data) {
            Ok(x) => x,
            Err(_) => {
                warn!(
                    "Skipping tag {:?}: does not correspond to any animation id",
                    tag.data
                );
                continue;
            }
        };
        let is_looping = tag.repeat.is_empty();
        if !(tag.from..=tag.to).all(|x| frames.contains_key(&x)) {
            info!("Skipping tag {:?}: some frames are absent", tag.data);
            continue;
        }

        let mut start = 0;
        let mut max_track_id = 0;
        let mut clips = Vec::new();
        for frame_id in tag.from..=tag.to {
            let mut shared_duration = None;
            for (track_id, duration, action) in &frames[&frame_id] {
                clips.push(Clip {
                    track_id: *track_id,
                    id: clips.len() as u32,
                    start,
                    len: *duration,
                    action: *action,
                });
                assert!(shared_duration.is_none() || shared_duration == Some(*duration));
                shared_duration = Some(*duration);
                max_track_id = max_track_id.max(*track_id);
            }
            start += shared_duration.unwrap();
        }
        result.insert(anim_id, (is_looping, max_track_id + 1, clips));
    }

    Ok(result)
}

fn collect_frames(
    resolver: &FsResolver,
    sheet: &Sheet,
    layer: Option<&str>,
) -> anyhow::Result<HashMap<u32, Vec<(u32, u32, ClipAction)>>> {
    let mut result = HashMap::<u32, Vec<(u32, u32, ClipAction)>>::new();
    for frame in &sheet.frames {
        let pieces = frame.filename.split('.').collect::<Vec<_>>();
        ensure!(pieces.len() == 2, "two many pieces in frame filename");
        if layer.map(|x| x != pieces[1]).unwrap_or_default() {
            continue;
        }

        let Ok(frame_id) = pieces[0].parse() else {
            bail!("first piece of the frame name must be an integer");
        };
        let sprite_path = resolver.get_path(AssetRoot::Assets, &sheet.meta.image);

        // NOTE: we assume that the order of layer frames is the same
        //       as the desired draw order
        let frames = result.entry(frame_id).or_default();
        frames.push((
            frames.len() as u32,
            frame.duration,
            ClipAction::DrawSprite(ClipActionDrawSprite {
                layer: 1,
                texture_id: resolver.inverse_resolve::<Texture2D>(&sprite_path).unwrap(),
                local_pos: vec2(-(frame.frame.w as f32) * 0.5, -(frame.frame.h as f32) * 0.5),
                local_rotation: 0.0,
                rect: ImgRect {
                    x: frame.frame.x,
                    y: frame.frame.y,
                    w: frame.frame.w,
                    h: frame.frame.h,
                },
                sort_offset: 0.0f32,
                rotate_with_parent: false,
            }),
        ))
    }

    Ok(result)
}

static REQUIRED_ASEPRITE_VERSION: &str = "1.3";

#[derive(Debug, Deserialize)]
pub struct Sheet {
    pub frames: Vec<Frame>,
    pub meta: SheetMeta,
}

#[derive(Debug, Deserialize)]
pub struct Frame {
    pub filename: String,
    /// The rect describing the frame rect
    pub frame: FrameRect,
    /// Duration in milliseconds
    pub duration: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct FrameRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub struct SheetMeta {
    pub version: String,
    pub image: String,
    #[serde(rename = "frameTags")]
    pub frame_tags: Vec<SheetTag>,
}

#[derive(Debug, Deserialize)]
pub struct SheetTag {
    #[allow(dead_code)]
    pub name: String,
    pub from: u32,
    pub to: u32,
    #[serde(default)]
    pub repeat: String,
    /// Userdata
    #[serde(default)]
    pub data: String,
}
