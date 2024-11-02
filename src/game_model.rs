use macroquad::prelude::*;

use crate::GameState;

#[derive(Clone, Copy, Debug)]
pub struct GameModel {
    pub prev_state: GameState,
    pub state: GameState,
    pub target_pos: Vec2,
}

impl GameModel {
    pub fn gameover_just_happened(&self) -> bool {
        self.prev_state == GameState::Active && self.state == GameState::GameOver
    }
}