mod common;

use glam::vec2;
use lib_col::Aabb;

use crate::common::{TestCase, draw_aabb, run_tests_no_fuzz};

#[test]
fn basic_aabb() {
    run_tests_no_fuzz([
        RectOverlapTest {
            name: "basic no-overlap",
            rect1: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(100.0, 100.0),
            },
            rect2: Aabb {
                min: vec2(101.0, 101.0),
                max: vec2(202.0, 202.0),
            },
            expected: false,
        },
        RectOverlapTest {
            name: "basic no-overlap swap",
            rect1: Aabb {
                min: vec2(101.0, 101.0),
                max: vec2(202.0, 202.0),
            },
            rect2: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(100.0, 100.0),
            },
            expected: false,
        },
        RectOverlapTest {
            name: "basic overlap",
            rect1: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(103.0, 103.0),
            },
            rect2: Aabb {
                min: vec2(101.0, 101.0),
                max: vec2(202.0, 202.0),
            },
            expected: true,
        },
        RectOverlapTest {
            name: "basic overlap swap",
            rect1: Aabb {
                min: vec2(101.0, 101.0),
                max: vec2(202.0, 202.0),
            },
            rect2: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(103.0, 103.0),
            },
            expected: true,
        },
        RectOverlapTest {
            name: "overlap by x",
            rect1: Aabb {
                min: vec2(0.0, 20.0),
                max: vec2(60.0, 10.0),
            },
            rect2: Aabb {
                min: vec2(50.0, 0.0),
                max: vec2(100.0, 40.0),
            },
            expected: true,
        },
    ]);
}

#[derive(Debug, Clone, Copy)]
struct RectOverlapTest {
    name: &'static str,
    rect1: Aabb,
    rect2: Aabb,
    expected: bool,
}

impl TestCase for RectOverlapTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self) -> bool {
        self.rect1.overlaps(self.rect2) == self.expected
    }

    fn draw(&self, canvas: &mut svg::Document) {
        draw_aabb(canvas, self.rect1, "red");
        draw_aabb(canvas, self.rect2, "green");
    }
}
