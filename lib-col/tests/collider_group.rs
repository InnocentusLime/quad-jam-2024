mod common;

use common::{TestCase, draw_shape, run_tests_no_fuzz};
use glam::{Affine2, Mat2, vec2};
use std::collections::HashSet;
use lib_col::{Collider, CollisionSolver, Group, Shape};

use crate::common::entity;

#[derive(Debug, Clone, Copy)]
struct GroupTest {
    name: &'static str,
    expected: &'static [usize],
    group: Group,
}

impl GroupTest {
    fn get_query(&self) -> Collider {
        Collider {
            shape: Shape::Circle { radius: 10.0 },
            group: self.group,
            tf: Affine2::from_translation(vec2(0.0, 0.0)),
        }
    }
}

impl TestCase for GroupTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self) -> bool {
        let mut solver = CollisionSolver::new();
        solver.fill((0..CIRCLE_COUNT).map(|idx| (entity(idx), CIRCLES[idx])));
        let expected = self
            .expected
            .iter()
            .map(|x| entity(*x))
            .collect::<HashSet<_>>();
        let matched = solver
            .query_overlaps(self.get_query(), Group::empty())
            .collect::<HashSet<_>>();

        matched == expected
    }

    fn draw(&self, canvas: &mut svg::Document) {
        let colors = ["red", "green", "blue"];
        for (circle, color) in CIRCLES.into_iter().zip(colors) {
            draw_shape(canvas, color, circle.shape, circle.tf);
        }

        let query = self.get_query();
        draw_shape(canvas, "white", query.shape, query.tf);
    }
}

#[test]
fn test_group_intersections() {
    run_tests_no_fuzz(tests());
}

fn tests() -> impl IntoIterator<Item = GroupTest> {
    [
        GroupTest {
            name: "group[1, 2]",
            expected: &[1, 2],
            group: Group::from_id(1).union(Group::from_id(2)),
        },
        GroupTest {
            name: "group[0, 2]",
            expected: &[0, 2],
            group: Group::from_id(0).union(Group::from_id(2)),
        },
        GroupTest {
            name: "group[0, 1]",
            expected: &[0, 1],
            group: Group::from_id(0).union(Group::from_id(1)),
        },
    ]
}

const CIRCLE: Shape = Shape::Circle { radius: 2.0 };
const CIRCLE_COUNT: usize = 3;
static CIRCLES: [Collider; CIRCLE_COUNT] = [
    Collider {
        shape: CIRCLE,
        tf: Affine2 {
            translation: vec2(0.0, 1.5),
            matrix2: Mat2::IDENTITY,
        },
        group: Group::from_id(0),
    },
    Collider {
        shape: CIRCLE,
        tf: Affine2 {
            translation: vec2(1.0, -1.0),
            matrix2: Mat2::IDENTITY,
        },
        group: Group::from_id(1),
    },
    Collider {
        shape: CIRCLE,
        tf: Affine2 {
            translation: vec2(-1.0, -1.0),
            matrix2: Mat2::IDENTITY,
        },
        group: Group::from_id(2),
    },
];
