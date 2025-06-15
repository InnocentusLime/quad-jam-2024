///! A simple trick is used to here to assume that shape 1 is located at the origin.
///! Originally, shape1 is affected by tf1 and shape2 by tf2.
///!
///! If we apply inverse of tf2 to both shapes, shape2's tf gets canceled.
///! This allows much simpler checks. E.g. we can always assume that one of the rects
///! is axis-aligned.
///!
///! The following are the assumptions about how the untransformed shapes are placed:
///! 1. The circle center is placed in (0, 0)
///! 2. The rectangle's center is placed in (0, 0)
///!
///! The coordinate system is as follows: Y goes up, X goes right.
use glam::{Affine2, Vec2, vec2, vec4};

/// Transform both centers and performs a distance check.
pub fn collision_circle_circle(tf1: Affine2, r1: f32, tf2: Affine2, r2: f32) -> bool {
    let p1 = tf1.transform_point2(Vec2::ZERO);
    let p2 = tf2.transform_point2(Vec2::ZERO);
    Vec2::distance(p1, p2) <= r1 + r2
}

/// Apply the trick on the circle, keeping rect axis-aligned.
/// Then perform the separating axis theorem.
pub fn collision_circle_rect(tf1: Affine2, r1: f32, tf2: Affine2, wh2: Vec2) -> bool {
    let shape1tf = tf2.inverse() * tf1;
    let center1 = shape1tf.transform_point2(Vec2::ZERO);
    let Vec2 {
        x: r_axis,
        y: t_axis,
    } = wh2 / 2.0;
    let (l_axis, b_axis) = (-r_axis, -t_axis);

    // Separating axis theorem
    !(center1.x + r1 < l_axis
        || center1.x - r1 > r_axis
        || center1.y + r1 < b_axis
        || center1.y - r1 > t_axis)
}

pub fn collision_rect_rect(tf1: Affine2, wh1: Vec2, tf2: Affine2, wh2: Vec2) -> bool {
    collision_rect_rect_pass(tf1, wh1, tf2, wh2) && 
    collision_rect_rect_pass(tf2, wh2, tf1, wh1)
}

/// Apply the trick on rect1, keeping rect2 axis-aligned.
/// Then perform the separating axis theorem.
fn collision_rect_rect_pass(tf1: Affine2, wh1: Vec2, tf2: Affine2, wh2: Vec2) -> bool {
    let shape1tf = tf2.inverse() * tf1;
    let p1 = [
        shape1tf.transform_point2(vec2(-wh1.x, -wh1.y) / 2.0),
        shape1tf.transform_point2(vec2(wh1.x, -wh1.y) / 2.0),
        shape1tf.transform_point2(vec2(wh1.x, wh1.y) / 2.0),
        shape1tf.transform_point2(vec2(-wh1.x, wh1.y) / 2.0),
    ];
    let x_coords = vec4(p1[0].x, p1[1].x, p1[2].x, p1[3].x);
    let y_coords = vec4(p1[0].y, p1[1].y, p1[2].y, p1[3].y);
    let Vec2 {
        x: r_axis,
        y: t_axis,
    } = wh2 / 2.0;
    let (l_axis, b_axis) = (-r_axis, -t_axis);

    // Separating axis theorem
    !(x_coords.max_element() < l_axis
        || x_coords.min_element() > r_axis
        || y_coords.max_element() < b_axis
        || y_coords.min_element() > t_axis)
}
