//! Various utils to work with shapes. This features include:
//! * Projecting a shape onto an axis
//! * Separating axis theorem

use glam::{Affine2, Vec2, Vec4, vec2};

pub const MAX_AXIS_NORMALS: usize = 8;
pub const SHAPE_TOI_EPSILON: f32 = std::f32::EPSILON * 100.0f32;
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

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Rect { width: f32, height: f32 },
    Circle { radius: f32 },
}

impl Shape {
    /// Tries to apply the separating axis theorem.
    /// This function tries the axes given by [Shape::separating_axes].
    /// Ref: https://en.wikipedia.org/wiki/Hyperplane_separation_theorem
    pub fn is_separated(&self, other: &Shape, tf1: Affine2, tf2: Affine2) -> bool {
        let mut axis_buff = [Vec2::ZERO; MAX_AXIS_NORMALS * 2];
        let n1 = self.separating_axes(tf1, 0, &mut axis_buff);
        let n2 = other.separating_axes(tf2, n1, &mut axis_buff);

        (0..n1 + n2)
            .map(|idx| axis_buff[idx])
            .any(|axis_normal| self.try_separating_axis(other, tf1, tf2, axis_normal))
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
        l_proj[1] < r_proj[0]
    }

    /// Computes time of impact by using the separating axis theorem.
    /// In addition provides the impact normal.
    /// While this is implemented for circles, the result might not be
    /// as precise as desired.
    pub fn time_of_impact(
        &self,
        other: &Shape,
        tf1: Affine2,
        tf2: Affine2,
        direction: Vec2,
        t_max: f32,
    ) -> Option<(f32, Vec2)> {
        let mut axis_buff = [Vec2::ZERO; MAX_AXIS_NORMALS * 2];
        let n1 = self.separating_axes(tf1, 0, &mut axis_buff);
        let n2 = other.separating_axes(tf2, n1, &mut axis_buff);

        (0..n1 + n2)
            .map(|idx| axis_buff[idx])
            .filter_map(|axis_normal| {
                self.candidate_time_of_impact(other, tf1, tf2, axis_normal, direction, t_max)
            })
            .max_by(|(t1, _), (t2, _)| f32::total_cmp(t1, t2))
    }

    /// Computes the time of impact for a fixed axis.
    /// The axis is encoded with its normal: axis_normal.
    /// `axis_normal` must be a normalized vector.
    pub fn candidate_time_of_impact(
        &self,
        other: &Shape,
        tf1: Affine2,
        tf2: Affine2,
        axis_normal: Vec2,
        direction: Vec2,
        t_max: f32,
    ) -> Option<(f32, Vec2)> {
        let proj1 = self.project(tf1, axis_normal);
        let proj2 = other.project(tf2, axis_normal);
        let dproj = axis_normal.dot(direction);

        // Do not process cases when movement is parallel to the
        // separation axis.
        if dproj <= SHAPE_TOI_EPSILON {
            return None;
        }

        let t = if proj1[0] < proj2[0] {
            (proj2[0] - proj1[1]) / dproj
        } else {
            (proj1[0] - proj2[1]) / dproj
        };

        let push_normal = if dproj >= 0.0 {
            -axis_normal
        } else {
            axis_normal
        };

        if t <= 0.0 || t > t_max {
            None
        } else {
            Some((t, push_normal))
        }
    }

    /// Provides potential separating axes for a shape.
    /// * `tf` -- transform for `self`
    /// * `offset` -- offset into the buffer
    /// * `out` -- the buffer to write into. Must be at least [MAX_AXIS_NORMALS] long.
    ///
    /// The returned value is the amount of axes written.
    pub fn separating_axes(&self, tf: Affine2, offset: usize, out: &mut [Vec2]) -> usize {
        match self {
            Shape::Rect { .. } => {
                let normals = rect_normals(tf);
                let n = normals.len();
                out[offset..offset + n].copy_from_slice(&normals);
                n
            }
            Shape::Circle { .. } => {
                let normals = circle_normals(tf);
                let n = normals.len();
                out[offset..offset + n].copy_from_slice(&normals);
                n
            }
        }
    }

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
        let proj = match *self {
            Shape::Rect { width, height } => project_rect(vec2(width, height), tf, axis),
            Shape::Circle { radius } => project_circle(radius, tf, axis),
        };
        debug_assert!(proj[0] <= proj[1], "projection not ordered");

        proj
    }
}

/// Returns transformed rectangle normals
pub fn rect_normals(tf: Affine2) -> [Vec2; 4] {
    RECT_NORMALS.map(|n| tf.transform_vector2(n))
}

/// Returns transformed rectangle points
pub fn rect_points(size: Vec2, tf: Affine2) -> [Vec2; 4] {
    RECT_VERTICES
        .map(|v| v * size / 2.0)
        .map(|v| tf.transform_point2(v))
}

/// Projects a rectangle transformed by tf onto axis:
/// * `size` -- the rectangle sides (width, height)
/// * `tf` -- rectangle transform
/// * `axis` -- the axis
pub fn project_rect(size: Vec2, tf: Affine2, axis: Vec2) -> [f32; 2] {
    let projections = Vec4::from_array(rect_points(size, tf).map(|v| v.dot(axis)));
    [projections.min_element(), projections.max_element()]
}

/// Returns transformed circle normals
pub fn circle_normals(tf: Affine2) -> [Vec2; 8] {
    CIRCLE_NORMALS.map(|n| tf.transform_vector2(n))
}

/// Returns transformed circle points
pub fn circle_points(radius: f32, tf: Affine2) -> [Vec2; 8] {
    CIRCLE_VERTICES
        .map(|v| v * radius)
        .map(|v| tf.transform_point2(v))
}

/// Projects a circle transformed by tf onto axis:
/// * `radius` -- the circle radius
/// * `tf` -- circle transform
/// * `axis` -- the axis
pub fn project_circle(radius: f32, tf: Affine2, axis: Vec2) -> [f32; 2] {
    let projections = circle_points(radius, tf).map(|v| v.dot(axis));
    let projections1 = Vec4::from_slice(&projections[0..4]);
    let projections2 = Vec4::from_slice(&projections[4..8]);
    [
        f32::min(projections1.min_element(), projections2.min_element()),
        f32::max(projections1.max_element(), projections2.max_element()),
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
