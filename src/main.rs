use game::decide_next_state;
use level::LevelDef;
use lib_game::NextState;
use lib_game::{draw_physics_debug, AppState, FontKey, Game, Render, TextureKey};
use macroquad::prelude::*;
use render::render_toplevel_ui;
use shipyard::World;

use crate::game::*;
use crate::goal::*;
use crate::player::*;
use crate::tile::*;

mod components;
mod game;
mod goal;
mod level;
mod player;
mod render;
mod tile;

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

struct Project {
    do_ai: bool,
}

impl Project {
    fn new() -> Project {
        Project { do_ai: true }
    }

    fn disable_ai(&mut self, _world: &mut World, _args: &[&str]) {
        self.do_ai = false;
    }

    fn enable_ai(&mut self, _world: &mut World, _args: &[&str]) {
        self.do_ai = true;
    }
}

impl Game for Project {
    fn debug_commands(
        &self,
    ) -> &[(
        &'static str,
        &'static str,
        fn(&mut Self, &mut World, &[&str]),
    )] {
        &[
            ("noai", "disable ai", Self::disable_ai),
            ("ai", "enable ai", Self::enable_ai),
        ]
    }

    fn debug_draws(&self) -> &[(&'static str, fn(&World))] {
        &[
            ("phys", draw_physics_debug),
            ("smell", debug_draw_tile_smell),
        ]
    }

    async fn next_level(
        &self,
        prev: Option<&str>,
        app_state: &AppState,
        _world: &World,
    ) -> NextState {
        let Some(prev) = prev else {
            return NextState::Load("assets/levels/level1.ron".to_string());
        };

        if *app_state == AppState::GameOver {
            return NextState::Load(prev.to_string());
        }

        // FIXME: do not crash
        let prev_level = load_string(prev).await.unwrap();
        let level = ron::from_str::<LevelDef>(prev_level.as_str()).unwrap();

        match level.next_level {
            Some(x) => NextState::Load(x),
            None => NextState::AppState(AppState::GameDone),
        }
    }

    async fn init(&self, path: &str, world: &mut World) {
        let level_data = load_string(path).await.unwrap();
        let level = ron::from_str::<LevelDef>(level_data.as_str()).unwrap();

        init_level(world, level);
    }

    fn input_phase(&self, input: &lib_game::InputModel, dt: f32, world: &mut World) {
        world.run_with_data(player_controls, (input, dt));
        if self.do_ai { /* No enemies yet */ }
    }

    fn plan_physics_queries(&self, _dt: f32, _world: &mut World) {}

    fn update(&self, dt: f32, world: &mut World) -> Option<lib_game::AppState> {
        world.run_with_data(tick_smell, dt);
        world.run(player_step_smell);
        world.run(check_goal);

        world.run(decide_next_state)
    }

    fn render_export(&self, app_state: &AppState, world: &World, render: &mut Render) {
        if app_state.is_presentable() {
            world.run_with_data(render::render_tiles, render);
            world.run_with_data(render::render_player, render);
            world.run_with_data(render::render_rays, render);
            world.run_with_data(render::render_goal, render);
            world.run_with_data(render::render_game_ui, render);
        }

        render_toplevel_ui(app_state, render);
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

    let mut app = lib_game::App::new(&window_conf()).await.unwrap();

    load_graphics(&mut app.render).await.unwrap();

    app.run(&mut Project::new()).await;
}
