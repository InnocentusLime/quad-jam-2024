use crate::GameState;

#[derive(Clone, Copy, Debug)]
pub struct GameModel {
    pub prev_state: GameState,
    pub state: GameState,
}

impl GameModel {
    pub fn gameover_just_happened(&self) -> bool {
        self.prev_state == GameState::Active && self.state == GameState::GameOver
    }
}