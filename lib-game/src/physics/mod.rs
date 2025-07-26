use hecs::World;
use macroquad::prelude::*;
use quad_col::*;

mod components;
mod debug;

use crate::components::Transform;
pub use components::*;
pub use debug::*;

const CHAR_MOVEMENT_ITERS: usize = 10;
const CHAR_NORMAL_NUDGE: f32 = 0.001;
const CHAR_SKIN: f32 = 0.01;

pub struct PhysicsState {
    solver: CollisionSolver,
}

impl PhysicsState {
    pub fn new() -> Self {
        Self {
            solver: CollisionSolver::new(),
        }
    }

    pub fn import_positions_and_info(&mut self, world: &mut World) {
        self.solver.clear();
        let it = world.query_mut::<(&BodyTag, &Transform)>();
        let cold = it
            .into_iter()
            .map(|(ent, (info, tf))| (ent, get_entity_collider(tf, info)));
        self.solver.fill(cold);
    }

    pub fn apply_kinematic_moves(&mut self, world: &mut World) {
        for (_, (tf, info, kin)) in
            &mut world.query::<(&mut Transform, &BodyTag, &mut KinematicControl)>()
        {
            let mut character = get_entity_collider(tf, info);
            character.group = kin.collision;

            let dr = conv::topleft_corner_vector_to_crate(kin.dr);
            let new_tf = process_character_movement(&self.solver, dr, character);
            tf.pos = conv::crate_vector_to_topleft_corner(new_tf.translation);
        }
    }

    pub fn export_collision_queries<const ID: usize>(&mut self, world: &mut World) {
        for (_, (tf, query)) in &mut world.query::<(&Transform, &mut CollisionQuery<ID>)>() {
            let query_collider = get_query_collider(tf, query);
            query.collision_list.clear();
            query
                .collision_list
                .extend(self.solver.query_overlaps(query_collider).map(|(e, _)| *e));
        }
    }

    pub fn export_all_queries(&mut self, world: &mut World) {
        self.export_collision_queries::<0>(world);
        self.export_collision_queries::<1>(world);
        self.export_collision_queries::<2>(world);
        self.export_collision_queries::<3>(world);
        self.export_collision_queries::<4>(world);
        self.export_collision_queries::<5>(world);
        self.export_collision_queries::<6>(world);
        self.export_collision_queries::<7>(world);
    }
}

fn process_character_movement(
    solver: &CollisionSolver,
    mut dr: Vec2,
    mut character: Collider,
) -> Affine2 {
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
    }

    character.tf
}

fn get_query_collider<const ID: usize>(tf: &Transform, query: &CollisionQuery<ID>) -> Collider {
    let shape_pos = world_tf_to_phys(*tf) * world_tf_to_phys(query.extra_tf);
    Collider {
        tf: shape_pos,
        shape: query.collider,
        group: query.group,
    }
}

fn get_entity_collider(tf: &Transform, info: &BodyTag) -> Collider {
    let col_tf = conv::topleft_corner_tf_to_crate(tf.pos, tf.angle);
    Collider {
        shape: info.shape,
        group: info.groups,
        tf: col_tf,
    }
}

fn world_tf_to_phys(tf: Transform) -> Affine2 {
    conv::topleft_corner_tf_to_crate(tf.pos, tf.angle)
}
