use macroquad::prelude::*;
use log::info;

use crate::{cmd_storage::StrTrie, screentext::SCREENCON_LINES_ONSCREEN};

const CHAR_BACKSPACE: char = '\u{0008}';
const CHAR_ESCAPE: char  = '\u{001b}';
const CHAR_ENTER: char = '\u{000d}';
const MAX_CMD_LEN: usize = 100;

struct CommandEntry<T> {
    cmd: &'static str,
    description: &'static str,
    payload: fn(&mut T, &[&str]),
}

pub struct CommandCenter<T> {
    buff: String,
    cmd_table: StrTrie,
    cmds: Vec<CommandEntry<T>>,
}

impl<T> CommandCenter<T> {
    pub fn new() -> Self {
        Self {
            buff: String::with_capacity(MAX_CMD_LEN),
            cmd_table: StrTrie::new(),
            cmds: Vec::new(),
        }
    }

    pub fn add_command(
        &mut self,
        cmd: &'static str,
        description: &'static str,
        payload: fn(&mut T, &[&str]),
    ) {
        if cmd == "help" {
            panic!("Do not add help");
        }

        let id = self.cmds.len();
        self.cmds.push(CommandEntry {
            cmd,
            description,
            payload,
        });

        self.cmd_table.add_entry(cmd, id);
    }

    pub fn should_pause(&self) -> bool {
        !self.buff.is_empty()
    }

    pub fn input(&mut self, ch: char, input: &mut T) {
        match (ch, self.buff.is_empty()) {
            (CHAR_BACKSPACE, false) => {
                self.buff.pop();
            },
            ('/' | ':', true) => self.buff.push(ch),
            (_, true) => (),
            (CHAR_ENTER, false) => self.submit(input),
            (CHAR_ESCAPE, false) => self.reset(),
            (ch, false) => self.append_ch(ch),
        }
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

    fn append_ch(&mut self, ch: char) {
        if self.buff.len() >= MAX_CMD_LEN { 
            return; 
        }
        if !Self::is_cmd_char(ch) {
            return;
        }

        self.buff.push(ch);
    }

    fn is_cmd_char(ch: char) -> bool {
        ch.is_alphabetic() ||
        ch == ' ' ||
        ch == '.' ||
        ch == ',' ||
        ch == '_'
    }

    fn reset(&mut self) {
        self.buff.clear();
    }

    fn submit(&mut self, input: &mut T) {
        info!("COMMAND: {}", self.buff);

        match self.buff.as_bytes()[0] {
            b':' => self.perform_command(input),
            _ => (),
        }
        
        self.reset();
    }

    fn perform_command(&mut self, input: &mut T) {
        let s = &self.buff[1..];
        let mut parts = s.split_ascii_whitespace();
        let Some(cmd) = parts.next() else { return; };

        if cmd == "help" {
            self.perform_help();
            return;
        }

        let Some(entry) = self.cmd_table.resolve_str(cmd)
        else {
            error!("No such command: {cmd:?}");
            return;
        };

        let args = parts.collect::<Vec<_>>();

        (self.cmds[entry].payload)(input, &args);
    }

    fn perform_help(&self) {
        for cmd in self.cmds.iter() {
            info!("{} -- {}", cmd.cmd, cmd.description);
        }
    }
}