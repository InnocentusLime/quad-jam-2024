use macroquad::prelude::*;
use shipyard::{IntoIter, View, ViewMut, World};

use crate::{game_model::GameModel, physics::{PhysBox, RapierHandle}, Follower, Pos};
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

pub struct Render {
    // ball_emit: particles::Emitter,
    // pl_emit: particles::Emitter,
    // brick_emit: particles::Emitter,
    // ball_exp: particles::Emitter,
}

impl Render {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
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

    pub fn draw(&mut self, world: &mut World) {
        self.setup_cam();

        clear_background(Color {
            r: 0.0,
            g: 0.0,
            b: 0.02,
            a: 1.0,
        });

        world.run(|follow: View<Follower>, pos: View<Pos>| {
            for (_, pos) in (&follow, &pos).iter() {
                draw_rectangle(
                    pos.0.x,
                    pos.0.y,
                    32.0,
                    32.0,
                    GREEN,
                );
            }
        });

        world.run(|phys: View<PhysBox>, pos: View<Pos>| {
            for (pbox, _) in (&phys, &pos).iter() {
                draw_rectangle_lines(
                    pbox.min.x,
                    pbox.min.y,
                    pbox.max.x - pbox.min.x,
                    pbox.max.y - pbox.min.y,
                    1.0,
                    RED,
                );
            }
        });
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