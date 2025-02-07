use std::collections::{HashMap, HashSet};

use macroquad::prelude::*;
use nalgebra::Translation2;
use rapier2d::prelude::*;
use shipyard::{Component, EntityId, IntoIter, View, ViewMut, World};

use crate::Pos;

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal)]
pub struct RapierHandle(RigidBodyHandle);

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

    pub fn spawn(&mut self, world: &mut World, ent: EntityId) {
        let mut iso = Isometry::identity();
        iso.append_translation_mut(&Translation2::new(2.0, 12.0));

        let body = self.bodies.insert(
            RigidBodyBuilder::new(RigidBodyType::Dynamic)
                .position(iso)
        );

        self.colliders.insert_with_parent(
            ColliderBuilder::new(SharedShape::cuboid(1.0, 1.0)),
            body.clone(),
            &mut self.bodies,
        );

        self.mapping.insert(ent, body);

        world.add_component(
            ent,
            RapierHandle(body),
        );
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
            let new_pos = self.bodies.get(rb.0)
                .unwrap()
                .translation();
            let new_pos = vec2(new_pos.x, new_pos.y);
            let new_pos = new_pos * 32.0 *
                    vec2(1.0, -1.0) +
                    vec2(0.0, screen_height()) +
                    vec2(0.0, -32.0);

            pos.0 = new_pos;
        });
    }
}