use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct InputModel {
    pub console_toggle_requested: bool,
    pub reset_requested: bool,
    pub left_movement_down: bool,
    pub right_movement_down: bool,
    pub up_movement_down: bool,
    pub down_movement_down: bool,
    pub confirmation_detected: bool,
    pub pause_requested: bool,
    pub fullscreen_toggle_requested: bool,
    pub attack_down: bool,
}

impl InputModel {
    pub fn capture() -> Self {
        // NOTE: for mobile
        // let (mx, my) = mouse_position();
        // let Vec2 { x: mx, y: my } = self.get_cam().screen_to_world(vec2(mx, my));

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
        let reset_requested = is_key_pressed(KeyCode::R);
        let console_toggle_requested = 
            is_key_pressed(KeyCode::GraveAccent) || is_key_pressed(KeyCode::Apostrophe);

        Self {
            console_toggle_requested,
            attack_down,
            reset_requested,
            left_movement_down,
            right_movement_down,
            confirmation_detected,
            up_movement_down,
            down_movement_down,
            pause_requested,
            fullscreen_toggle_requested,
        }
    }
}
