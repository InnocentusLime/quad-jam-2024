use super::prelude::*;

use std::borrow::Cow;
// use macroquad_particles::{self as particles, BlendMode, ColorCurve, EmitterConfig};

static WIN_TEXT: &'static str = "Congratulations!";
static GAMEOVER_TEXT: &'static str = "Game Over";
static COMPLETE_TEXT: &'static str = "Congratulations! You beat the game!";
static PAUSE_TEXT: &'static str = "Paused";
static PAUSE_HINT: &'static str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPress escape to resume";

static RESTART_HINT_DESK: &'static str = "Press Space to restart";
static RESTART_HINT_MOBILE: &'static str = "Tap the screen to restart";
static CONTINUE_HINT_DESK: &'static str = "Press Space to continue";
static CONTINUE_HINT_MOBILE: &'static str = "Tap the screen to continue";

static START_TEXT_DESK: &'static str = "Controls";
static START_HINT: &'static str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPRESS SPACE TO START\nGet ready to run!";
static START_TEXT_MOBILE: &'static str = "Tap to start";

pub const WALL_COLOR: Color = Color::from_rgba(51, 51, 84, 255);

pub fn tiles(render: &mut Render, tile_storage: View<TileStorage>, tiles: View<TileType>) {
    let Some(storage) = tile_storage.iter().next() else {
        return;
    };
    let iter = storage
        .iter_poses()
        .map(|(x, y, id)| (x, y, tiles.get(id).unwrap()));

    for (x, y, tile) in iter {
        match tile {
            TileType::Wall => {
                render.world.add_entity((
                    Tint(WALL_COLOR),
                    Scale(vec2(2.0, 2.0)),
                    Sprite {
                        origin: vec2(0.5, 0.5),
                        texture: TextureKey("wall"),
                    },
                    Transform {
                        pos: vec2(32.0 * x as f32, 32.0 * y as f32),
                        angle: 0.0,
                    },
                ));
            }
            TileType::Ground => (),
        }
    }
}

pub fn player(render: &mut Render, pos: View<Transform>, player: View<PlayerTag>) {
    for (_, pos) in (&player, &pos).iter() {
        render.world.add_entity((
            *pos,
            RectShape {
                origin: vec2(0.5, 0.5),
                width: 16.0,
                height: 16.0,
            },
            Tint(PURPLE),
        ));
    }
}

pub fn game_ui(
    render: &mut Render,
    score: View<PlayerScore>,
    health: View<Health>,
    player: View<PlayerTag>,
) {
    let font_size = 32;
    let off_y = 32.0;
    let ui_x = 536.0;
    let score = score.iter().next().unwrap().0;
    let player_health = (&player, &health).iter().next().unwrap().1 .0;
    let (game_state, game_state_color) = if player_health <= 0 {
        ("You are dead", RED)
    } else {
        ("", BLANK)
    };

    render.world.add_entity((
        GlyphText {
            font: FontKey("oegnek"),
            string: Cow::Owned(format!("Score:{score}")),
            font_size,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
        },
        Tint(YELLOW),
        Transform::from_xy(ui_x, off_y * 1.0),
    ));
    render.world.add_entity((
        GlyphText {
            font: FontKey("oegnek"),
            string: Cow::Owned(format!("Health:{player_health}")),
            font_size,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
        },
        Tint(YELLOW),
        Transform::from_xy(ui_x, off_y * 2.0),
    ));
    render.world.add_entity((
        GlyphText {
            font: FontKey("oegnek"),
            string: Cow::Borrowed(game_state),
            font_size: 64,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
        },
        Tint(game_state_color),
        Transform::from_xy(ui_x, off_y * 5.0),
    ));
}

pub fn goal(render: &mut Render, pos: View<Transform>, goal: View<GoalTag>) {
    for (pos, _) in (&pos, &goal).iter() {
        render.world.add_entity((
            *pos,
            Tint(GREEN),
            RectShape {
                origin: vec2(0.5, 0.5),
                width: 16.0,
                height: 16.0,
            },
        ));
    }
}

pub fn toplevel_ui(app_state: &AppState, render: &mut Render) {
    match app_state {
        AppState::Start => {
            render.world.add_entity(AnnouncementText {
                heading: start_text(),
                body: Some(START_HINT),
            });
        }
        AppState::GameOver => {
            render.world.add_entity(AnnouncementText {
                heading: GAMEOVER_TEXT,
                body: Some(game_restart_hint()),
            });
        }
        AppState::Win => {
            render.world.add_entity(AnnouncementText {
                heading: WIN_TEXT,
                body: Some(game_continue_hint()),
            });
        }
        AppState::Active { paused: true } => {
            render.world.add_entity(AnnouncementText {
                heading: PAUSE_TEXT,
                body: Some(PAUSE_HINT),
            });
        }
        AppState::PleaseRotate => {
            render.world.add_entity(AnnouncementText {
                heading: ORIENTATION_TEXT,
                body: Some(ORIENTATION_HINT),
            });
        }
        AppState::GameDone => {
            render.world.add_entity(AnnouncementText {
                heading: COMPLETE_TEXT,
                body: Some(game_restart_hint()),
            });
        }
        _ => (),
    }
}

fn start_text() -> &'static str {
    if lib_game::sys::on_mobile() {
        START_TEXT_MOBILE
    } else {
        START_TEXT_DESK
    }
}

fn game_restart_hint() -> &'static str {
    if lib_game::sys::on_mobile() {
        RESTART_HINT_MOBILE
    } else {
        RESTART_HINT_DESK
    }
}

fn game_continue_hint() -> &'static str {
    if lib_game::sys::on_mobile() {
        CONTINUE_HINT_MOBILE
    } else {
        CONTINUE_HINT_DESK
    }
}

// fn trail() -> particles::EmitterConfig {
//     particles::EmitterConfig {
//         emitting: true,
//         lifetime: 1.2,
//         lifetime_randomness: 0.7,
//         explosiveness: 0.01,
//         amount: 15,
//         initial_direction_spread: 0.4 * std::f32::consts::PI,
//         initial_velocity: 100.0,
//         size: 1.0,
//         gravity: vec2(0.0, 1000.0),
//         blend_mode: BlendMode::Alpha,
//         emission_shape: macroquad_particles::EmissionShape::Sphere { radius: BALL_RADIUS },
//         colors_curve: ColorCurve {
//             start: Color::from_hex(0xDDFBFF),
//             mid: BLANK,
//             end: BLANK,
//         },
//         ..Default::default()
//     }
// }

// fn explosion() -> particles::EmitterConfig {
//     particles::EmitterConfig {
//         one_shot: true,
//         emitting: false,
//         lifetime: 0.3,
//         lifetime_randomness: 0.7,
//         explosiveness: 0.99,
//         amount: 30,
//         initial_direction_spread: 2.0 * std::f32::consts::PI,
//         initial_velocity: 200.0,
//         size: 1.5,
//         gravity: vec2(0.0, 1000.0),
//         blend_mode: BlendMode::Alpha,
//         emission_shape: macroquad_particles::EmissionShape::Rect {
//             width: BOX_WIDTH,
//             height: BOX_HEIGHT,
//         },
//         colors_curve: ColorCurve {
//             start: Color::from_hex(0x333354),
//             mid: Color::from_hex(0x333354),
//             end: BLACK,
//         },
//         ..Default::default()
//     }
// }

// fn ball_explosion() -> particles::EmitterConfig {
//     particles::EmitterConfig {
//         one_shot: true,
//         emitting: false,
//         lifetime: 1.0,
//         lifetime_randomness: 0.7,
//         explosiveness: 0.99,
//         amount: 10,
//         initial_direction_spread: 2.0 * std::f32::consts::PI,
//         initial_velocity: 100.0,
//         size: 20.0,
//         gravity: vec2(0.0, -1000.0),
//         blend_mode: BlendMode::Alpha,
//         emission_shape: macroquad_particles::EmissionShape::Sphere { radius: BALL_RADIUS * 4.0 },
//         initial_angular_velocity: 5.0,
//         angular_accel: 0.0,
//         angular_damping: 0.01,
//         colors_curve: ColorCurve {
//             start: Color::from_hex(0xDDFBFF),
//             mid: Color { r: 1.0, g: 0.0, b: 0.0, a: 0.0, },
//             end: BLANK,
//         },
//         ..Default::default()
//     }
// }
