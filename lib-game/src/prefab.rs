use std::path::{Path, PathBuf};

use crate::components::*;
use crate::prelude::*;

use serde::Deserialize;

pub fn spawn_prefab(
    cmds: &mut CommandBuffer,
    resources: &Resources,
    prefab: AssetKey,
    tf: Transform,
) {
    let prefab = resources.prefabs.get(prefab).expect("Dangling prefab key");
    let ent = resources.world.reserve_entity();
    cmds.insert(ent, prefab);
    if prefab.has::<Transform>() {
        cmds.insert_one(ent, tf);
    }
}

pub fn register_libgame_components(prefab_factory: &mut PrefabFactory<Resources>) {
    prefab_factory.register_component_with_constructor("transform", Transform::from_pos);
    prefab_factory.register_component_with_constructor("body", BodyTagManifest::into_body_tag);

    prefab_factory.register_component_with_constructor_ctx(
        "sprite",
        SpriteManifest::into_sprite,
        SpriteManifest::dependencies,
    );
}

#[derive(Debug, Deserialize)]
pub struct SpriteManifest {
    pub layer: u32,
    pub texture: PathBuf,
    pub tex_rect_pos: UVec2,
    pub tex_rect_size: UVec2,
    pub sort_offset: f32,
    pub local_offset: Vec2,
}

impl SpriteManifest {
    pub fn into_sprite(self, resources: &mut Resources) -> anyhow::Result<Sprite> {
        let Some(texture) = resources.textures.resolve(&self.texture) else {
            anyhow::bail!("No such texture: {:?}", self.texture);
        };
        Ok(Sprite {
            layer: self.layer,
            texture,
            tex_rect_pos: self.tex_rect_pos,
            tex_rect_size: self.tex_rect_size,
            color: mimiq::WHITE,
            sort_offset: self.sort_offset,
            local_offset: self.local_offset,
        })
    }

    pub fn dependencies(data: &serde_json::value::RawValue) -> anyhow::Result<Vec<PathBuf>> {
        #[derive(Deserialize)]
        pub struct Deps<'a> {
            #[serde(borrow)]
            pub texture: &'a Path,
        }
        let deps = serde_json::from_str::<Deps>(data.get())?;
        Ok(vec![deps.texture.into()])
    }
}

#[derive(Debug, Deserialize)]
pub struct BodyTagManifest {
    pub groups: Vec<CollisionGroupManifest>,
    pub shape: Shape,
}

impl BodyTagManifest {
    pub fn into_body_tag(self) -> BodyTag {
        let groups = self
            .groups
            .into_iter()
            .map(CollisionGroupManifest::into_group)
            .fold(col_group::NONE, Group::union);
        BodyTag { groups, shape: self.shape }
    }
}

#[derive(Debug, Deserialize)]
pub enum CollisionGroupManifest {
    Level,
    Characters,
    Player,
    Attacks,
}

impl CollisionGroupManifest {
    pub fn into_group(self) -> Group {
        match self {
            CollisionGroupManifest::Level => col_group::LEVEL,
            CollisionGroupManifest::Characters => col_group::CHARACTERS,
            CollisionGroupManifest::Player => col_group::PLAYER,
            CollisionGroupManifest::Attacks => col_group::ATTACKS,
        }
    }
}
