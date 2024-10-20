use game_model::{player_won, GameModel};
use macroquad::prelude::*;
use miniquad::window::set_window_size;
use physics::Physics;
use render::Render;
use sound_director::SoundDirector;
use sys::*;
use ui::Ui;

mod physics;
mod render;
mod sys;
mod ui;
mod game_model;
mod sound_director;

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

async fn run() -> anyhow::Result<()> {
    set_default_filter_mode(FilterMode::Nearest);

    let mut phys = Physics::new();
    let mut render = Render::new().await?;
    let mut sounder = SoundDirector::new().await?;
    let ui = Ui::new().await?;

    let mut state = GameState::Start;
    let mut fullscreen = window_conf().fullscreen;
    let mut paused_state = state;

    // Save old size as leaving fullscreen will give window a different size
    // This value is our best bet as macroquad doesn't allow us to get window size
    let old_size = (window_conf().window_width, window_conf().window_height);

    build_textures_atlas();

    done_loading();

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

        let mut game_model = GameModel {
            dt: get_frame_time(),
            prev_state: state,
            state,
            old_physics: phys,
            physics: phys,
        };

        phys.new_frame();
        match state {
            GameState::Start if ui_model.confirmation_detected() => {
                state = GameState::Active;
            },
            GameState::Win | GameState::GameOver if ui_model.confirmation_detected() => {
                phys = Physics::new();
                game_model.old_physics = phys;
                state = GameState::Active;
            },
            GameState::Paused if ui_model.pause_requested() => {
                state = GameState::Active;
            },
            GameState::Active => {
                if ui_model.move_left() {
                    phys.move_player(dt, false);
                }

                if ui_model.move_right() {
                    phys.move_player(dt, true);
                }

                let hit_floor = phys.update(dt);

                if player_won(&phys) {
                    state = GameState::Win;
                } else if hit_floor {
                    state = GameState::GameOver;
                } else if ui_model.pause_requested() {
                    state = GameState::Paused;
                }
            },
            GameState::PleaseRotate if get_orientation() == 0.0 => {
                state = paused_state;
            },
            _ => (),
        };

        game_model.state = state;
        game_model.physics = phys;

        /*  =================== model is valid past this line ================ */

        render.draw(&game_model);
        ui.draw(ui_model);
        sounder.direct_sounds(&game_model);

        next_frame().await
    }
}