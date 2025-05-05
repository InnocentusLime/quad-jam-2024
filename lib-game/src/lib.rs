mod components;
mod input;
mod physics;
mod render;
mod sound_director;

pub mod sys;

pub use components::*;
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
}

impl AppState {
    /// Gives a hint whether the user should start
    /// rendering the game state or not
    pub fn is_presentable(&self) -> bool {
        match self {
            AppState::Active | AppState::GameOver | AppState::Paused | AppState::Win => true,
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

    state: AppState,
    paused_state: AppState,

    console_mode: u8,
    accumelated_time: f32,
    draw_world: bool,

    pub render: Render,
    sound: SoundDirector,
    physics: PhysicsState,
    world: World,
}

impl App {
    pub async fn new(conf: &Conf) -> anyhow::Result<Self> {
        Ok(Self {
            fullscreen: conf.fullscreen,
            old_size: (conf.window_width as u32, conf.window_height as u32),

            state: AppState::Start,
            paused_state: AppState::Start,

            console_mode: 0,
            accumelated_time: 0.0,
            draw_world: true,

            render: Render::new(),
            sound: SoundDirector::new().await?,
            physics: PhysicsState::new(),
            world: World::new(),
        })
    }

    /// Just runs the game. This is what you call after loading
    /// all the resources. The app takes a few callbacks to know
    /// what to do:
    /// * init_game -- what to do to set up a game
    /// * input_phase -- various input processing
    /// * pre_physics_query_phase -- last chance to properly plan all
    /// physics engine queries
    /// * update -- the crux of the logic
    /// * render -- export the world into render
    /// * debug_render -- draw some debug assist stuff on top of the world
    ///
    /// This method will run forever as it provides the application loop.
    pub async fn run(
        mut self,
        mut init_game: impl FnMut(&mut World),
        mut input_phase: impl FnMut(&InputModel, f32, &mut World),
        mut pre_physics_query_phase: impl FnMut(f32, &mut World),
        mut update: impl FnMut(f32, &mut World) -> Option<AppState>,
        mut render: impl FnMut(AppState, &World, &mut Render),
        mut debug_render: impl FnMut(&mut World),
    ) {
        ScreenCons::init_log();

        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            ScreenDump::new_frame();

            let input = InputModel::capture();
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);
            if self.next_state(&input) {
                self.world.clear();
                init_game(&mut self.world);
            }
            if matches!(self.state, AppState::Active) && do_tick {
                self.game_update(
                    &input,
                    &mut input_phase,
                    &mut pre_physics_query_phase,
                    &mut update,
                );
            }
            self.game_present(real_dt, &mut render);
            self.debug_info(&input, &mut debug_render);
            next_frame().await
        }
    }

    fn game_present(
        &mut self,
        real_dt: f32,
        mut render: impl FnMut(AppState, &World, &mut Render),
    ) {
        self.sound.run(&self.world);
        self.render.new_frame();
        render(self.state, &self.world, &mut self.render);
        self.render.render(!self.draw_world, real_dt);
    }

    fn game_update(
        &mut self,
        input: &InputModel,
        mut input_phase: impl FnMut(&InputModel, f32, &mut World),
        mut pre_physics_query_phase: impl FnMut(f32, &mut World),
        mut update: impl FnMut(f32, &mut World) -> Option<AppState>,
    ) {
        self.world
            .run_with_data(PhysicsState::remove_dead_handles, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::allocate_bodies, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::reset_forces, &mut self.physics);

        input_phase(&input, GAME_TICKRATE, &mut self.world);

        self.world
            .run_with_data(PhysicsState::import_positions_and_info, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::import_forces, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::apply_kinematic_moves, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::step, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::export_body_poses, &mut self.physics);

        pre_physics_query_phase(GAME_TICKRATE, &mut self.world);

        self.world
            .run_with_data(PhysicsState::export_beam_queries, &mut self.physics);
        self.world
            .run_with_data(PhysicsState::export_sensor_queries, &mut self.physics);

        let new_state = update(GAME_TICKRATE, &mut self.world);

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

    fn debug_info(
        &mut self, 
        input: &InputModel,
        client_debug: impl FnOnce(&mut World),
    ) {
        if input.console_toggle_requested {
            self.console_mode = (self.console_mode + 1) % 3;
        }

        if input.scroll_down {
            ScreenCons::scroll_forward();
        }
        if input.scroll_up {
            ScreenCons::scroll_back();
        }

        self.render.debug_render(|| client_debug(&mut self.world));

        let ent_count = self.world.borrow::<EntitiesView>().unwrap().iter().count();

        dump!("{}", self.accumelated_time);
        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");

        match self.console_mode {
            0 => (),
            1 => ScreenDump::draw(),
            2 => ScreenCons::draw(),
            _ => unreachable!("Illegal console mode"),
        }
    }

    fn next_state(&mut self, input: &InputModel) -> bool {
        /* Mobile device orientation enforcement */

        if sys::get_orientation() != 0.0 && self.state != AppState::PleaseRotate {
            self.paused_state = self.state;
            self.state = AppState::PleaseRotate;
            return false;
        }

        if sys::get_orientation() == 0.0 && self.state == AppState::PleaseRotate {
            self.state = self.paused_state;
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
