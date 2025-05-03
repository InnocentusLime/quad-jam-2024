use crate::components::*;
use crate::physics::*;

use shipyard::{IntoIter, View, World};

fn draw_one_sensors(pos: View<Transform>, sens_tag: View<OneSensorTag>) {
    for (tf, tag) in (&pos, &sens_tag).iter() {
        let color = if tag.col.is_some() {
            Color::new(0.00, 0.93, 0.80, 1.00)
        } else {
            GREEN
        };

        match tag.shape {
            ColliderTy::Box { width, height } => draw_rectangle_lines_ex(
                tf.pos.x,
                tf.pos.y,
                width,
                height,
                1.0,
                DrawRectangleParams {
                    offset: vec2(0.5, 0.5),
                    rotation: tf.angle,
                    color,
                },
            ),
            ColliderTy::Circle { radius } => {
                draw_circle_lines(tf.pos.x, tf.pos.y, radius, 1.0, color)
            }
        }
    }
}

fn draw_beams(pos: View<Transform>, beam_tag: View<BeamTag>) {
    for (tf, tag) in (&pos, &beam_tag).iter() {
        let color = GREEN;

        draw_rectangle_lines_ex(
            tf.pos.x,
            tf.pos.y,
            tag.length,
            tag.width,
            1.0,
            DrawRectangleParams {
                offset: vec2(0.0, 0.5),
                rotation: tf.angle,
                color,
            },
        );

        // TODO: optimise the frequest allocation away
        draw_text(
            &format!("Cols {}", tag.overlaps.len()),
            tf.pos.x,
            tf.pos.y,
            32.0,
            color,
        );
    }
}

fn draw_bodies(pos: View<Transform>, body_tag: View<BodyTag>) {
    for (tf, tag) in (&pos, &body_tag).iter() {
        let mut color = match tag.kind() {
            BodyKind::Static => DARKBLUE,
            BodyKind::Dynamic => RED,
            BodyKind::Kinematic => YELLOW,
        };
        if !tag.enabled {
            color.r *= 0.5;
            color.g *= 0.5;
            color.b *= 0.5;
        }

        match tag.shape() {
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
            ColliderTy::Circle { radius } => draw_circle(tf.pos.x, tf.pos.y, *radius, color),
        }
    }
}

pub fn draw_physics_debug(world: &World) {
    world.run(draw_one_sensors);
    world.run(draw_beams);
    world.run(draw_bodies);
}
