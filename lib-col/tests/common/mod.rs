use glam::{Affine2, Vec2, vec2};
use lib_col::{Shape, rect_points};
use svg::node::element::{Circle, Path, path::Data};

const TRANSFORM_COUNT: usize = 10;
const OUT_IMG_WIDTH: u32 = 600;
const OUT_IMG_HEIGHT: u32 = 600;
const LINE_THICKNESS: f32 = 1.0;

/// An interface for a test case for removing some boilerplate.
pub trait TestCase: Copy {
    /// The name of the test to use in the test report.
    fn name(&self) -> &'static str;

    /// Run the test and return success of failure.
    /// If you have a super helpful problem to report that
    /// the calling code can't see -- print it to stdout.
    fn check(&self) -> bool;

    /// Draw a visual aid to `canvas`.
    fn draw(&self, canvas: &mut svg::Document);
}

/// Additional API when your test in invariant against
/// various transformations.
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
    let mut document = svg::Document::new().set("viewBox", (0, 0, OUT_IMG_WIDTH, OUT_IMG_HEIGHT));
    case.draw(&mut document);
    svg::save("test-out.svg", &document).unwrap();
}

#[allow(dead_code)]
pub fn draw_vector(canvas: &mut svg::Document, color: &str, dir: Vec2, tf: Affine2) {
    let tf = Affine2::from_translation(vec2(
        OUT_IMG_WIDTH as f32 / 2.0,
        OUT_IMG_HEIGHT as f32 / 2.0,
    )) * tf;
    let start = tf.transform_point2(Vec2::ZERO);
    let end = start + 32.0 * dir;

    let data = Data::new()
        .move_to((start.x, start.y))
        .line_to((end.x, end.y));
    let path = Path::new()
        .set("fill", "none")
        .set("stroke", color)
        .set("stroke-width", LINE_THICKNESS)
        .set("d", data);
    *canvas = canvas.clone().add(path);
}

#[allow(dead_code)]
pub fn draw_shape(canvas: &mut svg::Document, color: &str, shape: Shape, tf: Affine2) {
    let tf = Affine2::from_translation(vec2(
        OUT_IMG_WIDTH as f32 / 2.0,
        OUT_IMG_HEIGHT as f32 / 2.0,
    )) * tf;
    match shape {
        Shape::Rect { width, height } => {
            let points = rect_points(vec2(width, height), tf);

            let data = Data::new()
                .move_to((points[0].x, points[0].y))
                .line_to((points[1].x, points[1].y))
                .line_to((points[2].x, points[2].y))
                .line_to((points[3].x, points[3].y))
                .close();
            let path = Path::new()
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", LINE_THICKNESS)
                .set("d", data);
            *canvas = canvas.clone().add(path);
        }
        Shape::Circle { radius } => {
            let center = tf.transform_point2(Vec2::ZERO);

            let circle = Circle::new()
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", LINE_THICKNESS)
                .set("cx", center.x)
                .set("cy", center.y)
                .set("r", radius);
            *canvas = canvas.clone().add(circle);
        }
    }
}
