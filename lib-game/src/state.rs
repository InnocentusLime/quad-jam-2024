use std::path::PathBuf;

use crate::collisions::CollisionSolver;
use crate::input::InputModel;
use crate::{DebugCommand, Resources};

use hecs::CommandBuffer;

pub struct StateRequest {
    pub name: &'static str,
    pub constructor: StateConstructor,
    pub dependencies: Vec<PathBuf>,
}

type StateConstructor = Box<dyn FnOnce(&mut Resources) -> Box<dyn State>>;

/// The trait containing all callbacks for the game,
/// that is run inside the App. It is usually best to
/// only keep configuration stuff inside this struct.
///
/// The application loop is structured as follows:
/// 1. Clearing the physics state
/// 2. Game::input_phase
/// 3. Physics simulation step and writeback
/// 4. Game::pre_physics_query_phase
/// 5. Handling of the physics queries
/// 6. Game::update
/// 7. Game::render
pub trait State: 'static {
    fn handle_command(&mut self, resources: &mut Resources, cmd: &DebugCommand) -> bool;

    fn input(
        &mut self,
        dt: f32,
        input_model: &InputModel,
        resources: &mut Resources,
        cmds: &mut CommandBuffer,
    );

    /// Set up all physics queries. This can be considered as a sort of
    /// pre-update phase.
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn plan_collision_queries(
        &mut self,
        dt: f32,
        resources: &mut Resources,
        cmds: &mut CommandBuffer,
    );

    /// Main update routine. You can request the App to transition
    /// into a new state by returning [Option::Some].
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn update(
        &mut self,
        dt: f32,
        resources: &mut Resources,
        collisions: &CollisionSolver,
        cmds: &mut CommandBuffer,
    ) -> Option<StateRequest>;
}

pub struct BootState {
    pub redirect: Option<StateRequest>,
}

impl State for BootState {
    fn handle_command(&mut self, _resources: &mut Resources, _cmd: &DebugCommand) -> bool {
        false
    }

    fn input(
        &mut self,
        _dt: f32,
        _input_model: &InputModel,
        _resources: &mut Resources,
        _cmds: &mut CommandBuffer,
    ) {
    }

    fn plan_collision_queries(
        &mut self,
        _dt: f32,
        _resources: &mut Resources,
        _cmds: &mut CommandBuffer,
    ) {
    }

    fn update(
        &mut self,
        _dt: f32,
        _resources: &mut Resources,
        _collisions: &CollisionSolver,
        _cmds: &mut CommandBuffer,
    ) -> Option<StateRequest> {
        self.redirect.take()
    }
}
