use std::{fs::File, path::Path};

use anyhow::Context;
use hashbrown::HashMap;
use lib_asset::FsResolver;

use crate::{Animation, AnimationId};

mod aseprite_decode;

pub fn load_animations_aseprite(
    resolver: &FsResolver,
    path: impl AsRef<Path>,
    layer: Option<&str>,
) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    let path = path.as_ref();
    let anim_file = File::open(path).with_context(|| format!("loading file {path:?}"))?;
    let sheet =
        serde_json::from_reader(anim_file).with_context(|| format!("decoding file {path:?}"))?;
    let anim = aseprite_decode::load_animations_from_aseprite(resolver, &sheet, layer)
        .with_context(|| format!("converting {path:?}"))?;
    Ok(anim)
}

pub fn load_animations_project(
    path: impl AsRef<Path>,
) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    let path = path.as_ref();
    let anim_file = File::open(path).with_context(|| format!("loading file {path:?}"))?;
    let anim =
        serde_json::from_reader(anim_file).with_context(|| format!("decoding file {path:?}"))?;
    Ok(anim)
}
