use macroquad::prelude::*;
use std::{fmt, sync::{LazyLock, Mutex}};

// TODO: increase
const SCREENCON_LINES: usize = 32;
const SCREENCON_CHARS_PER_LINE: usize = 255;
const SCREENCON_LINES_ONSCREEN: usize = 32;

#[derive(Clone)]
struct Line {
    buf: String,
    color: Color,
    background: Color,
}

struct ScreenCon {
    scroll_offset: usize,
    lines: Vec<Line>,
}

impl ScreenCon {
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

    // TODO: draw only visible
    // TODO: configurable visibility for disappearing logs and CMD-only mode
    fn draw(&self) {
        set_default_camera();
        let line_box = screen_height() / (SCREENCON_LINES_ONSCREEN as f32);
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(line_box);
        let dims = measure_text("A", None, font_size, font_scale);
        let line_height = dims.height;
        let spacing = (line_box - line_height) / 2.0;

        for (idx, line) in self.lines.iter().enumerate() {
            let idx = (idx + SCREENCON_LINES - self.scroll_offset) % SCREENCON_LINES;
            let y = idx as f32 * line_box;
            draw_rectangle(
                0.0,
                y,
                screen_width(),
                line_box,
                line.background,
            );
        }

        for (idx, line) in self.lines.iter().enumerate() {
            let idx = (idx + SCREENCON_LINES - self.scroll_offset) % SCREENCON_LINES;
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

static GLOBAL_CON: LazyLock<Mutex<ScreenCon>> = LazyLock::new(|| {
    Mutex::new(ScreenCon::new())
});

// TODO: write that do not move the cursor
struct ScreenConWriter {
    pen_text_color: Color,
    pen_back_color: Color,
    curr_line: usize,
}

impl ScreenConWriter {
    // TODO test that the buffer never overfills
    fn write_str_no_newline(&self, mut s: &str) {
        if s.len() > SCREENCON_CHARS_PER_LINE {
            s = &s[..SCREENCON_CHARS_PER_LINE];
        }

        let _lock_scope = {
            let mut lock = GLOBAL_CON.lock().unwrap();
            let line = &mut lock.lines[self.curr_line];

            line.buf.push_str(s);
            line.background = self.pen_back_color;
            line.color = self.pen_text_color;
        };
    }

    // TODO: auto scroll
    fn next_line(&mut self) {
        self.curr_line = (self.curr_line + 1) % SCREENCON_LINES;
        let _lock_scope = {
            let mut lock = GLOBAL_CON.lock().unwrap();
            let line = &mut lock.lines[self.curr_line];

            line.buf.clear();
        };
    }

    fn set_line(&self, s: &str) {
        // TODO: make "clear line" call
        let _lock_scope = {
            let mut lock = GLOBAL_CON.lock().unwrap();
            let line = &mut lock.lines[self.curr_line];
            line.buf.clear();
        };

        self.write_str_no_newline(s);
    }
}

impl fmt::Write for ScreenConWriter {
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

static GLOBAL_WRITER: Mutex<ScreenConWriter> = Mutex::new(
    ScreenConWriter {
        pen_back_color: BLANK,
        pen_text_color: WHITE,
        curr_line: 0,
    }
);

pub struct ScreenCons;

impl ScreenCons {
    pub fn draw() {
        GLOBAL_CON.lock().unwrap().draw();
    }

    pub fn set_color(text: Color, back: Color) {
        let mut lock = GLOBAL_WRITER.lock().unwrap();
        lock.pen_text_color = text;
        lock.pen_back_color = back;
    }

    // TODO: add color override
    pub fn set_line(s: &str) {
        GLOBAL_WRITER.lock().unwrap().set_line(s);
    }
}

impl fmt::Write for ScreenCons {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        GLOBAL_WRITER.lock().unwrap().write_str(s)
    }
}