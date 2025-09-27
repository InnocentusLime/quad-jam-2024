use std::fs::File;

use anyhow::Context;
use hashbrown::HashMap;

use crate::{Animation, AnimationId};

mod aseprite_decode;

pub fn load_animations_by_name(name: &str) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    let path = format!("art-project/{}.json", name);
    let anim_file = File::open(path).context("loading file")?;
    let sheet = serde_json::from_reader(anim_file)?;
    let anim = aseprite_decode::load_animations_from_aseprite(&sheet)?;
    Ok(anim)
}
