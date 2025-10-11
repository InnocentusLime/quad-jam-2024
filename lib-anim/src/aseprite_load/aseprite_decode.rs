use std::str::FromStr;

use hashbrown::HashMap;
use lib_asset::{FsResolver, TextureId};
use serde::Deserialize;
use thiserror::Error;

use crate::{Animation, AnimationId, Clip, ClipAction, ImgRect, Position};

static REQUIRED_ASEPRITE_VERSION: &'static str = "1.3";

#[derive(Debug, Error)]
pub enum LoadFromAsepriteError {
    #[error("Expected version to be {REQUIRED_ASEPRITE_VERSION}. Found {found:?}")]
    VersionMismatch { found: String },
    #[error("Duplicate frame id: {dup}")]
    DuplicateId { dup: u32 },
    #[error("Expected frame filename to be a number, found {found:?}")]
    BadFrameFilename { found: String },
    #[allow(dead_code)]
    #[error("Unknown animation name: {found:?}")]
    UnknownAnimationName { found: String },
    #[error("Exepcted repeat number to be a number, found: {found:?}")]
    RepeatNumberNotNumber { found: String },
}

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

pub fn load_clips_from_aseprite(
    resolver: &FsResolver,
    sheet: &Sheet,
) -> Result<HashMap<AnimationId, (Vec<Clip>, bool)>, LoadFromAsepriteError> {
    let version_prefix = String::from(REQUIRED_ASEPRITE_VERSION) + ".";
    if !sheet.meta.version.starts_with(&version_prefix) {
        return Err(LoadFromAsepriteError::VersionMismatch {
            found: sheet.meta.version.clone(),
        });
    }

    let mut id_to_frame = HashMap::with_capacity(sheet.frames.len());
    for frame in sheet.frames.iter() {
        let (id, new) = match frame.filename.parse::<u32>() {
            Ok(id) => (id, id_to_frame.insert(id, frame).is_none()),
            Err(_) => {
                return Err(LoadFromAsepriteError::BadFrameFilename {
                    found: frame.filename.clone(),
                });
            }
        };
        if !new {
            return Err(LoadFromAsepriteError::DuplicateId { dup: id });
        }
    }

    let mut result = HashMap::new();
    for anim in &sheet.meta.frame_tags {
        // let anim_id = AnimationId::from_str(&anim.data)
        //     .map_err(|_| LoadFromAsepriteError::UnknownAnimationName { found: anim.data.clone() })?;
        let anim_id = match AnimationId::from_str(&anim.data) {
            Ok(x) => x,
            Err(_) => {
                eprintln!("Skipping {:?}: unknown", anim.data);
                continue;
            }
        };
        let is_looping = if anim.repeat.is_empty() {
            true
        } else {
            // TODO: properly handle number of repetitions
            let _reapeat_n = anim.repeat.as_str().parse::<u32>().map_err(|_| {
                LoadFromAsepriteError::RepeatNumberNotNumber {
                    found: anim.repeat.clone(),
                }
            })?;
            false
        };

        // FIXME: this check is flakey. Just check
        // all frames.
        if anim.to as usize >= sheet.frames.len() {
            // TODO: log skip
            continue;
        }

        let mut cursor = 0u32;
        let mut clip_id = 0u32;
        let mut clips = Vec::with_capacity((anim.to - anim.from) as usize + 1);

        for frame_idx in anim.from..=anim.to {
            let frame = match id_to_frame.get(&frame_idx) {
                Some(x) => *x,
                // FIXME: DO NOT PUT THE ANIM IN
                None => break,
            };
            let rect = ImgRect {
                x: frame.frame.x,
                y: frame.frame.y,
                w: frame.frame.w,
                h: frame.frame.h,
            };
            let sprite_path = resolver.asset_path(&sheet.meta.image);
            let texture_id = TextureId::inverse_resolve(resolver, &sprite_path).unwrap();
            let action = ClipAction::DrawSprite {
                layer: 1,
                texture_id,
                local_pos: Position {
                    x: -(frame.frame.w as f32) * 0.5,
                    y: -(frame.frame.h as f32) * 0.5,
                },
                local_rotation: 0.0,
                rect,
                origin: Position { x: 0.0, y: 0.0 },
                sort_offset: 0.0f32,
            };
            clips.push(Clip {
                id: clip_id,
                start: cursor,
                len: frame.duration,
                action,
            });

            clip_id += 1;
            cursor += frame.duration;
        }

        result.insert(anim_id, (clips, is_looping));
    }

    Ok(result)
}

pub fn load_animations_from_aseprite(
    resolver: &FsResolver,
    sheet: &Sheet,
) -> Result<HashMap<AnimationId, Animation>, LoadFromAsepriteError> {
    let res = load_clips_from_aseprite(resolver, sheet)?
        .into_iter()
        .map(|(name, (clips, is_looping))| (name, Animation { clips, is_looping }))
        .collect();
    Ok(res)
}
