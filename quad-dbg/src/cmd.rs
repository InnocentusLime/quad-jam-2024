use log::info;

pub struct CommandCenter {
    buff: String,
}

impl CommandCenter {
    pub fn new() -> Self {
        Self {
            buff: String::new(),
        }
    }

    pub fn should_pause(&self) -> bool {
        !self.buff.is_empty()
    }

    pub fn reset(&mut self) {
        self.buff.clear();
    }

    pub fn submit(&mut self) {
        info!("COMMAND: {}", self.buff);
        self.reset();
    }

    pub fn input(&mut self, ch: char) {
        if !self.buff.is_empty() {
            self.buff.push(ch);
            return;
        }

        if ch != '/' && ch != ':' {
            return;
        }

        self.buff.push(ch);
    }
}