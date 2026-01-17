//! Various utils to work with shapes. This features include:
//! * Projecting a shape onto an axis
//! * Separating axis theorem

use glam::{Affine2, Vec2, vec2};

pub const MAX_AXIS_NORMALS: usize = 8;
pub const SHAPE_TOI_EPSILON: f32 = f32::EPSILON * 100.0f32;
pub static RECT_VERTICES: [Vec2; 4] = [
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, -1.0),
];
/// Untransformed rectangle normals
pub static RECT_NORMALS: [Vec2; 4] = [
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, -1.0),
    vec2(-1.0, 0.0),
];
pub const FRAC_SQRT_2_2: f32 = std::f32::consts::SQRT_2 / 2.0;
pub static CIRCLE_VERTICES: [Vec2; 8] = [
    vec2(1.0, 0.0),
    vec2(FRAC_SQRT_2_2, FRAC_SQRT_2_2),
    vec2(0.0, 1.0),
    vec2(-FRAC_SQRT_2_2, FRAC_SQRT_2_2),
    vec2(-1.0, 0.0),
    vec2(-FRAC_SQRT_2_2, -FRAC_SQRT_2_2),
    vec2(0.0, -1.0),
    vec2(FRAC_SQRT_2_2, -FRAC_SQRT_2_2),
];
pub const CIRCLE_SIDE: f32 = 0.7653668647301795434569199680607;
pub static CIRCLE_NORMALS: [Vec2; 8] = [
    vec2(
        -(0.0 - FRAC_SQRT_2_2) / CIRCLE_SIDE,
        (1.0 - FRAC_SQRT_2_2) / CIRCLE_SIDE,
    ),
    vec2(
        -(FRAC_SQRT_2_2 - 1.0) / CIRCLE_SIDE,
        (FRAC_SQRT_2_2 - 0.0) / CIRCLE_SIDE,
    ),
    vec2(
        -(1.0 - FRAC_SQRT_2_2) / CIRCLE_SIDE,
        (0.0 - (-FRAC_SQRT_2_2)) / CIRCLE_SIDE,
    ),
    vec2(
        -(FRAC_SQRT_2_2 - 0.0) / CIRCLE_SIDE,
        (-FRAC_SQRT_2_2 - (-1.0)) / CIRCLE_SIDE,
    ),
    vec2(
        -(0.0 - (-FRAC_SQRT_2_2)) / CIRCLE_SIDE,
        (-1.0 - (-FRAC_SQRT_2_2)) / CIRCLE_SIDE,
    ),
    vec2(
        -(-FRAC_SQRT_2_2 - (-1.0)) / CIRCLE_SIDE,
        (-FRAC_SQRT_2_2 - 0.0) / CIRCLE_SIDE,
    ),
    vec2(
        -(-1.0 - (-FRAC_SQRT_2_2)) / CIRCLE_SIDE,
        (0.0 - FRAC_SQRT_2_2) / CIRCLE_SIDE,
    ),
    vec2(
        -(-FRAC_SQRT_2_2 - 0.0) / CIRCLE_SIDE,
        (FRAC_SQRT_2_2 - 1.0) / CIRCLE_SIDE,
    ),
];

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum Shape {
    Rect { width: f32, height: f32 },
    Circle { radius: f32 },
}

impl Shape {
    pub fn write_vertices(self, tf: Affine2, out: &mut Vec<Vec2>) {
        match self {
            Shape::Rect { width, height } => out.extend(rect_points(vec2(width, height), tf)),
            Shape::Circle { radius } => out.extend(circle_points(radius, tf)),
        }
    }

    pub fn write_normals(self, tf: Affine2, out: &mut Vec<Vec2>) {
        match self {
            Shape::Rect { .. } => out.extend(rect_normals(tf)),
            Shape::Circle { .. } => out.extend(circle_normals(tf)),
        }
    }
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Rect {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Returns transformed rectangle normals
pub fn rect_normals(tf: Affine2) -> [Vec2; 4] {
    [
        tf.transform_vector2(RECT_NORMALS[0]),
        tf.transform_vector2(RECT_NORMALS[1]),
        tf.transform_vector2(RECT_NORMALS[2]),
        tf.transform_vector2(RECT_NORMALS[3]),
    ]
}

/// Returns transformed rectangle points
pub fn rect_points(size: Vec2, tf: Affine2) -> [Vec2; 4] {
    [
        tf.transform_point2(RECT_VERTICES[0] * size / 2.0),
        tf.transform_point2(RECT_VERTICES[1] * size / 2.0),
        tf.transform_point2(RECT_VERTICES[2] * size / 2.0),
        tf.transform_point2(RECT_VERTICES[3] * size / 2.0),
    ]
}

/// Returns transformed circle normals
pub fn circle_normals(tf: Affine2) -> [Vec2; 8] {
    [
        tf.transform_vector2(CIRCLE_NORMALS[0]),
        tf.transform_vector2(CIRCLE_NORMALS[1]),
        tf.transform_vector2(CIRCLE_NORMALS[2]),
        tf.transform_vector2(CIRCLE_NORMALS[3]),
        tf.transform_vector2(CIRCLE_NORMALS[4]),
        tf.transform_vector2(CIRCLE_NORMALS[5]),
        tf.transform_vector2(CIRCLE_NORMALS[6]),
        tf.transform_vector2(CIRCLE_NORMALS[7]),
    ]
}

/// Returns transformed circle points
pub fn circle_points(radius: f32, tf: Affine2) -> [Vec2; 8] {
    [
        tf.transform_point2(CIRCLE_VERTICES[0] * radius),
        tf.transform_point2(CIRCLE_VERTICES[1] * radius),
        tf.transform_point2(CIRCLE_VERTICES[2] * radius),
        tf.transform_point2(CIRCLE_VERTICES[3] * radius),
        tf.transform_point2(CIRCLE_VERTICES[4] * radius),
        tf.transform_point2(CIRCLE_VERTICES[5] * radius),
        tf.transform_point2(CIRCLE_VERTICES[6] * radius),
        tf.transform_point2(CIRCLE_VERTICES[7] * radius),
    ]
}

#[cfg(test)]
mod sanity_checks {
    use glam::Vec2;

    use super::{CIRCLE_NORMALS, CIRCLE_VERTICES};

    const NORMAL_EPSILON: f32 = std::f32::EPSILON * 32.0;
    const VERTEX_EPSILON: f32 = std::f32::EPSILON * 32.0;

    #[test]
    fn circle_vertices() {
        let computed_vertices = std::array::from_fn::<_, 8, _>(|idx| {
            let n = idx as f32;
            let angle = std::f32::consts::TAU / 8.0 * n;
            Vec2::from_angle(angle)
        });
        for idx in 0..8 {
            let diff = computed_vertices[idx] - CIRCLE_VERTICES[idx];
            assert!(
                diff.length() < VERTEX_EPSILON,
                "Vertex {idx}. vertices aren't close: {} and {}",
                computed_vertices[idx],
                CIRCLE_VERTICES[idx],
            );

            let length = CIRCLE_VERTICES[idx].length();
            assert!(
                (1.0 - length).abs() < NORMAL_EPSILON,
                "Vertex {idx}. Expected {length} to be close to {}",
                1.0,
            );
        }
    }

    #[test]
    fn circle_normals_length() {
        for (idx, normal) in CIRCLE_NORMALS.into_iter().enumerate() {
            let length = normal.length();
            assert!(
                (1.0 - length).abs() < NORMAL_EPSILON,
                "Normal {idx}. Expected {length} to be close to {}",
                1.0,
            );
        }
    }

    #[test]
    fn circle_normals_dir() {
        let vertices = std::array::from_fn::<_, 9, _>(|idx| CIRCLE_VERTICES[idx % 8]);
        for (nidx, window) in vertices.windows(2).enumerate() {
            let v1 = window[0];
            let v2 = window[1];
            let side = v1 - v2;
            let dot = side.normalize_or_zero().dot(CIRCLE_NORMALS[nidx]);
            assert!(
                (dot - 0.0).abs() < NORMAL_EPSILON,
                "Normal {nidx}. Expected {dot} to be close to {}. Vectors: {} and {}",
                0.0,
                side,
                CIRCLE_NORMALS[nidx],
            );
        }
    }
}
