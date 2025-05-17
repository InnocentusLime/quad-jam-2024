mod components;
mod dbg;
mod input;
mod physics;
mod render;
mod sound_director;

pub mod sys;

pub use components::*;
use dbg::init_debug_commands;
use hashbrown::{HashMap, HashSet};
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
    Active,
    GameOver,
    Win,
    Paused,
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
    fn init(&self, world: &mut World);

    /// Handle the user input. You also get the delta-time.
    fn input_phase(&self, input: &InputModel, dt: f32, world: &mut World);

    fn plan_physics_queries(&self, dt: f32, world: &mut World);

    /// Main update routine. You can request the App to transition
    /// into a new state by returning [Option::Some].
    fn update(&self, dt: f32, world: &mut World) -> Option<AppState>;

    /// Export the game world for rendering.
    fn render_export(&self, state: AppState, world: &World, render: &mut Render);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsoleMode {
    Hidden,
    Dump,
    Console,
}

impl ConsoleMode {
    fn scroll(self) -> Self {
        match self {
            ConsoleMode::Hidden => ConsoleMode::Dump,
            ConsoleMode::Dump => ConsoleMode::Console,
            ConsoleMode::Console => ConsoleMode::Hidden,
        }
    }
}

impl AppState {
    /// Gives a hint whether the user should start
    /// rendering the game state or not
    pub fn is_presentable(&self) -> bool {
        match self {
            AppState::Active
            | AppState::GameOver
            | AppState::Paused
            | AppState::Win
            | AppState::DebugFreeze => true,
            _ => false,
        }
    }
}

struct DebugStuff {
    cmd: CommandCenter<App>,
    console_mode: ConsoleMode,
}

impl DebugStuff {
    fn new() -> Self {
        ScreenCons::init_log();

        Self {
            cmd: CommandCenter::new(),
            console_mode: ConsoleMode::Hidden,
        }
    }

    fn draw(&self) { 
        let mut console_mode = self.console_mode;
        if self.cmd.should_pause() {
            console_mode = ConsoleMode::Console;
        }

        match console_mode {
            ConsoleMode::Hidden => (),
            ConsoleMode::Dump => ScreenDump::draw(),
            ConsoleMode::Console => ScreenCons::draw(),
        }

        self.cmd.draw();
    }

    fn input(&mut self, input: &InputModel, app: &mut App) {
        if input.scroll_down {
            ScreenCons::scroll_forward();
        }
        if input.scroll_up {
            ScreenCons::scroll_back();
        }

        if let Some(ch) = get_char_pressed() {
            self.cmd.input(ch, app);
        }

        if input.console_toggle_requested {
            self.console_mode = self.console_mode.scroll();
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

    state: AppState,
    accumelated_time: f32,
    paused_state: AppState,

    pub render: Render,
    sound: SoundDirector,
    physics: PhysicsState,
    world: World,

    draw_world: bool,
    freeze: bool,
    debug_draws: HashMap<String, fn(&World)>,
    enabled_debug_draws: HashSet<String>,
}

impl App {
    pub async fn new(conf: &Conf) -> anyhow::Result<Self> {
        Ok(Self {
            fullscreen: conf.fullscreen,
            old_size: (conf.window_width as u32, conf.window_height as u32),

            accumelated_time: 0.0,
            state: AppState::Start,
            paused_state: AppState::Start,

            render: Render::new(),
            sound: SoundDirector::new().await?,
            physics: PhysicsState::new(),
            world: World::new(),

            draw_world: true,
            freeze: false,
            debug_draws: HashMap::new(),
            enabled_debug_draws: HashSet::new(),
        })
    }

    /// Just runs the game. This is what you call after loading all the resources. 
    /// This method will run forever as it provides the application loop.
    pub async fn run(mut self, game: &dyn Game) {
        let mut debug = DebugStuff::new();
        self.debug_draws = game.debug_draws()
            .iter()
            .map(|(name, payload)| (name.to_string(), *payload))
            .collect();
        init_debug_commands(&mut debug.cmd);
        for (cmd, description, payload) in game.debug_commands() {
            let payload = *payload;
            debug.cmd.add_command(cmd, description, move |app, args| {
                payload(&mut app.world, args)
            });
        }
        
        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            ScreenDump::new_frame();

            let input = InputModel::capture();
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);
            debug.input(&input, &mut self);
            if self.next_state(&input, &debug) {
                self.world.clear();
                game.init(&mut self.world);
            }
            if matches!(self.state, AppState::Active) && do_tick {
                self.game_update(&input, game);
            }
            self.game_present(real_dt, game);
            self.debug_info();
            self.render.debug_render(|| {
                for debug in self.enabled_debug_draws.iter() {
                    (self.debug_draws[debug])(&self.world)
                }
            });
            debug.draw();
            next_frame().await
        }
    }

    fn game_present(&mut self, real_dt: f32, game: &dyn Game) {
        self.sound.run(&self.world);
        self.render.new_frame();
        game.render_export(self.state, &self.world, &mut self.render);
        self.render.render(!self.draw_world, real_dt);
    }

    fn game_update(&mut self, input: &InputModel, game: &dyn Game) {
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

        self.world.clear_all_removed_and_deleted();

        if let Some(new_state) = new_state {
            self.state = new_state;
        }
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

    fn next_state(&mut self, input: &InputModel, debug: &DebugStuff) -> bool {
        /* Mobile device orientation enforcement */

        if sys::get_orientation() != 0.0 && self.state != AppState::PleaseRotate {
            self.paused_state = self.state;
            self.state = AppState::PleaseRotate;
            return false;
        }

        if sys::get_orientation() == 0.0 && self.state == AppState::PleaseRotate {
            return false;
        }

        /* Debug freeze */
        if (debug.cmd.should_pause() || self.freeze) && self.state == AppState::Active {
            self.state = AppState::DebugFreeze;
            return false;
        }
        if !(debug.cmd.should_pause() || self.freeze) && self.state == AppState::DebugFreeze {
            self.state = AppState::Active;
            return false;
        }

        /* Normal state transitions */
        let (new_state, reset) = match self.state {
            AppState::Start if input.confirmation_detected => (AppState::Active, true),
            AppState::Win | AppState::GameOver if input.confirmation_detected => {
                (AppState::Active, true)
            }
            AppState::Paused if input.pause_requested => (AppState::Active, false),
            AppState::Active if input.pause_requested => (AppState::Paused, false),
            AppState::Active if input.reset_requested => (AppState::Active, true),
            _ => return false,
        };

        self.state = new_state;
        reset
    }
}
