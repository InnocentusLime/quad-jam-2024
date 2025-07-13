use hecs::World;
use macroquad::audio::{Sound, load_sound};

pub struct SoundDirector {
    _dead: Sound,
    // bsound: Sound,
    // bounce: Sound,
}

impl SoundDirector {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            _dead: load_sound("assets/dead.wav").await?,
            // bsound: load_sound("assets/break.wav").await?,
            // bounce: load_sound("assets/ball.wav").await?,
        })
    }

    pub fn run(&mut self, _world: &World) {}
}
