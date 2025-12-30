use hecs::{Entity, World};
use macroquad::prelude::*;

mod components;
mod debug;

use crate::{components::Transform, dump};
pub use components::*;
pub use debug::*;

const CHAR_MOVEMENT_ITERS: usize = 10;
const CHAR_NORMAL_NUDGE: f32 = 0.001;
const CHAR_SKIN: f32 = 0.01;

pub struct CollisionSolver {
    solver: lib_col::CollisionSolver,
    collision_buffer: Vec<Entity>,
}

impl CollisionSolver {
    pub fn new() -> Self {
        Self {
            solver: lib_col::CollisionSolver::new(),
            collision_buffer: Vec::with_capacity(100),
        }
    }

    pub fn import_colliders(&mut self, world: &mut World) {
        self.solver.clear();
        let it = world.query_mut::<(&BodyTag, &Transform)>();
        let cold = it
            .into_iter()
            .map(|(ent, (info, tf))| (ent, get_entity_collider(tf, info)));
        self.solver.fill(cold);
    }

    pub fn export_kinematic_moves(&mut self, world: &mut World) {
        for (_, (tf, info, kin)) in
            &mut world.query::<(&mut Transform, &BodyTag, &mut KinematicControl)>()
        {
            let mut character = get_entity_collider(tf, info);
            character.group = kin.collision;

            let dr = lib_col::conv::topleft_corner_vector_to_crate(kin.dr);
            let (new_tf, collided) =
                process_character_movement(&mut self.solver, dr, character, kin.slide);
            tf.pos = lib_col::conv::crate_vector_to_topleft_corner(new_tf.translation);
            kin.collided = collided;
        }
    }

    pub fn collisions_for<const ID: usize>(&self, query: &CollisionQuery<ID>) -> &[Entity] {
        let off = query.collision_slice.off;
        let len = query.collision_slice.len;
        &self.collision_buffer[off..(off + len)]
    }

    pub fn compute_collisions(&mut self, world: &mut World) {
        self.collision_buffer.clear();
        self.compute_collisions_query::<0>(world);
        self.compute_collisions_query::<1>(world);
        self.compute_collisions_query::<2>(world);
        self.compute_collisions_query::<3>(world);
        self.compute_collisions_query::<4>(world);
        self.compute_collisions_query::<5>(world);
        self.compute_collisions_query::<6>(world);
        self.compute_collisions_query::<7>(world);
    }

    pub fn compute_collisions_query<const ID: usize>(&mut self, world: &mut World) {
        for (_, (tf, query)) in &mut world.query::<(&Transform, &mut CollisionQuery<ID>)>() {
            let query_collider = get_query_collider(tf, query);
            let start = self.collision_buffer.len();
            self.collision_buffer
                .extend(self.solver.query_overlaps(query_collider, query.filter));
            let end = self.collision_buffer.len();
            query.collision_slice = CollisionQuerySlice {
                off: start,
                len: end - start,
            };
        }
    }
}

impl Default for CollisionSolver {
    fn default() -> Self {
        CollisionSolver::new()
    }
}

fn process_character_movement(
    solver: &mut lib_col::CollisionSolver,
    mut dr: Vec2,
    mut character: lib_col::Collider,
    slide: bool,
) -> (Affine2, bool) {
    let mut collided = false;
    for _ in 0..CHAR_MOVEMENT_ITERS {
        let offlen = dr.length();
        let direction = dr.normalize_or_zero();
        let cast = solver.query_shape_cast(character, direction, offlen);
        let Some((_entity, toi, normal)) = cast else {
            character.tf.translation += dr;
            break;
        };

        character.tf.translation += (toi - CHAR_SKIN) * direction;

        dr -= dr.dot(normal) * normal;
        dr += normal * CHAR_NORMAL_NUDGE;
        collided = true;
        if !slide {
            break;
        }
    }

    (character.tf, collided)
}

fn get_query_collider<const ID: usize>(
    tf: &Transform,
    query: &CollisionQuery<ID>,
) -> lib_col::Collider {
    let shape_pos = world_tf_to_phys(*tf);
    lib_col::Collider {
        tf: shape_pos,
        shape: query.collider,
        group: query.group,
    }
}

fn get_entity_collider(tf: &Transform, info: &BodyTag) -> lib_col::Collider {
    let col_tf = lib_col::conv::topleft_corner_tf_to_crate(tf.pos, tf.angle);
    lib_col::Collider {
        shape: info.shape,
        group: info.groups,
        tf: col_tf,
    }
}

fn world_tf_to_phys(tf: Transform) -> Affine2 {
    lib_col::conv::topleft_corner_tf_to_crate(tf.pos, tf.angle)
}

#[cfg(test)]
mod tests {
    use hecs::World;
    use lib_col::{Group, Shape};

    use crate::{BodyTag, CollisionQuery, CollisionSolver, Transform};

    // Tests proper buffer filling for collisions.
    // We do not care about the setup complexity.
    // All possible query configurations are tested in lib_col.
    #[test]
    fn test_buffer_offsets() {
        let mut world = World::new();
        let mut solver = CollisionSolver::new();
        let shape = Shape::Rect {
            width: 8.0,
            height: 8.0,
        };

        let col1 = world.spawn((
            Transform::from_xy(0.0, 0.0),
            BodyTag {
                shape,
                groups: Group::from_id(0),
            },
        ));
        let col2 = world.spawn((
            Transform::from_xy(0.0, 0.0),
            BodyTag {
                shape,
                groups: Group::from_id(1),
            },
        ));
        let q_1 = world.spawn((
            Transform::from_xy(0.0, 0.0),
            CollisionQuery::<0>::new(shape, Group::from_id(0), Group::from_id(0)),
        ));
        let q_2 = world.spawn((
            Transform::from_xy(0.0, 0.0),
            CollisionQuery::<0>::new(shape, Group::from_id(1), Group::from_id(1)),
        ));

        for _ in 0..3 {
            solver.import_colliders(&mut world);
            solver.compute_collisions(&mut world);
            assert_eq!(solver.collision_buffer.len(), 2);

            let q_1 = world.get::<&CollisionQuery<0>>(q_1).unwrap();
            assert_eq!(solver.collisions_for(&q_1), &[col1]);

            let q_2 = world.get::<&CollisionQuery<0>>(q_2).unwrap();
            assert_eq!(solver.collisions_for(&q_2), &[col2]);
        }
    }
}
