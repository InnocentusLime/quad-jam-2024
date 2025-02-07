use macroquad::prelude::*;
use shipyard::{IntoIter, View, ViewMut, World};
use crate::{ui::UiModel, Follower, Transform, Speed};

const PLAYER_SPEED_MAX: f32 = 128.0;
const PLAYER_ACC: f32 = 128.0;

pub struct Game {
}

impl Game {
    pub fn new() -> Self {
        Self { }
    }

    pub fn update(
        &mut self,
        dt: f32,
        _ui: &UiModel,
        world: &mut World,
    ) {
        let (mx, my) = mouse_position();

        world.run(|follow: View<Follower>, mut pos: ViewMut<Transform>, mut speed: ViewMut<Speed>| {
            for (_, pos, speed) in (&follow, &mut pos, &mut speed).iter() {
                let dv = (vec2(mx, my) - pos.0).normalize_or_zero();

                speed.0 += dv * PLAYER_ACC * dt;
                speed.0 = speed.0.clamp_length(0.0, PLAYER_SPEED_MAX);

                pos.0 += speed.0 * dt;
            }
        });
    }
}