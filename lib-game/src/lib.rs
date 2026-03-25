mod collisions;
mod components;
mod input;
mod prefab;
mod prelude;
mod render;

#[cfg(feature = "dbg")]
pub mod dbg;

pub mod state;

pub use collisions::CollisionSolver;
pub use components::*;
pub use input::InputModel;
pub use prefab::spawn_prefab;
pub use prelude::*;
pub use state::*;

use std::path::Path;
use std::rc::Rc;

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

pub fn run(init: AppInit) {
    let conf = mimiq::Conf { fs_root: "assets".into(), ..Default::default() };
    mimiq::run::<_, App>(conf, init);
}

#[derive(Debug)]
pub struct DebugCommand {
    pub command: String,
    pub args: Vec<String>,
}

pub struct AppInit {
    pub initial_state: StateRequest,
    pub prefab_factory: PrefabFactory<Resources>,
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
    pub render: render::Render,
    input: input::Input,
    col_solver: CollisionSolver,
    #[cfg(feature = "dbg")]
    debug: dbg::DebugStuff,
    cmds: CommandBuffer,
    asset_manager: AssetManager<Resources>,
    state: Box<dyn State>,
    queued_state: Option<StateRequest>,
}

impl mimiq::EventHandler<AppInit> for App {
    fn init(gl_ctx: Rc<mimiq::GlContext>, fs_server: mimiq::FsServerHandle, init: AppInit) -> Self {
        let resources = Resources::new(gl_ctx);

        let mut prefab_factory = init.prefab_factory;
        prefab::register_libgame_components(&mut prefab_factory);
        let asset_manager = AssetManager::new(fs_server, prefab_factory);

        info!("Lib-game version {}. Started.", env!("CARGO_PKG_VERSION"));

        Self {
            render: render::Render::new(&resources),
            col_solver: CollisionSolver::new(),
            cmds: CommandBuffer::new(),
            input: input::Input::new(),
            asset_manager,
            #[cfg(feature = "dbg")]
            debug: dbg::DebugStuff::new(),
            resources,
            state: Box::new(BootState { redirect: Some(init.initial_state) }),
            queued_state: None,
        }
    }

    fn file_ready(&mut self, event: mimiq::FileReady) {
        self.asset_manager.on_file_ready(&mut self.resources, event);
        self.queue_assets(
            self.asset_manager
                .iter_assets_to_load()
                .cloned()
                .collect::<Vec<_>>(),
        );

        let Some(queued_state) = self.queued_state.take() else {
            return;
        };
        let is_state_ready = queued_state
            .dependencies
            .iter()
            .all(|dep| self.asset_manager.is_loaded(dep));
        if is_state_ready {
            info!("queued state ready: {:?}", queued_state.name);
            self.resources.world.clear();
            self.state = (queued_state.constructor)(&mut self.resources);
        } else {
            self.queued_state = Some(queued_state)
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        #[cfg(not(feature = "dbg"))]
        let update = true;
        #[cfg(feature = "dbg")]
        let update = !self.debug.game_freeze_active();

        if !update {
            self.input.reset();
            return;
        }

        let dt = dt.as_secs_f32();
        match (self.update_inner(dt), self.queued_state.is_some()) {
            (None, _) | (Some(_), true) => (),
            (Some(request), false) => {
                info!(
                    "new state ({:?}) requested with deps: {:?}",
                    request.name, request.dependencies
                );
                self.queue_assets(request.dependencies.iter());
                self.queued_state = Some(request);
            }
        }
    }

    fn window_event(&mut self, event: WindowEvent, _window: &Window) {
        #[cfg(not(feature = "dbg"))]
        let allow_input = true;
        #[cfg(feature = "dbg")]
        let allow_input = !self.debug.game_freeze_active();

        if allow_input {
            self.input.handle_event(&event);
        }

        if event == WindowEvent::RedrawRequested {
            self.render.render(&mut self.resources);
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
    fn update_inner(&mut self, dt: f32) -> Option<StateRequest> {
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

    fn queue_assets<P: AsRef<Path>>(&mut self, asset_list: impl IntoIterator<Item = P>) {
        for unloaded in asset_list.into_iter() {
            let unloaded = unloaded.as_ref();
            if unloaded.starts_with("atlas/") {
                self.asset_manager
                    .load_image(unloaded, Resources::init_texture);
                continue;
            }
            if unloaded.starts_with("prefab/") {
                self.asset_manager
                    .load_prefab(unloaded, Resources::init_prefab);
                continue;
            }
            warn!("unknown dep: {unloaded:?}");
        }
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
        self.prefabs.insert(src, prefab);
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
        self.textures.insert(src, tex);
    }
}
