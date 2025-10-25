mod clips;

use egui::{epaint, vec2, Button, ComboBox, DragValue, Label, TextEdit};
use egui::{Color32, Key, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget, pos2};

pub use clips::*;
use hashbrown::HashMap;
use hecs::{Entity, World};
use lib_anim::{Animation, AnimationId};
use lib_asset::TextureId;
use strum::VariantArray;

use crate::AnimationPlay;

pub const PIXELS_PER_UNIT: f32 = 18.0;

pub struct AnimationEdit {
    pub target: Entity,
    freeze_play: bool,
    current_anim_id: AnimationId,
    clip_actions: HashMap<u32, lib_anim::ClipAction>,
    sequencer_state: SequencerState,
    tf: TimelineTf,
    clips: Clips,
    cursor_pos: u32,
    clip_label: String,
    track_label: String,
    track_color: [f32; 3],
    selected_clip: Option<u32>,
    selected_track: Option<u32>,
}

impl AnimationEdit {
    pub fn new() -> Self {
        Self {
            target: Entity::DANGLING,
            freeze_play: true,
            current_anim_id: AnimationId::BunnyWalkD,
            clip_actions: HashMap::new(),
            sequencer_state: SequencerState::Idle,
            clips: Clips::new(),
            cursor_pos: 0,
            clip_label: String::new(),
            selected_clip: None,
            selected_track: None,
            track_label: String::new(),
            track_color: [1.0; 3],
            tf: TimelineTf {
                zoom: 1.0,
                pan: 0.0,
            },
        }
    }

    pub fn animation_id(&self) -> AnimationId {
        self.current_anim_id
    }

    pub fn set_to_anim(&mut self, anims: &HashMap<AnimationId, Animation>) {
        let anim = &anims[&self.current_anim_id];
        self.clips.clear();
        self.clip_actions.clear();

        let mut last_track_id = self.clips.add_track("test".into(), Color32::RED);
        'clip_loop: for clip in anim.clips.iter() {
            for track_id in 0..=last_track_id {
                let Some(id) = self.clips.add_clip(track_id, "clip".into(), clip.start, clip.len) else {
                    break;
                };
                self.clip_actions.insert(id, clip.action);
                continue 'clip_loop;
            }
            last_track_id = self.clips.add_track("test".into(), Color32::RED);
            let id = self.clips.add_clip(last_track_id, "clip".into(), clip.start, clip.len).unwrap();
            self.clip_actions.insert(id, clip.action);
        }
    }

    pub fn write_back(&self, anims: &mut HashMap<AnimationId, Animation>) {
        let Some(anim) = anims.get_mut(&self.current_anim_id) else {
            return;
        };
        anim.clips.clear();
        anim.clips.extend(self.clips.iter_clips()
            .enumerate()
            .map(|(idx, clip)| lib_anim::Clip { 
                id: idx as u32, 
                start: clip.pos, 
                len: clip.len, 
                // FIXME: this can panic
                action: self.clip_actions[&(idx as u32)], 
            })
        )
    }
    
    pub fn ui(&mut self, ui: &mut Ui, anims: &mut HashMap<AnimationId, Animation>, world: &mut World) {
        ComboBox::new("target_id", "target_entity")
            .selected_text(format!("{:?}", self.target))
            .show_ui(ui, |ui| {
                for (entity, _) in world.query_mut::<&mut AnimationPlay>() {
                    ui.selectable_value(
                        &mut self.target, 
                        entity, 
                        format!("{entity:?}"),
                    );
                }
            });
        
        if self.target == Entity::DANGLING {
            return;
        }

        if let Ok(mut play) = world.get::<&mut AnimationPlay>(self.target) {
            play.pause = self.freeze_play;
            self.cursor_pos = play.cursor;
        } else {
            return;
        }
        
        let pick = ComboBox::new("animation_id", "animation")
            .selected_text(format!("{:?}", self.current_anim_id))
            .show_ui(ui, |ui| {
                for anim_id in AnimationId::VARIANTS {
                    let name: &'static str = anim_id.into();
                    let resp = ui.selectable_value(
                        &mut self.current_anim_id,
                        *anim_id, 
                        name
                    );
                    if resp.clicked() {
                        return true
                    }
                }
                false
            });
        if pick.inner.unwrap_or_default() {
            self.set_to_anim(anims);
        }

        ui.checkbox(&mut self.freeze_play, "Freeze");

        ui.group(|ui| {
            ui.set_min_size(vec2(200.0, 150.0));
            if let Some(clip_idx) = self.selected_clip {
                match self.clips.get(clip_idx) {
                    None => self.selected_clip = None,
                    Some(clip) => {
                        ui.label(clip.label.clone());
                        ui.label(format!("Track: {}", clip.track_id));
                        ui.label(format!("Pos: {}", clip.pos));
                        ui.label(format!("Length: {}", clip.len));
                        if let Some(act) = self.clip_actions.get_mut(&clip_idx) {
                            clip_action_ui(act, ui);
                        }
                    }
                }
            } else {
                ui.add_enabled(false, Label::new("No clip selected"));
            }
        });

        ui.horizontal(|ui| {
            TextEdit::singleline(&mut self.clip_label)
                .desired_width(150.0)
                .ui(ui);

            let resp =
                ui.add_enabled(self.selected_track.is_some(), Button::new("add clip"));
            if let Some(track_id) = self.selected_track {
                if resp.clicked() {
                    let id = self.clips.add_clip(
                        track_id,
                        self.clip_label.as_str().into(),
                        self.cursor_pos,
                        30,
                    );
                    self.clip_actions.insert(id.unwrap(), lib_anim::ClipAction::DrawSprite { 
                        layer: 0, 
                        texture_id: TextureId::WorldAtlas, 
                        local_pos: lib_anim::Position { x: 0.0, y: 0.0 }, 
                        local_rotation: 0.0, 
                        rect: lib_anim::ImgRect { x: 0, y: 0, w: 0, h: 0 }, 
                        origin: lib_anim::Position { x: 0.0, y: 0.0 }, 
                        sort_offset: 0.0, 
                    });
                }
            }

            let resp =
                ui.add_enabled(self.selected_clip.is_some(), Button::new("delete clip"));
            if let Some(idx) = self.selected_clip {
                if resp.clicked() {
                    self.clips.delete_clip(idx);
                }
            }
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_rgb(&mut self.track_color);
            TextEdit::singleline(&mut self.track_label)
                .desired_width(100.0)
                .ui(ui);

            if ui.button("Add track").clicked() {
                self.clips.add_track(self.track_label.clone().into(), Color32::from_rgb(
                    (self.track_color[0] * 255.0) as u8, 
                    (self.track_color[1] * 255.0) as u8, 
                    (self.track_color[2] * 255.0) as u8,
                ));
            }

            let resp =
                ui.add_enabled(self.selected_track.is_some(), Button::new("delete track"));
            if let Some(idx) = self.selected_track {
                if resp.clicked() {
                    self.clips.delete_track(idx);
                }
            }
        });

        Sequencer {
            state: &mut self.sequencer_state,
            clips: &mut self.clips,
            cursor_pos: &mut self.cursor_pos,
            size: Vec2::new(500.0, 200.0),
            tf: &mut self.tf,
            selected_clip: &mut self.selected_clip,
            selected_track: &mut self.selected_track,
        }
        .ui(ui);
        
        self.write_back(anims);
       
        if let Ok(mut play) = world.get::<&mut AnimationPlay>(self.target) {
            play.cursor = self.cursor_pos;
            play.animation = self.animation_id();
        }
    }
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
                        ui.selectable_value(
                            current_texture_id,
                            *texture_id, 
                            name
                        );
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
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimelineTf {
    pub zoom: f32,
    pub pan: f32,
}

impl TimelineTf {
    pub fn tf_pos(&self, pos: f32) -> f32 {
        self.zoom * (pos + self.pan)
    }

    pub fn tf_vector(&self, vec: f32) -> f32 {
        self.zoom * vec
    }

    pub fn inv_tf_vector(&self, vec: f32) -> f32 {
        vec / self.zoom
    }

    pub fn inv_tf_pos(&self, pos: f32) -> f32 {
        pos / self.zoom - self.pan
    }
}

#[derive(Debug)]
pub enum SequencerState {
    Idle,
    MoveClip {
        clip_id: u32,
        start_pos_x: f32,
        start_pos_y: f32,
        total_drag_delta_x: f32,
        total_drag_delta_y: f32,
    },
    ResizeClip {
        clip_id: u32,
        start_left: f32,
        start_right: f32,
        resize_left: bool,
        total_drag_delta: f32,
    },
    Pan {
        start_pan: f32,
        total_drag_delta: f32,
    },
}

pub struct Sequencer<'a> {
    pub cursor_pos: &'a mut u32,
    pub selected_clip: &'a mut Option<u32>,
    pub selected_track: &'a mut Option<u32>,
    pub clips: &'a mut Clips,
    pub state: &'a mut SequencerState,
    pub tf: &'a mut TimelineTf,
    pub size: Vec2,
}

impl<'a> Sequencer<'a> {
    fn timeline_input(&mut self, ui: &mut Ui, response: &Response, timeline_rect: Rect) {
        let Some(pointer) = response.hover_pos() else {
            return;
        };

        if let Some(selected_clip) = self.selected_clip.clone() {
            if self.clips.get(selected_clip).is_none() {
                *self.selected_clip = None;
            }
        }

        match *self.state {
            SequencerState::Idle => self.timeline_input_idle(ui, response, timeline_rect, pointer),
            SequencerState::MoveClip {
                clip_id,
                start_pos_x,
                start_pos_y,
                total_drag_delta_x,
                total_drag_delta_y,
            } => self.timeline_input_moving_clip(
                ui,
                response,
                clip_id,
                start_pos_x,
                start_pos_y,
                total_drag_delta_x,
                total_drag_delta_y,
            ),
            SequencerState::ResizeClip {
                clip_id,
                start_left,
                start_right,
                total_drag_delta,
                resize_left,
            } => self.timeline_input_resizing_clip(
                ui,
                response,
                clip_id,
                start_left,
                start_right,
                total_drag_delta,
                resize_left,
            ),
            SequencerState::Pan {
                start_pan,
                total_drag_delta,
            } => self.timeline_input_pan(ui, start_pan, total_drag_delta),
        }
    }

    fn timeline_input_idle(
        &mut self,
        ui: &mut Ui,
        response: &Response,
        timeline_rect: Rect,
        pointer: Pos2,
    ) {
        self.timeline_input_idle_clips(ui, timeline_rect, pointer);
        self.timeline_input_idle_pan_and_zoom(ui);
        self.timeline_input_idle_cursor(response, timeline_rect);
    }

    fn timeline_input_idle_clips(&mut self, ui: &mut Ui, timeline_rect: Rect, pointer: Pos2) {
        let left_button_down = ui
            .ctx()
            .input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        let x_pos = self.tf.inv_tf_pos(pointer.x - timeline_rect.left()).round() as u32;
        let y_pos = ((pointer.y - timeline_rect.top()) / CLIP_HEIGHT) as u32;
        let clip = self.clips.clip_containing_pos(y_pos, x_pos);

        if left_button_down {
            *self.selected_clip = clip.map(|x| x.id);
        }

        if let Some(clip) = clip {
            let track_y = self.clips.track_y(clip.track_id).unwrap();
            let action = clip.pointer_action(timeline_rect, pointer, *self.tf, track_y);
            Self::clip_action_to_cursor(ui, action);
            if left_button_down {
                *self.state = Self::clip_action_to_new_state(track_y, clip, action);
            }
        }
    }

    fn clip_action_to_cursor(ui: &mut Ui, action: ClipAction) {
        match action {
            ClipAction::Move => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
            ClipAction::Resize { .. } => {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal)
            }
        }
    }

    fn clip_action_to_new_state(track_y: u32, clip: &Clip, action: ClipAction) -> SequencerState {
        match action {
            ClipAction::Move => SequencerState::MoveClip {
                clip_id: clip.id,
                start_pos_x: clip.pos as f32,
                start_pos_y: track_y as f32 * CLIP_HEIGHT,
                total_drag_delta_x: 0.0f32,
                total_drag_delta_y: 0.0f32,
            },
            ClipAction::Resize { resize_left } => SequencerState::ResizeClip {
                clip_id: clip.id,
                resize_left,
                start_left: clip.pos as f32,
                start_right: (clip.pos + clip.len) as f32,
                total_drag_delta: 0.0f32,
            },
        }
    }

    fn timeline_input_idle_pan_and_zoom(&mut self, ui: &mut Ui) {
        let mut middle_button_down = false;
        let mut space_down = false;
        let mut plus_pressed = false;
        let mut minus_pressed = false;
        ui.ctx().input(|i| {
            middle_button_down = i.pointer.button_down(egui::PointerButton::Middle);
            space_down = i.key_down(Key::Space);
            // Due to miniquad stupidity, we must use Key::Plus
            plus_pressed = i.key_pressed(Key::Equals);
            minus_pressed = i.key_pressed(Key::Minus);
        });

        if plus_pressed {
            self.tf.zoom *= 1.3f32;
        } else if minus_pressed {
            self.tf.zoom /= 1.3f32;
        }

        if !middle_button_down && !space_down {
            return;
        }

        *self.state = SequencerState::Pan {
            start_pan: self.tf.pan,
            total_drag_delta: 0.0,
        };
    }

    fn timeline_input_idle_cursor(&mut self, response: &Response, timeline_rect: Rect) {
        let Some(pos) = response.interact_pointer_pos() else {
            return;
        };
        let y_pos = ((pos.y - timeline_rect.top()) / CLIP_HEIGHT) as u32;
        if response.clicked() {
            *self.cursor_pos = self.tf.inv_tf_pos(pos.x - timeline_rect.left()).round() as u32;
            *self.selected_track = self.clips.track_containing_pos(y_pos);
        }
    }

    fn timeline_input_moving_clip(
        &mut self,
        ui: &mut Ui,
        response: &Response,
        clip_id: u32,
        start_pos_x: f32,
        start_pos_y: f32,
        mut total_drag_delta_x: f32,
        mut total_drag_delta_y: f32,
    ) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        let left_button_down = ui
            .ctx()
            .input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        if !response.is_pointer_button_down_on() || !left_button_down {
            *self.state = SequencerState::Idle;
            return;
        }
        total_drag_delta_x += response.drag_delta().x;
        total_drag_delta_y += response.drag_delta().y;

        let Some(clip) = self.clips.get(clip_id) else {
            *self.state = SequencerState::Idle;
            return;
        };
        let new_track_y = ((start_pos_y + total_drag_delta_y).max(0.0) / CLIP_HEIGHT) as u32;
        let new_pos = (start_pos_x + self.tf.inv_tf_vector(total_drag_delta_x)) as u32;
        let new_len = clip.len;
        self.clips
            .set_clip_pos_len(clip_id, new_track_y, new_pos, new_len);
        *self.state = SequencerState::MoveClip {
            clip_id,
            start_pos_x,
            start_pos_y,
            total_drag_delta_x,
            total_drag_delta_y,
        }
    }

    fn timeline_input_resizing_clip(
        &mut self,
        ui: &mut Ui,
        response: &Response,
        clip_id: u32,
        start_left: f32,
        start_right: f32,
        mut total_drag_delta: f32,
        resize_left: bool,
    ) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        let left_button_down = ui
            .ctx()
            .input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        if !response.is_pointer_button_down_on() && !left_button_down {
            *self.state = SequencerState::Idle;
            return;
        }
        total_drag_delta += response.drag_delta().x;

        let final_size_delta = self.tf.inv_tf_vector(total_drag_delta);
        let (mut final_left, mut final_right) = (start_left, start_right);
        if resize_left {
            final_left = f32::min(final_right - 1.0, final_left + final_size_delta);
        } else {
            final_right = f32::max(final_left + 1.0, final_right + final_size_delta);
        };

        let Some(clip) = self.clips.get(clip_id) else {
            *self.state = SequencerState::Idle;
            return;
        };
        // We want to keep clip.len + clip.pos the same so
        // the right doesn't jitter
        let new_len = if resize_left {
            (clip.pos + clip.len) as f32 - final_left.round()
        } else {
            (final_right - final_left).round()
        };
        let new_pos = final_left.round() as u32;
        let new_track_y = self.clips.track_y(clip.track_id).unwrap();
        self.clips
            .set_clip_pos_len(clip_id, new_track_y, new_pos, new_len as u32);
        *self.state = SequencerState::ResizeClip {
            clip_id,
            start_left,
            start_right,
            resize_left,
            total_drag_delta,
        }
    }

    fn timeline_input_pan(&mut self, ui: &mut Ui, start_pan: f32, mut total_drag_delta: f32) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        let mut middle_button_down = false;
        let mut space_down = false;
        ui.ctx().input(|i| {
            middle_button_down = i.pointer.button_down(egui::PointerButton::Middle);
            space_down = i.key_down(Key::Space);
        });
        if !middle_button_down && !space_down {
            *self.state = SequencerState::Idle;
            return;
        }
        total_drag_delta += ui.ctx().input(|inp| inp.pointer.delta().x);
        self.tf.pan = start_pan + self.tf.inv_tf_vector(total_drag_delta);
        self.tf.pan = self.tf.pan.min(0.0);

        *self.state = SequencerState::Pan {
            start_pan,
            total_drag_delta,
        }
    }

    fn paint_timeline(&self, ui: &Ui, painter: &Painter, timeline_rect: Rect) {
        painter.rect_filled(timeline_rect, 0.0, ui.visuals().noninteractive().bg_fill);

        let track = self.selected_track.and_then(|idx| self.clips.get_track(idx));
        if let Some(track) = track {
            let dark_color = Color32::BLACK + track.color.additive().linear_multiply(0.05);
            // let dark_color = Color32::WHITE;
            let track_y = self.clips.track_y(track.id).unwrap();
            let track_selection = Rect::from_min_size(
                pos2(timeline_rect.left(), timeline_rect.top() + (track_y as f32) * CLIP_HEIGHT), 
                vec2(timeline_rect.width(), CLIP_HEIGHT),
            );
            painter.rect_filled(track_selection, 0.0, dark_color); 
        }

        let mut last_painted = None;
        let mut painted_count = 0;
        let contigious_paint = self.tf.tf_vector(1.0) >= PIXELS_PER_UNIT;
        for section in 1..300 {
            let local_pos = self.tf.tf_pos(section as f32);
            let too_close = last_painted
                .map(|x| local_pos - x < PIXELS_PER_UNIT)
                .unwrap_or(false);
            if too_close {
                continue;
            }

            let mark_x = timeline_rect.left() + local_pos;
            let mark_points = [
                pos2(mark_x, timeline_rect.top()),
                pos2(mark_x, timeline_rect.bottom()),
            ];
            let color = if painted_count % 5 == 0 || contigious_paint {
                ui.visuals().weak_text_color()
            } else {
                ui.visuals().extreme_bg_color
            };
            if local_pos >= 0.0 {
                painter.line_segment(mark_points, Stroke::new(1.0, color));
            }

            painted_count += 1;
            last_painted = Some(local_pos);
        }

        painter.rect(
            timeline_rect,
            0.0,
            Color32::TRANSPARENT,
            ui.visuals().noninteractive().fg_stroke,
            epaint::StrokeKind::Inside,
        );
    }

    fn paint_timeline_cursor(&self, painter: &Painter, timeline_rect: Rect) {
        let cur_x = self.tf.tf_pos(*self.cursor_pos as f32) + timeline_rect.left();
        painter.line_segment(
            [
                pos2(cur_x, timeline_rect.top()),
                pos2(cur_x, timeline_rect.bottom()),
            ],
            Stroke::new(1.0, Color32::RED),
        );
    }
}

impl<'a> Widget for Sequencer<'a> {
    fn ui(mut self, ui: &mut Ui) -> egui::Response {
        let (response, mut painter) = ui.allocate_painter(self.size, Sense::click_and_drag());
        let widget_rect = response.rect;
        if !ui.is_rect_visible(widget_rect) {
            return response;
        }

        self.clips.paint_track_labels(ui, &painter, widget_rect);

        let mut timeline_rect = widget_rect;
        timeline_rect.set_left(timeline_rect.left() + TRACK_LABEL_WIDTH);
        painter.set_clip_rect(timeline_rect);

        self.timeline_input(ui, &response, timeline_rect);
        self.paint_timeline(ui, &painter, timeline_rect);

        painter.set_clip_rect(timeline_rect.shrink(1.0));
        self.clips
            .paint_clips(ui, &painter, timeline_rect, *self.tf, *self.selected_clip);
        self.paint_timeline_cursor(&painter, timeline_rect);

        response
    }
}
