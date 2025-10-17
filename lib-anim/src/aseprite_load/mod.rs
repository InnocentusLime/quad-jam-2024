use std::{fs::File, path::Path};

use anyhow::Context;
use hashbrown::HashMap;

use crate::{Animation, AnimationId};

mod aseprite_decode;

pub fn load_animations_by_name(name: &str) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    let path = format!("project-aseprite/{}.json", name);
    load_animations(path)
}

pub fn load_animations(path: impl AsRef<Path>) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    let anim_file = File::open(path).context("loading file")?;
    let sheet = serde_json::from_reader(anim_file)?;
    let anim = aseprite_decode::load_animations_from_aseprite(&sheet)?;
    Ok(anim)
}
