mod common;

use common::{FuzzableTestCase, TestCase, draw_shape, run_tests};
use glam::{Affine2, Vec2, vec2};

use lib_col::{Collider, CollisionSolver, Group, Shape};

use crate::common::entity;

#[derive(Debug, Clone, Copy)]
struct TwoShapesTest {
    name: &'static str,
    tf1: Affine2,
    shape1: Shape,
    tf2: Affine2,
    shape2: Shape,
    expected_result: bool,
}

impl TestCase for TwoShapesTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self) -> bool {
        let mut solver = CollisionSolver::new();
        solver.fill([(
            entity(1),
            Collider {
                tf: self.tf1,
                shape: self.shape1,
                group: Group::from_id(0),
            },
        )]);
        let res = solver
            .query_overlaps(
                Collider {
                    tf: self.tf2,
                    shape: self.shape2,
                    group: Group::from_id(0),
                },
                Group::empty(),
            )
            .next()
            .is_some();
        if res != self.expected_result {
            println!("Mismatch!");
            false
        } else {
            true
        }
    }

    fn draw(&self, canvas: &mut svg::Document) {
        draw_shape(canvas, "red", self.shape1, self.tf1);
        draw_shape(canvas, "green", self.shape2, self.tf2);
    }
}

impl FuzzableTestCase for TwoShapesTest {
    fn transform(self, tf: Affine2) -> Self {
        TwoShapesTest {
            tf1: tf * self.tf1,
            tf2: tf * self.tf2,
            ..self
        }
    }
}

#[test]
fn test_simple_intersections() {
    run_tests(two_shapes_tests().into_iter().flat_map(swap_test));
}

fn two_shapes_tests() -> impl IntoIterator<Item = TwoShapesTest> {
    [
        // Circle-circle
        TwoShapesTest {
            name: "circles not intersecting",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 8.0 },
            tf2: Affine2::from_translation(vec2(16.0, 0.0)),
            shape2: Shape::Circle { radius: 4.0 },
            expected_result: false,
        },
        TwoShapesTest {
            name: "circles not intersecting (bigger)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 8.0 },
            tf2: Affine2::from_translation(vec2(16.0, 0.0)),
            shape2: Shape::Circle { radius: 6.0 },
            expected_result: false,
        },
        TwoShapesTest {
            name: "circles intersecting",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 12.0 },
            tf2: Affine2::from_translation(vec2(16.0, 0.0)),
            shape2: Shape::Circle { radius: 6.0 },
            expected_result: true,
        },
        TwoShapesTest {
            name: "circles intersecting (containing)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 12.0 },
            tf2: Affine2::from_translation(vec2(16.0, 0.0)),
            shape2: Shape::Circle { radius: 64.0 },
            expected_result: true,
        },
        // Rect-rect
        TwoShapesTest {
            name: "rects not intersecting (simple)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(64.0, 0.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 64.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rects intersecting (horiz)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(64.0, 0.0)),
            shape2: Shape::Rect {
                width: 66.0,
                height: 8.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rects not intersecting (rotated)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_angle_translation(std::f32::consts::FRAC_PI_3, vec2(64.0, 0.0)),
            shape2: Shape::Rect {
                width: 66.0,
                height: 8.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rects intersecting (rotated)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 128.0,
                height: 8.0,
            },
            tf2: Affine2::from_angle_translation(std::f32::consts::FRAC_PI_3, vec2(64.0, 0.0)),
            shape2: Shape::Rect {
                width: 66.0,
                height: 8.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rects not intersecting (top-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(0.0, 64.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 32.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rects intersecting (top-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(0.0, 64.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 256.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rects not intersecting (bot-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(0.0, -64.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 32.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rects intersecting (bot-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(0.0, -64.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 256.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rects not intersecting (left-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(-64.0, 0.0)),
            shape2: Shape::Rect {
                width: 32.0,
                height: 8.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rects intersecting (left-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(-64.0, 0.0)),
            shape2: Shape::Rect {
                width: 72.0,
                height: 8.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rects not intersecting (right-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(64.0, 0.0)),
            shape2: Shape::Rect {
                width: 32.0,
                height: 8.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rects intersecting (right-side)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            tf2: Affine2::from_translation(vec2(64.0, 0.0)),
            shape2: Shape::Rect {
                width: 72.0,
                height: 8.0,
            },
            expected_result: true,
        },
        // Rect-circle
        TwoShapesTest {
            name: "rect and circle not intersecting (right)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(32.0, 0.0)),
            shape2: Shape::Rect {
                width: 16.0,
                height: 8.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle intersecting (right)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(32.0, 0.0)),
            shape2: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (left)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(-32.0, 0.0)),
            shape2: Shape::Rect {
                width: 16.0,
                height: 8.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle intersecting (left)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(-32.0, 0.0)),
            shape2: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (top)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(0.0, 32.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 16.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle intersecting (top)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(0.0, 32.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 64.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (bot)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(0.0, -32.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 16.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle intersecting (bot)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(vec2(0.0, -32.0)),
            shape2: Shape::Rect {
                width: 8.0,
                height: 64.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (top-right)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(16.0, 16.0) + vec2(5.0, 6.0),
            ),
            shape2: Shape::Rect {
                width: 8.0,
                height: 10.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (bot-right)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(16.0, -16.0) + vec2(5.0, -6.0),
            ),
            shape2: Shape::Rect {
                width: 8.0,
                height: 10.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (top-left)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(-16.0, 16.0) + vec2(-5.0, 6.0),
            ),
            shape2: Shape::Rect {
                width: 8.0,
                height: 10.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (bot-left)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(-16.0, -16.0)
                    + vec2(-5.0, -6.0),
            ),
            shape2: Shape::Rect {
                width: 8.0,
                height: 10.0,
            },
            expected_result: false,
        },
        TwoShapesTest {
            name: "rect and circle intersecting (top-right)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(16.0, 16.0) + vec2(5.0, 6.0),
            ),
            shape2: Shape::Rect {
                width: 16.0,
                height: 10.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rect and circle intersecting (bot-right)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(16.0, -16.0) + vec2(5.0, -6.0),
            ),
            shape2: Shape::Rect {
                width: 16.0,
                height: 10.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rect and circle intersecting (top-left)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(-16.0, 16.0) + vec2(-5.0, 6.0),
            ),
            shape2: Shape::Rect {
                width: 16.0,
                height: 10.0,
            },
            expected_result: true,
        },
        TwoShapesTest {
            name: "rect and circle not intersecting (bot-left)",
            tf1: Affine2::from_translation(vec2(0.0, 0.0)),
            shape1: Shape::Circle { radius: 16.0 },
            tf2: Affine2::from_translation(
                Vec2::from_angle(std::f32::consts::FRAC_PI_4) * vec2(-16.0, -16.0)
                    + vec2(-5.0, -6.0),
            ),
            shape2: Shape::Rect {
                width: 16.0,
                height: 10.0,
            },
            expected_result: true,
        },
    ]
}

/// Some collision tests have asymmetric logic. For better coverage, it
/// is better to generate two tests, where the shapes are swapped for the
/// intersection test function.
fn swap_test(case: TwoShapesTest) -> impl IntoIterator<Item = TwoShapesTest> {
    let swapped_case = TwoShapesTest {
        tf1: case.tf2,
        shape1: case.shape2,
        tf2: case.tf1,
        shape2: case.shape1,
        ..case
    };

    [case, swapped_case]
}
