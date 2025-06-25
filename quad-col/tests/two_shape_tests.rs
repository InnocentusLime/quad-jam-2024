use glam::{Affine2, Vec2, vec2};
use imageproc::image;

use quad_col::{Shape, rect_points};

#[derive(Clone, Debug)]
struct TwoShapesTest {
    name: &'static str,
    tf1: Affine2,
    shape1: Shape,
    tf2: Affine2,
    shape2: Shape,
    expected_result: bool,
}

const TRANSFORM_COUNT: usize = 10;
const OUT_IMG_WIDTH: u32 = 1024;
const OUT_IMG_HEIGHT: u32 = 1024;

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

fn random_translation_and_angle() -> (Vec2, f32) {
    let trans_x_increment = rand::random_range(-8..8);
    let trans_y_increment = rand::random_range(-8..8);
    let angle_increment = rand::random_range(-3..3);

    let trans_x = trans_x_increment as f32 * 16.0;
    let trans_y = trans_y_increment as f32 * 16.0;
    let angle = std::f32::consts::FRAC_PI_2 / 2.0 * angle_increment as f32;

    (vec2(trans_x, trans_y), angle)
}

/// The initial test cases are quite simple. We can catch a few more bugs by
/// randomly offsetting and rotating the whole scene. Such transformation
/// will not change the intersection result.
fn transform_test(case: TwoShapesTest) -> impl IntoIterator<Item = TwoShapesTest> {
    let original_case = case.clone();
    let cases = std::iter::repeat_n(case, TRANSFORM_COUNT).map(|case| {
        let (translation, angle) = random_translation_and_angle();
        let scene_transform = Affine2::from_angle_translation(angle, translation);
        TwoShapesTest {
            tf1: scene_transform * case.tf1,
            tf2: scene_transform * case.tf2,
            ..case
        }
    });
    std::iter::once(original_case).chain(cases)
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

fn draw_shape(canvas: &mut image::RgbImage, color: image::Rgb<u8>, shape: Shape, tf: Affine2) {
    let tf = Affine2::from_translation(vec2(
        OUT_IMG_WIDTH as f32 / 2.0,
        OUT_IMG_HEIGHT as f32 / 2.0,
    )) * Affine2::from_scale(Vec2::splat(4.0))
        * tf;
    match shape {
        Shape::Rect { width, height } => {
            let points = rect_points(vec2(width, height), tf);
            let points = points.map(|v| imageproc::point::Point { x: v.x, y: v.y });
            imageproc::drawing::draw_hollow_polygon_mut(canvas, &points, color);
        }
        Shape::Circle { radius } => {
            let center = tf.transform_point2(Vec2::ZERO);
            imageproc::drawing::draw_hollow_circle_mut(
                canvas,
                (center.x as i32, center.y as i32),
                radius as i32,
                color,
            );
        }
    }
}

fn draw_test(case: &TwoShapesTest) {
    let mut img = image::RgbImage::new(OUT_IMG_WIDTH, OUT_IMG_HEIGHT);
    img.fill(0);
    draw_shape(&mut img, image::Rgb([255, 0, 0]), case.shape1, case.tf1);
    draw_shape(&mut img, image::Rgb([0, 255, 0]), case.shape2, case.tf2);
    img.save_with_format("test-out.png", image::ImageFormat::Png)
        .unwrap();
}

#[test]
fn test_simple_intersections() {
    let cases = two_shapes_tests()
        .into_iter()
        .flat_map(swap_test)
        .flat_map(transform_test);
    for case in cases {
        let res = !Shape::is_separated(&case.shape1, &case.shape2, case.tf1, case.tf2);
        // Dump a visual aid if the test is failing
        if res != case.expected_result {
            draw_test(&case);
        }
        assert_eq!(
            res, case.expected_result,
            "TEST: {:?}. Incorrect response for shape {:?} by {} and {:?} by {}",
            case.name, case.shape1, case.tf1, case.shape2, case.tf2,
        );
    }
}
