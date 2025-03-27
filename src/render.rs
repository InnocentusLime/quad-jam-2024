use jam_macro::method_system;
use macroquad::prelude::*;
use shipyard::{Get, IntoIter, Unique, View};

use crate::{physics::{ColliderTy, PhysicsInfo}, BallState, BoxTag, BruteTag, BulletTag, EnemyState, Health, PlayerDamageState, PlayerTag, TileStorage, TileType, Transform};
// use macroquad_particles::{self as particles, BlendMode, ColorCurve, EmitterConfig};

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

#[derive(Unique)]
pub struct Render {
    // ball_emit: particles::Emitter,
    // pl_emit: particles::Emitter,
    // brick_emit: particles::Emitter,
    // ball_exp: particles::Emitter,
    tiles: Texture2D,
    render_colliders: bool,
}

impl Render {
    pub async fn new() -> anyhow::Result<Self> {
        let tiles = load_texture("assets/tiles.png").await?;
        Ok(Self {
            tiles,
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

    #[method_system]
    pub fn new_frame(
        &mut self,
    ) {
        self.setup_cam();

        clear_background(Color {
            r: 0.0,
            g: 0.0,
            b: 0.02,
            a: 1.0,
        });
    }

    #[method_system]
    pub fn draw_tiles(
        &mut self,
        tile_storage: View<TileStorage>,
        tiles: View<TileType>,
    ) {
        for storage in tile_storage.iter() {
            storage.iter_poses()
                .map(|(x, y, id)| (x, y, tiles.get(id).unwrap()))
                .for_each(|(x, y, id)| match id {
                    TileType::Wall => draw_texture_ex(
                        &self.tiles,
                        32.0 * x as f32,
                        32.0 * y as f32,
                        Color::from_rgba(51, 51, 84, 255),
                        DrawTextureParams {
                            dest_size: Some(vec2(32.0, 32.0)),
                            source: Some(Rect {
                                x: 232.0,
                                y: 304.0,
                                w: 16.0,
                                h: 16.0,
                            }),
                            rotation: 0.0,
                            flip_x: false,
                            flip_y: false,
                            pivot: Some(vec2(0.5, 0.5)),
                        },
                    ),
                    TileType::Ground => (),
                });
        }
    }

    #[method_system]
    pub fn draw_player(
        &mut self,
        pos: View<Transform>,
        player: View<PlayerTag>,
        health: View<Health>,
        dmg: View<PlayerDamageState>,
    ) {
        for (_, pos, health, dmg) in (&player, &pos, &health, &dmg).iter() {
            let is_flickering = matches!(dmg, PlayerDamageState::Cooldown(_));
            let color = if is_flickering && (get_time() * 1000.0) as u32 % 2 == 0 {
               Color::new(0.0, 0.0, 0.0, 0.0)
            }
            else if health.0 <= 0 { Color::new(0.0, 0.0, 0.0, 0.0) }
            else { PURPLE };

            draw_rectangle_ex(
                pos.pos.x,
                pos.pos.y,
                16.0,
                16.0,
                DrawRectangleParams {
                    // offset: Vec2::ZERO,
                    offset: vec2(0.5, 0.5),
                    rotation: pos.angle,
                    color,
                },
            );
        }
    }

    #[method_system]
    pub fn draw_brute(
        &mut self,
        pos: View<Transform>,
        brute: View<BruteTag>,
        state: View<EnemyState>,
    ) {
        for (_, pos, state) in (&brute, &pos, &state).iter() {
            if matches!(state, EnemyState::Dead) {
                continue;
            }

            let is_flickering = matches!(state, EnemyState::Stunned { .. });
            let color = if is_flickering && (get_time() * 1000.0) as u32 % 2 == 0 {
               Color::new(0.0, 0.0, 0.0, 0.0)
            } else { RED };

            draw_circle(
                pos.pos.x,
                pos.pos.y,
                8.0,
                color,
            );
        }
    }

    #[method_system]
    pub fn draw_box(
        &mut self,
        pos: View<Transform>,
        boxt: View<BoxTag>,
    ) {
        for (_, pos) in (&boxt, &pos).iter() {
            draw_rectangle_ex(
                pos.pos.x,
                pos.pos.y,
                32.0,
                32.0,
                DrawRectangleParams {
                    // offset: Vec2::ZERO,
                    offset: vec2(0.5, 0.5),
                    rotation: pos.angle,
                    color: YELLOW,
                },
            );
        }
    }

    #[method_system]
    pub fn draw_ballohurt(
        &mut self,
        pos: View<Transform>,
        ball_state: View<BallState>,
    ) {
        for (pos, ball) in (&pos, &ball_state).iter() {
            match ball {
                BallState::InPocket => (),
                _ => draw_circle(
                    pos.pos.x,
                    pos.pos.y,
                    16.0,
                    GREEN,
                ),
            }
        }
    }

    #[method_system]
    pub fn draw_bullets(
        &mut self,
        pos: View<Transform>,
        bullet: View<BulletTag>,
    ) {
        for (pos, _) in (&pos, &bullet).iter() {
            draw_rectangle_ex(
                pos.pos.x,
                pos.pos.y,
                16.0,
                16.0,
                DrawRectangleParams {
                    // offset: Vec2::ZERO,
                    offset: vec2(0.5, 0.5),
                    rotation: pos.angle,
                    color: YELLOW,
                },
            );
        }
    }

    #[method_system]
    pub fn draw_colliders(
        &mut self,
        phys: View<PhysicsInfo>,
        pos: View<Transform>,
    ) {
        if self.render_colliders {
            for (col, tf) in (&phys, &pos).iter() {
                match col.col() {
                    ColliderTy::Box { width, height } => draw_rectangle_lines_ex(
                        tf.pos.x,
                        tf.pos.y,
                        *width,
                        *height,
                        1.0,
                        DrawRectangleParams {
                            // offset: Vec2::ZERO,
                            offset: vec2(0.5, 0.5),
                            rotation: tf.angle,
                            color: RED,
                        },
                    ),
                    ColliderTy::Circle { radius } => draw_circle_lines(
                        tf.pos.x,
                        tf.pos.y,
                        *radius,
                        1.0,
                        RED
                    ),
                }
            }
        }
    }

    fn setup_cam(&mut self) {
        // let view_width = (screen_width() / screen_height()) * physics::MAX_Y;
        // let mut cam = Camera2D::from_display_rect(Rect {
        //     x: -(view_width - physics::MAX_X) / 2.0,
        //     y: 0.0,
        //     w: view_width,
        //     h: physics::MAX_Y,
        // });
        // cam.zoom.y *= -1.0;

        // set_camera(&cam);
    }
}