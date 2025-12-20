use egui::Window;

use std::fmt;
use std::sync::{LazyLock, Mutex};

#[macro_export]
macro_rules! dump {
    ($($arg:tt)+) => {
        $crate::GLOBAL_DUMP.put_line(std::format_args!($($arg)+));
    };
}

const DUMP_LINE_CAPACITY: usize = 255;
const DUMP_CAPACITY: usize = 100;

pub static GLOBAL_DUMP: LazyLock<ScreenDump> = LazyLock::new(ScreenDump::new);

pub struct ScreenDump(Mutex<ScreenDumpBuff>);

impl ScreenDump {
    pub fn new() -> Self {
        ScreenDump(Mutex::new(ScreenDumpBuff::new()))
    }

    pub fn put_line(&self, args: fmt::Arguments) {
        let mut buff = self.0.lock().expect("Dangling mutex");
        let line = buff.get_next_line();
        fmt::write(line, args).expect("failed to write a line");
    }

    pub fn show(&self, ctx: &egui::Context) {
        let mut buff = self.0.lock().expect("Dangling mutex");
        Window::new("Value dump").show(ctx, |ui| {
            for line in buff.lines() {
                ui.label(line);
            }
        });
        buff.reset();
    }
}

impl Default for ScreenDump {
    fn default() -> Self {
        ScreenDump::new()
    }
}

struct ScreenDumpBuff {
    lines: Vec<String>,
    next_line: usize,
}

impl ScreenDumpBuff {
    fn new() -> Self {
        ScreenDumpBuff {
            lines: Vec::with_capacity(DUMP_CAPACITY),
            next_line: 0,
        }
    }

    fn get_next_line(&mut self) -> &mut String {
        assert!(self.next_line <= DUMP_CAPACITY);
        if self.next_line >= self.lines.len() {
            self.lines.push(String::with_capacity(DUMP_LINE_CAPACITY));
        }
        let res = &mut self.lines[self.next_line];
        self.next_line += 1;
        res
    }

    fn reset(&mut self) {
        self.next_line = 0;
        self.lines.iter_mut().for_each(String::clear);
    }

    fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter().take(self.next_line)
    }
}
