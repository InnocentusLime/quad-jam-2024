use macroquad::prelude::*;
use nalgebra::Translation2;
use rapier2d::prelude::*;

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
        }
    }

    pub fn get_pos(&self, h: &RigidBodyHandle) -> Option<Vec2> {
        let body = self.bodies.get(h.clone())?;
        let pos = body.position().translation;

        Some(vec2(pos.x, pos.y))
    }

    pub fn spawn(&mut self) -> RigidBodyHandle {
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

        body
    }

    pub fn step(&mut self) {
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
    }
}