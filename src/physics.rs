use macroquad::prelude::*;

pub const PUSH_EPSILON: f32 = 0.001;
pub const PLAYER_SPEED: f32 = 256.0;
pub const BALL_SPEED: f32 = 180.0;
pub const BOX_PER_LINE: usize = 15;
pub const BOX_LINE_COUNT: usize = 8;
pub const BOX_WIDTH: f32 = 40.0;
pub const BOX_HEIGHT: f32 = 20.0;
pub const BALL_RADIUS: f32 = 6.0;
pub const MAX_X: f32 = BOX_WIDTH * (BOX_PER_LINE as f32);
pub const MAX_Y: f32 = 410.0;
pub const PLAYER_WIDTH: f32 = 80.0;
pub const PLAYER_HEIGHT: f32 = 10.0;
pub const BALL_NUDGE: f32 = 0.4;

#[derive(Clone, Copy, Debug)]
pub struct Physics {
    pub player_x: f32,
    pub player_delta: f32,
    pub ball_pos: Vec2,
    pub ball_dir: Vec2,
    pub boxes: [[bool; BOX_PER_LINE]; BOX_LINE_COUNT],
}

impl Physics {
    pub fn new() -> Self {
        let player_start = MAX_X / 2.0 - PLAYER_WIDTH / 2.0;
        let mut boxes = [[true; BOX_PER_LINE]; BOX_LINE_COUNT];
        for x in 0..BOX_PER_LINE {
            boxes[0][x] = false;
        }

        Self {
            player_x: player_start,
            player_delta: 0.0,
            ball_pos: vec2(
                player_start + PLAYER_WIDTH / 2.0,
                MAX_Y - PLAYER_HEIGHT - BALL_RADIUS * 1.9 - BALL_RADIUS - PUSH_EPSILON
            ),
            ball_dir: vec2(-1.0, -1.0).normalize(),
            boxes,
        }
    }

    pub fn move_player(&mut self, dt: f32, right: bool) {
        let mut dx = PLAYER_SPEED * dt;
        if !right { dx *= -1.0; }

        self.player_x += dx;
        self.player_delta = dx;
    }

    pub fn new_frame(&mut self) {
        self.player_delta = 0.0;
    }

    pub fn update(&mut self, dt: f32) -> bool {
        let offset = self.ball_dir * BALL_SPEED * dt;
        let mut new_ball_pos = self.ball_pos + offset;

        self.player_x = self.player_x.clamp(0.0, MAX_X - PLAYER_WIDTH);

        if new_ball_pos.x - BALL_RADIUS < 0.0 {
            self.ball_dir.x *= -1.0;
            new_ball_pos.x = BALL_RADIUS;
        }

        if new_ball_pos.x + BALL_RADIUS > MAX_X {
            self.ball_dir.x *= -1.0;
            new_ball_pos.x = MAX_X - BALL_RADIUS;
        }

        if new_ball_pos.y - BALL_RADIUS < 0.0 {
            self.ball_dir.y *= -1.0;
            new_ball_pos.y = BALL_RADIUS;
        }

        if new_ball_pos.y + BALL_RADIUS > MAX_Y {
            // self.ball_dir.y *= -1.0;
            new_ball_pos.y = MAX_Y - BALL_RADIUS;
            return true;
        }

        for by in 0..BOX_LINE_COUNT {
            for bx in 0..BOX_PER_LINE {
                let box_rect = Self::box_rect(bx, by);

                if !self.boxes[by][bx] {
                    continue;
                }

                if !Self::ball_in_rect(new_ball_pos, box_rect) {
                    continue;
                }

                self.boxes[by][bx] = false;

                if Self::ball_bumped_vertically(self.ball_pos, box_rect) {
                    self.ball_dir.y *= -1.0;
                    if self.ball_pos.y > box_rect.bottom() {
                        new_ball_pos.y = box_rect.bottom() + BALL_RADIUS + PUSH_EPSILON;
                    } else {
                        new_ball_pos.y = box_rect.top() - BALL_RADIUS - PUSH_EPSILON;
                    }
                } else {
                    self.ball_dir.x *= -1.0;
                    if self.ball_pos.x > box_rect.right() {
                        new_ball_pos.x = box_rect.right() + BALL_RADIUS + PUSH_EPSILON;
                    } else {
                        new_ball_pos.x = box_rect.left() - BALL_RADIUS - PUSH_EPSILON;
                    }
                }
            }
        }

        let player_rect = self.player_rect();
        // The player paddle is kind of special
        // 1. We pretend it is curved with the height function of -0.2 * 2.0 * x
        // 2. Player paddle always pushes the ball to the top of it
        // 3. The horizontal component of ball's velocity can be affected if the paddle
        //     was moving horizontally during impact
        if Self::ball_in_rect(self.ball_pos, player_rect) {
            /* df/dx */
            let d_height = |x: f32| {
                -0.2 * 4.0 * x.powf(3.0)
                // -0.2 * 2.0 * x
            };
            /* tant */
            let tangent = |x: f32| {
                vec2(1.0, d_height(x)).normalize()
            };
            let normal = |x: f32| {
                let t = tangent(x);
                vec2(-t.y, -t.x)
            };
            let ball_x_on_surface = (
                (self.ball_pos.x - player_rect.left()) / player_rect.w
            ) * 2.0 - 1.0;

            let push_n = normal(ball_x_on_surface);
            self.ball_dir -= push_n * self.ball_dir.dot(push_n);
            self.ball_dir += push_n;
            self.ball_dir = self.ball_dir.normalize();

            if self.player_delta != 0.0 {
                self.ball_dir.x += BALL_NUDGE * (self.player_delta / self.player_delta);
            }
            self.ball_dir = self.ball_dir.normalize();

            new_ball_pos.y = player_rect.y - BALL_RADIUS - PUSH_EPSILON;
        }

        self.ball_pos = new_ball_pos;

        false
    }

    pub fn player_rect(&self) -> Rect {
        Rect {
            x: self.player_x,
            y: MAX_Y - PLAYER_HEIGHT - BALL_RADIUS * 1.9,
            w: PLAYER_WIDTH,
            h: PLAYER_HEIGHT,
        }
    }

    pub fn box_rect(x: usize, y: usize) -> Rect {
        Rect {
            x: (x as f32) * BOX_WIDTH,
            y: (y as f32) * BOX_HEIGHT,
            w: BOX_WIDTH,
            h: BOX_HEIGHT,
        }
    }

    fn ball_bumped_vertically(pos: Vec2, rect: Rect) -> bool {
        pos.x + BALL_RADIUS >= rect.left() &&
            pos.x - BALL_RADIUS <= rect.right()
    }

    fn ball_in_rect(pos: Vec2, rect: Rect) -> bool {
        pos.x + BALL_RADIUS >= rect.left() &&
        pos.x - BALL_RADIUS <= rect.right() &&
        pos.y + BALL_RADIUS >= rect.top() &&
        pos.y - BALL_RADIUS <= rect.bottom()
    }
}