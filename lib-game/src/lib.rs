mod collisions;
mod components;
mod render;

#[cfg(feature = "dbg")]
pub mod dbg;

pub mod sys;

pub use collisions::*;
pub use components::*;
pub use lib_asset::*;
pub use render::*;
use winit::{event::WindowEvent, window::Window};

use glam::*;
use hecs::{CommandBuffer, World};
use log::*;
use std::{path::Path, rc::Rc};

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
    fn handle_command(&mut self, cmd: &DebugCommand) -> bool;

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
    col_solver: CollisionSolver,
    pub world: World,
    #[cfg(feature = "dbg")]
    debug: dbg::DebugStuff,
    cmds: CommandBuffer,
    state: Box<dyn State>,

    render_world: bool,
}

impl mimiq::EventHandler<Box<dyn State>> for App {
    fn init(
        gl_ctx: Rc<mimiq::GlContext>,
        fs_server: mimiq::FsServerHandle,
        state: Box<dyn State>,
    ) -> Self {
        fs_server.submit_task("assets/atlas/bnuuy.png", 0);
        let resources = Resources::new(gl_ctx, fs_server);

        info!("Lib-game version: {}", env!("CARGO_PKG_VERSION"));

        Self {
            render: Render::new(&resources),
            col_solver: CollisionSolver::new(),
            world: World::new(),
            cmds: CommandBuffer::new(),
            #[cfg(feature = "dbg")]
            debug: dbg::DebugStuff::new(),
            resources,
            state,

            render_world: true,
        }
    }

    fn file_ready(&mut self, event: mimiq::FileReady) {
        let Ok(bytes) = event.bytes_result else {
            return;
        };
        let img = image::load_from_memory(&bytes).expect("Image load failed");

        let texture = self.resources.gl_ctx.new_texture(
            img,
            mimiq::Texture2DParams {
                internal_format: mimiq::Texture2DFormat::RGBA8,
                wrap: mimiq::TextureWrap::Clamp,
                min_filter: mimiq::FilterMode::Nearest,
                mag_filter: mimiq::FilterMode::Nearest,
            },
        );
        let texture = self.resources.textures.insert("atlas/bnuuy.png", texture);

        self.world.spawn((
            Transform::from_pos(vec2(64.0, 64.0)),
            BodyTag {
                groups: col_group::CHARACTERS,
                shape: Shape::Rect {
                    width: 32.0,
                    height: 64.0,
                },
            },
            Sprite {
                layer: 0,
                texture,
                color: mimiq::WHITE,
                sort_offset: 0.0,
                local_offset: Vec2::ZERO,
                tex_rect_pos: uvec2(0, 0),
                tex_rect_size: uvec2(67, 17),
            },
        ));
    }

    fn update(&mut self, dt: std::time::Duration) {
        #[cfg(feature = "dbg")]
        self.debug.new_update();
        #[cfg(feature = "dbg")]
        if !self.debug.should_pause() {
            if let Some(new_state) = self.update_inner(dt.as_secs_f32()) {
                self.state = new_state;
            }
        }
        #[cfg(not(feature = "dbg"))]
        if let Some(new_state) = self.update_inner(dt.as_secs_f32()) {
            self.state = new_state;
        }
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        match event {
            WindowEvent::RedrawRequested => {
                self.render.new_frame();
                #[cfg(feature = "dbg")]
                self.debug_draw();
                self.render.buffer_sprites(&mut self.world);
                self.render.render(&self.resources, self.render_world);
            }
            _ => (),
        }
    }

    #[cfg(feature = "dbg")]
    fn egui(&mut self, egui_ctx: &egui::Context) {
        self.debug_ui(egui_ctx);
    }
}

impl App {
    fn update_inner(&mut self, dt: f32) -> Option<Box<dyn State>> {
        self.col_solver.import_colliders(&mut self.world);
        self.col_solver.export_kinematic_moves(&mut self.world);

        self.state
            .plan_collision_queries(dt, &self.resources, &mut self.world, &mut self.cmds);
        self.cmds.run_on(&mut self.world);

        self.col_solver.compute_collisions(&mut self.world);

        let res = self.state.update(
            dt,
            &self.resources,
            &mut self.world,
            &self.col_solver,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        self.world.flush();
        res
    }
}

pub struct Resources {
    pub fs_server: mimiq::FsServerHandle,
    pub gl_ctx: Rc<mimiq::GlContext>,
    pub sprite_pipeline: mimiq::Pipeline<mimiq::util::BasicSpritePipelineMeta>,
    pub basic_pipeline: mimiq::Pipeline<mimiq::util::BasicPipelineMeta>,
    pub resolver: FsResolver,
    pub textures: AssetContainer<mimiq::Texture2D>,
}

impl Resources {
    pub fn new(gl_ctx: Rc<mimiq::GlContext>, fs_server: mimiq::FsServerHandle) -> Self {
        Resources {
            fs_server,
            sprite_pipeline: gl_ctx.new_pipeline(),
            basic_pipeline: gl_ctx.new_pipeline(),
            resolver: FsResolver::new(),
            textures: AssetContainer::new(),
            gl_ctx,
        }
    }
}
