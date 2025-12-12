mod clips;
mod save_ui;
mod sequencer;

use egui::{Button, ComboBox, DragValue, Label, TextEdit, WidgetText, vec2};
use egui::{Ui, Vec2, Widget};

use hashbrown::HashMap;
use hecs::{Entity, World};
use lib_anim::{Animation, AnimationId, AnimationPackId};
use lib_asset::{FsResolver, TextureId};
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
    track_label: String,
    selected_clip: Option<u32>,
    selected_track: Option<u32>,
    load_into_track: u32,
    layer_name: String,
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
            track_label: String::new(),
            tf: TimelineTf {
                zoom: 1.0,
                pan: 0.0,
            },
            load_into_track: 0,
            layer_name: String::new(),
        }
    }

    pub fn ui(
        &mut self,
        resolver: &FsResolver,
        ui: &mut Ui,
        anims: &mut HashMap<AnimationId, Animation>,
        world: &mut World,
    ) {
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
                self.open_save_pack = save_anim_pack_modal(ui, &mut self.current_pack_id, &anims);
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

        ui.horizontal(|ui| {
            let add_resp = ui.add_enabled(self.selected_track.is_some(), Button::new("add clip"));
            if let Some(track_id) = self.selected_track
                && add_resp.clicked()
            {
                clips.add_clip(track_id, play.cursor, 500);
            }

            let delete_resp =
                ui.add_enabled(self.selected_clip.is_some(), Button::new("delete clip"));
            let delete_pressed = ui.input(|input| input.key_pressed(egui::Key::Delete));
            if let Some(idx) = self.selected_clip
                && (delete_resp.clicked() || delete_pressed)
            {
                clips.delete_clip(idx);
            }
        });

        ui.horizontal(|ui| {
            TextEdit::singleline(&mut self.track_label)
                .desired_width(100.0)
                .ui(ui);

            if ui.button("Add track").clicked() {
                clips.add_track(self.track_label.clone().into());
            }

            let resp = ui.add_enabled(self.selected_track.is_some(), Button::new("delete track"));
            if let Some(idx) = self.selected_track
                && resp.clicked()
            {
                clips.delete_track(idx);
            }
        });

        Sequencer {
            state: &mut self.sequencer_state,
            clips: &mut clips,
            cursor_pos: &mut play.cursor,
            size: Vec2::new(500.0, 200.0),
            tf: &mut self.tf,
            selected_clip: &mut self.selected_clip,
            selected_track: &mut self.selected_track,
        }
        .ui(ui);
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
        ui.label(ClipWidget(clip).label());
        ui.label(format!("Track: {}", clip.track_id));
        ui.label(format!("Pos: {}", clip.start));
        ui.label(format!("Length: {}", clip.len));
        clip_action_ui(ui, clips.get_action_mut(clip_idx).unwrap());
    });
}

fn clip_action_ui(ui: &mut Ui, clip: &mut lib_anim::ClipAction) {
    let old_ty: lib_anim::ClipActionDiscriminants = (*clip).into();
    let mut new_ty = old_ty;
    enum_select(ui, "action_type", "Clip Action", &mut new_ty);
    if old_ty != new_ty {
        let new_clip = match new_ty {
            lib_anim::ClipActionDiscriminants::DrawSprite => lib_anim::ClipAction::DrawSprite {
                layer: 0,
                texture_id: TextureId::BunnyAtlas,
                local_pos: lib_anim::Position { x: 0.0, y: 0.0 },
                local_rotation: 0.0,
                rect: lib_anim::ImgRect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                },
                sort_offset: 0.0,
                rotate_with_parent: false,
            },
            lib_anim::ClipActionDiscriminants::AttackBox => lib_anim::ClipAction::AttackBox {
                local_pos: lib_anim::Position { x: 0.0, y: 0.0 },
                local_rotation: 0.0,
                team: lib_anim::Team::Player,
                group: lib_col::Group::empty(),
                shape: lib_col::Shape::Rect {
                    width: 0.0,
                    height: 0.0,
                },
                rotate_with_parent: false,
            },
            lib_anim::ClipActionDiscriminants::Invulnerability => {
                lib_anim::ClipAction::Invulnerability
            }
            lib_anim::ClipActionDiscriminants::LockInput => lib_anim::ClipAction::LockInput {
                allow_walk_input: false,
                allow_look_input: false,
            },
            lib_anim::ClipActionDiscriminants::Move => lib_anim::ClipAction::Move,
        };
        *clip = new_clip;
    }

    match clip {
        lib_anim::ClipAction::DrawSprite {
            layer,
            texture_id: current_texture_id,
            local_pos,
            local_rotation,
            rect,
            sort_offset,
            rotate_with_parent,
        } => {
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
        lib_anim::ClipAction::AttackBox {
            local_pos,
            local_rotation,
            team,
            group,
            rotate_with_parent,
            shape,
        } => {
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
                ui.label("team");
            });
            ui.horizontal(|ui| {
                group_ui(ui, group);
                ui.label("group");
            });
            ui.checkbox(rotate_with_parent, "rotate with parent");
            shape_ui(ui, shape);
        }
        lib_anim::ClipAction::Invulnerability => {
            ui.label("No data");
        }
        lib_anim::ClipAction::LockInput {
            allow_walk_input,
            allow_look_input,
        } => {
            ui.checkbox(allow_walk_input, "allow walk input");
            ui.checkbox(allow_look_input, "allow look input");
        }
        lib_anim::ClipAction::Move => {
            ui.label("No data");
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
