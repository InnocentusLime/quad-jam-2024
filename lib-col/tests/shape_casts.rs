mod common;

use common::{FuzzableTestCase, TestCase, draw_shape, draw_vector, run_tests};
use glam::{Affine2, Vec2, vec2};
use lib_col::{SHAPE_TOI_EPSILON, Shape, conv};

const TOI_ESTIMATE_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy)]
struct ShapeCastTest {
    name: &'static str,
    tf1: Affine2,
    shape1: Shape,
    tf2: Affine2,
    shape2: Shape,
    cast_dir: Vec2,
    toi_estimate: Option<(f32, Vec2)>,
    toi_max: f32,
}

impl TestCase for ShapeCastTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self) -> bool {
        let res = Shape::time_of_impact(
            &self.shape1,
            &self.shape2,
            self.tf1,
            self.tf2,
            self.cast_dir,
            self.toi_max,
        );

        match (res, self.toi_estimate) {
            (Some((result_toi, result_normal)), Some((target_toi, target_normal)))
                if (target_toi - result_toi).abs() < TOI_ESTIMATE_EPSILON
                    && vecs_same_dir(result_normal, target_normal) =>
            {
                true
            }
            (Some((result_toi, result_normal)), Some((target_toi, target_normal))) => {
                println!(
                    "Bad TOI! Expected result {} to be close to {}",
                    result_toi, target_toi,
                );
                println!(
                    "Bad normal! Expected result {} have same direction as {}",
                    result_normal, target_normal,
                );
                false
            }
            (None, None) => true,
            (Some(_), None) => {
                println!("False positive!");
                false
            }
            (None, Some(_)) => {
                println!("Missed!");
                false
            }
        }
    }

    fn draw(&self, canvas: &mut svg::Document) {
        draw_shape(canvas, "red", self.shape1, self.tf1);
        draw_shape(canvas, "green", self.shape2, self.tf2);
        draw_vector(canvas, "blue", self.cast_dir, self.tf1);
        if let Some((toi_estimate, _)) = self.toi_estimate {
            let impact_tf = Affine2 {
                translation: self.tf1.translation + toi_estimate * self.cast_dir,
                ..self.tf1
            };
            draw_shape(canvas, "yellow", self.shape1, impact_tf);
        }
    }
}

impl FuzzableTestCase for ShapeCastTest {
    fn transform(self, tf: Affine2) -> Self {
        ShapeCastTest {
            tf1: tf * self.tf1,
            tf2: tf * self.tf2,
            cast_dir: tf.transform_vector2(self.cast_dir),
            toi_estimate: self
                .toi_estimate
                .map(|(toi, normal)| (toi, tf.transform_vector2(normal))),
            ..self
        }
    }
}

/// Checks that the vectors are pointing in the same direction as follows:
/// * l and r do not have an obtuse angle: `l.dot(r) >= 0`
/// * l's perpendicular vector is also r's perpendicular vector: `l.perp().dot(r) <= eps`
fn vecs_same_dir(l: Vec2, r: Vec2) -> bool {
    l.perp().dot(r) <= SHAPE_TOI_EPSILON && l.dot(r) >= 0.0
}

#[test]
fn test_shape_casts() {
    run_tests(shape_cast_tests());
}

fn shape_cast_tests() -> impl IntoIterator<Item = ShapeCastTest> {
    [
        // Success rect-rect
        ShapeCastTest {
            name: "aabb (right cast)",
            tf1: Affine2::IDENTITY,
            shape1: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(32.0, 0.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            cast_dir: vec2(1.0, 0.0),
            toi_estimate: Some((24.0, vec2(-1.0, 0.0))),
            toi_max: 100.0,
        },
        ShapeCastTest {
            name: "aabb (left cast)",
            tf1: Affine2::IDENTITY,
            shape1: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(-32.0, 0.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            cast_dir: vec2(-1.0, 0.0),
            toi_estimate: Some((24.0, vec2(1.0, 0.0))),
            toi_max: 100.0,
        },
        ShapeCastTest {
            name: "aabb (top cast)",
            tf1: Affine2::IDENTITY,
            shape1: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(0.0, 32.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            cast_dir: vec2(0.0, 1.0),
            toi_estimate: Some((24.0, vec2(0.0, -1.0))),
            toi_max: 100.0,
        },
        ShapeCastTest {
            name: "aabb (bot cast)",
            tf1: Affine2::IDENTITY,
            shape1: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(0.0, -32.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            cast_dir: vec2(0.0, -1.0),
            toi_estimate: Some((24.0, vec2(0.0, 1.0))),
            toi_max: 100.0,
        },
        ShapeCastTest {
            name: "aabb (touch)",
            tf1: Affine2::IDENTITY,
            shape1: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(24.0, 0.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 10.0,
            },
            cast_dir: Vec2::from_angle((0.5f32).atan()),
            toi_estimate: Some((
                (8.0f32 * 8.0f32 + 16.0f32 * 16.0f32).sqrt(),
                vec2(-1.0, 0.0),
            )),
            toi_max: 100.0,
        },
        // Fail rect-rect
        ShapeCastTest {
            name: "aabb (right cast) (fail)",
            tf1: Affine2::IDENTITY,
            shape1: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(32.0, 0.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            cast_dir: vec2(1.0, 0.0),
            toi_estimate: None,
            toi_max: 10.0,
        },
        ShapeCastTest {
            name: "aabb (miss)",
            tf1: Affine2::IDENTITY,
            shape1: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(32.0, 0.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 8.0,
            },
            cast_dir: vec2(0.0, 1.0),
            toi_estimate: None,
            toi_max: 100.0,
        },
        ShapeCastTest {
            name: "character regression (miss)",
            tf1: Affine2::from_translation(vec2(32.0, -16.0)),
            shape1: Shape::Rect {
                width: 32.0,
                height: 16.0,
            },
            tf2: conv::topleft_corner_tf_to_crate(vec2(97.0, 128.0), 1.0471976),
            shape2: Shape::Circle { radius: 32.0 },
            cast_dir: vec2(-1.0, 0.0),
            toi_estimate: None,
            toi_max: 8.0,
        },
    ]
}
