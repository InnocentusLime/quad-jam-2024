use macroquad::{prelude::*, text};
use std::{fmt, sync::{LazyLock, Mutex}};

// TODO: increase
const SCREENCON_LINES: usize = 64;
const SCREENCON_CHARS_PER_LINE: usize = 255;
const SCREENCON_LINES_ONSCREEN: usize = 32;

#[derive(Clone)]
struct Line {
    buf: String,
    color: Color,
    background: Color,
}

struct ScreenText {
    scroll_offset: usize,
    lines: Vec<Line>,
}

impl ScreenText {
    fn new() -> Self {
        Self {
            scroll_offset: 0,
            lines: vec![Line {
                buf: String::with_capacity(SCREENCON_CHARS_PER_LINE),
                color: BLANK,
                background: BLANK,
            }; SCREENCON_LINES],
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

    // TODO test that the buffer never overfills
    fn write_str_no_newline(&mut self, mut s: &str) {
        // TODO move into Line API and handle all corner cases
        if s.len() > SCREENCON_CHARS_PER_LINE {
            s = &s[..SCREENCON_CHARS_PER_LINE];
        }

        let line = &mut self.text.lines[self.pen.curr_line];

        line.buf.push_str(s);
        line.background = self.pen.pen_back_color;
        line.color = self.pen.pen_text_color;
    }

    fn clear_curr_line(&mut self) {
        let line = &mut self.text.lines[self.pen.curr_line];

        line.buf.clear();
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

    pub fn get_color() -> (Color, Color) {
        let lock = GLOBAL_CON.lock().unwrap();

        (lock.pen.pen_back_color, lock.pen.pen_text_color)
    }

    pub fn set_color(text: Color, back: Color) {
        let mut lock = GLOBAL_CON.lock().unwrap();
        lock.pen.pen_text_color = text;
        lock.pen.pen_back_color = back;
    }

    pub fn scroll() -> usize {
        let lock = GLOBAL_CON.lock().unwrap();
        lock.text.scroll_offset
    }

    pub fn set_scroll(scroll: usize) {
        let mut lock = GLOBAL_CON.lock().unwrap();
        lock.text.scroll_offset = scroll;
    }

    pub fn put_event(msg: fmt::Arguments, level: Level) {
        let (back, text) = Self::log_level_cols(level);
        let (back_old, text_old) = Self::get_color();

        Self::set_color(text, back);
        fmt::write(&mut Self, msg).unwrap();
        Self::set_color(text_old, back_old);
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

        (back_col, text_col)
    }

    pub fn init_log() {
        static THIS: ScreenCons = ScreenCons;
        set_logger(&THIS).unwrap();
    }
}

impl fmt::Write for ScreenCons {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut lock = GLOBAL_CON.lock().unwrap();
        lock.write_str(s)
    }
}

impl Log for ScreenCons {
    fn enabled(&self, metadata: &Metadata) -> bool {
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

// TODO: hook up to log.rs