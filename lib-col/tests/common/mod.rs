use glam::{Affine2, Vec2, vec2};
use imageproc::drawing;
use lib_col::{Shape, rect_points};

const TRANSFORM_COUNT: usize = 10;
const OUT_IMG_WIDTH: u32 = 1024;
const OUT_IMG_HEIGHT: u32 = 1024;

/// An interface for a test case. All tests in this crate have one
/// thing in common. Their result must be the same if the scene gets
/// rotated or moved by some offset.
pub trait TestCase: Copy {
    /// The name of the test to use in the test report.
    fn name(&self) -> &'static str;

    /// Run the test and return success of failure.
    /// If you have a super helpful problem to report that
    /// the calling code can't see -- print it to stdout.
    fn check(&self) -> bool;

    /// Draw a visual aid to `canvas`.
    fn draw(&self, canvas: &mut image::RgbImage);
}

pub trait FuzzableTestCase: TestCase + Copy {
    /// Apply the transform on the test.
    fn transform(self, tf: Affine2) -> Self;
}

#[allow(dead_code)]
pub fn run_tests_no_fuzz<T: TestCase>(tests: impl IntoIterator<Item = T>) {
    for case in tests.into_iter() {
        println!("Running {:?}", case.name());
        if !case.check() {
            draw_test(&case);
            panic!("Test {:?} failed. Visual aid dumped.", case.name());
        }
    }
}

#[allow(dead_code)]
pub fn run_tests<T: FuzzableTestCase>(tests: impl IntoIterator<Item = T>) {
    let extended = tests.into_iter().flat_map(transform_test);
    for case in extended {
        println!("Running {:?}", case.name());
        if !case.check() {
            draw_test(&case);
            panic!("Test {:?} failed. Visual aid dumped.", case.name());
        }
    }
}

/// Generates a few copies of the same test, but applies a random transform
/// to each one.
fn transform_test<T: FuzzableTestCase>(case: T) -> impl IntoIterator<Item = T> {
    let original_case = case;
    let cases = std::iter::repeat_n(case, TRANSFORM_COUNT).map(|case| {
        let (translation, angle) = random_translation_and_angle();
        let scene_transform = Affine2::from_angle_translation(angle, translation);
        case.transform(scene_transform)
    });
    std::iter::once(original_case).chain(cases)
}

fn random_translation_and_angle() -> (Vec2, f32) {
    let trans_x_increment = rand::random_range(-3..3);
    let trans_y_increment = rand::random_range(-3..3);
    let angle_increment = rand::random_range(-3..3);

    let trans_x = trans_x_increment as f32 * 16.0;
    let trans_y = trans_y_increment as f32 * 16.0;
    let angle = std::f32::consts::FRAC_PI_3 / 1.5 * angle_increment as f32;

    (vec2(trans_x, trans_y), angle)
}

fn draw_test<T: TestCase>(case: &T) {
    let mut img = image::RgbImage::new(OUT_IMG_WIDTH, OUT_IMG_HEIGHT);
    img.fill(0);
    case.draw(&mut img);
    img.save_with_format("test-out.png", image::ImageFormat::Png)
        .unwrap();
}

#[allow(dead_code)]
pub fn draw_vector(canvas: &mut image::RgbImage, color: image::Rgb<u8>, dir: Vec2, tf: Affine2) {
    let tf = Affine2::from_translation(vec2(
        OUT_IMG_WIDTH as f32 / 2.0,
        OUT_IMG_HEIGHT as f32 / 2.0,
    )) * Affine2::from_scale(Vec2::splat(4.0))
        * tf;
    let start = tf.transform_point2(Vec2::ZERO);
    let end = start + 32.0 * dir;

    drawing::draw_line_segment_mut(canvas, (start.x, start.y), (end.x, end.y), color);
}

#[allow(dead_code)]
pub fn draw_shape(canvas: &mut image::RgbImage, color: image::Rgb<u8>, shape: Shape, tf: Affine2) {
    let tf = Affine2::from_translation(vec2(
        OUT_IMG_WIDTH as f32 / 2.0,
        OUT_IMG_HEIGHT as f32 / 2.0,
    )) * Affine2::from_scale(vec2(4.0, -4.0))
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
