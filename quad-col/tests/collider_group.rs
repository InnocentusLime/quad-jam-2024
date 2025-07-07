use glam::{Affine2, Mat2, vec2};
use hashbrown::HashSet;
use quad_col::{Collider, Group, Shape};

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

fn get_collisions(query: Collider) -> impl Iterator<Item = usize> {
    (0..CIRCLE_COUNT).filter(move |idx| CIRCLE_MATRIX[*idx].collides(&query))
}

#[test]
fn test_group_intersections() {
    let expected = [
        HashSet::from_iter([1, 2]),
        HashSet::from_iter([0, 2]),
        HashSet::from_iter([0, 1]),
    ];
    assert_eq!(expected.len(), CIRCLE_COUNT);
    for (idx, query) in QUERY_MATRIX.into_iter().enumerate() {
        let expected = &expected[idx];
        let found = get_collisions(query).collect::<HashSet<_>>();
        assert_eq!(expected, &found);
    }
}
