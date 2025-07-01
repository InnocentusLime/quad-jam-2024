use glam::{Affine2, Vec2, vec2};
use imageproc::drawing;
use quad_col::{rect_points, Shape};

const TOI_ESTIMATE_EPSILON: f32 = 0.00001;
const LENGTH_ESTIMATE_EPSILON: f32 = std::f32::EPSILON * 16.0;

const TRANSFORM_COUNT: usize = 10;
const OUT_IMG_WIDTH: u32 = 1024;
const OUT_IMG_HEIGHT: u32 = 1024;

#[derive(Debug, Clone)]
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
            toi_estimate: Some((8.0f32*8.0f32 + 16.0f32*16.0f32).sqrt()),
            toi_max: 100.0,
        },
    ]
}

fn random_translation_and_angle() -> (Vec2, f32) {
    let trans_x_increment = rand::random_range(-2..2);
    let trans_y_increment = rand::random_range(-2..2);
    let angle_increment = rand::random_range(-3..3);

    let trans_x = trans_x_increment as f32 * 16.0;
    let trans_y = trans_y_increment as f32 * 16.0;
    let angle = std::f32::consts::FRAC_PI_2 / 2.0 * angle_increment as f32;

    (vec2(trans_x, trans_y), angle)
}

/// The initial test cases are quite simple. We can catch a few more bugs by
/// randomly offsetting and rotating the whole scene. Such transformation
/// will not change the intersection result.
fn transform_test(case: ShapeCastTest) -> impl IntoIterator<Item = ShapeCastTest> {
    let original_case = case.clone();
    let cases = std::iter::repeat_n(case, TRANSFORM_COUNT).map(|case| {
        let (translation, angle) = random_translation_and_angle();
        let scene_transform = Affine2::from_angle_translation(angle, translation);
        ShapeCastTest {
            tf1: scene_transform * case.tf1,
            tf2: scene_transform * case.tf2,
            cast_dir: scene_transform.transform_vector2(case.cast_dir),
            ..case
        }
    });
    std::iter::once(original_case).chain(cases)
}

fn draw_cast_dir(canvas: &mut image::RgbImage, color: image::Rgb<u8>, dir: Vec2, tf: Affine2) {
    let tf = Affine2::from_translation(vec2(
        OUT_IMG_WIDTH as f32 / 2.0,
        OUT_IMG_HEIGHT as f32 / 2.0,
    )) * Affine2::from_scale(Vec2::splat(4.0))
        * tf;
    let start = tf.transform_point2(Vec2::ZERO);
    let end = start + 32.0 * dir;

    drawing::draw_line_segment_mut(
        canvas, 
        (start.x, start.y), 
        (end.x, end.y), 
        color,
    );
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

fn draw_test(case: &ShapeCastTest) {
    let mut img = image::RgbImage::new(OUT_IMG_WIDTH, OUT_IMG_HEIGHT);
    img.fill(0);
    draw_shape(&mut img, image::Rgb([255, 0, 0]), case.shape1, case.tf1);
    draw_shape(&mut img, image::Rgb([0, 255, 0]), case.shape2, case.tf2);
    draw_cast_dir(&mut img, image::Rgb([0, 0, 255]), case.cast_dir, case.tf1);
    img.save_with_format("test-out.png", image::ImageFormat::Png)
        .unwrap();
}

#[test]
fn test_shape_casts() {
    let cases = 
        shape_cast_tests()
        .into_iter()
        .flat_map(transform_test);
    for case in cases {
        assert!(
            (case.cast_dir.length() - 1.0).abs() < LENGTH_ESTIMATE_EPSILON,
            "TEST: {:?}. Bad cast_dir: {}",
            case.name,
            case.cast_dir.length(),
        );
        let res = Shape::time_of_impact(
            &case.shape1,
            &case.shape2,
            case.tf1,
            case.tf2,
            case.cast_dir,
            case.toi_max,
        );
        if res.is_some() != case.toi_estimate.is_some() {
            draw_test(&case);
        }
        assert_eq!(
            res.is_some(), 
            case.toi_estimate.is_some(),
            "TEST: {:?}. Incorrect response for shape {:?} by {} and {:?} by {}. Expected {}, got {}",
            case.name,
            case.shape1,
            case.tf1,
            case.shape2,
            case.tf2,
            case.toi_estimate.is_some(),
            res.is_some(),
        );
        let Some(target) = case.toi_estimate else {
            continue;
        };
        let res = res.unwrap();
        if (target - res).abs() >= TOI_ESTIMATE_EPSILON {
            draw_test(&case);
        }
        assert!(
            (target - res).abs() < TOI_ESTIMATE_EPSILON,
            "TEST: {:?}. Incorrect response for shape {:?} by {} and {:?} by {}. Expected {res} to be close to {target}",
            case.name,
            case.shape1,
            case.tf1,
            case.shape2,
            case.tf2,
        );
    }
}
