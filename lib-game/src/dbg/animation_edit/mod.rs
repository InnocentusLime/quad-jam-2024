mod clips;
mod save_ui;
mod sequencer;

use egui::{ComboBox, DragValue, Label, Modal, WidgetText, vec2};
use egui::{Ui, Widget};
use macroquad::math::Vec2;

use hashbrown::HashMap;
use hecs::{Entity, World};
use lib_asset::animation::*;
use lib_asset::level::CharacterInfo;
use lib_asset::{AnimationPackId, FsResolver, TextureId};
use strum::VariantArray;

use clips::*;
use save_ui::*;
use sequencer::*;

use crate::{AnimationPlay, CharacterLook};

pub struct AnimationEdit {
    pub playback: Entity,
    open_save_pack: bool,
    open_load_aseprite_modal: bool,
    current_pack_id: AnimationPackId,
    sequencer_state: SequencerState,
    tf: TimelineTf,
    selected_clip: Option<u32>,
    selected_track: Option<u32>,
    load_into_track: u32,
    layer_name: String,

    open_track_creation_modal: bool,
    track_label: String,
}

impl AnimationEdit {
    pub fn new() -> Self {
        Self {
            playback: Entity::DANGLING,
            open_save_pack: false,
            open_load_aseprite_modal: false,
            current_pack_id: AnimationPackId::Bunny,
            sequencer_state: SequencerState::Idle,
            selected_clip: None,
            selected_track: None,
            tf: TimelineTf {
                zoom: 1.0,
                pan: 0.0,
            },
            load_into_track: 0,
            layer_name: String::new(),
            track_label: String::new(),
            open_track_creation_modal: false,
        }
    }

    pub fn ui(
        &mut self,
        resolver: &FsResolver,
        ui: &mut Ui,
        anims: &mut HashMap<AnimationId, Animation>,
        world: &mut World,
    ) {
        let mut insert_pressed = false;
        let mut delete_pressed = false;
        let mut shift_down = false;
        ui.input(|st| {
            insert_pressed = st.key_pressed(egui::Key::Insert);
            delete_pressed = st.key_pressed(egui::Key::Delete);
            shift_down = st.modifiers.shift;
        });

        ComboBox::new("playback", "playback entity")
            .selected_text(format!("{:?}", self.playback))
            .show_ui(ui, |ui| {
                for (entity, _) in world.query_mut::<(&mut AnimationPlay, &mut CharacterLook)>() {
                    ui.selectable_value(&mut self.playback, entity, format!("{entity:?}"));
                }
            });
        let Ok(mut play_q) =
            world.query_one::<(&mut AnimationPlay, &mut CharacterLook)>(self.playback)
        else {
            self.playback = Entity::DANGLING;
            return;
        };
        let Some((play, look)) = play_q.get() else {
            self.playback = Entity::DANGLING;
            return;
        };

        ui.horizontal(|ui| {
            if ui.button("Load Pack").clicked() {
                load_anim_pack_ui(anims);
            }
            if ui.button("Save Pack").clicked() {
                self.open_save_pack = true;
            }
            if self.open_save_pack {
                self.open_save_pack = save_anim_pack_modal(ui, &mut self.current_pack_id, anims);
            }
        });

        let anim = anims.entry(play.animation).or_insert_with(Default::default);
        ui.horizontal(|ui| {
            animation_load_ui(
                ui,
                resolver,
                play.animation,
                anim,
                &mut self.open_load_aseprite_modal,
                &mut self.load_into_track,
                &mut self.layer_name,
            );
        });
        ui.horizontal(|ui| {
            ui.drag_angle(&mut look.0);
            ui.label("look");
        });
        enum_select(ui, "animation_id", "animation", &mut play.animation);
        ui.checkbox(&mut anim.is_looping, "is looping");
        ui.checkbox(&mut play.pause, "Pause");

        let mut clips = ClipsUi::new(&mut anim.tracks, &mut anim.clips);
        selected_clip_ui(ui, &mut clips, &mut self.selected_clip);

        self.track_creation_modal(&mut clips, ui);

        ui.horizontal(|_ui| {
            let add_clip = insert_pressed && !shift_down;
            if let Some(track_id) = self.selected_track
                && add_clip
            {
                clips.add_clip(track_id, play.cursor, 500);
            }

            let delete_track = delete_pressed && !shift_down;
            if let Some(idx) = self.selected_clip
                && delete_track
            {
                clips.delete_clip(idx);
            }
        });

        ui.horizontal(|_ui| {
            let add_track = insert_pressed && shift_down;
            if add_track && !self.open_track_creation_modal {
                self.open_track_creation_modal = true;
                self.track_label.clear();
            }

            let delete_track = delete_pressed && shift_down;
            if let Some(idx) = self.selected_track
                && delete_track
            {
                clips.delete_track(idx);
            }
        });

        Sequencer {
            state: &mut self.sequencer_state,
            clips: &mut clips,
            cursor_pos: &mut play.cursor,
            size: egui::vec2(500.0, 200.0),
            tf: &mut self.tf,
            selected_clip: &mut self.selected_clip,
            selected_track: &mut self.selected_track,
        }
        .ui(ui);
    }

    fn track_creation_modal(&mut self, clips: &mut ClipsUi, ui: &mut Ui) {
        if !self.open_track_creation_modal {
            return;
        }

        Modal::new(egui::Id::new("New track")).show(ui.ctx(), |ui| {
            ui.set_width(250.0);
            ui.heading("Create track");
            ui.text_edit_singleline(&mut self.track_label);

            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    self.open_track_creation_modal = false;
                    clips.add_track(self.track_label.clone());
                }
                if ui.button("Cancel").clicked() {
                    self.open_track_creation_modal = false;
                }
            });
        });
    }
}

fn selected_clip_ui(ui: &mut Ui, clips: &mut ClipsUi, selected_clip: &mut Option<u32>) {
    ui.group(|ui| {
        ui.set_min_size(vec2(200.0, 300.0));
        let Some(clip_idx) = *selected_clip else {
            ui.add_enabled(false, Label::new("No clip selected"));
            return;
        };
        let Some(clip) = clips.get(clip_idx) else {
            *selected_clip = None;
            return;
        };
        let track = clips.get_track(clip.track_id).unwrap();
        ui.label(format!("Track: {}", track.name));
        let track_y = clips.track_y(clip.track_id).unwrap();
        let (mut start, mut len) = (clip.start, clip.len);
        ui.horizontal(|ui| {
            DragValue::new(&mut start).ui(ui);
            ui.label("start");
        });
        ui.horizontal(|ui| {
            DragValue::new(&mut len).ui(ui);
            ui.label("len");
        });
        clips.set_clip_pos_len(clip.id, track_y, start, len);

        clip_action_ui(ui, clips.get_action_mut(clip_idx).unwrap());
    });
}

fn clip_action_ui(ui: &mut Ui, clip: &mut ClipAction) {
    let old_ty: ClipActionDiscriminants = (*clip).into();
    let mut new_ty = old_ty;
    enum_select(ui, "action_type", "Clip Action", &mut new_ty);
    if old_ty != new_ty {
        let new_clip = match new_ty {
            ClipActionDiscriminants::DrawSprite => ClipAction::DrawSprite(ClipActionDrawSprite {
                layer: 0,
                texture_id: TextureId::BunnyAtlas,
                local_pos: Vec2::ZERO,
                local_rotation: 0.0,
                rect: ImgRect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                },
                sort_offset: 0.0,
                rotate_with_parent: false,
            }),
            ClipActionDiscriminants::AttackBox => ClipAction::AttackBox(ClipActionAttackBox {
                local_pos: Vec2::ZERO,
                local_rotation: 0.0,
                team: Team::Player,
                group: lib_col::Group::empty(),
                shape: lib_col::Shape::Rect {
                    width: 0.0,
                    height: 0.0,
                },
                rotate_with_parent: false,
                graze_value: 0.0,
            }),
            ClipActionDiscriminants::Invulnerability => {
                ClipAction::Invulnerability(ClipActionInvulnerability)
            }
            ClipActionDiscriminants::LockInput => ClipAction::LockInput(ClipActionLockInput {
                allow_walk_input: false,
                allow_look_input: false,
            }),
            ClipActionDiscriminants::Move => ClipAction::Move(ClipActionMove),
            ClipActionDiscriminants::Spawn => ClipAction::Spawn(ClipActionSpawn {
                local_look: 0.0,
                local_pos: Vec2::ZERO,
                rotate_with_parent: false,
                character_info: CharacterInfo::BasicBullet {},
            }),
        };
        *clip = new_clip;
    }

    ui.separator();
    match clip {
        ClipAction::DrawSprite(ClipActionDrawSprite {
            layer,
            texture_id: current_texture_id,
            local_pos,
            local_rotation,
            rect,
            sort_offset,
            rotate_with_parent,
        }) => {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(layer).range(0..=10));
                ui.label("layer");
            });
            ComboBox::new("texture_id", "texture")
                .selected_text(format!("{current_texture_id:?}"))
                .show_ui(ui, |ui| {
                    for texture_id in TextureId::VARIANTS {
                        let name: &'static str = texture_id.into();
                        ui.selectable_value(current_texture_id, *texture_id, name);
                    }
                });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut local_pos.x).range(-256.0..=256.0));
                ui.add(DragValue::new(&mut local_pos.y).range(-256.0..=256.0));
                ui.label("local pos");
            });
            ui.horizontal(|ui| {
                ui.drag_angle(local_rotation);
                ui.label("local rotation");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut rect.x).range(0..=512));
                ui.add(DragValue::new(&mut rect.y).range(0..=512));
                ui.label("texture rect pos");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut rect.w).range(0..=512));
                ui.add(DragValue::new(&mut rect.h).range(0..=512));
                ui.label("texture rect size");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(sort_offset).range(-64.0..=64.0));
                ui.label("sort offset");
            });
            ui.checkbox(rotate_with_parent, "rotate with parent");
        }
        ClipAction::AttackBox(ClipActionAttackBox {
            local_pos,
            local_rotation,
            team,
            group,
            rotate_with_parent,
            graze_value,
            shape,
        }) => {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut local_pos.x).range(-256.0..=256.0));
                ui.add(DragValue::new(&mut local_pos.y).range(-256.0..=256.0));
                ui.label("local pos");
            });
            ui.horizontal(|ui| {
                ui.drag_angle(local_rotation);
                ui.label("local rotation");
            });
            ui.horizontal(|ui| {
                enum_select(ui, "team_id", "team", team);
            });
            ui.horizontal(|ui| {
                group_ui(ui, group);
                ui.label("group");
            });
            ui.checkbox(rotate_with_parent, "rotate with parent");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(graze_value).range(0.0..=30.0));
                ui.label("graze value");
            });
            shape_ui(ui, shape);
        }
        ClipAction::Invulnerability(_) => {
            ui.label("No data");
        }
        ClipAction::LockInput(ClipActionLockInput {
            allow_walk_input,
            allow_look_input,
        }) => {
            ui.checkbox(allow_walk_input, "allow walk input");
            ui.checkbox(allow_look_input, "allow look input");
        }
        ClipAction::Move(_) => {
            ui.label("No data");
        }
        ClipAction::Spawn(ClipActionSpawn {
            local_pos,
            local_look,
            rotate_with_parent,
            character_info,
        }) => {
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut local_pos.x).range(-256.0..=256.0));
                ui.add(DragValue::new(&mut local_pos.y).range(-256.0..=256.0));
                ui.label("local pos");
            });
            ui.horizontal(|ui| {
                ui.drag_angle(local_look);
                ui.label("local look");
            });
            ui.checkbox(rotate_with_parent, "rotate with parent");
            character_info_ui(ui, character_info);
        }
    }
}

fn group_ui(ui: &mut Ui, group: &mut lib_col::Group) {
    let response = ui.button("Configure");
    let flags_ui = |ui: &mut Ui| {
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

fn group_flags_ui(ui: &mut Ui, group: &mut lib_col::Group) {
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

fn shape_ui(ui: &mut Ui, shape: &mut lib_col::Shape) {
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

fn character_info_ui(ui: &mut Ui, character_info: &mut CharacterInfo) {
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

fn enum_select<T>(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    label: impl Into<WidgetText>,
    current_value: &mut T,
) where
    T: Copy + PartialEq,
    T: strum::VariantArray,
    for<'a> &'a T: Into<&'static str>,
{
    let current_text: &'static str = (&*current_value).into();
    ComboBox::new(id_salt, label)
        .selected_text(current_text)
        .show_ui(ui, |ui| {
            for selected_value in T::VARIANTS {
                let text: &'static str = selected_value.into();
                ui.selectable_value(current_value, *selected_value, text);
            }
        });
}
