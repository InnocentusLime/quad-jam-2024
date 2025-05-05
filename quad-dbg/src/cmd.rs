use macroquad::prelude::*;
use log::info;

use crate::screentext::SCREENCON_LINES_ONSCREEN;

pub struct CommandCenter {
    buff: String,
}

impl CommandCenter {
    pub fn new() -> Self {
        Self {
            buff: String::new(),
        }
    }

    pub fn should_pause(&self) -> bool {
        !self.buff.is_empty()
    }

    pub fn reset(&mut self) {
        self.buff.clear();
    }

    pub fn submit(&mut self) {
        info!("COMMAND: {}", self.buff);
        self.reset();
    }

    pub fn input(&mut self, ch: char) {
        if !self.buff.is_empty() {
            if ch == '\u{0008}' {
                self.buff.pop();
            } else {
                self.buff.push(ch);
            }
            return;
        }

        if ch != '/' && ch != ':' {
            return;
        }

        self.buff.push(ch);
    }

    pub fn draw(&self) {
        if self.buff.is_empty() {
            return;
        }

        set_default_camera();

        let line_box = screen_height() / (SCREENCON_LINES_ONSCREEN as f32);
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(line_box);
        let dims = measure_text("A", None, font_size, font_scale);
        let line_height = dims.height;
        let spacing = (line_box - line_height) / 2.0;

        let rect_y = (SCREENCON_LINES_ONSCREEN - 1) as f32 * line_box;
        draw_rectangle(0.0, rect_y, screen_width(), line_box, BLUE);

        let text_y = (SCREENCON_LINES_ONSCREEN - 1) as f32 * line_box + line_box - spacing;
        let res = draw_text_ex(
            &self.buff,
            0.0,
            text_y,
            TextParams {
                font: None,
                font_size,
                font_scale,
                font_scale_aspect,
                rotation: 0.0,
                color: WHITE,
            },
        );

        draw_rectangle(
            res.width + 0.1 * font_size as f32 * font_scale, 
            rect_y, 
            0.1 * font_size as f32 * font_scale, 
            font_size as f32 * font_scale, 
            WHITE,
        );
    }
}