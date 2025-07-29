use glam::{Affine2, Mat2, vec2};
use hashbrown::HashSet;
use hecs::World;
use quad_col::{Collider, CollisionSolver, Group, Shape};

const CIRCLE: Shape = Shape::Circle { radius: 2.0 };
const CIRCLE_COUNT: usize = 3;
static CIRCLE_MATRIX: [Collider; CIRCLE_COUNT] = [
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
static QUERY_MATRIX: [Collider; CIRCLE_COUNT] = [
    Collider {
        group: Group::from_id(1).union(Group::from_id(2)),
        ..CIRCLE_MATRIX[0]
    },
    Collider {
        group: Group::from_id(0).union(Group::from_id(2)),
        ..CIRCLE_MATRIX[1]
    },
    Collider {
        group: Group::from_id(0).union(Group::from_id(1)),
        ..CIRCLE_MATRIX[2]
    },
];

#[repr(transparent)]
struct ColliderComponent(Collider);

#[test]
fn test_world_empty() {
    let mut world = World::new();
    for circle in CIRCLE_MATRIX {
        world.spawn((ColliderComponent(circle),));
    }
    let mut solver = CollisionSolver::new();
    let it = world.query_mut::<&ColliderComponent>();
    solver.fill(
        it.into_iter()
            .map(|(entity, component)| (entity, component.0)),
    );
    let overlaps = solver
        .query_overlaps(
            Collider {
                tf: Affine2::IDENTITY,
                shape: Shape::Circle { radius: 64.0 },
                group: Group::empty(),
            },
            Group::empty(),
        )
        .collect::<Vec<_>>();
    assert!(overlaps.is_empty())
}

#[test]
fn test_world_simple() {
    let mut world = World::new();
    let spawned = CIRCLE_MATRIX
        .into_iter()
        .map(ColliderComponent)
        .map(|c| world.spawn((c,)))
        .collect::<Vec<_>>();
    let expected = [
        HashSet::from_iter([spawned[1], spawned[2]]),
        HashSet::from_iter([spawned[0], spawned[2]]),
        HashSet::from_iter([spawned[0], spawned[1]]),
    ];
    assert_eq!(expected.len(), CIRCLE_COUNT);
    let mut solver = CollisionSolver::new();
    let it = world.query_mut::<&ColliderComponent>();
    solver.fill(
        it.into_iter()
            .map(|(entity, component)| (entity, component.0)),
    );

    for (idx, query) in QUERY_MATRIX.into_iter().enumerate() {
        let expected = &expected[idx];
        let (got_ent, _) = solver
            .query_overlaps(query, Group::empty())
            .map(|(e, c)| (*e, *c))
            .collect::<(HashSet<_>, Vec<_>)>();
        assert_eq!(&got_ent, expected);
    }
}
