use core::fmt;
use std::fmt::Write;

use macroquad::prelude::*;

const DBG_FONT_SIZE: u16 = 16;
const DBG_MSG_LIFE: f32 = 3.0;
const DBG_MSG_CAP: usize = 30;

#[derive(Clone)]
struct DebugMsg {
    color: Color,
    line: String,
    spawn_time: f32,
}

struct DebugMsgStoreCell {
    dbg_event_cur: usize,
    dbg_events: [DebugMsg; DBG_MSG_CAP],
}

impl DebugMsgStoreCell {
    fn put_event(&mut self, msg: &fmt::Arguments, color: Color) {
        let cell = &mut self.dbg_events[self.dbg_event_cur];

        cell.line.clear();
        write!(&mut cell.line, "{}", msg).unwrap();
        cell.spawn_time = get_time() as f32;
        cell.color = color;

        self.dbg_event_cur = (self.dbg_event_cur + 1) % DBG_MSG_CAP;
    }
}

struct DebugMsgStore(std::sync::Mutex<DebugMsgStoreCell>);

static DEBUG_CELL: std::sync::LazyLock<DebugMsgStore> = std::sync::LazyLock::new(|| {
    DebugMsgStore(std::sync::Mutex::new(DebugMsgStoreCell {
        dbg_event_cur: 0,
        dbg_events: std::array::from_fn(|_| DebugMsg {
            line: String::with_capacity(255),
            spawn_time: -1.0,
            color: WHITE,
        }),
    }))
});

#[derive(Clone, Copy, Debug)]
struct OnScreenLog;

static ON_SCREEN_LOG: OnScreenLog = OnScreenLog;

impl Log for OnScreenLog {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO: fix
        true
    }

    fn log(&self, record: &Record) {
        let color = match record.level() {
            Level::Error => RED,
            Level::Warn => YELLOW,
            Level::Info => GREEN,
            Level::Debug => WHITE,
            Level::Trace => GRAY,
        };

        let mut cell = DEBUG_CELL.0.lock().unwrap();
        cell.put_event(record.args(), color);
    }

    fn flush(&self) { /* NOOP */ }
}

pub fn init_on_screen_log() {
    set_logger(&ON_SCREEN_LOG).unwrap();
}

pub struct Debug {
    text_cursor_x: f32,
    text_cursor_y: f32,
}

impl Debug {
    pub fn new() -> Self {
        Self {
            text_cursor_x: 0.0,
            text_cursor_y: DBG_FONT_SIZE as f32,
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

    pub fn draw_events(&mut self) {
        let dbg_cell = DEBUG_CELL.0.lock().unwrap();

        let time = get_time() as f32;
        for msg in &dbg_cell.dbg_events {
            if msg.spawn_time + DBG_MSG_LIFE < time {
                continue;
            }

            self.put_debug_text(msg.line.as_str(), msg.color);
            self.new_dbg_line();
        }
    }
}