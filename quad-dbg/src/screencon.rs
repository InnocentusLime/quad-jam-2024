use macroquad::prelude::*;
use std::{fmt, sync::{LazyLock, Mutex}};

const SCREENCON_LINES: usize = 1024;
const SCREENCON_CHARS_PER_LINE: usize = 255;
const SCREENCON_LINES_ONSCREEN: usize = 32;

#[derive(Clone)]
struct Line {
    buf: String,
    color: Color,
    background: Color,
}

impl Line {
    fn new() -> Self {
        Self {
            buf: String::with_capacity(SCREENCON_CHARS_PER_LINE),
            color: BLANK,
            background: BLANK,
        }
    }

    fn put(&mut self, mut s: &str) {
        let used = self.buf.len();
        if used >= SCREENCON_CHARS_PER_LINE {
            return;
        }
        let remaining = SCREENCON_CHARS_PER_LINE - used;

        if s.len() > remaining {
            s = &s[..remaining];
        }

        self.buf.push_str(s);
    }

    fn clear(&mut self) {
        self.buf.clear();
    }
}

struct ScreenText {
    scroll_offset: usize,
    lines: Vec<Line>,
}

impl ScreenText {
    fn new() -> Self {
        Self {
            scroll_offset: 0,
            lines: vec![Line::new(); SCREENCON_LINES],
        }
    }

    fn iter_visible_lines(&'_ self) -> impl Iterator<Item = (usize, &'_ Line)> + '_ {
        self.lines.iter()
            .cycle()
            .skip(self.scroll_offset)
            .take(SCREENCON_LINES_ONSCREEN)
            .enumerate()
    }

    fn last_visible_line(&self) -> usize {
        (self.scroll_offset + (SCREENCON_LINES_ONSCREEN - 1)) % SCREENCON_LINES
    }

    fn draw(&self) {
        set_default_camera();
        let line_box = screen_height() / (SCREENCON_LINES_ONSCREEN as f32);
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(line_box);
        let dims = measure_text("A", None, font_size, font_scale);
        let line_height = dims.height;
        let spacing = (line_box - line_height) / 2.0;

        for (idx, line) in self.iter_visible_lines() {
            let y = idx as f32 * line_box;
            draw_rectangle(
                0.0,
                y,
                screen_width(),
                line_box,
                line.background,
            );
        }

        for (idx, line) in self.iter_visible_lines() {
            let y = idx as f32 * line_box + line_box - spacing;
            draw_text_ex(
                &line.buf,
                0.0,
                y,
                TextParams {
                    font: None,
                    font_size,
                    font_scale,
                    font_scale_aspect,
                    rotation: 0.0,
                    color: line.color,
                },
            );
        }
    }
}

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
                pen_back_color: BLANK,
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
        let lock = GLOBAL_CON.lock().unwrap();

        lock.get_color()
    }

    pub fn set_color(text: Color, back: Color) {
        let mut lock = GLOBAL_CON.lock().unwrap();
        lock.set_color(text, back);
    }

    pub fn scroll() -> usize {
        let lock = GLOBAL_CON.lock().unwrap();
        lock.text.scroll_offset
    }

    pub fn set_scroll(scroll: usize) {
        let mut lock = GLOBAL_CON.lock().unwrap();
        lock.text.scroll_offset = scroll;
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

    fn log_level_cols(level: Level) -> (Color, Color) {
        let text_col = match level {
            Level::Error => RED,
            Level::Warn => YELLOW,
            Level::Info => GREEN,
            Level::Debug => WHITE,
            Level::Trace => GRAY,
        };

        let back_col = Color::new(0.0, 0.0, 0.0, 0.8);

        (text_col, back_col)
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

#[cfg(test)]
mod tests {
    use crate::screencon::SCREENCON_CHARS_PER_LINE;

    use super::Line;

    #[test]
    fn test_line_overfill() {
        let mut line = Line::new();

        let samples = [
            "1"; 3000
        ];

        for (idx, s) in samples.into_iter().enumerate() {
            if idx % 500 == 0 {
                line.clear();
            }

            line.put(s);

            assert!(line.buf.len() <= SCREENCON_CHARS_PER_LINE);
            assert!(line.buf.capacity() <= SCREENCON_CHARS_PER_LINE);
        }
    }
}