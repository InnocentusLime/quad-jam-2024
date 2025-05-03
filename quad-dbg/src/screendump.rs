use crate::screentext::*;
use macroquad::prelude::*;

use std::{
    fmt,
    sync::{LazyLock, Mutex},
};

struct ScreenPen {
    pen_text_color: Color,
    pen_back_color: Color,
    curr_line: usize,
}

struct ScreenDumpImpl {
    pen: ScreenPen,
    text: ScreenText,
}

impl ScreenDumpImpl {
    fn new() -> Self {
        ScreenDumpImpl {
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

    fn next_line(&mut self) {
        if self.pen.curr_line == SCREENCON_LINES {
            return;
        }

        self.pen.curr_line = self.pen.curr_line + 1;
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

    fn wipe(&mut self) {
        self.pen.curr_line = 0;
        for line in self.text.lines.iter_mut() {
            line.clear();
        }
    }
}

impl<'a> fmt::Write for ScreenDumpImpl {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.is_ascii() {
            return Err(fmt::Error);
        }

        let mut next = Some(s);
        while let Some(mut curr) = next.take() {
            if curr.len() == 0 {
                break;
            }

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

static GLOBAL_DUMP: LazyLock<Mutex<ScreenDumpImpl>> =
    LazyLock::new(|| Mutex::new(ScreenDumpImpl::new()));

pub struct ScreenDump;

impl ScreenDump {
    pub fn draw() {
        GLOBAL_DUMP.lock().unwrap().text.draw();
    }

    fn scope<R>(scope: impl FnOnce(&mut ScreenDumpImpl) -> R) -> R {
        let mut lock = GLOBAL_DUMP.lock().unwrap();
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

    pub fn new_frame() {
        Self::scope(|con| con.wipe());
    }
}

impl fmt::Write for ScreenDump {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut lock = GLOBAL_DUMP.lock().unwrap();
        lock.write_str(s)
    }
}

#[macro_export]
macro_rules! dump {
    ($($arg:tt)+) => {
        std::fmt::write(&mut $crate::ScreenDump,
            std::format_args!(
                "{}:{} {}\n",
                std::file!(),
                std::line!(),
                std::format_args!($($arg)+)
            ),
        ).unwrap();
    };
}
