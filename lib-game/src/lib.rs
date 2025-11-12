mod animations;
mod collisions;
mod components;
mod dbg;
mod health;
mod input;
mod render;
mod sound_director;

pub mod sys;

pub use collisions::*;
pub use components::*;
use dbg::DebugStuff;
use hashbrown::HashMap;
pub use input::*;
use lib_anim::{Animation, AnimationId};
use lib_asset::{FsResolver, TextureId};
use lib_level::TILE_SIDE;
pub use render::*;
pub use sound_director::*;

use hecs::{CommandBuffer, Entity, World};
use macroquad::prelude::*;

use lib_dbg::*;

use crate::animations::{
    collect_active_events, delete_animation_events, update_anims, update_attacks,
    update_invulnerability,
};

#[cfg(not(target_family = "wasm"))]
use dbg::AnimationEdit;

const GAME_TICKRATE: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Start,
    Active { paused: bool },
    GameOver,
    Win,
    GameDone,
    PleaseRotate,
    DebugFreeze,
}

#[derive(Debug)]
pub enum NextState {
    Load(String),
    AppState(AppState),
}

/// The trait containing all callbacks for the game,
/// that is run inside the App. It is usually best to
/// only keep configuration stuff inside this struct.
///
/// The application loop is structured as follows:
/// 1. Clearing the physics state
/// 2. Game::input_phase
/// 3. Physics simulation step and writeback
/// 4. Game::pre_physics_query_phase
/// 5. Handling of the physics queries
/// 6. Game::update
/// 7. Game::render
pub trait Game: 'static {
    /// Return the debug commands of this game. These commands
    /// will be added to the App's command registry.
    fn debug_commands(
        &self,
    ) -> &[(
        &'static str,
        &'static str,
        fn(&mut Self, &mut World, &[&str]),
    )];

    /// Return the list of the debug draws. Debug draws are batches
    /// of (usually, macroquad) draw calls to assist you at debugging
    /// the game logic.
    ///
    /// These debug draws can be used in `dde` and `ddd` and will
    /// show up in `ddl`
    fn debug_draws(&self) -> &[(&'static str, fn(&World))];

    /// Put all the appropriate data into the ECS World.
    /// The ECS world should be the only place where the state
    /// is located.
    fn init(
        &mut self,
        resources: &Resources,
        world: &mut World,
        render: &mut Render,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Handle the user input. You also get the delta-time.
    fn input_phase(
        &mut self,
        input: &InputModel,
        dt: f32,
        resources: &Resources,
        world: &mut World,
    );

    /// Set up all physics queries. This can be considered as a sort of
    /// pre-update phase.
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn plan_collision_queries(
        &mut self,
        dt: f32,
        resources: &Resources,
        world: &mut World,
        cmds: &mut CommandBuffer,
    );

    /// Main update routine. You can request the App to transition
    /// into a new state by returning [Option::Some].
    /// This phase accepts a command buffer. The commands get executed right
    /// after the this phase.
    fn update(
        &mut self,
        dt: f32,
        resources: &Resources,
        world: &mut World,
        cmds: &mut CommandBuffer,
    ) -> Option<AppState>;

    /// Export the game world for rendering.
    fn render_export(
        &self,
        state: &AppState,
        resources: &Resources,
        world: &World,
        render: &mut Render,
    );
}

impl AppState {
    /// Gives a hint whether the user should start
    /// rendering the game state or not
    pub fn is_presentable(&self) -> bool {
        match self {
            AppState::Active { .. }
            | AppState::GameOver
            | AppState::Win
            | AppState::DebugFreeze => true,
            _ => false,
        }
    }
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
    pub resources: Resources,
    accumelated_time: f32,

    camera: Camera2D,
    pub render: Render,
    sound: SoundDirector,
    collisions: CollisionSolver,
    active_events: HashMap<AnimationEvent, Entity>,
    world: World,
    cmds: CommandBuffer,

    draw_world: bool,
    freeze: bool,
}

impl App {
    pub async fn new(conf: &Conf) -> anyhow::Result<Self> {
        Ok(Self {
            fullscreen: conf.fullscreen,
            old_size: (conf.window_width as u32, conf.window_height as u32),

            state: AppState::Start,
            resources: Resources::new(),
            accumelated_time: 0.0,

            camera: Camera2D::default(),
            render: Render::new(),
            sound: SoundDirector::new().await?,
            collisions: CollisionSolver::new(),
            active_events: HashMap::new(),
            world: World::new(),
            cmds: CommandBuffer::new(),

            draw_world: true,
            freeze: false,
        })
    }

    /// Just runs the game. This is what you call after loading all the resources.
    /// This method will run forever as it provides the application loop.
    pub async fn run<G: Game>(mut self, game: &mut G) {
        let mut debug = DebugStuff::<G>::new(
            game.debug_draws()
                .iter()
                .map(|(name, payload)| (name.to_string(), *payload)),
            game.debug_commands().iter().map(|(x, y, z)| (*x, *y, *z)),
        );
        #[cfg(not(target_family = "wasm"))]
        let mut anim_edit = AnimationEdit::new();

        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        // TODO: remove
        let animations = lib_anim::AnimationPackId::Bunny
            .load_animation_pack(&self.resources.resolver)
            .await
            .unwrap();
        self.resources.animations = animations;

        loop {
            ScreenDump::new_frame();

            let input = InputModel::capture(&self.camera);
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);
            debug.input(&input, &mut self, game);

            // NOTE: this is a simple demo to show egui working
            #[cfg(not(target_family = "wasm"))]
            egui_macroquad::ui(|egui_ctx| {
                egui::Window::new("animation_edit").show(egui_ctx, |ui| {
                    anim_edit.ui(
                        &self.resources.resolver,
                        ui,
                        &mut self.resources.animations,
                        &mut self.world,
                    );
                });
            });

            let load_level = self.next_state(&input, &debug);
            if load_level {
                info!("Loading level");
                let level = lib_level::load_level(&self.resources.resolver, "test_room")
                    .await
                    .unwrap();
                self.render.add_texture(
                    TextureId::WorldAtlas,
                    &level
                        .map
                        .atlas
                        .load_texture(&self.resources.resolver)
                        .await
                        .unwrap(),
                );
                self.render.set_atlas(
                    TextureId::WorldAtlas,
                    level.map.atlas_margin,
                    level.map.atlas_spacing,
                );
                self.render.set_tilemap(&level);

                self.resources.level = Some(level);
                self.world.clear();
                game.init(&self.resources, &mut self.world, &mut self.render)
                    .await;
            }

            dump!("game state: {:?}", self.state);
            if matches!(self.state, AppState::Active { paused: false }) && do_tick {
                if let Some(next_state) = self.game_update(&input, game) {
                    self.state = next_state;
                }
            }

            self.game_present(real_dt, game);
            self.debug_info();
            debug.draw(&self.camera, &mut self.render, &self.world);
            next_frame().await
        }
    }

    fn game_present<G: Game>(&mut self, real_dt: f32, game: &G) {
        self.sound.run(&self.world);

        self.update_camera();
        self.render.new_frame();
        self.render.put_tilemap_into_sprite_buffer();
        self.render
            .put_anims_into_sprite_buffer(&mut self.world, &self.resources.animations);
        game.render_export(&self.state, &self.resources, &self.world, &mut self.render);
        self.render.render(&self.camera, !self.draw_world, real_dt);

        #[cfg(not(target_family = "wasm"))]
        egui_macroquad::draw();
    }

    fn game_update<G: Game>(&mut self, input: &InputModel, game: &mut G) -> Option<AppState> {
        game.input_phase(&input, GAME_TICKRATE, &self.resources, &mut self.world);

        update_anims(GAME_TICKRATE, &mut self.world, &self.resources);
        collect_active_events(&mut self.world, &mut self.active_events);
        update_attacks(
            &mut self.world,
            &self.resources,
            &mut self.cmds,
            &mut self.active_events,
        );
        update_invulnerability(&mut self.world, &self.resources);

        self.collisions.import_colliders(&mut self.world);
        self.collisions.export_kinematic_moves(&mut self.world);

        game.plan_collision_queries(
            GAME_TICKRATE,
            &self.resources,
            &mut self.world,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        self.collisions.export_queries(&mut self.world);

        health::collect_damage(&mut self.world);
        health::update_cooldown(GAME_TICKRATE, &mut self.world);
        health::apply_damage(&mut self.world);

        let new_state = game.update(
            GAME_TICKRATE,
            &self.resources,
            &mut self.world,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        delete_animation_events(
            &mut self.world,
            &self.resources,
            &mut self.cmds,
            &mut self.active_events,
        );
        self.world.flush();

        new_state
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
        if self.accumelated_time >= 2.0 * GAME_TICKRATE {
            warn!(
                "LAG by {:.2}ms",
                (self.accumelated_time - 2.0 * GAME_TICKRATE) * 1000.0
            );
            self.accumelated_time = 0.0;
            false
        } else if self.accumelated_time >= GAME_TICKRATE {
            self.accumelated_time -= GAME_TICKRATE;
            true
        } else {
            false
        }
    }

    fn debug_info(&mut self) {
        let ent_count = self.world.iter().count();

        dump!("{}", self.accumelated_time);
        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
    }

    fn next_state<G: Game>(&mut self, input: &InputModel, debug: &DebugStuff<G>) -> bool {
        /* Debug freeze */
        if (debug.should_pause() || self.freeze)
            && self.state == (AppState::Active { paused: false })
        {
            self.state = AppState::DebugFreeze;
            return false;
        }
        if !(debug.should_pause() || self.freeze) && self.state == AppState::DebugFreeze {
            self.state = AppState::Active { paused: false };
            return false;
        }

        /* Normal state transitions */
        match self.state {
            AppState::GameDone | AppState::GameOver if input.confirmation_detected => {
                self.state = AppState::Start;
                false
            }
            AppState::Win if input.confirmation_detected => {
                self.state = AppState::GameDone;
                false
            }
            AppState::Start if input.confirmation_detected => {
                self.state = AppState::Active { paused: false };
                true
            }
            AppState::Active { paused } if input.pause_requested => {
                self.state = AppState::Active { paused: !paused };
                false
            }
            _ => false,
        }
    }

    fn update_camera(&mut self) {
        let view_height = 17.0 * TILE_SIDE as f32;
        let view_width = ((screen_width() / screen_height()) * view_height).floor();
        self.camera = Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: view_width,
            h: view_height,
        });
        self.camera.zoom.y *= -1.0;

        // FIXME: magic numbers!
        self.camera.target = vec2(
            (0.5 * TILE_SIDE as f32) * 16.0,
            (0.5 * TILE_SIDE as f32) * 17.0,
        );
    }
}

pub struct Resources {
    pub resolver: FsResolver,
    pub level: Option<lib_level::LevelDef>,
    pub animations: HashMap<AnimationId, Animation>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            resolver: FsResolver::new(),
            level: None,
            animations: HashMap::new(),
        }
    }
}
