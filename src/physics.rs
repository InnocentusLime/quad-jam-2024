use std::collections::HashMap;

use macroquad::prelude::*;
use nalgebra::Translation2;
use rapier2d::{na::{Isometry, Isometry2, Vector2}, parry::query::{DefaultQueryDispatcher, PersistentQueryDispatcher, ShapeCastOptions}, prelude::*};
use shipyard::{Component, EntityId, Get, IntoIter, View, ViewMut, World};

use crate::Transform;

pub const PIXEL_PER_METER : f32 = 32.0;
pub const MAX_KINEMATICS_ITERS: i32 = 20;
pub const KINEMATIC_SKIN: f32 = 0.001;
pub const PUSH_SKIN: f32 = KINEMATIC_SKIN + 0.05;
pub const KINEMATIC_NORMAL_NUDGE: f32 = 1.0e-4;
pub const LENGTH_EPSILON: f32 = 1.0e-5;

#[derive(Clone, Copy, Debug)]
pub enum BodyKind {
    Dynamic,
    Static,
    Kinematic,
}

#[derive(Clone, Copy, Debug)]
pub enum ColliderTy {
    Box {
        width: f32,
        height: f32,
    },
}

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal)]
pub struct PhysicsInfo {
    col: ColliderTy,
    body: RigidBodyHandle,
}

impl PhysicsInfo {
    pub fn col(&self) -> &ColliderTy { &self.col }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct PhysBox {
    pub min: Vec2,
    pub max: Vec2,
}

pub struct PhysicsState {
    pub islands: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub pipeline: PhysicsPipeline,
    pub query_pipeline: QueryPipeline,
    pub integration_parameters: IntegrationParameters,
    pub gravity: Vector<Real>,
    pub hooks: Box<dyn PhysicsHooks>,
    pub mapping: HashMap<EntityId, RigidBodyHandle>,
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
            kinematic_cols: Vec::new(),
            manifolds: Vec::new(),
        }
    }

    pub fn spawn(
        &mut self,
        world: &mut World,
        entity: EntityId,
        collision: ColliderTy,
        kind: BodyKind,
    ) {
        let rap_ty = match kind {
            BodyKind::Dynamic => RigidBodyType::Dynamic,
            BodyKind::Static => RigidBodyType::Fixed,
            BodyKind::Kinematic => RigidBodyType::KinematicPositionBased,
        };

        let trans = world.run(|tf: View<Transform>| tf.get(entity).map(|x| *x))
            .unwrap();
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
            ColliderTy::Box { width, height } =>
            SharedShape::cuboid(
                width / 2.0 / PIXEL_PER_METER,
                height / 2.0 / PIXEL_PER_METER,
            ),
        };

        self.colliders.insert_with_parent(
            ColliderBuilder::new(collider_shape),
            body.clone(),
            &mut self.bodies,
        );
        self.mapping.insert(entity, body);
        world.add_component(
            entity,
            PhysicsInfo {
                body,
                col: collision,
            }
        );

        world.add_component(
            entity,
            PhysBox {
                min: Vec2::ZERO,
                max: Vec2::ZERO,
            },
        );
    }

    pub fn world_ang_to_phys(ang: f32) -> f32 {
        std::f32::consts::PI - ang
    }

    pub fn phys_ang_to_world(ang: f32) -> f32 {
        std::f32::consts::PI - ang
    }

    pub fn phys_to_world(p: Vec2) -> Vec2 {
        let mut out = p;

        out *= PIXEL_PER_METER;
        out.y *= -1.0;

        out
    }

    pub fn world_to_phys(p: Vec2) -> Vec2 {
        let mut out = p;

        out.y *= -1.0;
        out /= PIXEL_PER_METER;

        out
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
                    if !bod.is_dynamic()  { return true; }

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
                info!("CONT: {}", manifold.points.len());

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

                    info!("{:?}",
                        manifold.data.normal * delta_vel_per_contact.max(0.0) * mass_ratio,
                    );

                    body.apply_impulse_at_point(
                        manifold.data.normal * delta_vel_per_contact.max(0.0) * mass_ratio,
                        contact_point,
                        true,
                    );
                }
            }
        }
    }

    // Adapted code of the character controller from rapier2d
    pub fn move_kinematic(
        &mut self,
        world: &mut World,
        kinematic: EntityId,
        dr: Vec2,
    ) {
        self.kinematic_cols.clear();

        let dr = Self::world_to_phys(dr);
        let rbh = world.run(|rbs: ViewMut<PhysicsInfo>|
            (&rbs).get(kinematic).map(|x| x.body)
        )
        .expect("Failed to compute RB stuff");
        let rb = self.bodies.get(rbh).unwrap();
        let (kin_pos, kin_shape) = ((
            rb.position(),
            self.colliders.get(rb.colliders()[0])
                .unwrap()
                .shared_shape()
                .clone()
        ));

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
            let Some((handle, hit)) = self.query_pipeline.cast_shape(
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

            trans_rem = Self::get_slide_part(&hit, trans_rem);
        }

        let old_trans = kin_pos.translation.vector;
        self.move_kinematic_pushes(&*kin_shape);

        self.bodies.get_mut(rbh).unwrap().set_next_kinematic_translation(
            (old_trans + final_trans).into()
        );
    }

    pub fn step(&mut self, world: &mut World) {
        // GC the dead handles
        world.run(|view: View<PhysicsInfo>| for remd in view.removed_or_deleted() {
            let Some(rb) = self.mapping.remove(&remd)
                else { continue; };

            info!("ent:{remd:?} body:{rb:?} deletted");

            self.bodies.remove(
                rb,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            );
        });

        // Import the new positions to world
        world.run(|rbs: View<PhysicsInfo>, pos: View<Transform>| for (rb, pos) in (&rbs, &pos).iter() {
            let new_pos = Self::world_to_phys(pos.pos);
            let new_pos = Vector2::new(new_pos.x, new_pos.y);
            let new_ang = rapier2d::na::Unit::from_angle(
                Self::world_ang_to_phys(pos.angle)
            );
            let body = self.bodies.get_mut(rb.body).unwrap();

            // NOTE: perhaps we should do an epsilon compare here?
            if new_pos != *body.translation() || new_ang != *body.rotation() {
                body.set_position(Isometry {
                    rotation: new_ang,
                    translation: new_pos.into(),
                }, true);
            }
        });


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

        // Export the new positions to world
        world.run(|rbs: View<PhysicsInfo>, mut pos: ViewMut<Transform>| for (rb, pos) in (&rbs, &mut pos).iter() {
            let rb  = self.bodies.get(rb.body)
                .unwrap();
            let new_pos = rb.translation();
            let new_pos = vec2(new_pos.x, new_pos.y);
            let new_pos = Self::phys_to_world(new_pos);
            let new_angle = Self::phys_ang_to_world(rb.rotation().angle());

            pos.pos = new_pos;
            pos.angle = new_angle;
        });

        world.run(|rbs: View<PhysicsInfo>, mut pbox: ViewMut<PhysBox>| for (rb, pbox) in (&rbs, &mut pbox).iter() {
            let aabb = self.bodies.get(rb.body)
                .unwrap()
                .colliders()
                .iter()
                .map(|x| self.colliders.get(*x).unwrap().compute_aabb())
                .fold(Aabb::new_invalid(), |acc, x| acc.merged(&x));

            *pbox = PhysBox {
                min: Self::phys_to_world(vec2(aabb.mins.x, aabb.mins.y)),
                max: Self::phys_to_world(vec2(aabb.maxs.x, aabb.maxs.y)),
            };
        });
    }
}