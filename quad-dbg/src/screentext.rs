use macroquad::prelude::*;

pub(crate) const SCREENCON_LINES: usize = 1024;
pub(crate) const SCREENCON_CHARS_PER_LINE: usize = 255;
pub(crate) const SCREENCON_LINES_ONSCREEN: usize = 32;

#[derive(Clone)]
pub(crate) struct Line {
    buf: String,
    pub(crate) color: Color,
    pub(crate) background: Color,
}

impl Line {
    fn new() -> Self {
        Self {
            buf: String::with_capacity(SCREENCON_CHARS_PER_LINE),
            color: BLANK,
            background: BLANK,
        }
    }

    pub(crate) fn put(&mut self, mut s: &str) {
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

    pub(crate) fn clear(&mut self) {
        self.buf.clear();
    }
}

pub(crate) struct ScreenText {
    pub(crate) scroll_offset: usize,
    pub(crate) lines: Vec<Line>,
}

impl ScreenText {
    pub(crate) fn new() -> Self {
        Self {
            scroll_offset: 0,
            lines: vec![Line::new(); SCREENCON_LINES],
        }
    }

    pub(crate) fn iter_visible_lines(&'_ self) -> impl Iterator<Item = (usize, &'_ Line)> + '_ {
        self.lines.iter()
            .cycle()
            .skip(self.scroll_offset)
            .take(SCREENCON_LINES_ONSCREEN)
            .enumerate()
    }

    pub(crate) fn last_visible_line(&self) -> usize {
        (self.scroll_offset + (SCREENCON_LINES_ONSCREEN - 1)) % SCREENCON_LINES
    }

    pub(crate) fn draw(&self) {
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

#[cfg(test)]
mod tests {
    use super::{SCREENCON_CHARS_PER_LINE,Line};

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