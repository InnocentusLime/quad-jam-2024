mod collisions;
mod components;
mod render;

#[cfg(feature = "dbg")]
pub mod dbg;

pub mod sys;

use std::path::Path;

pub use collisions::*;
pub use components::*;
pub use lib_asset::*;
pub use render::*;

#[macro_export]
#[cfg(feature = "dbg")]
macro_rules! dump {
    ($($arg:tt)+) => {
        $crate::dbg::GLOBAL_DUMP.put_line(std::format_args!($($arg)+));
    };
}

#[macro_export]
#[cfg(not(feature = "dbg"))]
macro_rules! dump {
    ($($arg:tt)+) => {
        /* NOOP */
    };
}

#[derive(Debug)]
pub struct DebugCommand {
    pub command: String,
    pub args: Vec<String>,
}

use hecs::{CommandBuffer, World};
use macroquad::prelude::*;

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
pub trait Game: 'static {
    fn handle_command(&mut self, app: &mut App, cmd: &DebugCommand) -> bool;

    /// Set up all physics queries. This can be considered as a sort of
    /// pre-update phase.
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn plan_collision_queries(
        &mut self,
        dt: f32,
        resources: &Resources,
        world: &mut World,
        cmds: &mut CommandBuffer,
    );

    /// Main update routine. You can request the App to transition
    /// into a new state by returning [Option::Some].
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn update(
        &mut self,
        dt: f32,
        resources: &Resources,
        world: &mut World,
        collisions: &CollisionSolver,
        cmds: &mut CommandBuffer,
    );
}

/// The app run all the boilerplate code to make the game tick.
/// The following features are provided:
/// * State transitions and handling
/// * Debugging
/// * Physics handling
/// * Consistent tickrate timing
/// * Sound playing
/// * Integration with log-rs
/// * Drawing of the `dump!` macro
pub struct App {
    pub resources: Resources,
    pub render: Render,
    col_solver: CollisionSolver,
    pub world: World,
    cmds: CommandBuffer,

    render_world: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            resources: Resources::new(),
            render: Render::new(),
            col_solver: CollisionSolver::new(),
            world: World::new(),
            cmds: CommandBuffer::new(),

            render_world: true,
        }
    }

    /// Just runs the game. This is what you call after loading all the resources.
    /// This method will run forever as it provides the application loop.
    pub async fn run<G: Game>(mut self, game: &mut G) {
        #[cfg(feature = "dbg")]
        let mut debug = dbg::DebugStuff::new(game);

        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            #[cfg(feature = "dbg")]
            debug.ui(&mut self, game);

            let dt = get_frame_time();
            #[cfg(feature = "dbg")]
            debug.new_update();
            self.game_update(game, dt);

            self.game_present(dt);

            #[cfg(feature = "dbg")]
            debug.draw(&mut self);

            next_frame().await
        }
    }

    fn game_present(&mut self, real_dt: f32) {
        self.render.new_frame();
        self.render.buffer_sprites(&mut self.world);
        self.render
            .render(&self.resources, self.render_world, real_dt);
    }

    fn game_update<G: Game>(&mut self, game: &mut G, dt: f32) {
        self.col_solver.import_colliders(&mut self.world);
        self.col_solver.export_kinematic_moves(&mut self.world);

        game.plan_collision_queries(
            dt,
            &self.resources,
            &mut self.world,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        self.col_solver.compute_collisions(&mut self.world);

        game.update(
            dt,
            &self.resources,
            &mut self.world,
            &self.col_solver,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        self.world.flush();
    }
}

pub struct Resources {
    pub resolver: FsResolver,
    pub textures: AssetContainer<Texture2D>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            resolver: FsResolver::new(),
            textures: AssetContainer::new(),
        }
    }

    pub async fn load_texture(&mut self, path: impl AsRef<Path>) -> AssetKey {
        let src_path = path.as_ref();
        let path = self.resolver.get_path(AssetRoot::Assets, src_path);
        let path = path.to_string_lossy();
        let texture = load_texture(&path).await.unwrap();
        self.textures.insert(src_path, texture)
    }
}

impl Default for Resources {
    fn default() -> Self {
        Resources::new()
    }
}
