use crate::{physics::*, GameState};

#[derive(Clone, Copy, Debug)]
pub struct GameModel {
    pub dt: f32,
    pub prev_state: GameState,
    pub state: GameState,
    pub old_physics: Physics,
    pub physics: Physics,
}

impl GameModel {
    pub fn ball_bounced(&self) -> bool {
        self.old_physics.ball_dir != self.physics.ball_dir
    }

    pub fn ball_bounced_off_left_wall(&self) -> bool {
        (
            self.old_physics.ball_pos +
            self.old_physics.ball_dir * BALL_SPEED * self.dt -
            BALL_RADIUS
        ).x < 0.0
    }

    pub fn ball_bounced_off_right_wall(&self) -> bool {
        (
            self.old_physics.ball_pos +
            self.old_physics.ball_dir * BALL_SPEED * self.dt +
            BALL_RADIUS
        ).x > MAX_X
    }

    pub fn broken_box(&self) -> Option<(usize, usize)> {
        for by in 0..BOX_LINE_COUNT {
            for bx in 0..BOX_PER_LINE {
                if self.physics.boxes[by][bx] == self.old_physics.boxes[by][bx] {
                    continue;
                }

                return Some((bx, by));
            }
        }

        None
    }

    pub fn gameover_just_happened(&self) -> bool {
        self.prev_state == GameState::Active && self.state == GameState::GameOver
    }
}

pub fn player_won(phy: &Physics) -> bool {
    phy.boxes.iter().flat_map(|x| x.iter()).all(|x| !*x)
}