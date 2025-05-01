use components::RayTag;
use lib_game::{CameraDef, FontKey, Render, TextureKey};
use logic::{decide_next_state, Game};
use macroquad::prelude::*;
use shipyard::{IntoIter, UniqueView, ViewMut};

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

async fn load_graphics(render: &mut Render) -> anyhow::Result<()> {
    set_default_filter_mode(FilterMode::Nearest);

    let tiles = load_texture("assets/tiles.png").await?;
    render.add_texture(
        TextureKey("wall"),
        &tiles,
        Some(Rect {
            x: 232.0,
            y: 304.0,
            w: 16.0,
            h: 16.0,
        }),
    );

    render.add_font(
        FontKey("oegnek"),
        &load_ttf_font("assets/oegnek.ttf").await?,
    );

    build_textures_atlas();

    Ok(())
}

#[macroquad::main(window_conf)]
async fn main() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        lib_game::sys::panic_screen(&format!("Driver panicked:\n{}", info));
        hook(info);
    }));

    set_max_level(STATIC_MAX_LEVEL);

    let mut app = lib_game::App::new(&window_conf()).await.unwrap();

    load_graphics(&mut app.render).await.unwrap();

    app.run(
        |world| {
            let game = Game::new(world);
            world.add_unique(game);
        },
        |input, dt, world| {
            world.run_with_data(Game::player_controls, (input, dt));
            world.run_with_data(Game::player_ray_controls, input);
            world.run(Game::brute_ai);
        },
        |_dt, world| {
            world.run(Game::player_sensor_pose);
        },
        |dt, world| {
            world.run(Game::update_camera);
            world.run(Game::player_ammo_pickup);
            world.run(Game::player_ray_effect);
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
        |app_state, world, render_world| {
            world.run(|game: UniqueView<Game>| render_world.add_unique(CameraDef {
                rotation: game.camera().rotation,
                zoom: game.camera().zoom,
                target: game.camera().target,
                offset: game.camera().offset,
            }));

            world.run_with_data(render::render_tiles, render_world);
            world.run_with_data(render::render_player, render_world);
            world.run_with_data(render::render_brute, render_world);
            world.run_with_data(render::render_boxes, render_world);
            world.run_with_data(render::render_rays, render_world);
            world.run_with_data(render::render_ammo, render_world);
            world.run_with_data(render::render_game_ui, render_world);

            // FIXME: this is a very dirty hack.
            // a better thing would be to temporarily spawn a ray, that gets deletted next frame.
            // We want:
            // 1. gc dead handles at the start of the frame
            // 2. before that, gc all deleted entities
            // 3. add ToDelete tag
            world.run(|mut ray_tag: ViewMut<RayTag>| for tag in (&mut ray_tag).iter() {
                tag.active = false;
            })
        },
    ).await
}


