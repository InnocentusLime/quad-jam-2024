use std::any::TypeId;

use crate::animation::{Animation, Clip};
use egui::{Color32, Painter, Pos2, Rect, Stroke, TextStyle, Ui, WidgetText, pos2, vec2};

use super::TimelineTf;

pub const CLIP_HEIGHT: f32 = 20.0;
pub const CLIP_RESIZE_ZONE: f32 = 4.0;
pub const CLIP_RENDER_EPSILON: f32 = 5.0;
pub const TRACK_LABEL_WIDTH: f32 = 100.0;
pub const TRACK_MARK_WIDTH: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
pub enum UiClipGesture {
    Move,
    Resize { resize_left: bool },
}

pub struct ClipWidget(pub Clip);

impl ClipWidget {
    pub fn rect(&self, track_y: u32, timeline_rect: Rect, tf: TimelineTf) -> Rect {
        let top = timeline_rect.top() + (track_y as f32) * CLIP_HEIGHT;
        let left = timeline_rect.left() + tf.tf_pos(self.0.start as f32);
        let width = tf.tf_vector(self.0.len as f32);

        Rect::from_min_size(pos2(left, top), vec2(width, CLIP_HEIGHT))
    }

    pub fn paint(
        &self,
        ui: &Ui,
        painter: &Painter,
        timeline_rect: Rect,
        tf: TimelineTf,
        track_y: u32,
        color: Color32,
        clip_name: &str,
        selected: bool,
    ) {
        let dark_color = Color32::BLACK + color.additive().linear_multiply(0.2);
        let text_color = invert_color(color);
        let padding = ui.spacing().button_padding;
        let this_rect = self.rect(track_y, timeline_rect, tf);
        let left_resize_rect = Rect {
            max: pos2(this_rect.min.x + CLIP_RESIZE_ZONE, this_rect.max.y),
            ..this_rect
        };
        let right_resize_rect = Rect {
            min: pos2(this_rect.max.x - CLIP_RESIZE_ZONE, this_rect.min.y),
            ..this_rect
        };
        let move_rect = Rect {
            min: pos2(this_rect.min.x + CLIP_RESIZE_ZONE, this_rect.min.y),
            max: pos2(this_rect.max.x - CLIP_RESIZE_ZONE, this_rect.max.y),
        };

        if this_rect.width() > 2.0 * CLIP_RESIZE_ZONE + CLIP_RENDER_EPSILON {
            painter.rect_filled(left_resize_rect, 0.0, dark_color);
            painter.rect_filled(right_resize_rect, 0.0, dark_color);
            painter.rect_filled(move_rect, 0.0, color);
            if selected {
                painter.rect(
                    this_rect,
                    0.0,
                    Color32::TRANSPARENT,
                    ui.visuals().selection.stroke,
                    egui::StrokeKind::Inside,
                );
            }
        } else {
            let mini_rect_width = this_rect.width().max(CLIP_RENDER_EPSILON);
            let mini_rect =
                Rect::from_min_size(this_rect.min, vec2(mini_rect_width, this_rect.height()));
            painter.rect(
                mini_rect,
                0.0,
                color,
                Stroke::new(1.0, dark_color),
                egui::StrokeKind::Inside,
            );
            if selected {
                painter.rect(
                    mini_rect,
                    0.0,
                    Color32::TRANSPARENT,
                    ui.visuals().selection.stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }

        if move_rect.width() > 2.0 * padding.x + CLIP_RENDER_EPSILON {
            let text_gal = WidgetText::from(clip_name).into_galley(
                ui,
                Some(egui::TextWrapMode::Truncate),
                move_rect.width() - 2.0 * padding.x,
                TextStyle::Button,
            );
            let text_pos = ui
                .layout()
                .align_size_within_rect(text_gal.size(), move_rect.shrink2(padding))
                .min;
            painter.galley(text_pos, text_gal, text_color);
        }

        let border_stroke = ui.visuals().widgets.inactive.bg_stroke;
        painter.rect(
            this_rect,
            0.0,
            Color32::TRANSPARENT,
            border_stroke,
            egui::StrokeKind::Inside,
        );
    }

    pub fn gesture(
        &self,
        timeline_rect: Rect,
        pointer: Pos2,
        tf: TimelineTf,
        track_y: u32,
    ) -> UiClipGesture {
        let this_rect = self.rect(track_y, timeline_rect, tf);
        let local_off = pointer.x - this_rect.left();
        let resize_left = local_off <= CLIP_RESIZE_ZONE;
        let resize_right = local_off >= this_rect.width() - CLIP_RESIZE_ZONE;

        if resize_left || resize_right {
            UiClipGesture::Resize { resize_left }
        } else {
            UiClipGesture::Move
        }
    }
}

pub struct ClipsUi<'a>(pub &'a mut Animation);

impl<'a> ClipsUi<'a> {
    pub fn paint_track_labels(
        &self,
        ui: &mut Ui,
        painter: &Painter,
        widget_rect: Rect,
        selected_track: Option<(TypeId, u32)>,
    ) {
        for (track_kind, track_id, track_y, track) in self.0.all_tracks() {
            let top = widget_rect.top() + (track_y as f32) * CLIP_HEIGHT;
            let padding = ui.spacing().button_padding;
            let color = track_color(track_y);

            let text_gal = WidgetText::from(&track.name).into_galley(
                ui,
                Some(egui::TextWrapMode::Truncate),
                TRACK_LABEL_WIDTH - TRACK_MARK_WIDTH - 2.0 * padding.x,
                TextStyle::Button,
            );

            let rect = Rect::from_min_size(
                pos2(widget_rect.left() + TRACK_MARK_WIDTH, top),
                vec2(TRACK_LABEL_WIDTH - TRACK_MARK_WIDTH, CLIP_HEIGHT),
            );
            if selected_track == Some((track_kind, track_id)) {
                painter.rect_filled(rect, 0.0, darken_color(color));
            }

            let text_pos = ui
                .layout()
                .align_size_within_rect(text_gal.size(), rect.shrink2(padding))
                .min;
            painter.galley(text_pos, text_gal, Color32::WHITE);

            let mark_rect = Rect::from_min_size(
                pos2(widget_rect.left(), top),
                vec2(TRACK_MARK_WIDTH, CLIP_HEIGHT),
            );
            painter.rect_filled(mark_rect, 0.0, color);
        }
    }

    pub fn paint_clips(
        &self,
        ui: &mut Ui,
        painter: &Painter,
        timeline_rect: Rect,
        tf: TimelineTf,
        selected_clip: Option<(TypeId, u32)>,
    ) {
        for (clip_kind, clip_name, clip_id, clip_y, clip) in self.0.all_clips() {
            ClipWidget(clip).paint(
                ui,
                painter,
                timeline_rect,
                tf,
                clip_y,
                track_color(clip_y),
                clip_name,
                selected_clip == Some((clip_kind, clip_id)),
            );
        }
    }
}

pub fn track_color(id: u32) -> Color32 {
    match id {
        0 => Color32::RED,
        1 => Color32::GREEN,
        2 => Color32::YELLOW,
        3 => Color32::PURPLE,
        _ => Color32::WHITE,
    }
}

fn invert_color(col: Color32) -> Color32 {
    let arr = col.to_normalized_gamma_f32();
    let arr_inv = [1.0 - arr[0], 1.0 - arr[1], 1.0 - arr[2]];
    Color32::from_rgb(
        (arr_inv[0] * 255.0) as u8,
        (arr_inv[1] * 255.0) as u8,
        (arr_inv[2] * 255.0) as u8,
    )
}

pub fn darken_color(color: Color32) -> Color32 {
    Color32::BLACK + color.additive().linear_multiply(0.05)
}
