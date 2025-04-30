use logic::{decide_next_state, Game};
use macroquad::prelude::*;
use render::Render;

mod util;
mod components;
mod logic;
mod render;

fn window_conf() -> Conf {
    Conf {
        window_title: "Project Swarm".to_owned(),
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
        lib_game::sys::panic_screen(&format!("Driver panicked:\n{}", info));
        hook(info);
    }));

    set_max_level(STATIC_MAX_LEVEL);

    set_default_filter_mode(FilterMode::Nearest);

    let mut render = Render::new().await.unwrap();

    build_textures_atlas();

    let app = lib_game::App::new(&window_conf()).await.unwrap();

    app.run(
        |world| {
            let game = Game::new(world);
            world.add_unique(game);
        },
        |input, dt, world| {
            world.run_with_data(Game::player_controls, (input, dt));
            world.run(Game::player_ray_align);
            world.run(Game::brute_ai);
            // FIXME: refactor this system into two smaller pieces
            world.run_with_data(Game::player_shooting, input);
        },
        |_dt, world| {
            world.run(Game::player_sensor_pose);
        },
        |dt, world| {
            world.run(Game::update_camera);
            world.run(Game::player_ammo_pickup);
            world.run(Game::reset_amo_pickup);
            world.run_with_data(Game::enemy_states, dt);
            world.run(Game::enemy_state_data);
            world.run(Game::player_damage);
            world.run_with_data(Game::player_damage_state, dt);
            world.run(Game::reward_enemies);
            world.run(Game::count_rewards);
            world.run_with_data(Game::ray_tick, dt);

            world.run(decide_next_state)
        },
        |app_state, _dt, world| {
            render.render(world);
            render.render_ui(app_state);
        },
    ).await
}


