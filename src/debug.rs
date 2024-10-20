use macroquad::prelude::*;

use crate::ui::UiModel;

const DBG_FONT_SIZE: u16 = 16;
const DBG_MSG_LIFE: f32 = 3.0;
const DBG_MSG_CAP: usize = 30;

#[derive(Clone)]
struct DebugMsg {
    line: String,
    spawn_time: f32,
}

pub struct Debug {
    text_cursor_x: f32,
    text_cursor_y: f32,
    // TODO: make this a shared global resource
    dbg_event_cur: usize,
    dbg_events: [DebugMsg; DBG_MSG_CAP],
}

impl Debug {
    pub fn new() -> Self {
        Self {
            text_cursor_x: 0.0,
            text_cursor_y: DBG_FONT_SIZE as f32,
            dbg_event_cur: 0,
            dbg_events: std::array::from_fn(|_| DebugMsg {
                line: String::with_capacity(255),
                spawn_time: -1.0,
            }),
        }
    }

    pub fn new_frame(&mut self) {
        self.text_cursor_x = 0.0;
        self.text_cursor_y = DBG_FONT_SIZE as f32;

        set_default_camera();
    }

    pub fn new_dbg_line(&mut self) {
        self.text_cursor_x = 0.0;
        self.text_cursor_y += DBG_FONT_SIZE as f32;
    }

    pub fn put_debug_text(&mut self, text: &str, color: Color) {
        draw_text(
            text,
            self.text_cursor_x,
            self.text_cursor_y,
            DBG_FONT_SIZE as f32,
            color,
        );

        self.text_cursor_x += measure_text(text, None, DBG_FONT_SIZE, 1.0).width;
    }

    pub fn draw_ui_debug(&mut self, ui: &UiModel) {
        let input_pairs = [
            ("U", ui.move_up()),
            ("L", ui.move_left()),
            ("D", ui.move_down()),
            ("R", ui.move_right()),
        ];

        for (glyph, flag) in input_pairs.into_iter() {
            self.put_debug_text(
                glyph,
                if flag { RED } else { WHITE }
            );
        }

        self.new_dbg_line();
    }

    pub fn put_event(&mut self, msg: &str) {
        let cell = &mut self.dbg_events[self.dbg_event_cur];

        cell.line.clear();
        cell.line.push_str(msg);
        cell.spawn_time = get_time() as f32;

        self.dbg_event_cur = (self.dbg_event_cur + 1) % DBG_MSG_CAP;
    }

    pub fn draw_events(&mut self) {
        let time = get_time() as f32;
        for msg in self.dbg_events.clone() {
            if msg.spawn_time + DBG_MSG_LIFE < time {
                continue;
            }

            self.put_debug_text(msg.line.as_str(), WHITE);
            self.new_dbg_line();
        }
    }
}