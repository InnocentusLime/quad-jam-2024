//! Various utils to work with shapes. This features include:
//! * Projecting a shape onto an axis
//! * Separating axis theorem

use glam::{Affine2, Vec2, vec2, vec4};

use super::Shape;

pub const MAX_AXIS_NORMALS: usize = 32;
/// Untransformed rectangle normals
pub static RECT_NORMALS: [Vec2; 4] = [
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, -1.0),
    vec2(-1.0, 0.0),
];

impl Shape {
    /// Projects a shape onto an axis:
    /// * `tf` -- shape transform
    /// * `axis` -- the axis to project onto. Must be a unit vector
    ///
    /// The result is two numbers, which represent a line segment on the axis.
    /// ```graphics
    /// ----------*----------axis--------*---->
    ///           result[0]              result[1]
    /// ```
    pub fn project(&self, tf: Affine2, axis: Vec2) -> [f32; 2] {
        match self {
            Shape::Rect { size } => project_rect(*size, tf, axis),
            Shape::Circle { radius } => project_circle(*radius, tf, axis),
        }
    }

    /// Try applying the separating axis theorem.
    /// The axis is encoded with its normal: axis_normal.
    /// `axis_normal` must be a normalized vector.
    pub fn try_separating_axis(
        &self,
        other: &Shape,
        tf1: Affine2,
        tf2: Affine2,
        axis_normal: Vec2,
    ) -> bool {
        // If an axis separates shapes A and B, then their projections on axis normal
        // do not intersect.
        let proj1 = self.project(tf1, axis_normal);
        let proj2 = other.project(tf2, axis_normal);
        let (l_proj, r_proj) = if proj1[0] < proj2[0] {
            (proj1, proj2)
        } else {
            (proj2, proj1)
        };

        // We now have l_proj and r_proj. Intersection happens only in this case:
        // *--------------*
        // l_proj[0]      l_proj[1]
        //       *--------------*
        //       r_proj[0]      r_proj[1]
        l_proj[1] > r_proj[0]
    }

    /// Provides potential separating axes between two shapes.
    /// * `other` -- the other shape
    /// * `tf1` -- transform for `self`
    /// * `tf2` -- transform for `other`
    /// * `out` -- the buffer to write into. Must be at least [MAX_AXIS_NORMALS] long.
    ///
    /// The returned value is the amount of axes written.
    pub fn separating_axes(
        &self,
        other: &Shape,
        tf1: Affine2,
        tf2: Affine2,
        out: &mut [Vec2],
    ) -> usize {
        debug_assert!(out.len() >= MAX_AXIS_NORMALS, "Buffer too small");
        match (self, other) {
            (Shape::Circle { .. }, Shape::Circle { .. }) => {
                let center1 = tf1.transform_point2(Vec2::ZERO);
                let center2 = tf2.transform_point2(Vec2::ZERO);
                out[0] = (center1 - center2).perp();
                1
            }
            (Shape::Rect { .. }, Shape::Rect { .. }) => {
                out[0..3].copy_from_slice(&rect_normals(tf1));
                out[3..8].copy_from_slice(&rect_normals(tf2));
                8
            }
            // For Circle-Rect separation, rectangle's normals work well enough.
            (Shape::Rect { .. }, Shape::Circle { .. }) => {
                out[0..3].copy_from_slice(&rect_normals(tf1));
                4
            }
            // For Circle-Rect separation, rectangle's normals work well enough.
            (Shape::Circle { .. }, Shape::Rect { .. }) => {
                out[0..3].copy_from_slice(&rect_normals(tf2));
                4
            }
        }
    }

    /// Tries to apply the separating axis theorem.
    /// This function tries the axes given by [Shape::separating_axes].
    pub fn is_separated(&self, other: &Shape, tf1: Affine2, tf2: Affine2) -> bool {
        let mut axis_buff = [Vec2::ZERO; MAX_AXIS_NORMALS];
        let axis_count = self.separating_axes(other, tf1, tf2, &mut axis_buff);
        debug_assert!(axis_count < MAX_AXIS_NORMALS);

        (0..axis_count)
            .map(|idx| axis_buff[idx])
            .any(|axis_normal| self.try_separating_axis(other, tf1, tf2, axis_normal))
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
        tf.transform_point2(vec2(-size.x, -size.y) / 2.0),
        tf.transform_point2(vec2(size.x, -size.y) / 2.0),
        tf.transform_point2(vec2(size.x, size.y) / 2.0),
        tf.transform_point2(vec2(-size.x, size.y) / 2.0),
    ]
}

/// Projects a rectangle transformed by tf onto axis:
/// * `size` -- the rectangle sides (width, height)
/// * `tf` -- rectangle transform
/// * `axis` -- the axis
pub fn project_rect(size: Vec2, tf: Affine2, axis: Vec2) -> [f32; 2] {
    let points = rect_points(size, tf);
    let projections = vec4(
        points[0].dot(axis),
        points[1].dot(axis),
        points[2].dot(axis),
        points[3].dot(axis),
    );
    [projections.min_element(), projections.max_element()]
}

/// Projects a circle transformed by tf onto axis:
/// * `radius` -- the circle radius
/// * `tf` -- circle transform
/// * `axis` -- the axis
pub fn project_circle(radius: f32, tf: Affine2, axis: Vec2) -> [f32; 2] {
    let center = tf.transform_point2(Vec2::ZERO);
    let center_projection = center.dot(axis);
    [center_projection - radius, center_projection + radius]
}
