use std::sync::{LazyLock, Mutex};
use std::{collections::VecDeque, fmt};

use egui::scroll_area::ScrollBarVisibility;
use egui::{Color32, Modal, RichText, ScrollArea, TextEdit};
use log::Log;

const MAX_CMD_LEN: usize = 100;
const CMD_WIDTH: f32 = 500.0;
const LINEBUFF_SIZE: usize = 1000;
const TEXT_CAPACITY: usize = 1000;
const LOG_HEIGHT: f32 = 400.0;

#[derive(Debug)]
pub struct Command {
    pub command: String,
    pub args: Vec<String>,
}

pub struct CommandCenter {
    buff: String,
}

impl CommandCenter {
    pub fn new() -> Self {
        Self {
            buff: String::with_capacity(MAX_CMD_LEN),
        }
    }

    pub fn should_pause(&self) -> bool {
        !self.buff.is_empty()
    }

    pub fn show(&mut self, ctx: &egui::Context, ch: Option<char>) -> Option<Command> {
        let (close, submit, begin_command) = ctx.input(|inp| {
            let close = inp.key_pressed(egui::Key::Escape);
            let submit = inp.key_pressed(egui::Key::Enter);
            // FIXME: broken with macroquad backend this doesn't work
            //        inp.key_pressed(egui::Key::Colon)
            let begin_command = ch == Some(':');
            (close, submit, begin_command)
        });
        if begin_command {
            self.buff.push(':');
        }
        if close {
            self.buff.clear();
        }

        if self.buff.is_empty() {
            return None;
        }

        let command = Modal::new(egui::Id::new("console"))
            .show(ctx, |ui| self.cmd_ui(ui, submit, begin_command));
        command.inner
    }

    fn cmd_ui(&mut self, ui: &mut egui::Ui, submit: bool, begin_command: bool) -> Option<Command> {
        ui.set_width(CMD_WIDTH);

        GLOBAL_CON.0.lock().unwrap().show(ui);
        let output = TextEdit::singleline(&mut self.buff)
            .cursor_at_end(true)
            .desired_width(CMD_WIDTH)
            .show(ui);
        if output.response.lost_focus() && submit {
            echo_command(&self.buff);
            let res = parse_command(&self.buff);
            self.buff.clear();
            return res;
        }
        if begin_command {
            output.response.request_focus();
        }

        None
    }
}

fn parse_command(s: &str) -> Option<Command> {
    let s = &s[1..];
    let mut parts = s.split_ascii_whitespace();
    let command = parts.next()?.to_string();
    Some(Command {
        command,
        args: parts.map(|x| x.to_string()).collect(),
    })
}

fn echo_command(s: &str) {
    let mut buff = GLOBAL_CON.0.lock().expect("dangling console log");
    let line = buff.get_next_log_line(log::Level::Debug, true);
    line.clear();
    line.push_str("> ");
    line.push_str(&s[1..]);
}

pub static GLOBAL_CON: LazyLock<GlobalConsole> =
    LazyLock::new(|| GlobalConsole(Mutex::new(ConsoleBuffer::new())));

pub struct GlobalConsole(Mutex<ConsoleBuffer>);

impl Log for GlobalConsole {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if record.level() == log::Level::Debug || record.level() == log::Level::Trace {
            return;
        }

        let mut buff = self.0.lock().expect("dangling console log");
        let text = buff.get_next_log_line(record.level(), false);
        fmt::write(text, *record.args()).expect("fmt fail");
    }

    fn flush(&self) {
        /* NOOP */
    }
}

struct ConsoleBuffer(VecDeque<ConsoleLogRecord>);

impl ConsoleBuffer {
    fn new() -> Self {
        ConsoleBuffer(VecDeque::with_capacity(LINEBUFF_SIZE))
    }

    fn get_next_log_line(&mut self, level: log::Level, command_echo: bool) -> &mut String {
        if self.0.len() < LINEBUFF_SIZE {
            self.0.push_back(ConsoleLogRecord {
                command_echo,
                level,
                text: String::with_capacity(TEXT_CAPACITY),
            });
        } else {
            let mut elem = self.0.pop_front().unwrap();
            elem.command_echo = command_echo;
            elem.text.clear();
            elem.level = level;
            self.0.push_back(elem);
        }
        &mut self.0.back_mut().unwrap().text
    }

    fn show(&self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
            .max_height(LOG_HEIGHT)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.set_min_height(LOG_HEIGHT);
                ui.set_min_width(CMD_WIDTH);
                for line in self.0.iter() {
                    ui.horizontal(|ui| {
                        if line.command_echo {
                            ui.colored_label(Color32::WHITE, &line.text);
                        } else {
                            let rich_text = RichText::new(format_level(line.level))
                                .color(level_color(line.level))
                                .monospace();
                            ui.label(rich_text);
                            ui.label(&line.text);
                        }
                    });
                }
            });
    }
}

struct ConsoleLogRecord {
    command_echo: bool,
    level: log::Level,
    text: String,
}

fn format_level(level: log::Level) -> String {
    format!("{level:<5}")
}

fn level_color(level: log::Level) -> egui::Color32 {
    match level {
        log::Level::Error => Color32::RED,
        log::Level::Warn => Color32::YELLOW,
        log::Level::Info => Color32::GREEN,
        log::Level::Debug => Color32::GRAY,
        log::Level::Trace => Color32::GRAY,
    }
}
