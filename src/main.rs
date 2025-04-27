use quad_dbg::{dump, ScreenCons, ScreenDump};
use game::{decide_next_state, Game};
use macroquad::prelude::*;
use miniquad::window::set_window_size;
use physics::{BodyTag, PhysicsState};
use render::Render;
use shipyard::{EntitiesView, EntityId, UniqueViewMut, ViewMut, World, Component, Unique};
use sound_director::SoundDirector;
use sys::*;
use ui::{Ui, UiModel};

mod util;
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

#[derive(Debug, Clone, Copy)]
pub enum RewardState {
    Locked,
    Pending,
    Counted,
}

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct RewardInfo {
    pub state: RewardState,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy)]
#[derive(Unique)]
pub struct PlayerScore(pub u32);

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub enum BallState {
    InPocket,
    Throwing {
        to: Vec2,
    },
    Retracting,
    Capturing {
        enemy: EntityId,
    },
    Spinning {
        enemy: EntityId,
    },
}

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
#[repr(transparent)]
pub struct Health(i32);

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub enum EnemyState {
    Free,
    Stunned { left: f32 },
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Component)]
pub enum PlayerGunState {
    Empty,
    Full,
}

// TODO: this is a hack, because deleting entities
// in shipyard is unreasonably difficult
#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct BulletTag {
    is_picked: bool,
}

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct BoxTag;

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct PlayerTag;

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct DamageTag;

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct PlayerDamageSensorTag;

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct BruteTag;

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct RayTag {
    len: f32,
    life_left: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Component)]
pub enum TileType {
    Wall,
    Ground,
}

#[derive(Debug, Component)]
pub struct TileStorage {
    width: usize,
    height: usize,
    mem: Vec<EntityId>,
}

impl TileStorage {
    pub fn from_data(
        width: usize,
        height: usize,
        mem: Vec<EntityId>,
    ) -> Option<TileStorage> {
        if mem.len() != width * height { return None; }

        Some(TileStorage {
            width,
            height,
            mem,
        })
    }
    pub fn new(width: usize, height: usize) -> TileStorage {
        TileStorage::from_data(
            width,
            height,
            vec![
                EntityId::dead();
                width * height
            ],
        ).unwrap()
    }

    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

    pub fn get(&self, x: usize, y: usize) -> Option<EntityId> {
        debug_assert!(self.mem.len() < self.width * self.height);

        if x < self.width { return None; }
        if y < self.height { return None; }

        Some(self.mem[y * self.width + x])
    }

    pub fn set(&mut self, x: usize, y: usize, val: EntityId) {
        debug_assert!(self.mem.len() < self.width * self.height);

        if x < self.width { return; }
        if y < self.height { return; }

        self.mem[y * self.width + x] = val;
    }

    /// Returns the iterator over elements of form (x, y, entity)
    pub fn iter_poses(&'_ self) -> impl Iterator<Item = (usize, usize, EntityId)> + '_ {
        self.mem.iter()
            .enumerate()
            .map(|(idx, val)| (idx % self.width, idx / self.width, *val))
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Transform {
    pub pos: Vec2,
    pub angle: f32,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Speed(pub Vec2);

#[derive(Debug, Clone, Copy)]
#[derive(Unique)]
pub struct DeltaTime(pub f32);

#[derive(Debug, Clone, Copy, Component)]
pub enum PlayerDamageState {
    Hittable,
    Cooldown(f32),
}

fn reset_game(world: &mut World) {
    let ents = world.borrow::<EntitiesView>().unwrap()
        .iter().collect::<Vec<_>>();

    for ent in ents {
        world.delete_entity(ent);
    }

    let game = Game::new(world);

    world.add_unique(game);
}

async fn run() -> anyhow::Result<()> {
    set_max_level(STATIC_MAX_LEVEL);
    ScreenCons::init_log();

    info!("Rapier version: {}", rapier2d::VERSION);
    info!("Project version: {}", env!("CARGO_PKG_VERSION"));

    set_default_filter_mode(FilterMode::Nearest);

    let mut state = AppState::Start;

    let mut world = World::new();

    let mut ui = Ui::new().await?;
    let mut render = Render::new().await?;
    let mut physics = PhysicsState::new();
    let mut sound = SoundDirector::new().await?;

    world.add_unique(ui.update(state));
    world.add_unique(DeltaTime(0.0));

    let game = Game::new(&mut world);
    world.add_unique(game);

    let mut fullscreen = window_conf().fullscreen;
    let mut paused_state = state;
    let mut accumelated_time = 0.0f32;
    let mut perf_time = 0.0f32;
    let mut perf_ticks = 0;
    let mut console_mode = 0;

    // Save old size as leaving fullscreen will give window a different size
    // This value is our best bet as macroquad doesn't allow us to get window size
    let old_size = (window_conf().window_width, window_conf().window_height);

    build_textures_atlas();

    done_loading();

    info!("Done loading");

    loop {
        ScreenDump::new_frame();

        if get_orientation() != 0.0 && state != AppState::PleaseRotate {
            paused_state = state;
            state = AppState::PleaseRotate;
        }

        let ui_model = ui.update(state);
        let real_dt = get_frame_time();
        let fixed_dt = 1.0 / 60.0;
        let mut do_tick = false;
        accumelated_time += real_dt;
        perf_time += real_dt;

        if accumelated_time >= 2.0*fixed_dt {
            warn!("LAG");
            accumelated_time = 0.0;
            perf_time = 0.0;
            perf_ticks = 0;
        } else if accumelated_time >= fixed_dt {
            do_tick = true;
            accumelated_time -= fixed_dt;
            perf_ticks += 1;
        }

        world.run(|mut ui_model_res: UniqueViewMut<UiModel>| *ui_model_res = ui_model);

        if ui_model.fullscreen_toggle_requested() {
            // NOTE: macroquad does not update window config when it goes fullscreen
            set_fullscreen(!fullscreen);

            if fullscreen {
                set_window_size(old_size.0 as u32, old_size.1 as u32);
            }

            fullscreen = !fullscreen;
        }

        world.run_with_data(PhysicsState::allocate_bodies, &mut physics);
        // No way to clear all insertion tracking :(
        world.run(|info: ViewMut<BodyTag>| info.clear_all_inserted());

        match state {
            AppState::Start if ui_model.confirmation_detected() => {
                info!("Starting the game");
                state = AppState::Active;
            },
            AppState::Win | AppState::GameOver if ui_model.confirmation_detected() => {
                state = AppState::Active;
                reset_game(&mut world);
            },
            AppState::Paused if ui_model.pause_requested() => {
                info!("Unpausing");
                state = AppState::Active;
            },
            AppState::Active if ui_model.pause_requested() => {
                info!("Pausing");
                state = AppState::Paused;
            },
            AppState::Active if ui_model.reset_requested() => {
                info!("Resetting");
                reset_game(&mut world);
            }
            AppState::Active if !ui_model.pause_requested() => {
                if do_tick {
                    world.run(|mut dt: UniqueViewMut<DeltaTime>| {
                        dt.0 = fixed_dt
                    });

                    world.run_with_data(PhysicsState::reset_forces, &mut physics);

                    world.run(Game::brute_ai);
                    world.run(Game::player_controls);

                    world.run_with_data(PhysicsState::import_positions_and_info, &mut physics);
                    world.run_with_data(PhysicsState::import_forces, &mut physics);
                    world.run_with_data(PhysicsState::apply_kinematic_moves, &mut physics);
                    world.run_with_data(PhysicsState::step, &mut physics);
                    world.run_with_data(PhysicsState::export_body_poses, &mut physics);

                    world.run(Game::player_sensor_pose);
                    world.run(Game::player_ray_align);

                    world.run_with_data(PhysicsState::export_beam_queries, &mut physics);
                    world.run_with_data(PhysicsState::export_sensor_queries, &mut physics);

                    world.run(Game::update_camera);
                    world.run(Game::player_ammo_pickup);
                    world.run(Game::reset_amo_pickup);
                    world.run(Game::enemy_states);
                    world.run(Game::enemy_state_data);
                    world.run(Game::player_damage);
                    world.run(Game::player_shooting);
                    world.run(Game::player_damage_state);
                    world.run(Game::reward_enemies);
                    world.run(Game::count_rewards);
                    world.run(Game::ray_tick);
                }

                if let Some(new_state) = world.run(decide_next_state) {
                    state = new_state;
                }
            },
            AppState::PleaseRotate if get_orientation() == 0.0 => {
                state = paused_state;
            },
            _ => (),
        };

        world.run(|mut dt: UniqueViewMut<DeltaTime>| {
            dt.0 = real_dt
        });
        render.render(&world);
        world.run_with_data(Ui::draw, &mut ui);
        sound.run(&world);

        world.run_with_data(PhysicsState::remove_dead_handles, &mut physics);
        world.clear_all_removed_and_deleted();

        let ent_count = world.borrow::<EntitiesView>()
            .unwrap().iter().count();

        if is_key_pressed(KeyCode::GraveAccent) {
            console_mode = (console_mode + 1) % 3;
        }

        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
        dump!("Perf timing: {} & {perf_ticks}", (perf_time / fixed_dt) as i32);

        match console_mode {
            0 => (),
            1 => ScreenDump::draw(),
            2 => ScreenCons::draw(),
            _ => unreachable!("Illegal console mode"),
        }

        next_frame().await
    }
}