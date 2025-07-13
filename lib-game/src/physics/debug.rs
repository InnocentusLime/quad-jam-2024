use crate::components::*;
use crate::physics::*;

use hecs::World;

fn draw_queries<const ID: usize>(world: &World) {
    for (_, (tf, query)) in &mut world.query::<(&Transform, &CollisionQuery<ID>)>() {
        let color = if query.has_collided() {
            Color::new(0.00, 0.93, 0.80, 1.00)
        } else {
            GREEN
        };

        match query.collider {
            Shape::Rect { width, height } => draw_rectangle_lines_ex(
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
            Shape::Circle { radius } => draw_circle_lines(tf.pos.x, tf.pos.y, radius, 1.0, color),
        }
    }
}

fn draw_bodies(world: &World) {
    for (_, (tf, tag)) in &mut world.query::<(&Transform, &BodyTag)>() {
        let color = DARKBLUE;

        match tag.shape {
            Shape::Rect { width, height } => draw_rectangle_ex(
                tf.pos.x,
                tf.pos.y,
                width,
                height,
                DrawRectangleParams {
                    offset: vec2(0.5, 0.5),
                    rotation: tf.angle,
                    color,
                },
            ),
            Shape::Circle { radius } => draw_circle(tf.pos.x, tf.pos.y, radius, color),
        }
    }
}

pub fn draw_physics_debug(world: &World) {
    draw_bodies(world);
    draw_queries::<0>(world);
    draw_queries::<1>(world);
    draw_queries::<2>(world);
    draw_queries::<3>(world);
    draw_queries::<4>(world);
    draw_queries::<5>(world);
    draw_queries::<6>(world);
    draw_queries::<7>(world);
}
