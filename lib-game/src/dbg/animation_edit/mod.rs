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

use crate::AnimationPlay;

pub struct AnimationEdit {
    pub playback: Entity,
    open_save_pack: bool,
    current_pack_id: AnimationPackId,
    sequencer_state: SequencerState,
    tf: TimelineTf,
    track_label: String,
    selected_clip: Option<u32>,
    selected_track: Option<u32>,
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
            track_label: String::new(),
            tf: TimelineTf {
                zoom: 1.0,
                pan: 0.0,
            },
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
                for (entity, _) in world.query_mut::<&mut AnimationPlay>() {
                    ui.selectable_value(&mut self.playback, entity, format!("{entity:?}"));
                }
            });
        let Ok(mut play) = world.get::<&mut AnimationPlay>(self.playback) else {
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

        let anim = anims.get_mut(&play.animation).unwrap();
        ui.horizontal(|ui| {
            animation_load_ui(ui, resolver, play.animation, anim);
        });
        enum_select(ui, "animation_id", "animation", &mut play.animation);
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
        clip_action_ui(clips.get_action_mut(clip_idx).unwrap(), ui);
    });
}

fn clip_action_ui(clip: &mut lib_anim::ClipAction, ui: &mut Ui) {
    match clip {
        lib_anim::ClipAction::DrawSprite {
            layer,
            texture_id: current_texture_id,
            local_pos,
            local_rotation,
            rect,
            origin,
            sort_offset,
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
                ui.add(DragValue::new(local_rotation).range(0.0..=std::f32::consts::TAU));
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
                ui.add(DragValue::new(&mut origin.x).range(-256.0..=256.0));
                ui.add(DragValue::new(&mut origin.y).range(-256.0..=256.0));
                ui.label("origin");
            });
            ui.horizontal(|ui| {
                ui.add(DragValue::new(sort_offset).range(-64.0..=64.0));
                ui.label("sort offset");
            });
        }
    }
}
