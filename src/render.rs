use std::borrow::Cow;

use macroquad::prelude::*;
use shipyard::{Get, IntoIter, UniqueView, View};

use crate::components::*;
use lib_game::*;
// use macroquad_particles::{self as particles, BlendMode, ColorCurve, EmitterConfig};

static WIN_TEXT: &'static str = "Congratulations!";
static GAMEOVER_TEXT: &'static str = "Game Over";
static PAUSE_TEXT: &'static str = "Paused";
static PAUSE_HINT: &'static str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPress escape to resume";

static RESTART_HINT_DESK: &'static str = "Press Space to restart";
static RESTART_HINT_MOBILE: &'static str = "Tap the screen to restart";

static START_TEXT_DESK: &'static str = "Controls";
static START_HINT: &'static str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPRESS SPACE TO START\nGet ready to run!";
static START_TEXT_MOBILE: &'static str = "Tap to start";

pub const WALL_COLOR: Color = Color::from_rgba(51, 51, 84, 255);

pub fn render_tiles(render: &mut Render, tile_storage: View<TileStorage>, tiles: View<TileType>) {
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

pub fn render_player(
    render: &mut Render,
    pos: View<Transform>,
    player: View<PlayerTag>,
    dmg: View<PlayerDamageState>,
) {
    for (_, pos, dmg) in (&player, &pos, &dmg).iter() {
        let is_flickering = matches!(dmg, PlayerDamageState::Cooldown(_));

        let r_player = render.world.add_entity((
            *pos,
            RectShape {
                origin: vec2(0.5, 0.5),
                width: 16.0,
                height: 16.0,
            },
            Tint(PURPLE),
        ));

        if is_flickering {
            render.world.add_component(r_player, Flicker);
        }
    }
}

pub fn render_main_cell(
    render: &mut Render,
    pos: View<Transform>,
    brute: View<MainCellTag>,
    state: View<EnemyState>,
    hp: View<Health>,
) {
    for (_, pos, state, hp) in (&brute, &pos, &state, &hp).iter() {
        if matches!(state, EnemyState::Dead) {
            continue;
        }

        let k = hp.0 as f32 / crate::enemy::BRUTE_SPAWN_HEALTH as f32;
        let is_flickering = matches!(state, EnemyState::Stunned { .. });
        let color = Color::new(RED.r * k, RED.g * k, RED.b * k, 1.0);

        let r_enemy = render
            .world
            .add_entity((*pos, CircleShape { radius: 12.0 }, Tint(color)));

        if is_flickering {
            render.world.add_component(r_enemy, Flicker);
        }
    }
}

pub fn render_brute(
    render: &mut Render,
    pos: View<Transform>,
    brute: View<BruteTag>,
    state: View<EnemyState>,
) {
    for (_, pos, state) in (&brute, &pos, &state).iter() {
        if matches!(state, EnemyState::Dead) {
            continue;
        }

        let k = 0.4;
        let is_flickering = matches!(state, EnemyState::Stunned { .. });
        let color = Color::new(BLUE.r * k, BLUE.g * k, BLUE.b * k, 1.0);

        let r_enemy = render
            .world
            .add_entity((*pos, CircleShape { radius: 6.0 }, Tint(color)));

        if is_flickering {
            render.world.add_component(r_enemy, Flicker);
        }
    }
}

pub fn render_stalker(
    render: &mut Render,
    pos: View<Transform>,
    brute: View<StalkerTag>,
    state: View<EnemyState>,
    hp: View<Health>,
) {
    for (_, pos, state, hp) in (&brute, &pos, &state, &hp).iter() {
        if matches!(state, EnemyState::Dead) {
            continue;
        }

        let k = hp.0 as f32 / crate::enemy::BRUTE_SPAWN_HEALTH as f32;
        let is_flickering = matches!(state, EnemyState::Stunned { .. });
        let color = Color::new(BLUE.r * k, BLUE.g * k, BLUE.b * k, 1.0);

        let r_enemy = render
            .world
            .add_entity((*pos, CircleShape { radius: 6.0 }, Tint(color)));

        if is_flickering {
            render.world.add_component(r_enemy, Flicker);
        }
    }
}

pub fn render_boxes(render: &mut Render, pos: View<Transform>, boxt: View<BoxTag>) {
    for (_, pos) in (&boxt, &pos).iter() {
        render.world.add_entity((
            *pos,
            RectShape {
                origin: vec2(0.5, 0.5),
                width: 32.0,
                height: 32.0,
            },
            Tint(WALL_COLOR),
        ));
    }
}

pub fn render_rays(
    render: &mut Render,
    pos: View<Transform>,
    ray: View<RayTag>,
    beam: View<BeamTag>,
) {
    for (pos, ray, beam) in (&pos, &ray, &beam).iter() {
        if !ray.shooting {
            continue;
        }

        render.world.add_entity((
            Tint(GREEN),
            *pos,
            RectShape {
                origin: vec2(0.0, 0.5),
                height: crate::player::PLAYER_RAY_WIDTH,
                width: beam.length,
            },
            Scale(vec2(1.0, 1.0)),
            Timed::new(crate::player::PLAYER_RAY_LINGER),
            VertShrinkFadeoutAnim,
        ));
    }
}

pub fn render_ammo(render: &mut Render, pos: View<Transform>, bullet: View<BulletTag>) {
    for (pos, bul) in (&pos, &bullet).iter() {
        if matches!(bul, BulletTag::PickedUp) {
            continue;
        }

        render.world.add_entity((
            *pos,
            Tint(YELLOW),
            RectShape {
                origin: vec2(0.5, 0.5),
                width: 16.0,
                height: 16.0,
            },
        ));
    }
}

pub fn render_game_ui(
    render: &mut Render,
    score: UniqueView<PlayerScore>,
    health: View<Health>,
    player: View<PlayerTag>,
    state: View<EnemyState>,
) {
    let font_size = 32;
    let off_y = 32.0;
    let ui_x = 536.0;
    let score = score.0;
    let player_health = (&player, &health).iter().next().unwrap().1 .0;
    let alive_enemy_count = state
        .iter()
        .filter(|x| !matches!(x, EnemyState::Dead))
        .count();
    let (game_state, game_state_color) = if alive_enemy_count == 0 {
        ("You win", GREEN)
    } else if player_health <= 0 {
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

pub fn render_goal(render: &mut Render, pos: View<Transform>, goal: View<GoalTag>) {
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

pub fn render_toplevel_ui(app_state: &AppState, render: &mut Render) {
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
                body: Some(game_restart_hint()),
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
