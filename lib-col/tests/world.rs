mod common;

use common::{TestCase, draw_shape, run_tests_no_fuzz};
use glam::{Affine2, Mat2, vec2};
use hashbrown::HashSet;
use hecs::{Entity, World};
use lib_col::{Collider, CollisionSolver, Group, Shape};

#[derive(Debug, Clone, Copy)]
struct WorldTest {
    name: &'static str,
    expected: &'static [usize],
    circle_groups: [Group; CIRCLE_COUNT],
    query_group: Group,
    query_filter: Group,
}

impl WorldTest {
    fn get_query(&self) -> (Collider, Group) {
        let col = Collider {
            shape: Shape::Circle { radius: 10.0 },
            group: self.query_group,
            tf: Affine2::from_translation(vec2(0.0, 0.0)),
        };

        (col, self.query_filter)
    }

    fn spawn_entities(&self, world: &mut World) -> Vec<Entity> {
        CIRCLES
            .into_iter()
            .zip(self.circle_groups)
            .map(|(collider, group)| Collider { group, ..collider })
            .map(ColliderComponent)
            .map(|x| world.spawn((x,)))
            .collect::<Vec<_>>()
    }

    fn fill_solver(&self, world: &mut World, solver: &mut CollisionSolver) {
        solver.fill(
            world
                .query_mut::<&ColliderComponent>()
                .into_iter()
                .map(|(entity, component)| (entity, component.0)),
        );
    }
}

impl TestCase for WorldTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn check(&self) -> bool {
        let mut world = World::new();
        let mut solver = CollisionSolver::new();
        let spawned = self.spawn_entities(&mut world);
        self.fill_solver(&mut world, &mut solver);

        let (col, filter) = self.get_query();
        let expected = self
            .expected
            .into_iter()
            .map(|idx| spawned[*idx])
            .collect::<HashSet<_>>();
        let actual = solver
            .query_overlaps(col, filter)
            .map(|(entitiy, _)| *entitiy)
            .collect::<HashSet<_>>();

        actual == expected
    }

    fn draw(&self, canvas: &mut image::RgbImage) {
        let colors = [
            image::Rgb([255, 0, 0]),
            image::Rgb([0, 255, 0]),
            image::Rgb([0, 0, 255]),
        ];
        for (circle, color) in CIRCLES.into_iter().zip(colors) {
            draw_shape(canvas, color, circle.shape, circle.tf);
        }

        let (query, _) = self.get_query();
        draw_shape(canvas, image::Rgb([255, 255, 255]), query.shape, query.tf);
    }
}

#[test]
fn test_world() {
    run_tests_no_fuzz(tests());
}

fn tests() -> impl IntoIterator<Item = WorldTest> {
    [
        WorldTest {
            name: "empty",
            expected: &[],
            circle_groups: [Group::from_id(0), Group::from_id(1), Group::from_id(2)],
            query_group: Group::empty(),
            query_filter: Group::empty(),
        },
        WorldTest {
            name: "group[1, 2]",
            expected: &[1, 2],
            circle_groups: [Group::from_id(0), Group::from_id(1), Group::from_id(2)],
            query_group: pair_group(1, 2),
            query_filter: Group::empty(),
        },
        WorldTest {
            name: "group[0, 2]",
            expected: &[0, 2],
            circle_groups: [Group::from_id(0), Group::from_id(1), Group::from_id(2)],
            query_group: pair_group(0, 2),
            query_filter: Group::empty(),
        },
        WorldTest {
            name: "group[0, 1]",
            expected: &[0, 1],
            circle_groups: [Group::from_id(0), Group::from_id(1), Group::from_id(2)],
            query_group: pair_group(0, 1),
            query_filter: Group::empty(),
        },
        WorldTest {
            name: "group[1, 2] & filter[1, 2]",
            expected: &[0],
            circle_groups: [pair_group(1, 2), pair_group(0, 2), pair_group(0, 1)],
            query_group: pair_group(1, 2),
            query_filter: pair_group(1, 2),
        },
        WorldTest {
            name: "group[0, 2] & filter[0, 2]",
            expected: &[1],
            circle_groups: [pair_group(1, 2), pair_group(0, 2), pair_group(0, 1)],
            query_group: pair_group(0, 2),
            query_filter: pair_group(0, 2),
        },
        WorldTest {
            name: "group[0, 1] & filter[0, 1]",
            expected: &[2],
            circle_groups: [pair_group(1, 2), pair_group(0, 2), pair_group(0, 1)],
            query_group: pair_group(0, 1),
            query_filter: pair_group(0, 1),
        },
    ]
}

fn pair_group(x: u32, y: u32) -> Group {
    Group::from_id(x).union(Group::from_id(y))
}

#[repr(transparent)]
struct ColliderComponent(Collider);

const CIRCLE: Shape = Shape::Circle { radius: 2.0 };
const CIRCLE_COUNT: usize = 3;
static CIRCLES: [Collider; CIRCLE_COUNT] = [
    Collider {
        shape: CIRCLE,
        tf: Affine2 {
            translation: vec2(0.0, 1.5),
            matrix2: Mat2::IDENTITY,
        },
        group: Group::empty(),
    },
    Collider {
        shape: CIRCLE,
        tf: Affine2 {
            translation: vec2(1.0, -1.0),
            matrix2: Mat2::IDENTITY,
        },
        group: Group::empty(),
    },
    Collider {
        shape: CIRCLE,
        tf: Affine2 {
            translation: vec2(-1.0, -1.0),
            matrix2: Mat2::IDENTITY,
        },
        group: Group::empty(),
    },
];
