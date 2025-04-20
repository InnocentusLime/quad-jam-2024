use std::collections::HashMap;

use jam_macro::method_system;
use macroquad::prelude::*;
use nalgebra::Translation2;
use rapier2d::{na::{Isometry, Isometry2, UnitComplex, Vector2}, parry::{query::{DefaultQueryDispatcher, PersistentQueryDispatcher, ShapeCastOptions}, shape::{Ball, Cuboid}}, prelude::*};
use shipyard::{Component, EntitiesView, EntityId, Get, IntoIter, IntoWithId, Unique, UniqueView, View, ViewMut};

use crate::{wrap_method, DeltaTime, Transform};

pub const PIXEL_PER_METER : f32 = 32.0;
pub const MAX_KINEMATICS_ITERS: i32 = 20;
pub const KINEMATIC_SKIN: f32 = 0.001;
pub const PUSH_SKIN: f32 = KINEMATIC_SKIN + 0.05;
pub const KINEMATIC_NORMAL_NUDGE: f32 = 1.0e-4;
pub const LENGTH_EPSILON: f32 = 1.0e-5;

pub mod groups {
    use rapier2d::prelude::*;

    pub const LEVEL: Group = Group::GROUP_1;
    pub const NPCS: Group = Group::GROUP_2;
    pub const PLAYER: Group = Group::GROUP_3;
    pub const PROJECTILES: Group = Group::GROUP_4;

    pub const LEVEL_INTERACT: Group =
        LEVEL
            .union(NPCS)
            .union(PLAYER)
            .union(PROJECTILES);
    pub const PLAYER_INTERACT: Group =
        LEVEL;
    pub const NPCS_INTERACT: Group =
        LEVEL
            .union(PROJECTILES)
            .union(NPCS);
    pub const PROJECTILES_INTERACT: Group =
        LEVEL;
}

#[derive(Clone, Copy, Debug)]
pub enum ColliderTy {
    Box {
        width: f32,
        height: f32,
    },
    Circle {
        radius: f32,
    }
}

#[derive(Clone, Debug, Component)]
pub struct BeamTag {
    pub overlap_filter: InteractionGroups,
    pub overlaps: Vec<EntityId>,
}

impl BeamTag {
    pub fn new(overlap_filter: InteractionGroups) -> Self {
        Self {
            overlap_filter,
            overlaps: Vec::with_capacity(32),
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct OneSensorTag {
    pub col: Option<EntityId>,
}

impl OneSensorTag {
    pub fn new() -> Self {
        Self {
            col: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Component)]
#[derive(PartialEq, Eq, Hash)]
pub enum BodyTag {
    Static,
    Dynamic,
    Kinematic,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct KinematicControl {
    pub dr: Vec2,
    pub slide: bool,
}

impl KinematicControl {
    pub fn new() -> Self {
        Self {
            dr: Vec2::ZERO,
            slide: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct ForceApplier {
    pub force: Vec2,
}

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal, Insertion)]
pub struct PhysicsInfo {
    pub enabled: bool,
    pub groups: InteractionGroups,
    shape: ColliderTy,
    mass: f32,
}

impl PhysicsInfo {
    pub fn new(groups: InteractionGroups, shape: ColliderTy, mass: f32, enabled: bool) -> Self {
        Self {
            enabled,
            groups,
            mass,
            shape,
        }
    }

    pub fn shape(&self) -> &ColliderTy { &self.shape }
}

enum CompleteBodyKind {
    Interractable(BodyTag),
    Sensor,
}

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
    kinematic_cols: Vec<(Vector2<f32>, Isometry2<f32>, ShapeCastHit)>,
    manifolds: Vec<ContactManifold>,
}

impl PhysicsState {
    pub fn new() -> Self {
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
            kinematic_cols: Vec::new(),
            manifolds: Vec::new(),
        }
    }

    fn spawn_body(
        &mut self,
        is_enabled: bool,
        trans: &Transform,
        entity: EntityId,
        collision: ColliderTy,
        kind: CompleteBodyKind,
        groups: InteractionGroups,
        mass: f32,
    ) {
        let rap_ty = match kind {
            CompleteBodyKind::Interractable(BodyTag::Dynamic) => RigidBodyType::Dynamic,
            CompleteBodyKind::Interractable(BodyTag::Static) => RigidBodyType::Fixed,
            CompleteBodyKind::Interractable(BodyTag::Kinematic) => RigidBodyType::KinematicPositionBased,
            CompleteBodyKind::Sensor => RigidBodyType::Fixed,
        };

        let start_pos = Self::world_to_phys(trans.pos);
        let iso = Isometry2 {
            translation: Translation2::new(start_pos.x, start_pos.y),
            rotation: rapier2d::na::Unit::from_angle(
                Self::world_ang_to_phys(trans.angle)
            ),
        };
        let body = self.bodies.insert(
            RigidBodyBuilder::new(rap_ty)
                .position(iso)
                .soft_ccd_prediction(2.0)
                .linear_damping(1.0)
                .angular_damping(1.0)
        );
        let collider_shape = match collision {
            ColliderTy::Box { width, height } => SharedShape::cuboid(
                width / 2.0 / PIXEL_PER_METER,
                height / 2.0 / PIXEL_PER_METER,
            ),
            ColliderTy::Circle { radius } => SharedShape::ball(
                radius / PIXEL_PER_METER,
            ),
        };

        self.colliders.insert_with_parent(
            ColliderBuilder::new(collider_shape)
                .collision_groups(groups)
                .mass(mass)
                .sensor(matches!(kind, CompleteBodyKind::Sensor))
                .enabled(is_enabled)
            ,
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

    fn get_slide_part(hit: &ShapeCastHit, trans: Vector2<f32>) -> Vector2<f32> {
        let dist_to_surface = trans.dot(&hit.normal1);
        let (normal_part, penetration_part) = if dist_to_surface < 0.0 {
            (Vector2::zeros(), dist_to_surface * *hit.normal1)
        } else {
            (dist_to_surface * *hit.normal1, Vector2::zeros())
        };

        trans - normal_part - penetration_part +
            *hit.normal1 * KINEMATIC_NORMAL_NUDGE
    }

    fn move_kinematic_pushes(
        &mut self,
        kin_shape: &dyn Shape,
        kin_groups: InteractionGroups,
    ) {
        let dispatcher = DefaultQueryDispatcher;

        for (rem, pos, hit) in &self.kinematic_cols {
            let push = *hit.normal1 * rem.dot(
                &hit.normal1
            );
            let char_box = kin_shape.compute_aabb(pos)
                .loosened(PUSH_SKIN);

            self.manifolds.clear();
            self.query_pipeline.colliders_with_aabb_intersecting_aabb(
                &char_box,
                |handle| {
                    let Some(col) = self.colliders.get(*handle)
                        else { return true; };
                    let Some(bodh) = col.parent()
                        else { return true; };
                    let Some(bod) = self.bodies.get(bodh)
                        else { return true; };
                    if !bod.is_dynamic() { return true; }
                    if !col.collision_groups().test(kin_groups) {
                        return true;
                    }

                    self.manifolds.clear();
                    let pos12 = pos.inv_mul(col.position());
                    let _ = dispatcher.contact_manifolds(
                        &pos12,
                        &*kin_shape,
                        col.shape(),
                        PUSH_SKIN,
                        &mut self.manifolds,
                        &mut None,
                    );

                    for m in &mut self.manifolds {
                        m.data.rigid_body2 = Some(bodh);
                        m.data.normal = pos * m.local_n1;
                    }

                    true
                });
            let velocity_to_transfer = push * self.integration_parameters.dt.recip();
            for manifold in &self.manifolds {
                let body_handle = manifold.data.rigid_body2.unwrap();
                let body = &mut self.bodies[body_handle];
                // info!("CONT: {}", manifold.points.len());

                for pt in &manifold.points {
                    if pt.dist > PUSH_SKIN {
                        continue;
                    }

                    let body_mass = body.mass();
                    let contact_point = body.position() * pt.local_p2;
                    let delta_vel_per_contact = (velocity_to_transfer
                        - body.velocity_at_point(&contact_point))
                    .dot(&manifold.data.normal);
                    let char_mass = 1.0;
                    let mass_ratio = body_mass * char_mass / (body_mass + char_mass);

                    // info!("{:?}",
                    //     manifold.data.normal * delta_vel_per_contact.max(0.0) * mass_ratio,
                    // );

                    body.apply_impulse_at_point(
                        manifold.data.normal * delta_vel_per_contact.max(0.0) * mass_ratio,
                        contact_point,
                        true,
                    );
                }
            }
        }
    }

    fn cast_shape(
        &mut self,
        tf: Transform,
        groups: InteractionGroups,
        dir: Vec2,
        shape: ColliderTy,
    ) -> Option<f32> {
        let predicate = Some(
            &|_, col: &Collider| -> bool {
                col.is_enabled()
            } as &dyn Fn(ColliderHandle, &Collider) -> bool
        );
        let shape = match shape {
            ColliderTy::Box { width, height } => {
                &Cuboid::new(rapier2d::na::Vector2::new(
                    width / 2.0 / PIXEL_PER_METER,
                    height / 2.0 / PIXEL_PER_METER,
                )) as &dyn Shape
            },
            ColliderTy::Circle { radius } => {
                &Ball::new(
                    radius / PIXEL_PER_METER,
                ) as &dyn Shape
            },
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
        ) else { return None; };

        Some(hit.time_of_impact)
    }

    fn all_collisions(
        &mut self,
        tf: Transform,
        groups: InteractionGroups,
        shape: ColliderTy,
        writeback: &mut Vec<EntityId>
    ) {
        let predicate = Some(
            &|_, col: &Collider| -> bool {
                col.is_enabled() && !col.is_sensor()
            } as &dyn Fn(ColliderHandle, &Collider) -> bool
        );
        let shape = match shape {
            ColliderTy::Box { width, height } => {
                &Cuboid::new(rapier2d::na::Vector2::new(
                    width / 2.0 / PIXEL_PER_METER,
                    height / 2.0 / PIXEL_PER_METER,
                )) as &dyn Shape
            },
            ColliderTy::Circle { radius } => {
                &Ball::new(
                    radius / PIXEL_PER_METER,
                ) as &dyn Shape
            },
        };
        let shape_pos = Self::world_tf_to_phys(tf);
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
            }
        );
    }

    fn any_collisions(
        &mut self,
        tf: Transform,
        groups: InteractionGroups,
        shape: ColliderTy,
    ) -> Option<EntityId> {
        let predicate = Some(
            &|_, col: &Collider| -> bool {
                col.is_enabled()
            } as &dyn Fn(ColliderHandle, &Collider) -> bool
        );
        let shape = match shape {
            ColliderTy::Box { width, height } => {
                &Cuboid::new(rapier2d::na::Vector2::new(
                    width / 2.0 / PIXEL_PER_METER,
                    height / 2.0 / PIXEL_PER_METER,
                )) as &dyn Shape
            },
            ColliderTy::Circle { radius } => {
                &Ball::new(
                    radius / PIXEL_PER_METER,
                ) as &dyn Shape
            },
        };
        let shape_pos = Self::world_tf_to_phys(tf);
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
        ) else { return None; };

        let col = self.colliders.get(handle).unwrap();

        Some(self.mapping_inv[&col.parent().unwrap()])
    }

    fn move_kinematic(
        &mut self,
        rbh: RigidBodyHandle,
        dr: Vec2,
        slide: bool,
    ) -> bool {
        let predicate = Some(
            &|_, col: &Collider| -> bool {
                col.is_enabled() && !col.is_sensor()
            } as &dyn Fn(ColliderHandle, &Collider) -> bool
        );
        self.kinematic_cols.clear();

        let dr = Self::world_to_phys(dr);
        let rb = self.bodies.get(rbh).unwrap();
        let (kin_pos, kin_shape) = (
            rb.position(),
            self.colliders.get(rb.colliders()[0])
                .unwrap()
                .shared_shape()
                .clone()
        );
        let groups = self.colliders.get(rb.colliders()[0]).unwrap().collision_groups();

        let mut final_trans = rapier2d::na::Vector2::zeros();
        let mut trans_rem = rapier2d::na::Vector2::new(
            dr.x,
            dr.y,
        );

        let mut max_iters = MAX_KINEMATICS_ITERS;
        while let Some((off_dir, off_len)) = UnitVector::try_new_and_get(
            trans_rem,
            LENGTH_EPSILON,
        ) {
            if max_iters <= 0 { break; }
            max_iters -= 1;

            let shape_pos = Translation::from(final_trans) * kin_pos;
            let Some((_handle, hit)) = self.query_pipeline.cast_shape(
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
                    predicate,
                    ..QueryFilter::default()
                },
            )
            else {
                final_trans += trans_rem;
                trans_rem.fill(0.0);
                break;
            };

            let allowed_dist = hit.time_of_impact;
            let allowed_trans = *off_dir * allowed_dist;

            final_trans += allowed_trans;
            trans_rem -= allowed_trans;

            self.kinematic_cols.push((
                trans_rem,
                shape_pos,
                hit,
            ));

            if slide {
                trans_rem = Self::get_slide_part(&hit, trans_rem);
            }
        }

        let has_collided = !self.kinematic_cols.is_empty();
        let old_trans = kin_pos.translation.vector;
        self.move_kinematic_pushes(&*kin_shape, groups);

        self.bodies.get_mut(rbh).unwrap().set_next_kinematic_translation(
            (old_trans + final_trans).into()
        );

        has_collided
    }

    pub fn allocate_bodies(
        &mut self,
        info: View<PhysicsInfo>,
        tf: View<Transform>,
        body_tag: View<BodyTag>,
    ) {
        for (entity, info) in info.inserted().iter().with_id() {
            let Ok(kind) = body_tag.get(entity)
                else { continue; };
            let trans = tf.get(entity).unwrap();

            info!("Allocate physics body for {entity:?}");

            self.spawn_body(
                info.enabled,
                trans,
                entity,
                info.shape,
                CompleteBodyKind::Interractable(*kind),
                info.groups,
                info.mass,
            );
        }
    }

    pub fn allocate_one_sensors(
        &mut self,
        info: View<PhysicsInfo>,
        tf: View<Transform>,
        sens_tag: View<OneSensorTag>,
    ) {
        for (entity, info) in info.inserted().iter().with_id() {
            let Ok(_) = sens_tag.get(entity)
                else { continue; };
            let trans = tf.get(entity).unwrap();

            info!("Allocate one sensor for {entity:?}");

            self.spawn_body(
                info.enabled,
                trans,
                entity,
                info.shape,
                CompleteBodyKind::Sensor,
                info.groups,
                info.mass,
            );
        }
    }

    pub fn remove_dead_handles(
        &mut self,
        rbs: View<PhysicsInfo>,
    ) {
        // GC the dead handles
        for remd in rbs.removed_or_deleted() {
            let Some(rb) = self.mapping.remove(&remd)
                else { continue; };
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
        };
    }

    pub fn reset_forces(
        &mut self,
        mut force: ViewMut<ForceApplier>,
    ) {
        for force in (&mut force).iter() {
            force.force = Vec2::ZERO;
        }
    }

    pub fn import_forces(
        &mut self,
        body_tag: View<BodyTag>,
        force: View<ForceApplier>,
    ) {
        for (ent, (body_tag, force)) in (&body_tag, &force).iter().with_id() {
            if *body_tag != BodyTag::Dynamic {
                warn!("Force applier attached to a non-dynamic body: {ent:?}");
                continue;
            }

            let force = Self::world_to_phys(force.force);
            let rbh = self.mapping[&ent];
            let body = self.bodies.get_mut(rbh).unwrap();
            body.add_force(nalgebra::vector![force.x, force.y], true);
        }
    }

    pub fn import_positions_and_info(
        &mut self,
        rbs: View<PhysicsInfo>,
        pos: ViewMut<Transform>,
    ) {
        // Enable-disable
        for (ent, info) in rbs.iter().with_id() {
            // TODO: this being None is okay for beams only. Probably
            // should note that somewhere
            // Alternatively, can tag beams differently
            let Some(&rbh) = self.mapping.get(&ent)
                else { continue; };
            let body = self.bodies.get_mut(rbh).unwrap();

            body.set_enabled(info.enabled);

            for col in body.colliders() {
                let col = self.colliders.get_mut(*col).unwrap();
                col.set_collision_groups(info.groups);
            }
        }

        // Import the new positions to world
        for (ent, (_, pos)) in (&rbs, &pos).iter().with_id() {
            // TODO: this being None is okay for beams only. Probably
            // should note that somewhere
            // Alternatively, can tag beams differently
            let Some(&rbh) = self.mapping.get(&ent)
                else { continue; };
            let body = self.bodies.get_mut(rbh).unwrap();
            let new_pos= Self::world_tf_to_phys(*pos);

            // NOTE: perhaps we should do an epsilon compare here?
            if new_pos != *body.position() {
                body.set_position(Self::world_tf_to_phys(*pos), true);
            }
        };
    }

    pub fn step(&mut self) {
        // Step simulation
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
            &()
        );

        // Reset forces
        for (_, body) in self.bodies.iter_mut() {
            body.reset_forces(false);
        }
    }

    pub fn apply_kinematic_moves(
        &mut self,
        mut kin: ViewMut<KinematicControl>,
    ) {
        for (ent, kin) in (&mut kin).iter().with_id() {
            let rbh = self.mapping[&ent];
            self.move_kinematic(
                rbh,
                kin.dr,
                kin.slide,
            );

            kin.dr = Vec2::ZERO;
        }
    }

    pub fn export_body_poses(
        &mut self,
        rbs: View<PhysicsInfo>,
        body_tag: View<BodyTag>,
        mut pos: ViewMut<Transform>,
    ) {
        for (ent, (_, _, pos)) in (&rbs, &body_tag, &mut pos).iter().with_id() {
            let rbh = self.mapping[&ent];
            let rb  = self.bodies.get(rbh).unwrap();
            let new_pos = rb.translation();
            let new_pos = vec2(new_pos.x, new_pos.y);
            let new_pos = Self::phys_to_world(new_pos);
            let new_angle = Self::phys_ang_to_world(rb.rotation().angle());

            pos.pos = new_pos;
            pos.angle = new_angle;
        }
    }

    pub fn export_sensor_queries(
        &mut self,
        tf: View<Transform>,
        info: View<PhysicsInfo>,
        mut sens: ViewMut<OneSensorTag>,
    ) {
        for (tf, info, sens) in (&tf, &info, &mut sens).iter() {
            let res = self.any_collisions(
                *tf,
                info.groups,
                info.shape,
            );

            sens.col = res;
        }
    }

    pub fn export_beam_queries(
        &mut self,
        tf: View<Transform>,
        mut info: ViewMut<PhysicsInfo>,
        mut beam: ViewMut<BeamTag>,
    ) {
        for (tf, info, beam) in (&tf, &mut info, &mut beam).iter() {
            let ColliderTy::Box { height, .. } = info.shape
                else { panic!("Beam support only box colliders"); };

            let dir = Vec2::from_angle(tf.angle);
            // FIXME: a dirty hack!
            let beamlen = self.cast_shape(
                *tf,
                info.groups,
                dir,
                ColliderTy::Box { width: 1.0, height },
            ).unwrap_or(1000.0);
            let beam_shape = ColliderTy::Box {
                height,
                width: beamlen,
            };
            let beam_tf = Transform {
                pos: tf.pos + dir * (beamlen / 2.0),
                angle: tf.angle,
            };

            info.shape = beam_shape;
            beam.overlaps.clear();
            self.all_collisions(
                beam_tf,
                beam.overlap_filter,
                beam_shape,
                &mut beam.overlaps,
            );
        }
    }
}