use crate::DebugCommand;

use egui::{Modal, TextEdit};

const MAX_CMD_LEN: usize = 100;
const CMD_WIDTH: f32 = 500.0;

pub struct CommandCenter {
    buff: String,
}

impl CommandCenter {
    pub fn new() -> Self {
        Self { buff: String::with_capacity(MAX_CMD_LEN) }
    }

    pub fn should_pause(&self) -> bool {
        !self.buff.is_empty()
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<DebugCommand> {
        let (close, submit, begin_command) = ctx.input(|inp| {
            let close = inp.key_pressed(egui::Key::Escape);
            let submit = inp.key_pressed(egui::Key::Enter);
            let begin_command = inp.key_pressed(egui::Key::Colon);
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

    fn cmd_ui(
        &mut self,
        ui: &mut egui::Ui,
        submit: bool,
        begin_command: bool,
    ) -> Option<DebugCommand> {
        ui.set_width(CMD_WIDTH);

        let output = TextEdit::singleline(&mut self.buff)
            .cursor_at_end(true)
            .desired_width(CMD_WIDTH)
            .show(ui);
        if output.response.lost_focus() && submit {
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

fn parse_command(s: &str) -> Option<DebugCommand> {
    let s = &s[1..];
    let mut parts = s.split_ascii_whitespace();
    let command = parts.next()?.to_string();
    Some(DebugCommand { command, args: parts.map(|x| x.to_string()).collect() })
}
