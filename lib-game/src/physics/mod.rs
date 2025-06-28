use std::collections::HashMap;

use macroquad::prelude::*;
use nalgebra::Translation2;
use rapier2d::{
    na::{Isometry2, UnitComplex, Vector2},
    parry::{
        query::ShapeCastOptions,
        shape::{Ball, Cuboid},
    },
    prelude::*,
};
use shipyard::{EntityId, Get, IntoIter, View, ViewMut, World};

mod components;
mod debug;

use crate::components::Transform;
pub use components::*;
pub use debug::*;

pub use rapier2d::prelude::InteractionGroups;

pub const PIXEL_PER_METER: f32 = 32.0;
pub const MOVE_KINEMATIC_MAX_ITERS: i32 = 20;
pub const KINEMATIC_SKIN: f32 = 0.001;
pub const PUSH_SKIN: f32 = KINEMATIC_SKIN + 0.05;
pub const KINEMATIC_NORMAL_NUDGE: f32 = 1.0e-4;
pub const LENGTH_EPSILON: f32 = 1.0e-5;

pub struct PhysicsState {
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    pipeline: PhysicsPipeline,
    query_pipeline: QueryPipeline,
    integration_parameters: IntegrationParameters,
    gravity: Vector<Real>,
    hooks: Box<dyn PhysicsHooks + Send + Sync>,
    mapping: HashMap<EntityId, RigidBodyHandle>,
    mapping_inv: HashMap<RigidBodyHandle, EntityId>,
}

impl PhysicsState {
    pub fn new() -> Self {
        info!(
            "lib-game physics backend: rapier version {}",
            rapier2d::VERSION
        );

        Self {
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            pipeline: PhysicsPipeline::new(),
            query_pipeline: QueryPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            gravity: Vector::zeros(), //Vector::y() * -9.81,
            hooks: Box::new(()),
            mapping: HashMap::new(),
            mapping_inv: HashMap::new(),
        }
    }

    fn spawn_body(
        &mut self,
        is_enabled: bool,
        trans: &Transform,
        entity: EntityId,
        collision: ColliderTy,
        kind: BodyKind,
        groups: InteractionGroups,
        mass: f32,
    ) {
        let rap_ty = match kind {
            BodyKind::Static => RigidBodyType::Fixed,
            BodyKind::Kinematic => RigidBodyType::KinematicPositionBased,
        };

        let start_pos = Self::world_to_phys(trans.pos);
        let iso = Isometry2 {
            translation: Translation2::new(start_pos.x, start_pos.y),
            rotation: rapier2d::na::Unit::from_angle(Self::world_ang_to_phys(trans.angle)),
        };
        let body = self.bodies.insert(
            RigidBodyBuilder::new(rap_ty)
                .position(iso)
                .soft_ccd_prediction(2.0)
                .linear_damping(0.4)
                .angular_damping(1.0),
        );
        let collider_shape = match collision {
            ColliderTy::Box { width, height } => SharedShape::cuboid(
                width / 2.0 / PIXEL_PER_METER,
                height / 2.0 / PIXEL_PER_METER,
            ),
            ColliderTy::Circle { radius } => SharedShape::ball(radius / PIXEL_PER_METER),
        };

        self.colliders.insert_with_parent(
            ColliderBuilder::new(collider_shape)
                .collision_groups(groups)
                .mass(mass)
                .enabled(is_enabled)
                .friction(0.001),
            body.clone(),
            &mut self.bodies,
        );
        self.mapping.insert(entity, body);
        self.mapping_inv.insert(body, entity);
    }

    fn world_ang_to_phys(ang: f32) -> f32 {
        std::f32::consts::PI - ang
    }

    fn phys_ang_to_world(ang: f32) -> f32 {
        std::f32::consts::PI - ang
    }

    fn phys_to_world(p: Vec2) -> Vec2 {
        let mut out = p;

        out *= PIXEL_PER_METER;
        out.y *= -1.0;

        out
    }

    fn world_to_phys(p: Vec2) -> Vec2 {
        let mut out = p;

        out.y *= -1.0;
        out /= PIXEL_PER_METER;

        out
    }

    fn world_tf_to_phys(tf: Transform) -> rapier2d::na::Isometry2<f32> {
        let ang = Self::world_ang_to_phys(tf.angle);
        let pos = Self::world_to_phys(tf.pos);

        rapier2d::na::Isometry2 {
            rotation: UnitComplex::from_angle(ang),
            translation: Translation2::new(pos.x, pos.y),
        }
    }

    #[allow(dead_code)]
    fn cast_shape(
        &mut self,
        tf: Transform,
        groups: InteractionGroups,
        dir: Vec2,
        shape: ColliderTy,
    ) -> Option<f32> {
        let predicate = Some(&|_, col: &Collider| -> bool { col.is_enabled() }
            as &dyn Fn(ColliderHandle, &Collider) -> bool);
        let shape = match shape {
            ColliderTy::Box { width, height } => &Cuboid::new(rapier2d::na::Vector2::new(
                width / 2.0 / PIXEL_PER_METER,
                height / 2.0 / PIXEL_PER_METER,
            )) as &dyn Shape,
            ColliderTy::Circle { radius } => &Ball::new(radius / PIXEL_PER_METER) as &dyn Shape,
        };
        let dir = Self::world_to_phys(dir.normalize_or_zero());
        let shape_pos = Self::world_tf_to_phys(tf);
        let Some((_, hit)) = self.query_pipeline.cast_shape(
            &self.bodies,
            &self.colliders,
            &shape_pos,
            &vector![dir.x, dir.y],
            shape,
            ShapeCastOptions {
                max_time_of_impact: Real::MAX,
                target_distance: 0.0,
                stop_at_penetration: true,
                compute_impact_geometry_on_penetration: true,
            },
            QueryFilter {
                groups: Some(groups),
                predicate,
                ..QueryFilter::default()
            },
        ) else {
            return None;
        };

        Some(hit.time_of_impact)
    }

    fn all_collisions(
        &mut self,
        shape_pos: Isometry2<f32>,
        groups: InteractionGroups,
        shape: ColliderTy,
        writeback: &mut Vec<EntityId>,
    ) {
        let predicate = Some(
            &|_, col: &Collider| -> bool { col.is_enabled() && !col.is_sensor() }
                as &dyn Fn(ColliderHandle, &Collider) -> bool,
        );
        let shape = match shape {
            ColliderTy::Box { width, height } => &Cuboid::new(rapier2d::na::Vector2::new(
                width / 2.0 / PIXEL_PER_METER,
                height / 2.0 / PIXEL_PER_METER,
            )) as &dyn Shape,
            ColliderTy::Circle { radius } => &Ball::new(radius / PIXEL_PER_METER) as &dyn Shape,
        };
        self.query_pipeline.intersections_with_shape(
            &self.bodies,
            &self.colliders,
            &shape_pos,
            shape,
            QueryFilter {
                groups: Some(groups),
                predicate,
                ..QueryFilter::default()
            },
            |handle| {
                let col = self.colliders.get(handle).unwrap();

                writeback.push(self.mapping_inv[&col.parent().unwrap()]);

                true
            },
        );
    }

    fn any_collisions(
        &mut self,
        shape_pos: Isometry2<f32>,
        groups: InteractionGroups,
        shape: ColliderTy,
    ) -> Option<EntityId> {
        let predicate = Some(&|_, col: &Collider| -> bool { col.is_enabled() }
            as &dyn Fn(ColliderHandle, &Collider) -> bool);
        let shape = match shape {
            ColliderTy::Box { width, height } => &Cuboid::new(rapier2d::na::Vector2::new(
                width / 2.0 / PIXEL_PER_METER,
                height / 2.0 / PIXEL_PER_METER,
            )) as &dyn Shape,
            ColliderTy::Circle { radius } => &Ball::new(radius / PIXEL_PER_METER) as &dyn Shape,
        };
        let Some(handle) = self.query_pipeline.intersection_with_shape(
            &self.bodies,
            &self.colliders,
            &shape_pos,
            shape,
            QueryFilter {
                groups: Some(groups),
                predicate,
                ..QueryFilter::default()
            },
        ) else {
            return None;
        };

        let col = self.colliders.get(handle).unwrap();

        Some(self.mapping_inv[&col.parent().unwrap()])
    }

    fn move_kinematic(&mut self, rbh: RigidBodyHandle, dr: Vec2, slide: bool) -> bool {
        let mut translation_remaining = Self::world_to_phys(dr);
        let mut translation_current = Vec2::ZERO;
        let mut has_collided = false;
        for _ in 0..MOVE_KINEMATIC_MAX_ITERS {
            let off_len = translation_remaining.length();
            let Some(off_dir) = translation_remaining.try_normalize() else {
                break;
            };
            let Some(hit) = self.move_kinematic_cast(rbh, translation_current, off_dir, off_len)
            else {
                translation_current += translation_remaining;
                break;
            };

            has_collided = true;
            let translation_allowed = off_dir * hit.time_of_impact;
            translation_current += translation_allowed;
            translation_remaining -= translation_allowed;

            // If sliding is enabled, realign the remaining translation, with the surface.
            if slide {
                translation_remaining = Self::get_slide_part(&hit, translation_remaining);
            }
        }

        self.apply_kinematic_offset(rbh, translation_current);
        has_collided
    }

    fn get_slide_part(hit: &ShapeCastHit, translation: Vec2) -> Vec2 {
        let hit_normal = vec2(hit.normal1.x, hit.normal1.y);
        let dist_to_surface = translation.dot(hit_normal);
        let (normal_part, penetration_part) = if dist_to_surface < 0.0 {
            (Vec2::ZERO, dist_to_surface * hit_normal)
        } else {
            (dist_to_surface * hit_normal, Vec2::ZERO)
        };

        // Add the normal to gently push the object out of collision
        translation - normal_part - penetration_part + hit_normal * KINEMATIC_NORMAL_NUDGE
    }

    fn apply_kinematic_offset(&mut self, rbh: RigidBodyHandle, offset: Vec2) {
        let offset = Vector::new(offset.x, offset.y);
        let old_trans = self.bodies.get(rbh).unwrap().position().translation.vector;
        self.bodies
            .get_mut(rbh)
            .unwrap()
            .set_next_kinematic_translation((old_trans + offset).into());
    }

    fn move_kinematic_cast(
        &mut self,
        rbh: RigidBodyHandle,
        translation_current: Vec2,
        off_dir: Vec2,
        off_len: f32,
    ) -> Option<ShapeCastHit> {
        let translation_current = Vector2::new(translation_current.x, translation_current.y);
        let off_dir = Vector2::new(off_dir.x, off_dir.y);
        let predicate = |_, col: &Collider| -> bool { col.is_enabled() && !col.is_sensor() };
        let rb = self.bodies.get(rbh).unwrap();
        let kin_pos = rb.position();
        let kin_shape = self
            .colliders
            .get(rb.colliders()[0])
            .unwrap()
            .shared_shape()
            .clone();
        let groups = self
            .colliders
            .get(rb.colliders()[0])
            .unwrap()
            .collision_groups();
        let shape_pos = Translation::from(translation_current) * kin_pos;
        let hit = self.query_pipeline.cast_shape(
            &self.bodies,
            &self.colliders,
            &shape_pos,
            &off_dir,
            &*kin_shape.0,
            ShapeCastOptions {
                target_distance: KINEMATIC_SKIN,
                max_time_of_impact: off_len,
                stop_at_penetration: false,
                compute_impact_geometry_on_penetration: true,
            },
            QueryFilter {
                exclude_rigid_body: Some(rbh),
                groups: Some(groups),
                predicate: Some(&predicate),
                ..QueryFilter::default()
            },
        );

        hit.map(|(_, hit)| hit)
    }

    pub fn allocate_bodies(&mut self, info: ViewMut<BodyTag>, tf: View<Transform>) {
        for (entity, info) in info.inserted().iter().with_id() {
            let trans = tf.get(entity).unwrap();

            info!("Allocate physics body for {entity:?}");

            self.spawn_body(
                info.enabled,
                trans,
                entity,
                info.shape,
                info.kind,
                info.groups.into_interaction_groups(),
                info.mass,
            );
        }

        info.clear_all_inserted()
    }

    pub fn remove_dead_handles(&mut self, rbs: View<BodyTag>) {
        // GC the dead handles
        for remd in rbs.removed_or_deleted() {
            let Some(rb) = self.mapping.remove(&remd) else {
                continue;
            };
            self.mapping_inv.remove(&rb);

            info!("ent:{remd:?} body:{rb:?} deletted");

            self.bodies.remove(
                rb,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            );
        }
    }

    pub fn import_positions_and_info(&mut self, rbs: View<BodyTag>, pos: ViewMut<Transform>) {
        // Enable-disable
        for (ent, info) in rbs.iter().with_id() {
            let body = &mut self.bodies[self.mapping[&ent]];

            body.set_enabled(info.enabled);

            for col in body.colliders() {
                let col = self.colliders.get_mut(*col).unwrap();
                col.set_collision_groups(info.groups.into_interaction_groups());
            }
        }

        // Import the new positions to world
        for (ent, (_, pos)) in (&rbs, &pos).iter().with_id() {
            let body = &mut self.bodies[self.mapping[&ent]];
            let new_pos = Self::world_tf_to_phys(*pos);

            // NOTE: perhaps we should do an epsilon compare here?
            if new_pos != *body.position() {
                body.set_position(Self::world_tf_to_phys(*pos), true);
            }
        }
    }

    pub fn step(&mut self) {
        self.query_pipeline.update(&self.colliders);
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &*self.hooks,
            &(),
        );
    }

    pub fn apply_kinematic_moves(&mut self, mut kin: ViewMut<KinematicControl>) {
        for (ent, kin) in (&mut kin).iter().with_id() {
            let rbh = self.mapping[&ent];
            self.move_kinematic(rbh, kin.dr, kin.slide);

            kin.dr = Vec2::ZERO;
        }
    }

    pub fn export_body_poses(&mut self, body_tag: View<BodyTag>, mut pos: ViewMut<Transform>) {
        for (ent, (_, pos)) in (&body_tag, &mut pos).iter().with_id() {
            let rb = &self.bodies[self.mapping[&ent]];
            let new_pos = rb.translation();
            let new_pos = vec2(new_pos.x, new_pos.y);
            let new_pos = Self::phys_to_world(new_pos);
            let new_angle = Self::phys_ang_to_world(rb.rotation().angle());

            pos.pos = new_pos;
            pos.angle = new_angle;
        }
    }

    pub fn export_collision_queries<const ID: usize>(
        &mut self,
        tf: View<Transform>,
        mut query: ViewMut<CollisionQuery<ID>>,
    ) {
        for (tf, query) in (&tf, &mut query).iter() {
            let shape_pos = Self::world_tf_to_phys(*tf) * Self::world_tf_to_phys(query.extra_tf);
            match &mut query.collision_list {
                CollisionList::One(one) => {
                    let res = self.any_collisions(
                        shape_pos,
                        query.group.into_interaction_groups(),
                        query.collider,
                    );
                    *one = res;
                }
                CollisionList::Many(list) => {
                    self.all_collisions(
                        shape_pos,
                        query.group.into_interaction_groups(),
                        query.collider,
                        list,
                    );
                }
            }
        }
    }

    pub fn export_all_queries(&mut self, world: &mut World) {
        world.run_with_data(Self::export_collision_queries::<0>, self);
        world.run_with_data(Self::export_collision_queries::<1>, self);
        world.run_with_data(Self::export_collision_queries::<2>, self);
        world.run_with_data(Self::export_collision_queries::<3>, self);
        world.run_with_data(Self::export_collision_queries::<4>, self);
        world.run_with_data(Self::export_collision_queries::<5>, self);
        world.run_with_data(Self::export_collision_queries::<6>, self);
        world.run_with_data(Self::export_collision_queries::<7>, self);
    }
}
