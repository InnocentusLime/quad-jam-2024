mod clips;
mod save_ui;
mod sequencer;

use std::any::TypeId;

use egui::{Button, ComboBox, DragValue, Label, Modal, Response, WidgetText, vec2};
use egui::{Ui, Widget};
use macroquad::math::Vec2;

use hashbrown::HashMap;
use hecs::{Entity, World};
use lib_asset::AnimationPackId;

use save_ui::*;
use sequencer::*;

use crate::animation::Animation;
use crate::{AnimationId, AnimationPlay, AttackBox, CLIP_TYPES, CharacterLook};

pub struct AnimationEdit {
    pub playback: Entity,
    open_save_pack: bool,
    current_pack_id: AnimationPackId,
    sequencer_state: SequencerState,
    tf: TimelineTf,
    selected_clip: Option<(TypeId, u32)>,
    selected_track: Option<(TypeId, u32)>,

    open_track_creation_modal: bool,
    track_kind: TypeId,
    track_label: String,

    open_global_offset_modal: bool,
    global_offset: Vec2,
}

impl AnimationEdit {
    pub fn new() -> Self {
        Self {
            playback: Entity::DANGLING,
            open_save_pack: false,
            current_pack_id: AnimationPackId::Bunny,
            sequencer_state: SequencerState::Idle,
            selected_clip: None,
            selected_track: None,
            track_kind: TypeId::of::<AttackBox>(),
            tf: TimelineTf {
                zoom: 1.0,
                pan: 0.0,
            },
            track_label: String::new(),
            open_track_creation_modal: false,
            open_global_offset_modal: false,
            global_offset: Vec2::ZERO,
        }
    }

    pub fn ui(
        &mut self,
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
            animation_load_ui(ui, play.animation, anim);
        });
        ui.horizontal(|ui| {
            ui.drag_angle(&mut look.0);
            ui.label("look");
        });
        enum_select(ui, "animation_id", "animation", &mut play.animation);
        ui.checkbox(&mut anim.is_looping, "is looping");
        ui.checkbox(&mut play.pause, "Pause");

        let poen_global_offset_button = Button::new("Global offset");
        let global_offset_resp =
            ui.add_enabled(!self.open_global_offset_modal, poen_global_offset_button);
        if global_offset_resp.clicked() {
            self.open_global_offset_modal = true;
        }

        selected_clip_ui(ui, anim, &mut self.selected_clip);

        self.track_creation_modal(anim, ui);
        self.global_offset_modal(&global_offset_resp, anim, ui);

        ui.horizontal(|_ui| {
            let add_clip = insert_pressed && !shift_down;
            if let Some((kind, track_id)) = self.selected_track
                && add_clip
            {
                anim.add_clip(kind, track_id, play.cursor, 500);
            }

            let delete_track = delete_pressed && !shift_down;
            if let Some((kind, idx)) = self.selected_clip
                && delete_track
            {
                anim.delete_clip(kind, idx);
            }
        });

        ui.horizontal(|_ui| {
            let add_track = insert_pressed && shift_down;
            if add_track && !self.open_track_creation_modal {
                self.open_track_creation_modal = true;
                self.track_label.clear();
            }

            let delete_track = delete_pressed && shift_down;
            if let Some((kind, idx)) = self.selected_track
                && delete_track
            {
                anim.delete_track(kind, idx);
            }
        });

        Sequencer {
            state: &mut self.sequencer_state,
            anim,
            cursor_pos: &mut play.cursor,
            size: egui::vec2(500.0, 200.0),
            tf: &mut self.tf,
            selected_clip: &mut self.selected_clip,
            selected_track: &mut self.selected_track,
        }
        .ui(ui);
    }

    fn track_creation_modal(&mut self, anim: &mut Animation, ui: &mut Ui) {
        if !self.open_track_creation_modal {
            return;
        }

        Modal::new(egui::Id::new("New track")).show(ui.ctx(), |ui| {
            ui.set_width(250.0);
            ui.heading("Create track");
            ui.text_edit_singleline(&mut self.track_label);
            let action_track = &anim.action_tracks[&self.track_kind];
            let current_text = action_track.manifest_key();
            ComboBox::new("track-kind", current_text)
                .selected_text(current_text)
                .show_ui(ui, |ui| {
                    for selected_value in CLIP_TYPES {
                        let selected_text = anim.action_tracks[&selected_value].manifest_key();
                        ui.selectable_value(&mut self.track_kind, selected_value, selected_text);
                    }
                });

            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    self.open_track_creation_modal = false;
                    anim.add_track(self.track_kind, self.track_label.clone());
                }
                if ui.button("Cancel").clicked() {
                    self.open_track_creation_modal = false;
                }
            });
        });
    }

    fn global_offset_modal(&mut self, response: &Response, anim: &mut Animation, ui: &mut Ui) {
        let popup_id = ui.make_persistent_id("offset_modal");
        if response.clicked() {
            self.global_offset = Vec2::ZERO;
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        let popup_res = egui::popup_above_or_below_widget(
            ui,
            popup_id,
            response,
            egui::AboveOrBelow::Below,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ui.horizontal(|ui| {
                    anim.global_offset(-self.global_offset);
                    ui.add(DragValue::new(&mut self.global_offset.x).range(-256.0..=256.0));
                    ui.add(DragValue::new(&mut self.global_offset.y).range(-256.0..=256.0));
                    ui.label("global offset");
                    anim.global_offset(self.global_offset);
                });

                if ui.button("Apply").clicked() {
                    // Close and apply the global offset
                    return true;
                }

                // Do not close and do not apply
                false
            },
        );

        match popup_res {
            None if self.open_global_offset_modal => {
                anim.global_offset(-self.global_offset);
                self.open_global_offset_modal = false;
            }
            Some(true) => {
                self.open_global_offset_modal = false;
                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
            }
            _ => self.open_global_offset_modal = popup_res.is_some(),
        }
    }
}

fn selected_clip_ui(ui: &mut Ui, anim: &mut Animation, selected_clip: &mut Option<(TypeId, u32)>) {
    ui.group(|ui| {
        ui.set_min_size(vec2(200.0, 300.0));
        let Some((kind, clip_id)) = *selected_clip else {
            ui.add_enabled(false, Label::new("No clip selected"));
            return;
        };
        let Some(clip) = anim.get_clip(kind, clip_id) else {
            *selected_clip = None;
            return;
        };
        let track = anim.get_track(kind, clip.track_id).unwrap();
        ui.label(format!("Track: {}", track.name));
        let (mut start, mut len) = (clip.start, clip.len);
        ui.horizontal(|ui| {
            DragValue::new(&mut start).ui(ui);
            ui.label("start");
        });
        ui.horizontal(|ui| {
            DragValue::new(&mut len).ui(ui);
            ui.label("len");
        });
        anim.set_clip_pos_len(kind, clip_id, clip.track_id, start, len);

        ui.separator();
        anim.clip_editor_ui(kind, clip_id, ui);
    });
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
