use glam::{Affine2, Vec2, vec2};
use lib_col::Aabb;

use crate::common::{TestCase, draw_aabb, draw_vector, run_tests_no_fuzz};

mod common;

#[test]
fn raycast_aabb() {
    run_tests_no_fuzz([
        RectRaycastTest {
            name: "basic-left",
            rect: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(10.0, 10.0),
            },
            t_max: f32::INFINITY,
            origin: vec2(-10.0, 5.0),
            direction: Vec2::X,
            expected: true,
        },
        RectRaycastTest {
            name: "basic-left too-short",
            rect: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(10.0, 10.0),
            },
            t_max: 1.0,
            origin: vec2(-10.0, 5.0),
            direction: Vec2::X,
            expected: false,
        },
        RectRaycastTest {
            name: "basic-left miss 1",
            rect: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(10.0, 10.0),
            },
            t_max: f32::INFINITY,
            origin: vec2(-10.0, 5.0),
            direction: Vec2::Y,
            expected: false,
        },
        RectRaycastTest {
            name: "basic-left miss 2",
            rect: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(10.0, 10.0),
            },
            t_max: f32::INFINITY,
            origin: vec2(-10.0, 5.0),
            direction: Vec2::NEG_Y,
            expected: false,
        },
        RectRaycastTest {
            name: "basic-left on edge",
            rect: Aabb {
                min: vec2(0.0, 0.0),
                max: vec2(10.0, 10.0),
            },
            t_max: f32::INFINITY,
            origin: vec2(0.0, 5.0),
            direction: Vec2::X,
            expected: true,
        },
    ]);
}

#[derive(Debug, Clone, Copy)]
struct RectRaycastTest {
    name: &'static str,
    rect: Aabb,
    t_max: f32,
    origin: Vec2,
    direction: Vec2,
    expected: bool,
}

impl TestCase for RectRaycastTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self) -> bool {
        self.rect
            .cast_point(self.origin, self.direction, self.t_max)
            == self.expected
    }

    fn draw(&self, canvas: &mut svg::Document) {
        draw_aabb(canvas, self.rect, "red");
        let tf = Affine2::from_translation(self.origin);
        draw_vector(canvas, "green", self.direction, tf);
    }
}
