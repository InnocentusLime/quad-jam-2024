use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct InputModel {
    pub console_toggle_requested: bool,
    pub left_movement_down: bool,
    pub right_movement_down: bool,
    pub up_movement_down: bool,
    pub down_movement_down: bool,
    pub confirmation_detected: bool,
    pub pause_requested: bool,
    pub fullscreen_toggle_requested: bool,
    pub attack_down: bool,
    pub dash_pressed: bool,
    pub scroll_up: bool,
    pub scroll_down: bool,
    pub aim: Vec2,
}

impl InputModel {
    pub fn capture(camera: &Camera2D) -> Self {
        let (mx, my) = mouse_position();
        let aim = camera.screen_to_world(vec2(mx, my));

        // TODO: handle mobile
        let left_movement_down = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
        let right_movement_down = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
        let up_movement_down = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
        let down_movement_down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
        let confirmation_detected =
            is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left);
        let pause_requested = is_key_pressed(KeyCode::Escape);
        let fullscreen_toggle_requested = is_key_pressed(KeyCode::F11);
        let attack_down = is_mouse_button_down(MouseButton::Left);
        let console_toggle_requested =
            is_key_pressed(KeyCode::GraveAccent) || is_key_pressed(KeyCode::Apostrophe);
        let scroll_up = is_key_down(KeyCode::PageUp);
        let scroll_down = is_key_down(KeyCode::PageDown);
        let dash_pressed = is_mouse_button_pressed(MouseButton::Right);

        Self {
            console_toggle_requested,
            attack_down,
            dash_pressed,
            left_movement_down,
            right_movement_down,
            confirmation_detected,
            up_movement_down,
            down_movement_down,
            pause_requested,
            fullscreen_toggle_requested,
            scroll_up,
            scroll_down,
            aim,
        }
    }
}
