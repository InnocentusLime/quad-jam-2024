mod collisions;
mod components;
mod dbg;
mod input;
mod render;
mod sound_director;

pub mod sys;

pub use collisions::*;
pub use components::*;
use dbg::DebugStuff;
pub use input::*;
use lib_level::TILE_SIDE;
pub use render::*;
pub use sound_director::*;

use hecs::{CommandBuffer, World};
use macroquad::prelude::*;

use lib_dbg::*;

const GAME_TICKRATE: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Start,
    Active { paused: bool },
    GameOver,
    Win,
    GameDone,
    PleaseRotate,
    DebugFreeze,
}

#[derive(Debug)]
pub enum NextState {
    Load(String),
    AppState(AppState),
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
pub trait Game: 'static {
    /// Return the debug commands of this game. These commands
    /// will be added to the App's command registry.
    fn debug_commands(
        &self,
    ) -> &[(
        &'static str,
        &'static str,
        fn(&mut Self, &mut World, &[&str]),
    )];

    /// Return the list of the debug draws. Debug draws are batches
    /// of (usually, macroquad) draw calls to assist you at debugging
    /// the game logic.
    ///
    /// These debug draws can be used in `dde` and `ddd` and will
    /// show up in `ddl`
    fn debug_draws(&self) -> &[(&'static str, fn(&World))];

    /// Put all the appropriate data into the ECS World.
    /// The ECS world should be the only place where the state
    /// is located.
    fn init(
        &mut self,
        data: &str,
        world: &mut World,
        render: &mut Render,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Used by the app to consult what should be the next
    /// level to load. For now the data returned is just forwarded
    /// to `init`.
    fn next_level(
        &mut self,
        prev: Option<&str>,
        app_state: &AppState,
        world: &World,
    ) -> impl std::future::Future<Output = NextState> + Send;

    /// Handle the user input. You also get the delta-time.
    fn input_phase(&mut self, input: &InputModel, dt: f32, world: &mut World);

    /// Set up all physics queries. This can be considered as a sort of
    /// pre-update phase.
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn plan_collision_queries(&mut self, dt: f32, world: &mut World, cmds: &mut CommandBuffer);

    /// Main update routine. You can request the App to transition
    /// into a new state by returning [Option::Some].
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn update(&mut self, dt: f32, world: &mut World, cmds: &mut CommandBuffer) -> Option<AppState>;

    /// Export the game world for rendering.
    fn render_export(&self, state: &AppState, world: &World, render: &mut Render);
}

impl AppState {
    /// Gives a hint whether the user should start
    /// rendering the game state or not
    pub fn is_presentable(&self) -> bool {
        match self {
            AppState::Active { .. }
            | AppState::GameOver
            | AppState::Win
            | AppState::DebugFreeze => true,
            _ => false,
        }
    }
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
    fullscreen: bool,
    old_size: (u32, u32),

    loaded_level: Option<String>,
    state: AppState,
    accumelated_time: f32,

    camera: Camera2D,
    pub render: Render,
    sound: SoundDirector,
    collisions: CollisionSolver,
    world: World,
    cmds: CommandBuffer,

    draw_world: bool,
    freeze: bool,
}

impl App {
    pub async fn new(conf: &Conf) -> anyhow::Result<Self> {
        Ok(Self {
            fullscreen: conf.fullscreen,
            old_size: (conf.window_width as u32, conf.window_height as u32),

            loaded_level: None,
            state: AppState::Start,
            accumelated_time: 0.0,

            camera: Camera2D::default(),
            render: Render::new(),
            sound: SoundDirector::new().await?,
            collisions: CollisionSolver::new(),
            world: World::new(),
            cmds: CommandBuffer::new(),

            draw_world: true,
            freeze: false,
        })
    }

    /// Just runs the game. This is what you call after loading all the resources.
    /// This method will run forever as it provides the application loop.
    pub async fn run<G: Game>(mut self, game: &mut G) {
        let mut debug = DebugStuff::<G>::new(
            game.debug_draws()
                .iter()
                .map(|(name, payload)| (name.to_string(), *payload)),
            game.debug_commands().iter().map(|(x, y, z)| (*x, *y, *z)),
        );

        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            ScreenDump::new_frame();

            let input = InputModel::capture(&self.camera);
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);
            debug.input(&input, &mut self, game);

            // NOTE: this is a simple demo to show egui working
            #[cfg(not(target_family = "wasm"))]
            egui_macroquad::ui(|egui_ctx| {
                egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                    ui.label("Test");
                });
            });

            if let Some(next_state) = self.next_state(&input, &debug, game).await {
                match next_state {
                    NextState::AppState(next_state) => self.state = next_state,
                    NextState::Load(data) => {
                        info!("Loading: {data}");
                        self.world.clear();
                        game.init(data.as_str(), &mut self.world, &mut self.render)
                            .await;
                        self.state = AppState::Active { paused: false };
                        self.loaded_level = Some(data);
                    }
                }
            }

            dump!("game state: {:?}", self.state);
            if matches!(self.state, AppState::Active { paused: false }) && do_tick {
                if let Some(next_state) = self.game_update(&input, game) {
                    self.state = next_state;
                }
            }

            self.game_present(real_dt, game);
            self.debug_info();
            debug.draw(&self.camera, &mut self.render, &self.world);
            next_frame().await
        }
    }

    fn game_present<G: Game>(&mut self, real_dt: f32, game: &G) {
        self.update_camera();
        self.sound.run(&self.world);
        self.render.new_frame();
        game.render_export(&self.state, &self.world, &mut self.render);
        self.render.render(&self.camera, !self.draw_world, real_dt);

        #[cfg(not(target_family = "wasm"))]
        egui_macroquad::draw();
    }

    fn game_update<G: Game>(&mut self, input: &InputModel, game: &mut G) -> Option<AppState> {
        game.input_phase(&input, GAME_TICKRATE, &mut self.world);

        self.collisions.import_colliders(&mut self.world);
        self.collisions.export_kinematic_moves(&mut self.world);

        game.plan_collision_queries(GAME_TICKRATE, &mut self.world, &mut self.cmds);
        self.cmds.run_on(&mut self.world);

        self.collisions.export_queries(&mut self.world);

        let new_state = game.update(GAME_TICKRATE, &mut self.world, &mut self.cmds);
        self.cmds.run_on(&mut self.world);

        self.world.flush();

        new_state
    }

    fn fullscreen_toggles(&mut self, input: &InputModel) {
        if !input.fullscreen_toggle_requested {
            return;
        }

        // NOTE: macroquad does not update window config when it goes fullscreen
        set_fullscreen(!self.fullscreen);

        if self.fullscreen {
            miniquad::window::set_window_size(self.old_size.0 as u32, self.old_size.1 as u32);
        }

        self.fullscreen = !self.fullscreen;
    }

    fn update_ticking(&mut self, real_dt: f32) -> bool {
        self.accumelated_time += real_dt;
        if self.accumelated_time >= 2.0 * GAME_TICKRATE {
            warn!(
                "LAG by {:.2}ms",
                (self.accumelated_time - 2.0 * GAME_TICKRATE) * 1000.0
            );
            self.accumelated_time = 0.0;
            false
        } else if self.accumelated_time >= GAME_TICKRATE {
            self.accumelated_time -= GAME_TICKRATE;
            true
        } else {
            false
        }
    }

    fn debug_info(&mut self) {
        let ent_count = self.world.iter().count();

        dump!("{}", self.accumelated_time);
        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
    }

    async fn next_state<G: Game>(
        &mut self,
        input: &InputModel,
        debug: &DebugStuff<G>,
        game: &mut G,
    ) -> Option<NextState> {
        /* Debug freeze */
        if (debug.should_pause() || self.freeze)
            && self.state == (AppState::Active { paused: false })
        {
            return Some(NextState::AppState(AppState::DebugFreeze));
        }
        if !(debug.should_pause() || self.freeze) && self.state == AppState::DebugFreeze {
            return Some(NextState::AppState(AppState::Active { paused: false }));
        }

        /* Normal state transitions */
        match self.state {
            AppState::GameDone if input.confirmation_detected => {
                self.loaded_level = None;
                self.state = AppState::Start;
                let verdict = game
                    .next_level(self.loaded_level.as_deref(), &self.state, &self.world)
                    .await;
                Some(verdict)
            }
            AppState::Win | AppState::GameOver | AppState::Start if input.confirmation_detected => {
                let verdict = game
                    .next_level(self.loaded_level.as_deref(), &self.state, &self.world)
                    .await;
                Some(verdict)
            }
            AppState::Active { paused } if input.pause_requested => {
                Some(NextState::AppState(AppState::Active { paused: !paused }))
            }
            _ => None,
        }
    }

    fn update_camera(&mut self) {
        let view_height = 17.0 * TILE_SIDE as f32;
        let view_width = ((screen_width() / screen_height()) * view_height).floor();
        self.camera = Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: view_width,
            h: view_height,
        });
        self.camera.zoom.y *= -1.0;

        // FIXME: magic numbers!
        self.camera.target = vec2(
            (0.5 * TILE_SIDE as f32) * 16.0,
            (0.5 * TILE_SIDE as f32) * 17.0,
        );
    }
}
