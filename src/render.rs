use std::borrow::Cow;

use macroquad::prelude::*;
use shipyard::{Get, IntoIter, UniqueView, View, World};

use crate::logic::*;
use lib_game::*;
use crate::components::*;
// use macroquad_particles::{self as particles, BlendMode, ColorCurve, EmitterConfig};

const FONT_SCALE: f32 = 1.0;
const MAIN_FONT_SIZE: u16 = 32;
const HINT_FONT_SIZE: u16 = 16;
const VERTICAL_ORIENT_HORIZONTAL_PADDING: f32 = 16.0;

static WIN_TEXT: &'static str = "Congratulations!";
static GAMEOVER_TEXT: &'static str = "Game Over";
static PAUSE_TEXT: &'static str = "Paused";
static PAUSE_HINT: &'static str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPress escape to resume";
static ORIENTATION_TEXT: &'static str = "Wrong Orientation";

static RESTART_HINT_DESK: &'static str = "Press Space to restart";
static RESTART_HINT_MOBILE: &'static str = "Tap the screen to restart";
static ORIENTATION_HINT: &'static str = "Please re-orient your device\ninto landscape";

static START_TEXT_DESK: &'static str = "Controls";
static START_HINT: &'static str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPRESS SPACE TO START\nGet ready to run!";
static START_TEXT_MOBILE: &'static str = "Tap to start";


pub const WALL_COLOR: Color = Color::from_rgba(51, 51, 84, 255);

pub fn render_tiles(
    export_world: &mut World,
    tile_storage: View<TileStorage>,
    tiles: View<TileType>,
) {
    let Some(storage) = tile_storage.iter().next()
        else { return; };
    let iter = storage.iter_poses()
            .map(|(x, y, id)| (x, y, tiles.get(id).unwrap()));

    for (x, y, tile) in iter {
        match tile {
            TileType::Wall => { export_world.add_entity((
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
            )); },
            TileType::Ground => (),
        }
    }
}

pub fn render_player(
    export_world: &mut World,
    pos: View<Transform>,
    player: View<PlayerTag>,
    dmg: View<PlayerDamageState>,
) {
    for (_, pos, dmg) in (&player, &pos, &dmg).iter() {
        let is_flickering = matches!(dmg, PlayerDamageState::Cooldown(_));

        let r_player = export_world.add_entity((
            *pos,
            RectShape {
                origin: vec2(0.5, 0.5),
                width: 16.0,
                height: 16.0,
            },
            Tint(PURPLE),
        ));

        if is_flickering {
            export_world.add_component(r_player, Flicker);
        }
    }
}

pub fn render_brute(
    export_world: &mut World,
    pos: View<Transform>,
    brute: View<BruteTag>,
    state: View<EnemyState>,
    hp: View<Health>,
) {
    for (_, pos, state, hp) in (&brute, &pos, &state, &hp).iter() {
        if matches!(state, EnemyState::Dead) {
            continue;
        }

        let k = hp.0 as f32 / BRUTE_SPAWN_HEALTH as f32;
        let is_flickering = matches!(state, EnemyState::Stunned { .. });
        let color = Color::new(RED.r * k, RED.g * k, RED.b * k, 1.0);

        let r_enemy = export_world.add_entity((
            *pos,
            CircleShape { radius: 8.0 },
            Tint(color),
        ));

        if is_flickering {
            export_world.add_component(r_enemy, Flicker);
        }
    }
}

pub fn render_boxes(
    export_world: &mut World,
    pos: View<Transform>,
    boxt: View<BoxTag>,
) {
    for (_, pos) in (&boxt, &pos).iter() {
        export_world.add_entity((
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
    export_world: &mut World,
    pos: View<Transform>,
    ray: View<RayTag>,
    beam: View<BeamTag>,
) {
    for (pos, ray, beam) in (&pos, &ray, &beam).iter() {
        if !ray.shooting { continue; }

        export_world.add_entity((
            Tint(GREEN),
            *pos,
            RectShape {
                origin: vec2(0.0, 0.5),
                height: PLAYER_RAY_WIDTH,
                width: beam.length,
            },
            Scale(vec2(1.0, 1.0)),
            Timed::new(PLAYER_RAY_LINGER),
            VertShrinkFadeoutAnim,
        ));
    }
}

pub fn render_ammo(
    export_world: &mut World,
    pos: View<Transform>,
    bullet: View<BulletTag>,
) {
    for (pos, bul) in (&pos, &bullet).iter() {
        if bul.is_picked { continue; }

        export_world.add_entity((
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
    export_world: &mut World,
    score: UniqueView<PlayerScore>,
    health: View<Health>,
    player: View<PlayerTag>,
    gun: View<PlayerGunState>,
    state: View<EnemyState>,
) {
    let font_size = 32;
    let off_y = 32.0;
    let ui_x = 600.0;
    let score = score.0;
    let player_health = (&player, &health).iter().next().unwrap().1.0;
    let player_gun = *(&gun,).iter().next().unwrap();
    let alive_enemy_count = state.iter()
        .filter(|x| !matches!(x, EnemyState::Dead))
        .count();

    let gun_state = match player_gun {
        PlayerGunState::Empty => "Your gun is empty",
        PlayerGunState::Full => "Gun loaded",
    };
    let (game_state, game_state_color) = if alive_enemy_count == 0 {
        ("You win", GREEN)
    } else if player_health <= 0 {
        ("You are dead", RED)
    } else {
        ("", BLANK)
    };

    export_world.add_entity((
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
    export_world.add_entity((
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
    export_world.add_entity((
        GlyphText {
            font: FontKey("oegnek"),
            string: Cow::Borrowed(gun_state),
            font_size,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
        },
        Tint(YELLOW),
        Transform::from_xy(ui_x, off_y * 3.0),
    ));
    export_world.add_entity((
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

pub struct Render {
    // ball_emit: particles::Emitter,
    // pl_emit: particles::Emitter,
    // brick_emit: particles::Emitter,
    // ball_exp: particles::Emitter,
    tiles: Texture2D,
    oegnek: Font,
    render_world: bool,
    render_colliders: bool,
}

impl Render {
    pub async fn new() -> anyhow::Result<Self> {
        let tiles = load_texture("assets/tiles.png").await?;
        Ok(Self {
            tiles,
            oegnek: load_ttf_font("assets/oegnek.ttf").await?,
            render_world: true,
            render_colliders: false,
            // ball_emit: particles::Emitter::new(EmitterConfig {
            //     texture: None,
            //     ..trail()
            // }),
            // pl_emit: particles::Emitter::new(EmitterConfig {
            //     texture: None,
            //     ..trail()
            // }),
            // brick_emit:  particles::Emitter::new(EmitterConfig {
            //     texture: None,
            //     ..explosion()
            // }),
            // ball_exp:  particles::Emitter::new(EmitterConfig {
            //     texture: Some(sad),
            //     ..ball_explosion()
            // }),
        })
    }

    pub fn render_ui(
        &mut self,
        state: AppState,
    ) {
        set_camera(&self.get_cam());

        if lib_game::sys::on_mobile() && state == AppState::Active {
            /* Mobile controls */
        }

        match state {
            AppState::Start => self.draw_announcement_text(
                true,
                Self::start_text(),
                Some(START_HINT),
            ),
            AppState::GameOver => self.draw_announcement_text(
                true,
                GAMEOVER_TEXT,
                Some(Self::game_restart_hint()),
            ),
            AppState::Win => self.draw_announcement_text(
                false,
                WIN_TEXT,
                Some(Self::game_restart_hint()),
            ),
            AppState::Paused => self.draw_announcement_text(
                true,
                PAUSE_TEXT,
                Some(PAUSE_HINT),
            ),
            AppState::PleaseRotate => self.draw_announcement_text(
                true,
                ORIENTATION_TEXT,
                Some(ORIENTATION_HINT),
            ),
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