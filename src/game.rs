use macroquad::prelude::*;
use shipyard::{IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use crate::{method_as_system, ui::UiModel, DeltaTime, Follower, Speed, Transform};

const PLAYER_SPEED_MAX: f32 = 128.0;
const PLAYER_ACC: f32 = 128.0;

#[derive(Unique)]
pub struct Game {
}

impl Game {
    pub fn new() -> Self {
        Self { }
    }

    pub fn update_follower(
        &mut self,
        follow: View<Follower>,
        mut pos: ViewMut<Transform>,
        mut speed: ViewMut<Speed>,
        dt: UniqueView<DeltaTime>,
    ) {
        let dt = dt.0;
        // TODO: do not use here
        let (mx, my) = mouse_position();

        for (_, pos, speed) in (&follow, &mut pos, &mut speed).iter() {
            let dv = (vec2(mx, my) - pos.pos).normalize_or_zero();

            speed.0 += dv * PLAYER_ACC * dt;
            speed.0 = speed.0.clamp_length(0.0, PLAYER_SPEED_MAX);

            pos.pos += speed.0 * dt;
        }
    }
}

method_as_system!(
    Game::update_follower as game_update_follower(
        this: Game,
        follow: View<Follower>,
        pos: ViewMut<Transform>,
        speed: ViewMut<Speed>,
        dt: UniqueView<DeltaTime>
    )
);