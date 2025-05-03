mod input;
mod physics;
mod render;
mod sound_director;
mod components;

pub mod sys;

pub use physics::*;
pub use input::*;
pub use sound_director::*;
pub use components::*;
pub use render::*;

use shipyard::{World, EntitiesView};
use macroquad::prelude::*;

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

        // FIXME: dirty hack
        init_game(&mut self.world);

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            ScreenDump::new_frame();

            let input = InputModel::capture();

            if is_key_pressed(KeyCode::GraveAccent) {
                self.console_mode = (self.console_mode + 1) % 3;
            }

            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);

            self.fullscreen_toggles(&input);

            self.rotate_states();
            self.next_state(&input, &mut init_game);
            if self.state == AppState::Active && input.reset_requested {
                self.world.clear();
                init_game(&mut self.world);
            }

            self.world.run_with_data(PhysicsState::allocate_bodies, &mut self.physics);

            if matches!(self.state, AppState::Active) && do_tick {
                self.world.run_with_data(PhysicsState::reset_forces, &mut self.physics);

                input_phase(&input, GAME_TICKRATE, &mut self.world);

                self.world.run_with_data(PhysicsState::import_positions_and_info, &mut self.physics);
                self.world.run_with_data(PhysicsState::import_forces, &mut self.physics);
                self.world.run_with_data(PhysicsState::apply_kinematic_moves, &mut self.physics);
                self.world.run_with_data(PhysicsState::step, &mut self.physics);
                self.world.run_with_data(PhysicsState::export_body_poses, &mut self.physics);

                pre_physics_query_phase(GAME_TICKRATE, &mut self.world);

                self.world.run_with_data(PhysicsState::export_beam_queries, &mut self.physics);
                self.world.run_with_data(PhysicsState::export_sensor_queries, &mut self.physics);

                let new_state = update(GAME_TICKRATE, &mut self.world);

                if let Some(new_state) = new_state {
                    self.state = new_state;
                }
            }

            self.sound.run(&self.world);
            self.render.new_frame();
            render(self.state, &self.world, &mut self.render);
            self.render.render(!self.draw_world, real_dt);

            dump!("{}", self.accumelated_time);
            self.debug_info(&mut debug_render);

            self.world.run_with_data(PhysicsState::remove_dead_handles, &mut self.physics);
            self.world.clear_all_removed_and_deleted();

            next_frame().await
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
        if self.accumelated_time >= 2.0*GAME_TICKRATE {
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

    fn debug_info(&mut self, client_debug: impl FnOnce(&mut World)) {
        self.render.debug_render(|| client_debug(&mut self.world));

        let ent_count = self.world.borrow::<EntitiesView>()
            .unwrap().iter().count();

        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");

        match self.console_mode {
            0 => (),
            1 => ScreenDump::draw(),
            2 => ScreenCons::draw(),
            _ => unreachable!("Illegal console mode"),
        }
    }

    fn rotate_states(&mut self) {
        if sys::get_orientation() != 0.0 && self.state != AppState::PleaseRotate {
            self.paused_state = self.state;
            self.state = AppState::PleaseRotate;
        }
        if sys::get_orientation() == 0.0 && self.state == AppState::PleaseRotate {
            self.state = self.paused_state;
        }
    }

    fn next_state(
        &mut self,
        input: &InputModel,
        mut init_game: impl FnMut(&mut World),
    ) {
        let new_state = match self.state {
            AppState::Start if input.confirmation_detected => AppState::Active,
            AppState::Win | AppState::GameOver if input.confirmation_detected => AppState::Active,
            AppState::Paused if input.pause_requested => AppState::Active,
            AppState::Active if input.pause_requested => AppState::Paused,
            AppState::Active if input.reset_requested => AppState::Active,
            _ => return,
        };

        if new_state == AppState::Active && matches!(self.state, AppState::GameOver | AppState::Win) {
            self.world.clear();
            init_game(&mut self.world);
        }

        self.state = new_state;
    }
}