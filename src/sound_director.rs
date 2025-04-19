use macroquad::audio::{self, load_sound, PlaySoundParams, Sound};
use shipyard::World;

pub struct SoundDirector {
    dead: Sound,
    // bsound: Sound,
    // bounce: Sound,
}

impl SoundDirector {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            dead: load_sound("assets/dead.wav").await?,
            // bsound: load_sound("assets/break.wav").await?,
            // bounce: load_sound("assets/ball.wav").await?,
        })
    }

    pub fn run(&mut self, world: &World) {
        world.run_with_data(Self::direct_sounds, self);
    }

    fn direct_sounds(&mut self) {
    }
}