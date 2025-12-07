mod animations;
mod collisions;
mod components;
mod dbg;
mod health;
mod input;
mod render;

pub mod sys;

pub use collisions::*;
pub use components::*;
use dbg::DebugStuff;
use hashbrown::HashMap;
pub use input::*;
use lib_anim::{Animation, AnimationId, AnimationPackId};
use lib_asset::{FontId, FsResolver, TextureId};
use lib_level::TILE_SIDE;
pub use render::*;

use hecs::{CommandBuffer, Entity, World};
use macroquad::prelude::*;

use lib_dbg::*;

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
    fn handle_command(&mut self, app: &mut App, cmd: &Command) -> bool;

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
    collisions: CollisionSolver,
    clip_action_objects: HashMap<ClipActionObject, Entity>,
    pub world: World,
    cmds: CommandBuffer,

    render_world: bool,
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
            collisions: CollisionSolver::new(),
            clip_action_objects: HashMap::new(),
            world: World::new(),
            cmds: CommandBuffer::new(),

            render_world: true,
            freeze: false,
        })
    }

    /// Just runs the game. This is what you call after loading all the resources.
    /// This method will run forever as it provides the application loop.
    pub async fn run<G: Game>(mut self, game: &mut G) {
        let mut debug = DebugStuff::new();
        debug.debug_draws.extend(
            game.debug_draws()
                .iter()
                .map(|(name, payload)| (name.to_string(), *payload)),
        );
        #[cfg(not(target_family = "wasm"))]
        let mut anim_edit = AnimationEdit::new();

        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            let input = InputModel::capture(&self.camera);
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);

            egui_macroquad::ui(|egui_ctx| {
                #[cfg(not(target_family = "wasm"))]
                egui::Window::new("animation_edit").show(egui_ctx, |ui| {
                    anim_edit.ui(
                        &self.resources.resolver,
                        ui,
                        &mut self.resources.animations,
                        &mut self.world,
                    );
                });
                let cmd = debug.cmd_center.show(egui_ctx, get_char_pressed());
                if let Some(cmd) = cmd {
                    self.handle_command(&mut debug, game, cmd);
                }
                GLOBAL_DUMP.show(egui_ctx);
            });

            let load_level = self.next_state(&input, &debug);
            if load_level {
                info!("Loading level");
                let level = lib_level::load_level(&self.resources.resolver, "test_room")
                    .await
                    .unwrap();
                self.resources.load_texture(level.map.atlas).await;
                self.render.set_atlas(
                    &self.resources,
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

            self.render.debug_render(&self.camera, || {
                for debug_draw_name in debug.enabled_debug_draws.iter() {
                    let draw = debug.debug_draws[debug_draw_name];
                    draw(&self.world);
                }
            });

            egui_macroquad::draw();

            next_frame().await
        }
    }

    fn game_present<G: Game>(&mut self, real_dt: f32, game: &G) {
        self.update_camera();
        self.render.new_frame();
        self.render.put_tilemap_into_sprite_buffer();
        self.render
            .put_anims_into_sprite_buffer(&mut self.world, &self.resources.animations);
        game.render_export(&self.state, &self.resources, &self.world, &mut self.render);
        self.render
            .render(&self.resources, &self.camera, self.render_world, real_dt);
    }

    fn game_update<G: Game>(&mut self, input: &InputModel, game: &mut G) -> Option<AppState> {
        game.input_phase(&input, GAME_TICKRATE, &self.resources, &mut self.world);

        animations::update(GAME_TICKRATE, &mut self.world, &self.resources);
        animations::collect_clip_action_objects(&mut self.world, &mut self.clip_action_objects);
        animations::update_attack_boxes(
            &mut self.world,
            &self.resources,
            &mut self.cmds,
            &mut self.clip_action_objects,
        );
        health::reset(&mut self.world);
        animations::update_invulnerability(&mut self.world, &self.resources);
        health::update_cooldown(GAME_TICKRATE, &mut self.world);

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
        health::apply_damage(&mut self.world);
        health::apply_cooldown(&mut self.world);

        let new_state = game.update(
            GAME_TICKRATE,
            &self.resources,
            &mut self.world,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        animations::delete_clip_action_objects(
            &mut self.world,
            &self.resources,
            &mut self.cmds,
            &mut self.clip_action_objects,
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
        let lag_ms = (self.accumelated_time - 2.0 * GAME_TICKRATE) * 1000.0;
        if lag_ms > 1.0 {
            warn!("LAG by {lag_ms:.2}ms");
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

        dump!("Dt: {:.2}", self.accumelated_time);
        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
    }

    fn next_state(&mut self, input: &InputModel, debug: &DebugStuff) -> bool {
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

    fn handle_command<G: Game>(&mut self, debug: &mut DebugStuff, game: &mut G, cmd: Command) {
        match cmd.command.as_str() {
            "f" => self.freeze = true,
            "uf" => self.freeze = false,
            "hw" => self.render_world = false,
            "sw" => self.render_world = true,
            "reset" => self.state = AppState::Start,
            "dde" => {
                if cmd.args.len() < 1 {
                    error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !debug.debug_draws.contains_key(dd_name) {
                    error!("No such debug draw: {:?}", dd_name);
                    return;
                }
                debug.enabled_debug_draws.insert(dd_name.to_owned());
            }
            "ddd" => {
                if cmd.args.len() < 1 {
                    error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !debug.enabled_debug_draws.contains(dd_name) {
                    error!("No enabled debug draw: {:?}", dd_name);
                    return;
                }
                debug.enabled_debug_draws.remove(dd_name);
            }
            unmatched => {
                if !game.handle_command(self, &cmd) {
                    error!("Unknown command: {unmatched:?}");
                }
            }
        }
    }
}

pub struct Resources {
    pub resolver: FsResolver,
    pub level: Option<lib_level::LevelDef>,
    pub animations: HashMap<AnimationId, Animation>,
    pub textures: HashMap<TextureId, Texture2D>,
    pub fonts: HashMap<FontId, Font>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            resolver: FsResolver::new(),
            level: None,
            animations: HashMap::new(),
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub async fn load_texture(&mut self, texture_id: TextureId) {
        let texture = texture_id.load_texture(&self.resolver).await.unwrap();
        self.textures.insert(texture_id, texture);
    }

    pub async fn load_font(&mut self, font_id: FontId) {
        let font = font_id.load_font(&self.resolver).await.unwrap();
        self.fonts.insert(font_id, font);
    }

    /// **ADDITIVLY** loads an animations pack
    pub async fn load_animation_pack(&mut self, pack_id: AnimationPackId) {
        let pack = pack_id.load_animation_pack(&self.resolver).await.unwrap();
        self.animations.extend(pack);
    }
}
