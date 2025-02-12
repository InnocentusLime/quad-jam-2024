use debug::{init_on_screen_log, Debug};
use game::{game_player_controls, game_update_follower, Game};
use macroquad::prelude::*;
use miniquad::window::set_window_size;
use physics::{physics_step, PhysicsState};
use render::{render_draw, Render};
use shipyard::{Component, Unique, UniqueViewMut, World};
use sound_director::{sound_director_sounds, SoundDirector};
use sys::*;
use ui::{ui_render, Ui, UiModel};

mod util;
mod debug;
mod game;
mod render;
mod sys;
mod ui;
mod sound_director;
mod physics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppState {
    Start,
    Active,
    GameOver,
    Win,
    Paused,
    PleaseRotate,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Boring Arcanoid".to_owned(),
        high_dpi: true,
        window_width: 1600,
        window_height: 900,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        sys::panic_screen(&format!("Driver panicked:\n{}", info));
        hook(info);
    }));

    if let Err(e) = run().await {
        sys::panic_screen(&format!("Driver exitted with error:\n{:?}", e));
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Transform {
    pub pos: Vec2,
    pub angle: f32,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Speed(pub Vec2);


#[derive(Debug, Clone, Copy, Component)]
pub struct Follower;

#[derive(Debug, Clone, Copy)]
#[derive(Unique)]
pub struct DeltaTime(pub f32);

async fn run() -> anyhow::Result<()> {
    set_max_level(STATIC_MAX_LEVEL);
    init_on_screen_log();

    info!("Rapier version: {}", rapier2d::VERSION);
    info!("Project version: {}", env!("CARGO_PKG_VERSION"));

    set_default_filter_mode(FilterMode::Nearest);

    let mut state = AppState::Start;
    let mut debug = Debug::new();
    let ui = Ui::new().await?;

    let mut world = World::new();
    world.add_unique(Render::new().await?);
    world.add_unique(PhysicsState::new());
    world.add_unique(SoundDirector::new().await?);
    world.add_unique(ui.update(state));
    world.add_unique(DeltaTime(0.0));
    world.add_unique(ui); // TODO: remove

    let game = Game::new(&mut world);
    world.add_unique(game);

    let mut fullscreen = window_conf().fullscreen;
    let mut paused_state = state;

    // Save old size as leaving fullscreen will give window a different size
    // This value is our best bet as macroquad doesn't allow us to get window size
    let old_size = (window_conf().window_width, window_conf().window_height);

    build_textures_atlas();

    done_loading();

    info!("Done loading");

    loop {
        if get_orientation() != 0.0 && state != AppState::PleaseRotate {
            paused_state = state;
            state = AppState::PleaseRotate;
        }

        let ui_model = world.run(|ui: UniqueViewMut<Ui>, mut ui_model: UniqueViewMut<UiModel>, mut dt: UniqueViewMut<DeltaTime>| {
            *ui_model = ui.update(state);
            dt.0 = get_frame_time();

            *ui_model
        });

        if ui_model.fullscreen_toggle_requested() {
            // NOTE: macroquad does not update window config when it goes fullscreen
            set_fullscreen(!fullscreen);

            if fullscreen {
                set_window_size(old_size.0 as u32, old_size.1 as u32);
            }

            fullscreen = !fullscreen;
        }

        match state {
            AppState::Start if ui_model.confirmation_detected() => {
                info!("Starting the game");
                state = AppState::Active;
            },
            AppState::Win | AppState::GameOver if ui_model.confirmation_detected() => {
                state = AppState::Active;
            },
            AppState::Paused if ui_model.pause_requested() => {
                info!("Unpausing");
                state = AppState::Active;
            },
            AppState::Active if ui_model.pause_requested() => {
                info!("Pausing");
                state = AppState::Paused;
            },
            AppState::Active if !ui_model.pause_requested() => {
                world.run(game_update_follower);
                world.run(game_player_controls);
                world.run(physics_step);
            },
            AppState::PleaseRotate if get_orientation() == 0.0 => {
                state = paused_state;
            },
            _ => (),
        };

        world.run(render_draw);
        world.run(ui_render);
        world.run(sound_director_sounds);

        debug.new_frame();
        debug.draw_ui_debug(&ui_model);
        debug.draw_events();

        world.clear_all_removed_and_deleted();

        next_frame().await
    }
}