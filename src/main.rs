use debug::{init_on_screen_log, Debug};
use game::Game;
use macroquad::prelude::*;
use miniquad::window::set_window_size;
use physics::PhysicsState;
use render::Render;
use shipyard::{Component, World};
use sound_director::SoundDirector;
use sys::*;
use ui::Ui;

mod debug;
mod game;
mod render;
mod sys;
mod ui;
mod sound_director;
mod physics;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
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
pub struct Pos(pub Vec2);

#[derive(Debug, Clone, Copy, Component)]
pub struct Speed(pub Vec2);


#[derive(Debug, Clone, Copy, Component)]
pub struct Follower;

async fn run() -> anyhow::Result<()> {
    set_max_level(STATIC_MAX_LEVEL);
    init_on_screen_log();

    set_default_filter_mode(FilterMode::Nearest);

    info!("Setting up Rapier");

    let mut rap = PhysicsState::new();

    info!("Rapier version: {}", rapier2d::VERSION);

    let mut game = Game::new();
    let mut debug = Debug::new();
    let mut render = Render::new().await?;
    let mut sounder = SoundDirector::new().await?;
    let ui = Ui::new().await?;

    let mut world = World::new();
    let _follower = world.add_entity((
        Speed(Vec2::ZERO),
        Pos(Vec2::ZERO),
        Follower,
    ));
    let phys_test2 = world.add_entity((
        Pos(vec2(
            0.0,
            300.0,
        )),
    ));

    // world.add_component(phys_test, component);

    info!("Project version: {}", env!("CARGO_PKG_VERSION"));

    info!("Runtime created");

    let mut state = GameState::Start;
    let mut fullscreen = window_conf().fullscreen;
    let mut paused_state = state;

    // Save old size as leaving fullscreen will give window a different size
    // This value is our best bet as macroquad doesn't allow us to get window size
    let old_size = (window_conf().window_width, window_conf().window_height);

    build_textures_atlas();

    done_loading();

    info!("Done loading");

    let poses = [
        vec2(0.0, 0.0),
        vec2(64.0, -3.0),
        vec2(128.0, 20.0),
    ];

    for pos in poses {
        let phys_test = world.add_entity((
            Pos(pos),
        ));
        rap.spawn(&mut world, phys_test);
    }
    rap.spawn_ground(&mut world, phys_test2);

    loop {
        let dt = get_frame_time();

        if get_orientation() != 0.0 && state != GameState::PleaseRotate {
            paused_state = state;
            state = GameState::PleaseRotate;
        }

        let ui_model = ui.update(state);

        if ui_model.fullscreen_toggle_requested() {
            // NOTE: macroquad does not update window config when it goes fullscreen
            set_fullscreen(!fullscreen);

            if fullscreen {
                set_window_size(old_size.0 as u32, old_size.1 as u32);
            }

            fullscreen = !fullscreen;
        }

        match state {
            GameState::Start if ui_model.confirmation_detected() => {
                info!("Starting the game");
                state = GameState::Active;
            },
            GameState::Win | GameState::GameOver if ui_model.confirmation_detected() => {
                state = GameState::Active;
            },
            GameState::Paused if ui_model.pause_requested() => {
                info!("Unpausing");
                state = GameState::Active;
            },
            GameState::Active => {
                /* Update game */
                if ui_model.pause_requested() {
                    info!("Pausing");
                    state = GameState::Paused;
                }

                game.update(dt, &ui_model, &mut world);
                rap.step(&mut world);
            },
            GameState::PleaseRotate if get_orientation() == 0.0 => {
                state = paused_state;
            },
            _ => (),
        };

        render.draw(&mut world);
        ui.draw(ui_model);
        // sounder.direct_sounds(&game_model);

        debug.new_frame();
        debug.draw_ui_debug(&ui_model);
        debug.draw_events();

        world.clear_all_removed_and_deleted();

        next_frame().await
    }
}