use macroquad::audio::{self, load_sound, PlaySoundParams, Sound};

use crate::game_model::GameModel;

pub struct SoundDirector {
    dead: Sound,
    bsound: Sound,
    bounce: Sound,
}

impl SoundDirector {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            dead: load_sound("assets/dead.wav").await?,
            bsound: load_sound("assets/break.wav").await?,
            bounce: load_sound("assets/ball.wav").await?,
        })
    }

    pub fn direct_sounds(&mut self, model: &GameModel) {
        if model.ball_bounced() && model.broken_box().is_none() {
            audio::play_sound(
                &self.bounce,
                PlaySoundParams {
                    looped: false,
                    volume: 0.23,
                }
            );
        } else if model.ball_bounced() {
            audio::play_sound(
                &self.bsound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.4,
                }
            );
        }

        if model.gameover_just_happened() {
            audio::play_sound(
                &self.dead,
                PlaySoundParams {
                    looped: false,
                    volume: 0.4,
                }
            );
        }
    }
}