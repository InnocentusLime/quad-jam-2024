use macroquad::prelude::*;
use crate::ui::UiModel;

const PLAYER_SPEED: f32 = 10.0;

pub struct Game {
    player_pos: Vec2,
    swarm_matrix: Vec<Vec<bool>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            player_pos: Vec2::ZERO,
            swarm_matrix: vec![vec![false; 1024]; 1024],
        }
    }

    pub fn update(&mut self, _dt: f32, _ui: &UiModel) {

    }
}