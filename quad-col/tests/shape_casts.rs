mod common;

use common::{TestCase, draw_shape, draw_vector, run_tests};
use glam::{Affine2, Vec2, vec2};
use quad_col::Shape;

const TOI_ESTIMATE_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy)]
struct ShapeCastTest {
    name: &'static str,
    tf1: Affine2,
    shape1: Shape,
    tf2: Affine2,
    shape2: Shape,
    cast_dir: Vec2,
    toi_estimate: Option<f32>,
    toi_max: f32,
}

impl TestCase for ShapeCastTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn transform(self, tf: Affine2) -> Self {
        ShapeCastTest {
            tf1: tf * self.tf1,
            tf2: tf * self.tf2,
            cast_dir: tf.transform_vector2(self.cast_dir),
            ..self
        }
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
            (Some(target), Some(result)) if (target - result).abs() < TOI_ESTIMATE_EPSILON => true,
            (Some(target), Some(result)) => {
                println!(
                    "Bad TOI! Expected result {} to be close to {}",
                    result, target
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

    fn draw(&self, canvas: &mut image::RgbImage) {
        draw_shape(canvas, image::Rgb([255, 0, 0]), self.shape1, self.tf1);
        draw_shape(canvas, image::Rgb([0, 255, 0]), self.shape2, self.tf2);
        draw_vector(canvas, image::Rgb([0, 0, 255]), self.cast_dir, self.tf1);
    }
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
            toi_estimate: Some(24.0),
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
            toi_estimate: Some(24.0),
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
            toi_estimate: Some(24.0),
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
            toi_estimate: Some(24.0),
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
                height: 8.0,
            },
            cast_dir: Vec2::from_angle((0.5f32).atan()),
            toi_estimate: Some((8.0f32 * 8.0f32 + 16.0f32 * 16.0f32).sqrt()),
            toi_max: 100.0,
        },
    ]
}
