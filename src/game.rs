use macroquad::prelude::*;
use crate::ui::UiModel;

const PLAYER_SPEED_MAX: f32 = 128.0;
const PLAYER_ACC: f32 = 128.0;

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub occupied: bool,
    pub speed: Vec2,
}

pub struct Game {
    player_speed: Vec2,
    player_pos: Vec2,
    swarm_matrix: Vec<Vec<Cell>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            player_speed: Vec2::ZERO,
            player_pos: Vec2::ZERO,
            swarm_matrix: vec![vec![Cell {
                occupied: false,
                speed: Vec2::ZERO,
            }; 1024]; 1024],
        }
    }

    pub fn update(&mut self, dt: f32, _ui: &UiModel) {
        let (mx, my) = mouse_position();
        let dv = (vec2(mx, my) - self.player_pos).normalize_or_zero();

        self.player_speed += dv * PLAYER_ACC * dt;
        self.player_speed = self.player_speed.clamp_length(0.0, PLAYER_SPEED_MAX);
        self.player_pos += self.player_speed * dt;
    }

    pub fn player_pos(&self) -> Vec2 {
        self.player_pos
    }

    pub fn swarm_matrix(&self) -> &Vec<Vec<Cell>> {
        &self.swarm_matrix
    }
}