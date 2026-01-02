use glam::{Vec2, vec2};
use lib_col::Aabb;

use crate::common::{TestCase, draw_aabb, run_tests_no_fuzz};

mod common;

#[test]
fn raycast_aabb() {
    run_tests_no_fuzz([
        ShapecastRectTest {
            name: "basic-left",
            rect1: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(10.0, 10.0),
            },
            rect2: Aabb {
                min: vec2(-12.0, 5.0),
                max: vec2(-12.0, 8.0),
            },
            t_max: f32::INFINITY,
            direction: Vec2::X,
            expected: true,
        },
        ShapecastRectTest {
            name: "basic-left too-short",
            rect1: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(10.0, 10.0),
            },
            rect2: Aabb {
                min: vec2(-12.0, 5.0),
                max: vec2(-12.0, 8.0),
            },
            t_max: 1.0,
            direction: Vec2::X,
            expected: false,
        },
    ]);
}

#[derive(Debug, Clone, Copy)]
struct ShapecastRectTest {
    name: &'static str,
    rect1: Aabb,
    rect2: Aabb,
    t_max: f32,
    direction: Vec2,
    expected: bool,
}

impl TestCase for ShapecastRectTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self) -> bool {
        self.rect1.cast_rect(self.rect2, self.direction, self.t_max) == self.expected
    }

    fn draw(&self, canvas: &mut svg::Document) {
        draw_aabb(canvas, self.rect1, "red");
        draw_aabb(canvas, self.rect2, "green");
    }
}
