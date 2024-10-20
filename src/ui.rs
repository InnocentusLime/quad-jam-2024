use macroquad::prelude::*;
use crate::{sys::*, GameState};

const FONT_SCALE: f32 = 1.0;
const MAIN_FONT_SIZE: u16 = 32;
const HINT_FONT_SIZE: u16 = 16;
const PADDLE_BUTTON_WIDTH: f32 = 64.0;
const VERTICAL_ORIENT_HORIZONTAL_PADDING: f32 = 16.0;

static WIN_TEXT: &'static str = "Congratulations!";
static GAMEOVER_TEXT: &'static str = "Game Over";
static PAUSE_TEXT: &'static str = "Paused";
static ORIENTATION_TEXT: &'static str = "Wrong Orientation";

static RESTART_HINT_DESK: &'static str = "Press Space to restart";
static RESTART_HINT_MOBILE: &'static str = "Tap the screen to restart";
static ORIENTATION_HINT: &'static str = "Please re-orient your device\ninto landscape";

static START_TEXT_DESK: &'static str = "Press Space to start";
static START_TEXT_MOBILE: &'static str = "Tap to start";

#[derive(Clone, Copy, Debug)]
pub struct InGameUiModel {
    state: GameState,
    left_movement_down: bool,
    right_movement_down: bool,
    confirmation_detected: bool,
    pause_requested: bool,
    fullscreen_toggle_requested: bool,
}

impl InGameUiModel {
    pub fn move_left(&self) -> bool {
        self.left_movement_down
    }

    pub fn move_right(&self) -> bool {
        self.right_movement_down
    }

    pub fn confirmation_detected(&self) -> bool {
        self.confirmation_detected
    }

    pub fn pause_requested(&self) -> bool {
        self.pause_requested
    }

    pub fn fullscreen_toggle_requested(&self) -> bool {
        self.fullscreen_toggle_requested
    }
}

pub struct Ui {
    oegnek: Font,
}

impl Ui {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            oegnek: load_ttf_font("assets/oegnek.ttf").await?,
        })
    }

    pub fn update(&self, state: GameState) -> InGameUiModel {
        let (mx, my) = mouse_position();
        let Vec2 { x: mx, y: my } = self.get_cam().screen_to_world(vec2(mx, my));
        let left_button_rect = self.move_left_button_rect();
        let right_button_rect = self.move_right_button_rect();


        let left_movement_down =
            is_key_down(KeyCode::A) ||
            is_key_down(KeyCode::Left) ||
            (left_button_rect.contains(vec2(mx, my)) &&
             is_mouse_button_down(MouseButton::Left) &&
             on_mobile());
        let right_movement_down =
            is_key_down(KeyCode::D) ||
            is_key_down(KeyCode::Right) ||
            (right_button_rect.contains(vec2(mx, my)) &&
             is_mouse_button_down(MouseButton::Left) &&
             on_mobile());
        let confirmation_detected =
            is_key_pressed(KeyCode::Space) ||
            is_mouse_button_pressed(MouseButton::Left);
        let pause_requested =
            is_key_pressed(KeyCode::Escape);
        let fullscreen_toggle_requested =
            is_key_pressed(KeyCode::F11);

        InGameUiModel {
            state,
            left_movement_down,
            right_movement_down,
            confirmation_detected,
            pause_requested,
            fullscreen_toggle_requested,
        }
    }

    pub fn draw(&self, model: InGameUiModel) {
        set_camera(&self.get_cam());

        if on_mobile() && model.state == GameState::Active {
            let left_button_rect = self.move_left_button_rect();
            let right_button_rect = self.move_right_button_rect();
            draw_rectangle(
                left_button_rect.x,
                left_button_rect.y,
                left_button_rect.w,
                left_button_rect.h,
                if model.move_left() { WHITE }
                else { Color::from_hex(0xDDFBFF) }
            );
            draw_rectangle(
                right_button_rect.x,
                right_button_rect.y,
                right_button_rect.w,
                right_button_rect.h,
                if model.move_right() { WHITE }
                else { Color::from_hex(0xDDFBFF) }
            );
        }

        match model.state {
            GameState::Start => self.draw_announcement_text(
                true,
                Self::start_text(),
                None,
            ),
            GameState::GameOver => self.draw_announcement_text(
                true,
                GAMEOVER_TEXT,
                Some(Self::game_restart_hint()),
            ),
            GameState::Win => self.draw_announcement_text(
                false,
                WIN_TEXT,
                Some(Self::game_restart_hint()),
            ),
            GameState::Paused => self.draw_announcement_text(true, PAUSE_TEXT, None),
            GameState::PleaseRotate => self.draw_announcement_text(
                true,
                ORIENTATION_TEXT,
                Some(ORIENTATION_HINT),
            ),
            _ => (),
        }
    }

    fn start_text() -> &'static str {
        if on_mobile() {
            START_TEXT_MOBILE
        } else {
            START_TEXT_DESK
        }
    }

    fn game_restart_hint() -> &'static str {
        if on_mobile() {
            RESTART_HINT_MOBILE
        } else {
            RESTART_HINT_DESK
        }
    }

    fn move_left_button_rect(&self) -> Rect {
        let view_rect = self.view_rect();

        Rect {
            x: view_rect.left(),
            y: view_rect.top(),
            w: PADDLE_BUTTON_WIDTH,
            h: view_rect.h,
        }
    }

    fn move_right_button_rect(&self) -> Rect {
        let view_rect = self.view_rect();

        Rect {
            x: view_rect.right() - PADDLE_BUTTON_WIDTH,
            y: view_rect.top(),
            w: PADDLE_BUTTON_WIDTH,
            h: view_rect.h,
        }
    }

    fn draw_announcement_text(&self, backdrop: bool, text: &str, hint: Option<&str>) {
        let view_rect = self.view_rect();

        if backdrop {
            draw_rectangle(
                view_rect.x,
                view_rect.y,
                view_rect.w,
                view_rect.h,
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.12,
                    a: 0.5,
                }
            );
        }

        let center = get_text_center(
            text,
            Some(&self.oegnek),
            MAIN_FONT_SIZE,
            FONT_SCALE,
            0.0
        );
        draw_text_ex(
            text,
            view_rect.left() + view_rect.w / 2.0 - center.x,
            view_rect.top() + view_rect.h / 2.0 - center.y,
            TextParams {
                font: Some(&self.oegnek),
                font_size: MAIN_FONT_SIZE,
                color: Color::from_hex(0xDDFBFF),
                font_scale: FONT_SCALE,
                ..Default::default()
            }
        );

        let Some(hint) = hint else { return; };
        let center = get_text_center(
            Self::find_longest_line(hint),
            Some(&self.oegnek),
            HINT_FONT_SIZE,
            FONT_SCALE,
            0.0
        );
        draw_multiline_text_ex(
            hint,
            view_rect.left() + view_rect.w / 2.0 - center.x,
            view_rect.top() + view_rect.h / 2.0 - center.y + (MAIN_FONT_SIZE as f32) * 1.5,
            None,
            TextParams {
                font: Some(&self.oegnek),
                font_size: HINT_FONT_SIZE,
                color: Color::from_hex(0xDDFBFF),
                font_scale: FONT_SCALE,
                ..Default::default()
            }
        );
    }

    fn find_longest_line(text: &str) -> &str {
        text.split('\n').max_by_key(|x| x.len())
            .unwrap_or("")
    }

    fn view_rect(&self) -> Rect {
        // Special case for misoriented mobile devices
        if screen_height() > screen_width() {
            let measure = measure_text(
                ORIENTATION_TEXT,
                Some(&self.oegnek),
                MAIN_FONT_SIZE,
                FONT_SCALE
            );
            let view_width = measure.width +
                2.0 * VERTICAL_ORIENT_HORIZONTAL_PADDING;

            return Rect {
                x: -VERTICAL_ORIENT_HORIZONTAL_PADDING,
                y: 0.0,
                w: view_width,
                h: view_width * (screen_height() / screen_width())
            }
        }

        let view_height = (MAIN_FONT_SIZE as f32) * 12.0;
        Rect {
            x: 0.0,
            y: 0.0,
            w: view_height * (screen_width() / screen_height()),
            h: view_height,
        }
    }

    fn get_cam(&self) -> Camera2D {
        let mut cam = Camera2D::from_display_rect(self.view_rect());
        cam.zoom.y *= -1.0;

        cam
    }
}