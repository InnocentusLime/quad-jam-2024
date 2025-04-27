use macroquad::prelude::*;
use crate::screentext::*;

use std::{fmt, sync::{LazyLock, Mutex}};

struct ScreenPen {
    pen_text_color: Color,
    pen_back_color: Color,
    curr_line: usize,
}

struct ScreenConsoleImpl {
    pen: ScreenPen,
    text: ScreenText,
}

// TODO: don't let scroll too far?
impl ScreenConsoleImpl {
    fn new() -> Self {
        ScreenConsoleImpl {
            pen: ScreenPen {
                pen_back_color: SCREENCON_DEFAULT_BACKGROUND,
                pen_text_color: WHITE,
                curr_line: 0,
            },
            text: ScreenText::new(),
        }
    }

    fn write_str_no_newline(&mut self, s: &str) {
        let line = &mut self.text.lines[self.pen.curr_line];

        line.background = self.pen.pen_back_color;
        line.color = self.pen.pen_text_color;
        line.put(s);
    }

    fn clear_curr_line(&mut self) {
        let line = &mut self.text.lines[self.pen.curr_line];

        line.clear();
    }

    fn next_line(&mut self) {
        self.pen.curr_line = (self.pen.curr_line + 1) % SCREENCON_LINES;
        let should_scroll =
            self.pen.curr_line > self.text.last_visible_line() ||
            self.text.last_visible_line() == SCREENCON_LINES - 1;

        if should_scroll {
            self.text.scroll_offset = (self.text.scroll_offset + 1) % SCREENCON_LINES;
        }

        self.clear_curr_line();
    }

    fn last_line(&self) -> usize {
        (self.pen.curr_line + SCREENCON_LINES - SCREENCON_LINES_ONSCREEN + 1) % SCREENCON_LINES
    }

    fn scroll(&self) -> usize {
        self.text.scroll_offset
    }

    fn set_scroll(&mut self, x: usize) {
        self.text.scroll_offset = x % SCREENCON_LINES;
    }

    fn set_color(&mut self, text: Color, back: Color) {
        self.pen.pen_text_color = text;
        self.pen.pen_back_color = back;
    }

    fn get_color(&self) -> (Color, Color) {
        (self.pen.pen_text_color, self.pen.pen_back_color)
    }
}

impl<'a> fmt::Write for ScreenConsoleImpl {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.is_ascii() {
            return Err(fmt::Error);
        }

        let mut next = Some(s);
        while let Some(mut curr) = next.take() {
            if curr.len() == 0 { break; }

            if let Some((line, rest)) = curr.split_once('\n') {
                curr = line;
                next = Some(rest);
            }

            self.write_str_no_newline(curr);

            if next.is_some() {
                self.next_line();
            }
        }

        Ok(())
    }
}

static GLOBAL_CON: LazyLock<Mutex<ScreenConsoleImpl>> = LazyLock::new(|| {
    Mutex::new(ScreenConsoleImpl::new())
});

pub struct ScreenCons;

impl ScreenCons {
    pub fn draw() {
        GLOBAL_CON.lock().unwrap().text.draw();
    }

    fn scope<R>(scope: impl FnOnce(&mut ScreenConsoleImpl) -> R) -> R {
        let mut lock = GLOBAL_CON.lock().unwrap();
        scope(&mut lock)
    }

    pub fn get_color() -> (Color, Color) {
        Self::scope(|con| con.get_color())
    }

    pub fn set_color(text: Color, back: Color) {
        Self::scope(|con| con.set_color(text, back))
    }

    pub fn scroll() -> usize {
        Self::scope(|con| con.scroll())
    }

    pub fn set_scroll(scroll: usize) {
        Self::scope(|con| con.set_scroll(scroll))
    }

    pub fn is_scroll_recent() -> bool {
        Self::scope(|con| {
            con.last_line() == con.scroll()
        })
    }

    pub fn snap_to_last_message() {
        Self::scope(|con| {
            con.set_scroll(con.last_line());
        })
    }

    pub fn put_event(msg: fmt::Arguments, level: Level) {
        Self::scope(|con| {
            let (text, back) = Self::log_level_cols(level);
            let (text_old, back_old) = con.get_color();

            con.set_color(text, back);
            fmt::write(con, msg).unwrap();
            con.set_color(text_old, back_old);
        })
    }

    pub fn init_log() {
        static THIS: ScreenCons = ScreenCons;
        set_logger(&THIS).unwrap();
    }

    pub fn scroll_forward() {
        Self::scope(|con| {
            let scroll = con.scroll();
            con.set_scroll(scroll.saturating_add(1));
        })
    }

    pub fn scroll_back() {
        Self::scope(|con| {
            let scroll = con.scroll();
            con.set_scroll(scroll.saturating_sub(1));
        })
    }

    fn log_level_cols(level: Level) -> (Color, Color) {
        let text_col = match level {
            Level::Error => RED,
            Level::Warn => YELLOW,
            Level::Info => GREEN,
            Level::Debug => WHITE,
            Level::Trace => GRAY,
        };

        let back_col = SCREENCON_DEFAULT_BACKGROUND;

        (text_col, back_col)
    }
}

impl fmt::Write for ScreenCons {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut lock = GLOBAL_CON.lock().unwrap();
        lock.write_str(s)
    }
}

impl Log for ScreenCons {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO: impl
        true
    }

    fn log(&self, record: &Record) {
        let msg = *record.args();
        let level = record.level();
        let file = record.file().unwrap_or("???");
        let line = record.line().unwrap_or(0);

        Self::put_event(
            format_args!("{}:{} {}\n", file, line, &msg),
            level
        );
    }

    fn flush(&self) { /* NOOP */ }
}