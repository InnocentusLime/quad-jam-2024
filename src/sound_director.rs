use macroquad::audio::{self, load_sound, PlaySoundParams, Sound};
use shipyard::Unique;

use crate::method_as_system;

#[derive(Unique)]
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

    pub fn direct_sounds(&mut self) {
    }
}

method_as_system!(
    SoundDirector::direct_sounds as sound_director_sounds(
        this: SoundDirector,
    )
);