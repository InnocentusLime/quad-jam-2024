use macroquad::prelude::*;
use shipyard::{Get, IntoIter, UniqueView, View, World};

use crate::{game::{Game, BRUTE_SPAWN_HEALTH, PLAYER_RAY_LINGER, PLAYER_RAY_WIDTH}, physics::{BeamTag, BodyTag, ColliderTy, OneSensorTag, PhysicsInfo}, BallState, BoxTag, BruteTag, BulletTag, EnemyState, Health, PlayerDamageState, PlayerGunState, PlayerScore, PlayerTag, RayTag, TileStorage, TileType, Transform};
// use macroquad_particles::{self as particles, BlendMode, ColorCurve, EmitterConfig};

pub const WALL_COLOR: Color = Color::from_rgba(51, 51, 84, 255);

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
    render_world: bool,
    render_colliders: bool,
}

impl Render {
    pub async fn new() -> anyhow::Result<Self> {
        let tiles = load_texture("assets/tiles.png").await?;
        Ok(Self {
            tiles,
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

    pub fn render(&mut self, world: &World) {
        world.run_with_data(Self::new_frame, self);

        if self.render_world {
            world.run_with_data(Self::draw_tiles, self);
            world.run_with_data(Self::draw_ballohurt, self);
            world.run_with_data(Self::draw_brute, self);
            world.run_with_data(Self::draw_player, self);
            world.run_with_data(Self::draw_box, self);
            world.run_with_data(Self::draw_box, self);
            world.run_with_data(Self::draw_bullets, self);
            world.run_with_data(Self::draw_rays, self);
        }
        // Debug rendering
        if self.render_colliders {
            world.run_with_data(Self::draw_bodies, self);
            world.run_with_data(Self::draw_one_sensors, self);
            world.run_with_data(Self::draw_beams, self);
        }
        // UI
        world.run_with_data(Self::draw_stats, self);
    }

    fn new_frame(
        &mut self,
        game: UniqueView<Game>,
    ) {
        set_camera(game.camera());

        clear_background(Color {
            r: 0.0,
            g: 0.0,
            b: 0.02,
            a: 1.0,
        });
    }

    fn draw_tiles(
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
                        WALL_COLOR,
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

    fn draw_player(
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

    fn draw_brute(
        &mut self,
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
            let color = if is_flickering && (get_time() * 1000.0) as u32 % 2 == 0 {
                Color::new(0.0, 0.0, 0.0, 0.0)
            } else {
                let mut res = RED;
                res.r *= k;
                res.g *= k;
                res.b *= k;

                res
            };

            draw_circle(
                pos.pos.x,
                pos.pos.y,
                8.0,
                color,
            );
        }
    }

    fn draw_box(
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
                    color: WALL_COLOR,
                },
            );
        }
    }

    fn draw_ballohurt(
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

    fn draw_bullets(
        &mut self,
        pos: View<Transform>,
        bullet: View<BulletTag>,
        score: UniqueView<PlayerScore>,
    ) {
        let ammo_hint = "AMMO";

        for (pos, bul) in (&pos, &bullet).iter() {
            if bul.is_picked { continue; }

            let mes = measure_text(
                &ammo_hint,
                None,
                16,
                1.0
            );

            if score.0 == 0 {
                draw_text(
                    &ammo_hint,
                    pos.pos.x - mes.width / 2.0,
                    pos.pos.y - 20.0,
                    16.0,
                    YELLOW,
                );
            }

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

    fn draw_rays(
        &mut self,
        pos: View<Transform>,
        ray: View<RayTag>,
    ) {
        for (pos, ray) in (&pos, &ray).iter() {
            let k = ray.life_left / PLAYER_RAY_LINGER;
            draw_rectangle_ex(
                pos.pos.x,
                pos.pos.y,
                ray.len,
                PLAYER_RAY_WIDTH * k,
                DrawRectangleParams {
                    offset: vec2(0.0, 0.5),
                    rotation: pos.angle,
                    color: Color {
                        a: k,
                        ..GREEN
                    },
                }
            );
        }
    }

    fn draw_one_sensors(
        &mut self,
        phys: View<PhysicsInfo>,
        pos: View<Transform>,
        sens_tag: View<OneSensorTag>,
    ) {
        for (col, tf, tag) in (&phys, &pos, &sens_tag).iter() {
            let color = if tag.col.is_some() {
                Color::new(0.00, 0.93, 0.80, 1.00)
            } else {
                GREEN
            };

            match col.shape() {
                ColliderTy::Box { width, height } => draw_rectangle_lines_ex(
                    tf.pos.x,
                    tf.pos.y,
                    *width,
                    *height,
                    1.0,
                    DrawRectangleParams {
                        offset: vec2(0.5, 0.5),
                        rotation: tf.angle,
                        color,
                    },
                ),
                ColliderTy::Circle { .. } => (),
            }
        }
    }

    fn draw_beams(
        &mut self,
        phys: View<PhysicsInfo>,
        pos: View<Transform>,
        beam_tag: View<BeamTag>,
    ) {
        // TODO: draw collision count
        for (col, tf, _tag) in (&phys, &pos, &beam_tag).iter() {
            let color = GREEN;

            match col.shape() {
                ColliderTy::Box { width, height } => draw_rectangle_lines_ex(
                    tf.pos.x,
                    tf.pos.y,
                    *width,
                    *height,
                    1.0,
                    DrawRectangleParams {
                        offset: vec2(0.0, 0.5),
                        rotation: tf.angle,
                        color,
                    },
                ),
                // Circle beams are not allowed
                ColliderTy::Circle { radius } => draw_circle_lines(
                    tf.pos.x,
                    tf.pos.y,
                    *radius,
                    1.0,
                    color,
                ),
            }
        }
    }

    fn draw_bodies(
        &mut self,
        phys: View<PhysicsInfo>,
        pos: View<Transform>,
        body_tag: View<BodyTag>,
    ) {
        for (col, tf, tag) in (&phys, &pos, &body_tag).iter() {
            let color = match tag {
                BodyTag::Static => DARKBLUE,
                BodyTag::Dynamic => RED,
                BodyTag::Kinematic => YELLOW,
            };

            match col.shape() {
                ColliderTy::Box { width, height } => draw_rectangle_ex(
                    tf.pos.x,
                    tf.pos.y,
                    *width,
                    *height,
                    DrawRectangleParams {
                        // offset: Vec2::ZERO,
                        offset: vec2(0.5, 0.5),
                        rotation: tf.angle,
                        color,
                    },
                ),
                ColliderTy::Circle { radius } => draw_circle(
                    tf.pos.x,
                    tf.pos.y,
                    *radius,
                    color,
                ),
            }
        }
    }

    fn draw_stats(
        &mut self,
        score: UniqueView<PlayerScore>,
        health: View<Health>,
        player: View<PlayerTag>,
        gun: View<PlayerGunState>,
        state: View<EnemyState>,
    ) {
        let ui_x = 600.0;
        let score = score.0;
        let player_health = (&player, &health).iter().next().unwrap().1.0;
        let player_gun = *(&gun,).iter().next().unwrap().0;

        draw_text(
            &format!("Score:{score}"),
            ui_x,
            32.0,
            32.0,
            YELLOW,
        );
        draw_text(
            &format!("Health:{player_health}"),
            ui_x,
            64.0,
            32.0,
            YELLOW,
        );
        match player_gun {
            PlayerGunState::Empty => draw_text(
                &"Your gun is not loaded",
                ui_x,
                96.0,
                32.0,
                YELLOW,
            ),
            PlayerGunState::Full => draw_text(
                &"Ready to shoot",
                ui_x,
                96.0,
                32.0,
                YELLOW,
            ),
        };

        let count = state.iter()
            .filter(|x| !matches!(x, EnemyState::Dead))
            .count();

        if count == 0 {
            draw_text(
                "YOU WIN!",
                ui_x,
                148.0,
                64.0,
                GREEN,
            );
        } else if player_health <= 0 {
            draw_text(
                "You are dead!",
                ui_x,
                148.0,
                64.0,
                RED,
            );
        }
    }
}