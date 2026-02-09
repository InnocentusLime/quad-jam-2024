use egui::{Color32, Painter, Pos2, Rect, Stroke, TextStyle, Ui, WidgetText, pos2, vec2};
use lib_asset::animation::*;
use macroquad::math::Vec2;

use super::TimelineTf;

pub const CLIP_HEIGHT: f32 = 20.0;
pub const CLIP_RESIZE_ZONE: f32 = 4.0;
pub const CLIP_RENDER_EPSILON: f32 = 5.0;
pub const TRACK_LABEL_WIDTH: f32 = 100.0;
pub const TRACK_MARK_WIDTH: f32 = 10.0;

#[derive(Clone, Copy)]
pub struct ClipPosition {
    pub kind: ClipKind,
    pub id: u32,
    pub track_id: u32,
    pub track_y: u32,
    pub start: u32,
    pub len: u32,
    pub name: &'static str,
}

impl ClipPosition {
    pub fn contains_pos(&self, pos: u32) -> bool {
        self.start <= pos && pos < self.end()
    }

    pub fn end(&self) -> u32 {
        self.start + self.len
    }
}

#[derive(Clone, Copy)]
pub struct TrackPosition<'a> {
    pub track_y: u32,
    pub track_id: u32,
    pub kind: ClipKind,
    pub name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::VariantArray, strum::IntoStaticStr)]
pub enum ClipKind {
    Invulnerability = 0,
    Move = 1,
    DrawSprite = 2,
    AttackBox = 3,
    LockInput = 4,
    Spawn = 5,
}

#[derive(Debug, Clone, Copy)]
pub enum UiClipGesture {
    Move,
    Resize { resize_left: bool },
}

pub struct ClipWidget(pub ClipPosition);

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
            let text_gal = self.label().into_galley(
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

    pub fn pointer_action(
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

    pub fn label(&self) -> WidgetText {
        WidgetText::from(self.0.name)
    }
}

pub struct ClipsUi<'a>(pub &'a mut Animation);

impl<'a> ClipsUi<'a> {
    pub fn get(&self, kind: ClipKind, clip_id: u32) -> Option<ClipPosition> {
        self.all_clips().find(|x| x.id == clip_id && x.kind == kind)
    }

    pub fn get_track(&self, kind: ClipKind, track_id: u32) -> Option<TrackPosition<'_>> {
        self.all_tracks()
            .find(|x| x.kind == kind && x.track_id == track_id)
    }

    pub fn track_with_y(&self, kind: ClipKind, track_y: u32) -> Option<TrackPosition<'_>> {
        self.all_tracks()
            .find(|x| x.kind == kind && x.track_y == track_y)
    }

    pub fn global_offset(&mut self, off: Vec2) {
        self.0.action_tracks.invulnerability.global_offset(off);
        self.0.action_tracks.r#move.global_offset(off);
        self.0.action_tracks.draw_sprite.global_offset(off);
        self.0.action_tracks.attack_box.global_offset(off);
        self.0.action_tracks.lock_input.global_offset(off);
        self.0.action_tracks.spawn.global_offset(off);
    }

    pub fn add_track(&mut self, kind: ClipKind, name: String) {
        match kind {
            ClipKind::Invulnerability => &mut self.0.action_tracks.invulnerability.add_track(name),
            ClipKind::Move => &mut self.0.action_tracks.r#move.add_track(name),
            ClipKind::DrawSprite => &mut self.0.action_tracks.draw_sprite.add_track(name),
            ClipKind::AttackBox => &mut self.0.action_tracks.attack_box.add_track(name),
            ClipKind::LockInput => &mut self.0.action_tracks.lock_input.add_track(name),
            ClipKind::Spawn => &mut self.0.action_tracks.spawn.add_track(name),
        };
    }

    pub fn delete_track(&mut self, kind: ClipKind, track_id: u32) {
        match kind {
            ClipKind::Invulnerability => {
                self.0.action_tracks.invulnerability.delete_track(track_id)
            }
            ClipKind::Move => self.0.action_tracks.r#move.delete_track(track_id),
            ClipKind::DrawSprite => self.0.action_tracks.draw_sprite.delete_track(track_id),
            ClipKind::AttackBox => self.0.action_tracks.attack_box.delete_track(track_id),
            ClipKind::LockInput => self.0.action_tracks.lock_input.delete_track(track_id),
            ClipKind::Spawn => self.0.action_tracks.spawn.delete_track(track_id),
        }
    }

    pub fn add_clip(&mut self, kind: ClipKind, track_id: u32, start: u32, len: u32) {
        match kind {
            ClipKind::Invulnerability => self
                .0
                .action_tracks
                .invulnerability
                .add_clip(track_id, start, len),
            ClipKind::Move => self.0.action_tracks.r#move.add_clip(track_id, start, len),
            ClipKind::DrawSprite => self
                .0
                .action_tracks
                .draw_sprite
                .add_clip(track_id, start, len),
            ClipKind::AttackBox => self
                .0
                .action_tracks
                .attack_box
                .add_clip(track_id, start, len),
            ClipKind::LockInput => self
                .0
                .action_tracks
                .lock_input
                .add_clip(track_id, start, len),
            ClipKind::Spawn => self.0.action_tracks.spawn.add_clip(track_id, start, len),
        }
    }

    pub fn delete_clip(&mut self, kind: ClipKind, clip_id: u32) {
        match kind {
            ClipKind::Invulnerability => self.0.action_tracks.invulnerability.delete_clip(clip_id),
            ClipKind::Move => self.0.action_tracks.r#move.delete_clip(clip_id),
            ClipKind::DrawSprite => self.0.action_tracks.draw_sprite.delete_clip(clip_id),
            ClipKind::AttackBox => self.0.action_tracks.attack_box.delete_clip(clip_id),
            ClipKind::LockInput => self.0.action_tracks.lock_input.delete_clip(clip_id),
            ClipKind::Spawn => self.0.action_tracks.spawn.delete_clip(clip_id),
        }
    }

    pub fn set_clip_pos_len(
        &mut self,
        kind: ClipKind,
        idx: u32,
        new_track: u32,
        new_pos: u32,
        new_len: u32,
    ) {
        match kind {
            ClipKind::Invulnerability => self
                .0
                .action_tracks
                .invulnerability
                .set_clip_pos_len(idx, new_track, new_pos, new_len),
            ClipKind::Move => self
                .0
                .action_tracks
                .r#move
                .set_clip_pos_len(idx, new_track, new_pos, new_len),
            ClipKind::DrawSprite => self
                .0
                .action_tracks
                .draw_sprite
                .set_clip_pos_len(idx, new_track, new_pos, new_len),
            ClipKind::AttackBox => self
                .0
                .action_tracks
                .attack_box
                .set_clip_pos_len(idx, new_track, new_pos, new_len),
            ClipKind::LockInput => self
                .0
                .action_tracks
                .lock_input
                .set_clip_pos_len(idx, new_track, new_pos, new_len),
            ClipKind::Spawn => self
                .0
                .action_tracks
                .spawn
                .set_clip_pos_len(idx, new_track, new_pos, new_len),
        }
    }

    pub fn paint_track_labels(
        &self,
        ui: &mut Ui,
        painter: &Painter,
        widget_rect: Rect,
        selected_track: Option<(ClipKind, u32)>,
    ) {
        for track in self.all_tracks() {
            let top = widget_rect.top() + (track.track_y as f32) * CLIP_HEIGHT;
            let padding = ui.spacing().button_padding;
            let color = track_color(track.track_y as u32);

            let text_gal = WidgetText::from(track.name).into_galley(
                ui,
                Some(egui::TextWrapMode::Truncate),
                TRACK_LABEL_WIDTH - TRACK_MARK_WIDTH - 2.0 * padding.x,
                TextStyle::Button,
            );

            let rect = Rect::from_min_size(
                pos2(widget_rect.left() + TRACK_MARK_WIDTH, top),
                vec2(TRACK_LABEL_WIDTH - TRACK_MARK_WIDTH, CLIP_HEIGHT),
            );
            if selected_track == Some((track.kind, track.track_id)) {
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
        selected_clip: Option<(ClipKind, u32)>,
    ) {
        for clip in self.all_clips() {
            ClipWidget(clip).paint(
                ui,
                painter,
                timeline_rect,
                tf,
                clip.track_y,
                track_color(clip.track_y),
                selected_clip == Some((clip.kind, clip.id)),
            );
        }
    }

    pub fn clip_containing_pos(&self, track_y: u32, pos: u32) -> Option<ClipPosition> {
        self.all_clips()
            .find(|x| x.track_y == track_y && x.contains_pos(pos))
    }

    fn all_clips(&self) -> impl Iterator<Item = ClipPosition> {
        let track_offsets = self.all_track_ofsets();
        let invulns = Self::clip_positions(
            track_offsets,
            ClipKind::Invulnerability,
            &self.0.action_tracks.invulnerability,
        );
        let moves =
            Self::clip_positions(track_offsets, ClipKind::Move, &self.0.action_tracks.r#move);
        let draw_sprites = Self::clip_positions(
            track_offsets,
            ClipKind::DrawSprite,
            &self.0.action_tracks.draw_sprite,
        );
        let attack_boxes = Self::clip_positions(
            track_offsets,
            ClipKind::AttackBox,
            &self.0.action_tracks.attack_box,
        );
        let lock_input = Self::clip_positions(
            track_offsets,
            ClipKind::LockInput,
            &self.0.action_tracks.lock_input,
        );
        let spawn =
            Self::clip_positions(track_offsets, ClipKind::Spawn, &self.0.action_tracks.spawn);

        invulns
            .chain(moves)
            .chain(draw_sprites)
            .chain(attack_boxes)
            .chain(lock_input)
            .chain(spawn)
    }

    fn clip_positions<T: ClipAction>(
        track_offsets: [u32; 6],
        kind: ClipKind,
        clips: &Clips<T>,
    ) -> impl Iterator<Item = ClipPosition> {
        clips
            .clips
            .iter()
            .enumerate()
            .map(move |(id, clip)| ClipPosition {
                kind,
                id: id as u32,
                track_id: clip.track_id,
                track_y: track_offsets[kind as usize] + clip.track_id,
                start: clip.start,
                len: clip.len,
                name: clip.action.name(),
            })
    }

    pub fn all_tracks(&self) -> impl Iterator<Item = TrackPosition<'_>> {
        let track_offsets = self.all_track_ofsets();
        self.all_kind_tracks().flat_map(move |(kind, tracks)| {
            tracks
                .iter()
                .enumerate()
                .map(move |(track_id, track)| TrackPosition {
                    track_y: track_offsets[kind as usize] + track_id as u32,
                    track_id: track_id as u32,
                    kind,
                    name: track.name.as_str(),
                })
        })
    }

    fn all_track_ofsets(&self) -> [u32; 6] {
        let mut track_offsets = [0; 6];
        let mut curr_off = 0;
        for (idx, (_, track)) in self.all_kind_tracks().enumerate() {
            track_offsets[idx] = curr_off;
            curr_off += track.len() as u32;
        }
        track_offsets
    }

    fn all_kind_tracks(&self) -> impl Iterator<Item = (ClipKind, &Vec<Track>)> {
        [
            (
                ClipKind::Invulnerability,
                &self.0.action_tracks.invulnerability.tracks,
            ),
            (ClipKind::Move, &self.0.action_tracks.r#move.tracks),
            (
                ClipKind::DrawSprite,
                &self.0.action_tracks.draw_sprite.tracks,
            ),
            (ClipKind::AttackBox, &self.0.action_tracks.attack_box.tracks),
            (ClipKind::LockInput, &self.0.action_tracks.lock_input.tracks),
            (ClipKind::Spawn, &self.0.action_tracks.spawn.tracks),
        ]
        .into_iter()
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
