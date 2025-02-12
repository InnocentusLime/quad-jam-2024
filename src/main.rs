use debug::{init_on_screen_log, Debug};
use game::Game;
use macroquad::prelude::*;
use miniquad::window::set_window_size;
use physics::{physics_move_kinematic, physics_spawn, physics_step, BodyKind, ColliderTy, PhysicsState};
use render::{render_draw, Render};
use shipyard::{Component, Unique, UniqueViewMut, World};
use sound_director::{sound_director_sounds, SoundDirector};
use sys::*;
use ui::{Ui, UiModel};

mod util;
mod debug;
mod game;
mod render;
mod sys;
mod ui;
mod sound_director;
mod physics;

pub const PLAYER_SPEED: f32 = 128.0;

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

fn spawn_walls(world: &mut World) {
    const WALL_THICK: f32 = 32.0;
    const WALL_SIDE: f32 = 480.0;

    let wall_data = [
        (WALL_SIDE / 2.0, WALL_SIDE - WALL_THICK / 2.0, WALL_SIDE, WALL_THICK),
        (WALL_SIDE / 2.0, WALL_THICK / 2.0, WALL_SIDE, WALL_THICK),
        (WALL_SIDE - WALL_THICK / 2.0, WALL_SIDE / 2.0, WALL_THICK, WALL_SIDE),
        (WALL_THICK / 2.0, WALL_SIDE / 2.0, WALL_THICK, WALL_SIDE),
    ];

    for (x, y, width, height) in wall_data {
        let wall = world.add_entity((
            Transform {
                pos: vec2(x, y),
                angle: 0.0f32,
            },
        ));
        physics_spawn(
            world,
            wall,
            ColliderTy::Box {
                width,
                height,
            },
            BodyKind::Static,
        );
    }
}

async fn run() -> anyhow::Result<()> {
    set_max_level(STATIC_MAX_LEVEL);
    init_on_screen_log();

    set_default_filter_mode(FilterMode::Nearest);

    let mut state = GameState::Start;
    let mut debug = Debug::new();
    let ui = Ui::new().await?;

    info!("Setting up Rapier");

    let mut world = World::new();
    world.add_unique(Render::new().await?);
    world.add_unique(PhysicsState::new());
    world.add_unique(SoundDirector::new().await?);
    world.add_unique(ui.update(state));
    world.add_unique(DeltaTime(0.0));

    info!("Rapier version: {}", rapier2d::VERSION);

    let mut game = Game::new();

    let _follower = world.add_entity((
        Speed(Vec2::ZERO),
        Transform {
            pos: Vec2::ZERO,
            angle: 0.0f32,
        },
        Follower,
    ));

    // world.add_component(phys_test, component);

    info!("Project version: {}", env!("CARGO_PKG_VERSION"));

    info!("Runtime created");

    let mut fullscreen = window_conf().fullscreen;
    let mut paused_state = state;

    // Save old size as leaving fullscreen will give window a different size
    // This value is our best bet as macroquad doesn't allow us to get window size
    let old_size = (window_conf().window_width, window_conf().window_height);

    build_textures_atlas();

    done_loading();

    info!("Done loading");

    let mut angle = 0.0;
    let poses = [
        vec2(200.0, 160.0),
        vec2(64.0, 250.0),
        vec2(128.0, 150.0),
        vec2(300.0, 250.0),
    ];
    let boxes = poses.map(|pos| {
        angle += 0.2;
        let the_box = world.add_entity((
            Transform {
                pos,
                angle,
            },
        ));
        physics_spawn(
            &mut world,
            the_box,
            ColliderTy::Box {
                width: 32.0,
                height: 32.0,
            },
            BodyKind::Dynamic,
        );

        the_box
    });

    spawn_walls(&mut world);

    let player = world.add_entity(
        Transform {
            pos: vec2(300.0, 300.0),
            angle: 0.0,
        }
    );
    physics_spawn(
        &mut world,
        player,
        ColliderTy::Box {
            width: 16.0,
            height: 16.0,
        },
        BodyKind::Kinematic,
    );

    loop {
        if get_orientation() != 0.0 && state != GameState::PleaseRotate {
            paused_state = state;
            state = GameState::PleaseRotate;
        }

        let (ui_model, DeltaTime(dt)) = world.run(|mut ui_model: UniqueViewMut<UiModel>, mut dt: UniqueViewMut<DeltaTime>| {
            *ui_model = ui.update(state);
            dt.0 = get_frame_time();
            (*ui_model, *dt)
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

                if is_key_pressed(KeyCode::Key1) {
                    world.delete_entity(boxes[0]);
                }
                if is_key_pressed(KeyCode::Key2) {
                    world.delete_entity(boxes[1]);
                }
                if is_key_pressed(KeyCode::Key3) {
                    world.delete_entity(boxes[2]);
                }
                if is_key_pressed(KeyCode::Key4) {
                    world.delete_entity(boxes[3]);
                }

                let mut dir = Vec2::ZERO;
                if is_key_down(KeyCode::A) {
                    dir += vec2(-1.0, 0.0);
                }
                if is_key_down(KeyCode::W) {
                    dir += vec2(0.0, -1.0);
                }
                if is_key_down(KeyCode::D) {
                    dir += vec2(1.0, 0.0);
                }
                if is_key_down(KeyCode::S) {
                    dir += vec2(0.0, 1.0);
                }

                // world.run(|mut pos: ViewMut<Transform>| {
                //     let dt = rapier2d::prelude::IntegrationParameters::default().dt;
                //     (&mut pos).get(player).unwrap().pos += dir.normalize_or_zero() * dt * 64.0;
                // });

                physics_move_kinematic(
                    &mut world,
                    player,
                    dir.normalize_or_zero() * dt * PLAYER_SPEED,
                );

                game.update(dt, &ui_model, &mut world);
                world.run(physics_step);
            },
            GameState::PleaseRotate if get_orientation() == 0.0 => {
                state = paused_state;
            },
            _ => (),
        };

        world.run(render_draw);
        ui.draw(ui_model);
        world.run(sound_director_sounds);

        debug.new_frame();
        debug.draw_ui_debug(&ui_model);
        debug.draw_events();

        world.clear_all_removed_and_deleted();

        next_frame().await
    }
}