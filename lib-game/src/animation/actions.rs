use anyhow::bail;
#[cfg(feature = "dev-env")]
use lib_asset::AssetContainer;
use lib_asset::{AssetKey, level::CharacterInfo};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::any::TypeId;

use crate::Resources;

pub trait ClipAction: std::fmt::Debug + Default + Copy + 'static {
    fn manifest_key() -> &'static str;

    fn global_offset(&mut self, _off: Vec2) {}

    #[cfg(feature = "dev-env")]
    fn editor_ui(&mut self, _resources: &AssetContainer<Texture2D>, ui: &mut egui::Ui) {
        ui.label("No data");
    }

    fn from_manifest(resources: &Resources, manifest: &serde_json::Value) -> anyhow::Result<Self>;

    fn to_manifest(&self, resources: &Resources) -> serde_json::Value;
}

pub const CLIP_TYPES: [TypeId; 6] = [
    TypeId::of::<Invulnerability>(),
    TypeId::of::<Move>(),
    TypeId::of::<DrawSprite>(),
    TypeId::of::<AttackBox>(),
    TypeId::of::<LockInput>(),
    TypeId::of::<Spawn>(),
];

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Invulnerability;

impl ClipAction for Invulnerability {
    fn manifest_key() -> &'static str {
        "invulnerability"
    }

    fn from_manifest(_resources: &Resources, manifest: &serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(manifest.clone()).map_err(anyhow::Error::from)
    }

    fn to_manifest(&self, _resource: &Resources) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Move;

impl ClipAction for Move {
    fn manifest_key() -> &'static str {
        "move"
    }

    fn from_manifest(_resources: &Resources, manifest: &serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(manifest.clone()).map_err(anyhow::Error::from)
    }

    fn to_manifest(&self, _resource: &Resources) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DrawSprite {
    pub layer: u32,
    pub texture_id: AssetKey,
    pub local_pos: Vec2,
    pub local_rotation: f32,
    pub rect_pos: UVec2,
    pub rect_size: UVec2,
    pub sort_offset: f32,
    pub rotate_with_parent: bool,
}

impl ClipAction for DrawSprite {
    fn global_offset(&mut self, off: Vec2) {
        self.local_pos += off;
    }

    #[cfg(feature = "dev-env")]
    fn editor_ui(&mut self, textures: &AssetContainer<Texture2D>, ui: &mut egui::Ui) {
        use egui::*;

        let selected_path = textures.inverse_resolve(self.texture_id);

        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.layer).range(0..=10));
            ui.label("layer");
        });
        ComboBox::new("texture_id", "texture")
            .selected_text(format!("{selected_path:?}"))
            .show_ui(ui, |ui| {
                for (path, key) in textures.iter() {
                    ui.selectable_value(&mut self.texture_id, key, path.to_string_lossy());
                }
            });
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.local_pos.x).range(-256.0..=256.0));
            ui.add(DragValue::new(&mut self.local_pos.y).range(-256.0..=256.0));
            ui.label("local pos");
        });
        ui.horizontal(|ui| {
            ui.drag_angle(&mut self.local_rotation);
            ui.label("local rotation");
        });
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.rect_pos.x).range(0..=512));
            ui.add(DragValue::new(&mut self.rect_pos.y).range(0..=512));
            ui.label("texture rect pos");
        });
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.rect_size.x).range(0..=512));
            ui.add(DragValue::new(&mut self.rect_size.y).range(0..=512));
            ui.label("texture rect size");
        });
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.sort_offset).range(-64.0..=64.0));
            ui.label("sort offset");
        });
        ui.checkbox(&mut self.rotate_with_parent, "rotate with parent");
    }

    fn manifest_key() -> &'static str {
        "draw_sprite"
    }

    fn from_manifest(resources: &Resources, manifest: &serde_json::Value) -> anyhow::Result<Self> {
        let raw_manifest: lib_asset::animation_manifest::DrawSprite =
            serde_json::from_value(manifest.clone())?;
        let Some(texture_id) = resources.textures.resolve(&raw_manifest.atlas_file) else {
            bail!("Texture {:?} is not loaded", raw_manifest.atlas_file);
        };
        Ok(DrawSprite {
            texture_id,
            layer: raw_manifest.layer,
            local_pos: raw_manifest.local_pos,
            local_rotation: raw_manifest.local_rotation,
            rect_pos: raw_manifest.rect_pos,
            rect_size: raw_manifest.rect_size,
            sort_offset: raw_manifest.sort_offset,
            rotate_with_parent: raw_manifest.rotate_with_parent,
        })
    }

    fn to_manifest(&self, resources: &Resources) -> serde_json::Value {
        let atlas_file = resources.textures.inverse_resolve(self.texture_id);
        let raw_manifest = lib_asset::animation_manifest::DrawSprite {
            layer: self.layer,
            atlas_file: atlas_file.to_path_buf(),
            local_pos: self.local_pos,
            local_rotation: self.local_rotation,
            rect_pos: self.rect_pos,
            rect_size: self.rect_size,
            sort_offset: self.sort_offset,
            rotate_with_parent: self.rotate_with_parent,
        };
        serde_json::to_value(raw_manifest).unwrap()
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
    fn global_offset(&mut self, off: Vec2) {
        self.local_pos += off;
    }

    #[cfg(feature = "dev-env")]
    fn editor_ui(&mut self, _resources: &AssetContainer<Texture2D>, ui: &mut egui::Ui) {
        use egui::*;

        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.local_pos.x).range(-256.0..=256.0));
            ui.add(DragValue::new(&mut self.local_pos.y).range(-256.0..=256.0));
            ui.label("local pos");
        });
        ui.horizontal(|ui| {
            ui.drag_angle(&mut self.local_rotation);
            ui.label("local rotation");
        });
        ui.horizontal(|ui| {
            group_ui(ui, &mut self.group);
            ui.label("group");
        });
        ui.checkbox(&mut self.rotate_with_parent, "rotate with parent");
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.graze_value).range(0.0..=30.0));
            ui.label("graze value");
        });
        shape_ui(ui, &mut self.shape);
    }

    fn manifest_key() -> &'static str {
        "attack_box"
    }

    fn from_manifest(_resources: &Resources, manifest: &serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(manifest.clone()).map_err(anyhow::Error::from)
    }

    fn to_manifest(&self, _resource: &Resources) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LockInput {
    pub allow_walk_input: bool,
    pub allow_look_input: bool,
}

impl ClipAction for LockInput {
    #[cfg(feature = "dev-env")]
    fn editor_ui(&mut self, _resources: &AssetContainer<Texture2D>, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.allow_walk_input, "allow walk input");
        ui.checkbox(&mut self.allow_look_input, "allow look input");
    }

    fn manifest_key() -> &'static str {
        "lock_input"
    }

    fn from_manifest(_resources: &Resources, manifest: &serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(manifest.clone()).map_err(anyhow::Error::from)
    }

    fn to_manifest(&self, _resource: &Resources) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
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
    fn global_offset(&mut self, off: Vec2) {
        self.local_pos += off;
    }

    #[cfg(feature = "dev-env")]
    fn editor_ui(&mut self, _resources: &AssetContainer<Texture2D>, ui: &mut egui::Ui) {
        use egui::*;

        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.local_pos.x).range(-256.0..=256.0));
            ui.add(DragValue::new(&mut self.local_pos.y).range(-256.0..=256.0));
            ui.label("local pos");
        });
        ui.horizontal(|ui| {
            ui.drag_angle(&mut self.local_look);
            ui.label("local look");
        });
        ui.checkbox(&mut self.rotate_with_parent, "rotate with parent");
        character_info_ui(ui, &mut self.character_info);
    }

    fn manifest_key() -> &'static str {
        "spawn"
    }

    fn from_manifest(_resources: &Resources, manifest: &serde_json::Value) -> anyhow::Result<Self> {
        serde_json::from_value(manifest.clone()).map_err(anyhow::Error::from)
    }

    fn to_manifest(&self, _resource: &Resources) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}

#[cfg(feature = "dev-env")]
fn group_ui(ui: &mut egui::Ui, group: &mut lib_col::Group) {
    let response = ui.button("Configure");
    let flags_ui = |ui: &mut egui::Ui| {
        group_flags_ui(ui, group);
    };

    let popup_id = ui.make_persistent_id("group_flags");
    if response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    }
    egui::popup_above_or_below_widget(
        ui,
        popup_id,
        &response,
        egui::AboveOrBelow::Below,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        flags_ui,
    );
}

#[cfg(feature = "dev-env")]
fn group_flags_ui(ui: &mut egui::Ui, group: &mut lib_col::Group) {
    use crate::col_group::*;

    ui.set_min_width(200.0);

    let mut level = group.includes(LEVEL);
    let mut characters = group.includes(CHARACTERS);
    let mut player = group.includes(PLAYER);

    ui.checkbox(&mut level, "level");
    ui.checkbox(&mut characters, "characters");
    ui.checkbox(&mut player, "player");

    *group = lib_col::Group::empty();
    if level {
        *group = group.union(LEVEL)
    }
    if characters {
        *group = group.union(CHARACTERS);
    }
    if player {
        *group = group.union(PLAYER);
    }
}

#[cfg(feature = "dev-env")]
fn shape_ui(ui: &mut egui::Ui, shape: &mut lib_col::Shape) {
    use egui::*;

    let shape_tys = ["Rect", "Shape"];
    let defaults = [
        lib_col::Shape::Rect {
            width: 0.0,
            height: 0.0,
        },
        lib_col::Shape::Circle { radius: 0.0 },
    ];
    let curr_id = match shape {
        lib_col::Shape::Rect { .. } => 0,
        lib_col::Shape::Circle { .. } => 1,
    };
    let mut new_id = curr_id;
    ComboBox::new("shape", "Shape")
        .selected_text(shape_tys[curr_id])
        .show_ui(ui, |ui| {
            for (id, label) in shape_tys.iter().enumerate() {
                ui.selectable_value(&mut new_id, id, *label);
            }
        });
    if curr_id != new_id {
        *shape = defaults[new_id];
    }
    match shape {
        lib_col::Shape::Rect { width, height } => {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(width).range(0.0..=300.0));
                ui.label("width");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(height).range(0.0..=300.0));
                ui.label("height");
            });
        }
        lib_col::Shape::Circle { radius } => {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(radius).range(0.0..=300.0));
                ui.label("radius");
            });
        }
    }
}

#[cfg(feature = "dev-env")]
fn character_info_ui(ui: &mut egui::Ui, character_info: &mut CharacterInfo) {
    use egui::*;

    let character_tys = [
        "Player",
        "Goal",
        "Damager",
        "Stabber",
        "BasicBullet",
        "Shooter",
    ];
    let defaults = [
        CharacterInfo::Player {},
        CharacterInfo::Goal {},
        CharacterInfo::Damager {},
        CharacterInfo::Stabber {},
        CharacterInfo::BasicBullet {},
    ];
    let curr_id = match character_info {
        CharacterInfo::Player { .. } => 0,
        CharacterInfo::Goal { .. } => 1,
        CharacterInfo::Damager { .. } => 2,
        CharacterInfo::Stabber { .. } => 3,
        CharacterInfo::BasicBullet { .. } => 4,
        CharacterInfo::Shooter {} => 5,
    };
    let mut new_id = curr_id;
    ComboBox::new("info", "CharacterInfo")
        .selected_text(character_tys[curr_id])
        .show_ui(ui, |ui| {
            for (id, label) in character_tys.iter().enumerate() {
                ui.selectable_value(&mut new_id, id, *label);
            }
        });
    if curr_id != new_id {
        *character_info = defaults[new_id];
    }
    match character_info {
        CharacterInfo::Player {} => {
            ui.label("No data");
        }
        CharacterInfo::Goal {} => {
            ui.label("No data");
        }
        CharacterInfo::Damager {} => {
            ui.label("No data");
        }
        CharacterInfo::Stabber {} => {
            ui.label("No data");
        }
        CharacterInfo::BasicBullet {} => {
            ui.label("No data");
        }
        CharacterInfo::Shooter {} => {
            ui.label("No data");
        }
    }
}
