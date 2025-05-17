mod components;
mod dbg;
mod input;
mod physics;
mod render;
mod sound_director;

pub mod sys;

pub use components::*;
use dbg::DebugStuff;
pub use input::*;
pub use physics::*;
pub use render::*;
pub use sound_director::*;

use macroquad::prelude::*;
use shipyard::{EntitiesView, World};

use quad_dbg::*;

const GAME_TICKRATE: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Start,
    Load,
    Active { paused: bool },
    GameOver,
    Win,
    PleaseRotate,
    DebugFreeze,
}

/// The trait containing all callbacks for the game,
/// that is run inside the App. Do not store the game
/// state in the structure itself. All game state should
/// be inside the ECS world.
///
/// The application loop is structured as follows:
/// 1. Clearing the physics state
/// 2. Game::input_phase
/// 3. Physics simulation step and writeback
/// 4. Game::pre_physics_query_phase
/// 5. Handling of the physics queries
/// 6. Game::update
/// 7. Game::render
pub trait Game {
    /// Return the debug commands of this game. These commands
    /// will be added to the App's command registry.
    fn debug_commands(&self) -> &[(&'static str, &'static str, fn(&mut World, &[&str]))];

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
    fn init(&self, data: &str, world: &mut World);

    /// Used by the app to consult what should be the next
    /// level to load. For now the data returned is just forwarded
    /// to `init`.
    fn next_level(&self, data: &str, app_state: &AppState, world: &World) -> String;

    /// Handle the user input. You also get the delta-time.
    fn input_phase(&self, input: &InputModel, dt: f32, world: &mut World);

    fn plan_physics_queries(&self, dt: f32, world: &mut World);

    /// Main update routine. You can request the App to transition
    /// into a new state by returning [Option::Some].
    fn update(&self, dt: f32, world: &mut World) -> Option<AppState>;

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

#[derive(Debug, Default)]
struct LoadedLevel(Option<String>);

impl LoadedLevel {
    pub fn from_string(x: String) -> LoadedLevel {
        LoadedLevel(Some(x))
    }

    pub fn get(&self) -> &str {
        self.0.as_ref().map(String::as_str).unwrap_or("null")
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

    loaded_level: LoadedLevel,
    state: AppState,
    accumelated_time: f32,

    camera: Camera2D,
    pub render: Render,
    sound: SoundDirector,
    physics: PhysicsState,
    world: World,

    draw_world: bool,
    freeze: bool,
}

impl App {
    pub async fn new(conf: &Conf) -> anyhow::Result<Self> {
        Ok(Self {
            fullscreen: conf.fullscreen,
            old_size: (conf.window_width as u32, conf.window_height as u32),

            loaded_level: LoadedLevel::default(),
            state: AppState::Start,
            accumelated_time: 0.0,

            camera: Camera2D::default(),
            render: Render::new(),
            sound: SoundDirector::new().await?,
            physics: PhysicsState::new(),
            world: World::new(),

            draw_world: true,
            freeze: false,
        })
    }

    /// Just runs the game. This is what you call after loading all the resources.
    /// This method will run forever as it provides the application loop.
    pub async fn run(mut self, game: &dyn Game) {
        let mut debug = DebugStuff::new(
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

            if let AppState::Load = &self.state {
                self.world.clear();
                game.init(self.loaded_level.get(), &mut self.world);
                self.state = AppState::Active { paused: false };
            }

            let input = InputModel::capture(&self.camera);
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);
            debug.input(&input, &mut self);

            if let Some(next_state) = self.next_state(&input, &debug, game) {
                self.state = next_state;
            }
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

    fn game_present(&mut self, real_dt: f32, game: &dyn Game) {
        self.sound.run(&self.world);
        self.render.new_frame();
        game.render_export(&self.state, &self.world, &mut self.render);
        self.render.render(&self.camera, !self.draw_world, real_dt);
    }

    fn game_update(&mut self, input: &InputModel, game: &dyn Game) -> Option<AppState> {
        self.world
            .run_with_data(PhysicsState::remove_dead_handles, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::allocate_bodies, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::reset_forces, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::reset_impulses, &mut self.physics);

        game.input_phase(&input, GAME_TICKRATE, &mut self.world);

        self.world
            .run_with_data(PhysicsState::import_positions_and_info, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::import_forces, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::import_impulses, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::apply_kinematic_moves, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::step, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::export_body_poses, &mut self.physics);

        game.plan_physics_queries(GAME_TICKRATE, &mut self.world);

        self.world
            .run_with_data(PhysicsState::export_beam_queries, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::export_sensor_queries, &mut self.physics);

        let new_state = game.update(GAME_TICKRATE, &mut self.world);

        self.update_camera();

        self.world.clear_all_removed_and_deleted();
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
            warn!("LAG");
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
        let ent_count = self.world.borrow::<EntitiesView>().unwrap().iter().count();

        dump!("{}", self.accumelated_time);
        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
    }

    fn next_state(&mut self, input: &InputModel, debug: &DebugStuff, game: &dyn Game) -> Option<AppState> {
        /* Debug freeze */
        if (debug.should_pause() || self.freeze)
            && self.state == (AppState::Active { paused: false })
        {
            return Some(AppState::DebugFreeze);
        }
        if !(debug.should_pause() || self.freeze) && self.state == AppState::DebugFreeze {
            return Some(AppState::Active { paused: false });
        }

        /* Normal state transitions */
        match self.state {
            AppState::Win | AppState::GameOver | AppState::Start if input.confirmation_detected => {
                let data = game.next_level(self.loaded_level.get(), &self.state, &self.world);
                self.loaded_level = LoadedLevel::from_string(data);
                Some(AppState::Load)
            }
            AppState::Active { paused } if input.pause_requested => {
                Some(AppState::Active { paused: !paused })
            }
            _ => None,
        }
    }

    fn update_camera(&mut self) {
        let view_height = 19.0 * 32.0;
        let view_width = (screen_width() / screen_height()) * view_height;
        self.camera = Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: view_width,
            h: view_height,
        });
        self.camera.zoom.y *= -1.0;

        // FIXME: magic numbers!
        self.camera.target = vec2(
            (0.5 * 32.0) * 17.0,
            (0.5 * 32.0) * 17.0,
        );
    }
}
