use lib_game::{draw_physics_debug, FontKey, Render, TextureKey};
use game::{decide_next_state, Game};
use macroquad::prelude::*;
use render::render_toplevel_ui;
use shipyard::UniqueViewMut;
use shipyard::World;

use crate::enemy::*;
use crate::player::*;
use crate::tile::*;

mod tile;
mod player;
mod enemy;
mod components;
mod game;
mod render;
mod util;

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
    render.ui_font = FontKey("oegnek");

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

    app.add_debug_draw("phys", draw_physics_debug);
    app.add_debug_draw("smell", debug_draw_tile_smell);

    let debug_commands: Vec<(&'static str, &'static str, fn(&mut World, &[&str]))> = vec![
        ("noai", "disable ai", |world: &mut World, _| {
            world.run(|mut game: UniqueViewMut<Game>| game.do_ai = false);
        }),
        ("ai", "enable ai", |world: &mut World, _| {
            world.run(|mut game: UniqueViewMut<Game>| game.do_ai = true);
        }),
    ];

    load_graphics(&mut app.render).await.unwrap();


    app.run(
        debug_commands,
        |world| {
            let game = Game::new(world);
            world.add_unique(game);
        },
        |input, dt, world| {
            world.run_with_data(player_controls, (input, dt));
            world.run_with_data(player_ray_controls, input);
            if world.run(Game::should_ai) {
                world.run_with_data(update_brain, dt); 
                world.run(brute_ai);
                world.run(stalker_ai);
            }
        },
        |_dt, world| {
            world.run(player_sensor_pose);
        },
        |dt, world| {
            world.run(Game::update_camera);
            world.run_with_data(tick_smell, dt);
            world.run(player_step_smell);
            world.run(player_ammo_pickup);
            world.run(player_ray_effect);
            world.run(Game::reset_amo_pickup);
            world.run_with_data(enemy_states, dt);
            world.run(cell_phys_data);
            world.run(player_damage);
            world.run_with_data(player_damage_state, dt);
            world.run(Game::reward_enemies);
            world.run(Game::count_rewards);

            world.run(decide_next_state)
        },
        |app_state, world, render| {
            if app_state.is_presentable() {
                world.run_with_data(render::prepare_world_cam, render);
                world.run_with_data(render::render_tiles, render);
                world.run_with_data(render::render_player, render);
                world.run_with_data(render::render_brute, render);
                world.run_with_data(render::render_stalker, render);
                world.run_with_data(render::render_boxes, render);
                world.run_with_data(render::render_rays, render);
                world.run_with_data(render::render_ammo, render);
                world.run_with_data(render::render_game_ui, render);
            }

            render_toplevel_ui(app_state, render);
        },
    )
    .await
}
