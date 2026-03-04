use crate::Render;
use crate::collisions::*;
use crate::components::*;

use hecs::World;
use mimiq::Color;

// fn draw_queries<const ID: usize>(world: &World) {
//     for (_, (tf, query)) in &mut world.query::<(&Transform, &CollisionQuery<ID>)>() {
//         let color = if query.has_collided() {
//             Color::new(0.00, 0.93, 0.80, 1.00)
//         } else {
//             GREEN
//         };

//         draw_shape_lines(tf, &query.collider, color);
//     }
// }

fn draw_bodies(world: &mut World, render: &mut Render) {
    for (_, (tf, tag)) in world.query_mut::<(&Transform, &BodyTag)>() {
        draw_shape(render, tf, &tag.shape, mimiq::DARKBLUE);
    }
}

pub fn draw_physics_debug(world: &mut World, render: &mut Render) {
    draw_bodies(world, render);
    //     draw_queries::<0>(world);
    //     draw_queries::<1>(world);
    //     draw_queries::<2>(world);
    //     draw_queries::<3>(world);
    //     draw_queries::<4>(world);
    //     draw_queries::<5>(world);
    //     draw_queries::<6>(world);
    //     draw_queries::<7>(world);
}

pub fn draw_shape(render: &mut Render, tf: &Transform, shape: &Shape, color: Color) {
    match *shape {
        Shape::Rect { width, height } => {
            render
                .gizmos
                .rect(color, tf.pos, vec2(width, height), tf.angle)
        }
        Shape::Circle { radius } => render.gizmos.circle(color, tf.pos, radius),
    }
}

// pub fn draw_shape_lines(tf: &Transform, shape: &Shape, color: Color) {
//     match *shape {
//         Shape::Rect { width, height } => draw_rectangle_lines_ex(
//             tf.pos.x,
//             tf.pos.y,
//             width,
//             height,
//             1.0,
//             DrawRectangleParams {
//                 offset: vec2(0.5, 0.5),
//                 rotation: tf.angle,
//                 color,
//             },
//         ),
//         Shape::Circle { radius } => draw_circle_lines(tf.pos.x, tf.pos.y, radius, 1.0, color),
//     }
// }
