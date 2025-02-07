use std::collections::{HashMap, HashSet};

use macroquad::prelude::*;
use nalgebra::Translation2;
use rapier2d::prelude::*;
use shipyard::{Component, EntityId, IntoIter, View, ViewMut, World};

use crate::Pos;

pub const PIXEL_PER_METER : f32 = 32.0;

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal)]
pub struct RapierHandle {
    body: RigidBodyHandle,
    collider: ColliderHandle,
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
            gravity: Vector::y() * -9.81,
            hooks: Box::new(()),
            mapping: HashMap::new(),
        }
    }

    pub fn spawn_ground(&mut self, world: &mut World, ent: EntityId) {
        let mut iso = Isometry::identity();
        iso.append_translation_mut(&Translation2::new(0.0, 0.0));

        let body = self.bodies.insert(
            RigidBodyBuilder::new(RigidBodyType::Fixed)
                .position(iso)
        );
        let collider = self.colliders.insert_with_parent(
            ColliderBuilder::new(SharedShape::cuboid(100.0, 0.5)),
            body.clone(),
            &mut self.bodies,
        );

        self.mapping.insert(ent, body);

        world.add_component(
            ent,
            RapierHandle {
                body,
                collider,
            },
        );

        world.add_component(
            ent,
            PhysBox {
                min: Vec2::ZERO,
                max: Vec2::ZERO,
            },
        );
    }

    pub fn spawn(&mut self, world: &mut World, ent: EntityId) {
        let mut iso = Isometry::identity();
        iso.append_translation_mut(&Translation2::new(0.0, 0.0));

        let body = self.bodies.insert(
            RigidBodyBuilder::new(RigidBodyType::Dynamic)
                .position(iso)
        );
        let collider = self.colliders.insert_with_parent(
            ColliderBuilder::new(SharedShape::cuboid(0.5, 0.5)),
            body.clone(),
            &mut self.bodies,
        );

        self.mapping.insert(ent, body);

        world.add_component(
            ent,
            RapierHandle {
                body,
                collider,
            },
        );

        world.add_component(
            ent,
            PhysBox {
                min: Vec2::ZERO,
                max: Vec2::ZERO,
            },
        );
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

    pub fn step(&mut self, world: &mut World) {
        // GC the dead handles
        world.run(|view: View<RapierHandle>| for remd in view.removed_or_deleted() {
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
        world.run(|rbs: View<RapierHandle>, pos: View<Pos>| for (rb, pos) in (&rbs, &pos).iter() {
            let new_pos = Self::world_to_phys(pos.0);
            let body = self.bodies.get_mut(rb.body).unwrap();

            body.set_position(
                Isometry {
                    translation: rapier2d::na::Translation2::new(
                        new_pos.x,
                        new_pos.y,
                    ),
                    ..*body.position()
                },
                true,
            );
        });


        // Step simulation
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
        world.run(|rbs: View<RapierHandle>, mut pos: ViewMut<Pos>| for (rb, pos) in (&rbs, &mut pos).iter() {
            let new_pos = self.bodies.get(rb.body)
                .unwrap()
                .translation();
            let new_pos = vec2(new_pos.x, new_pos.y);
            let new_pos = Self::phys_to_world(new_pos);

            pos.0 = new_pos;
        });

        world.run(|rbs: View<RapierHandle>, mut pbox: ViewMut<PhysBox>| for (rb, pbox) in (&rbs, &mut pbox).iter() {
            let aabb = self.colliders.get(rb.collider)
                .unwrap()
                .compute_aabb();

            *pbox = PhysBox {
                min: Self::phys_to_world(vec2(aabb.mins.x, aabb.mins.y)),
                max: Self::phys_to_world(vec2(aabb.maxs.x, aabb.maxs.y)),
            };
        });
    }
}