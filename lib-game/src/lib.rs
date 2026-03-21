mod collisions;
mod components;
mod input;
mod prefab;
mod render;

#[cfg(feature = "dbg")]
pub mod dbg;

pub mod sys;

pub use collisions::*;
pub use components::*;
pub use input::*;
pub use lib_asset::*;
pub use render::*;
use winit::{event::WindowEvent, window::Window};

use glam::*;
use hecs::{BuiltEntityClone, CommandBuffer, EntityBuilderClone, World};
use log::*;
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

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
    ) -> Option<Box<dyn State>>;
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
    input: Input,
    col_solver: CollisionSolver,
    #[cfg(feature = "dbg")]
    debug: dbg::DebugStuff,
    cmds: CommandBuffer,
    asset_manager: AssetManager<Resources>,
    state: Box<dyn State>,
}

impl mimiq::EventHandler<Box<dyn State>> for App {
    fn init(
        gl_ctx: Rc<mimiq::GlContext>,
        fs_server: mimiq::FsServerHandle,
        state: Box<dyn State>,
    ) -> Self {
        let resources = Resources::new(gl_ctx);
        let prefab_factory = prefab::make_prefab_factory();
        let mut asset_manager = AssetManager::new(fs_server, prefab_factory);
        asset_manager.load_prefab("test.json", Resources::init_prefab);

        info!("Lib-game version: {}", env!("CARGO_PKG_VERSION"));

        Self {
            render: Render::new(&resources),
            col_solver: CollisionSolver::new(),
            cmds: CommandBuffer::new(),
            input: Input::new(),
            asset_manager,
            #[cfg(feature = "dbg")]
            debug: dbg::DebugStuff::new(),
            resources,
            state,
        }
    }

    fn file_ready(&mut self, event: mimiq::FileReady) {
        self.asset_manager.on_file_ready(&mut self.resources, event);
        let assets_to_load = self
            .asset_manager
            .iter_assets_to_load()
            .cloned()
            .collect::<Vec<_>>();
        for unresolved in assets_to_load {
            if unresolved.starts_with("atlas/") {
                self.asset_manager
                    .load_image(&unresolved, Resources::init_texture);
                continue;
            }
            warn!("unknown dep: {unresolved:?}");
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        #[cfg(not(feature = "dbg"))]
        let update = true;
        #[cfg(feature = "dbg")]
        let update = !self.debug.should_pause();

        if !update {
            return;
        }

        if let Some(new_state) = self.update_inner(dt.as_secs_f32()) {
            self.state = new_state;
        }
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        self.input.handle_event(&event);
        match event {
            WindowEvent::RedrawRequested => self.render.render(&mut self.resources),
            _ => (),
        }
    }

    #[cfg(feature = "dbg")]
    fn egui(&mut self, egui_ctx: &egui::Context) {
        self.dump_common_info();
        self.debug_ui(egui_ctx);
        self.debug.new_update();
    }
}

impl App {
    fn update_inner(&mut self, dt: f32) -> Option<Box<dyn State>> {
        let input_model = self.input.get_input_model();
        self.state
            .input(dt, &input_model, &mut self.resources, &mut self.cmds);

        self.col_solver.import_colliders(&mut self.resources.world);
        self.col_solver
            .export_kinematic_moves(&mut self.resources.world);

        self.state
            .plan_collision_queries(dt, &mut self.resources, &mut self.cmds);
        self.cmds.run_on(&mut self.resources.world);

        self.col_solver
            .compute_collisions(&mut self.resources.world);

        let res = self
            .state
            .update(dt, &mut self.resources, &self.col_solver, &mut self.cmds);
        self.cmds.run_on(&mut self.resources.world);

        self.resources.world.flush();
        self.input.update();
        res
    }
}

pub struct Resources {
    pub world: World,
    pub gl_ctx: Rc<mimiq::GlContext>,
    pub sprite_pipeline: mimiq::Pipeline<mimiq::util::BasicSpritePipelineMeta>,
    pub basic_pipeline: mimiq::Pipeline<mimiq::util::BasicPipelineMeta>,
    pub textures: AssetContainer<mimiq::Texture2D>,
    pub prefabs: AssetContainer<BuiltEntityClone>,
}

impl Resources {
    pub fn new(gl_ctx: Rc<mimiq::GlContext>) -> Self {
        Resources {
            world: World::new(),
            sprite_pipeline: gl_ctx.new_pipeline(),
            basic_pipeline: gl_ctx.new_pipeline(),
            textures: AssetContainer::new(),
            prefabs: AssetContainer::new(),
            gl_ctx,
        }
    }

    fn init_prefab(&mut self, _fs_resolver: &FsResolver, prefab: BuiltEntityClone, src: &Path) {
        self.prefabs.insert(src.to_path_buf(), prefab);
    }

    fn init_texture(&mut self, _fs_resolver: &FsResolver, image: image::DynamicImage, src: &Path) {
        let tex = self.gl_ctx.new_texture(
            image,
            mimiq::Texture2DParams {
                internal_format: mimiq::Texture2DFormat::RGBA8,
                wrap: mimiq::TextureWrap::Clamp,
                min_filter: mimiq::FilterMode::Nearest,
                mag_filter: mimiq::FilterMode::Nearest,
            },
        );
        self.textures.insert(src.to_path_buf(), tex);
    }
}
