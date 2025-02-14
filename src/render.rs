use macroquad::prelude::*;
use shipyard::{Get, IntoIter, Unique, View};

use crate::{method_as_system, physics::{ColliderTy, PhysicsInfo}, MobType, TileStorage, TileType, Transform};
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

    pub fn draw(
        &mut self,
        phys: View<PhysicsInfo>,
        pos: View<Transform>,
        tile_storage: View<TileStorage>,
        tiles: View<TileType>,
        mob: View<MobType>,
    ) {
        self.setup_cam();

        clear_background(Color {
            r: 0.0,
            g: 0.0,
            b: 0.02,
            a: 1.0,
        });

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

        for (mob, pos) in (&mob, &pos).iter() {
            match mob {
                MobType::Player => draw_rectangle_ex(
                    pos.pos.x,
                    pos.pos.y,
                    16.0,
                    16.0,
                    DrawRectangleParams {
                        // offset: Vec2::ZERO,
                        offset: vec2(0.5, 0.5),
                        rotation: pos.angle,
                        color: PURPLE,
                    },
                ),
                MobType::Box => draw_rectangle_ex(
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
                ),
            }
        }

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

method_as_system!(
    Render::draw as render_draw(
        this: Render,
        phys: View<PhysicsInfo>,
        pos: View<Transform>,
        storage: View<TileStorage>,
        tiles: View<TileType>,
        mob: View<MobType>
    )
);