use crate::components::*;
use crate::physics::*;

use shipyard::{IntoIter, View, World};

fn draw_queries<const ID: usize>(pos: View<Transform>, query: View<CollisionQuery<ID>>) {
    for (tf, query) in (&pos, &query).iter() {
        let color = if query.has_collided() {
            Color::new(0.00, 0.93, 0.80, 1.00)
        } else {
            GREEN
        };

        match query.collider {
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

fn draw_bodies(pos: View<Transform>, body_tag: View<BodyTag>) {
    for (tf, tag) in (&pos, &body_tag).iter() {
        let mut color = match tag.kind() {
            BodyKind::Static => DARKBLUE,
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
    world.run(draw_bodies);
    world.run(draw_queries::<0>);
    world.run(draw_queries::<1>);
    world.run(draw_queries::<2>);
    world.run(draw_queries::<3>);
    world.run(draw_queries::<4>);
    world.run(draw_queries::<5>);
    world.run(draw_queries::<6>);
    world.run(draw_queries::<7>);
}
