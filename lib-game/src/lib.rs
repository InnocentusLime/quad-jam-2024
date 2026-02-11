mod animation;
mod attack;
mod character;
mod collisions;
mod components;
mod health;
mod input;
mod level_utils;
mod render;

#[cfg(feature = "dbg")]
pub mod dbg;

pub mod sys;

use hashbrown::HashMap;
use hecs::EntityBuilder;

pub use animation::*;
pub use attack::*;
pub use character::*;
pub use collisions::*;
pub use components::*;
pub use input::*;
pub use level_utils::*;
pub use lib_asset::animation_manifest::AnimationId;
pub use lib_asset::level::*;
pub use lib_asset::*;
pub use render::*;

#[macro_export]
#[cfg(feature = "dbg")]
macro_rules! dump {
    ($($arg:tt)+) => {
        $crate::dbg::GLOBAL_DUMP.put_line(std::format_args!($($arg)+));
    };
}

#[macro_export]
#[cfg(not(feature = "dbg"))]
macro_rules! dump {
    ($($arg:tt)+) => {
        /* NOOP */
    };
}

#[derive(Debug)]
pub struct DebugCommand {
    pub command: String,
    pub args: Vec<String>,
}

use hecs::{CommandBuffer, Entity, World};
use macroquad::prelude::*;

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
    fn handle_command(&mut self, app: &mut App, cmd: &DebugCommand) -> bool;

    /// Return the list of the debug draws. Debug draws are batches
    /// of (usually, macroquad) draw calls to assist you at debugging
    /// the game logic.
    ///
    /// These debug draws can be used in `dde` and `ddd` and will
    /// show up in `ddl`
    fn debug_draws(&self) -> &[(&'static str, fn(&World, &Resources))];

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
        collisions: &CollisionSolver,
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

    fn init_tile(
        &self,
        resources: &Resources,
        builder: &mut EntityBuilder,
        tile_x: u32,
        tile_y: u32,
        tile: TileIdx,
    );

    fn init_character(&self, resources: &Resources, builder: &mut EntityBuilder, def: CharacterDef);
}

impl AppState {
    /// Gives a hint whether the user should start
    /// rendering the game state or not
    pub fn is_presentable(&self) -> bool {
        matches!(
            self,
            AppState::Active { .. } | AppState::GameOver | AppState::Win | AppState::DebugFreeze
        )
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

    pub state: AppState,
    pub queued_level: Option<LevelId>,
    pub resources: Resources,
    accumelated_time: f32,

    camera: Camera2D,
    pub render: Render,
    col_solver: CollisionSolver,
    clip_action_objects: HashMap<ClipActionObject, Entity>,
    pub world: World,
    cmds: CommandBuffer,

    render_world: bool,
    #[allow(unused)]
    freeze: bool,
}

impl App {
    pub async fn new(conf: &Conf) -> anyhow::Result<Self> {
        let mut resources = Resources::new();
        resources.cfg = load_game_config(&resources.resolver).await?;
        // TODO: log cfg

        Ok(Self {
            fullscreen: conf.fullscreen,
            old_size: (conf.window_width as u32, conf.window_height as u32),

            state: AppState::Start,
            queued_level: None,
            resources,
            accumelated_time: 0.0,

            camera: Camera2D::default(),
            render: Render::new(),
            col_solver: CollisionSolver::new(),
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
        #[cfg(feature = "dbg")]
        let mut debug = dbg::DebugStuff::new(game);

        sys::done_loading();

        info!("Done loading");
        info!("lib-game version: {}", env!("CARGO_PKG_VERSION"));

        loop {
            #[cfg(feature = "dbg")]
            debug.ui(&mut self, game);

            let input = InputModel::capture(&self.camera);
            let real_dt = get_frame_time();
            let do_tick = self.update_ticking(real_dt);
            self.fullscreen_toggles(&input);

            self.next_state(&input);
            if let Some(queued_level) = self.queued_level.take() {
                self.load_level(game, queued_level).await;
                self.state = AppState::Active { paused: false };
            }

            if do_tick {
                #[cfg(feature = "dbg")]
                debug.new_update();
                dump!("game state: {:?}", self.state);
                if matches!(self.state, AppState::Active { paused: false })
                    && let Some(next_state) = self.game_update(&input, game)
                {
                    self.state = next_state;
                }
            }

            self.game_present(real_dt, game);

            #[cfg(feature = "dbg")]
            debug.draw(&mut self);

            next_frame().await
        }
    }

    async fn load_level<G: Game>(&mut self, game: &mut G, level_id: LevelId) {
        info!("Loading level");
        let level = self
            .resources
            .resolver
            .load::<LevelDef>(level_id)
            .await
            .unwrap();
        self.resources.load_texture(level.map.atlas).await;
        self.render.set_atlas(
            &self.resources,
            TextureId::WorldAtlas,
            level.map.atlas_margin,
            level.map.atlas_spacing,
        );

        self.world.clear();
        self.resources.level = level;
        self.spawn_tiles(game);
        self.spawn_characters(game);
    }

    fn spawn_tiles<G: Game>(&mut self, game: &G) {
        let level = &self.resources.level;
        for x in 0..level.map.width {
            for y in 0..level.map.height {
                let mut builder = EntityBuilder::new();
                let tile = level.map.tilemap[(x + y * level.map.width) as usize];
                game.init_tile(&self.resources, &mut builder, x, y, tile);
                self.world.spawn(builder.build());
            }
        }
    }

    fn spawn_characters<G: Game>(&mut self, game: &G) {
        for def in self.resources.level.characters.iter() {
            let mut builder = EntityBuilder::new();
            game.init_character(&self.resources, &mut builder, *def);
            self.world.spawn(builder.build());
        }
    }

    fn game_present<G: Game>(&mut self, real_dt: f32, game: &G) {
        self.update_camera();
        self.render.new_frame();
        self.render.buffer_tiles(&mut self.world);
        animation::buffer_sprites(&mut self.world, &self.resources, &mut self.render);
        game.render_export(&self.state, &self.resources, &self.world, &mut self.render);
        self.render
            .render(&self.resources, &self.camera, self.render_world, real_dt);
    }

    fn game_update<G: Game>(&mut self, input: &InputModel, game: &mut G) -> Option<AppState> {
        game.input_phase(input, GAME_TICKRATE, &self.resources, &mut self.world);

        animation::update(GAME_TICKRATE, &mut self.world, &self.resources);
        animation::collect_clip_action_objects(&mut self.world, &mut self.clip_action_objects);
        animation::update_attack_boxes(
            &mut self.world,
            &self.resources,
            &mut self.cmds,
            &self.clip_action_objects,
        );
        animation::update_spawned(
            &mut self.world,
            &self.resources,
            &mut self.cmds,
            game,
            &self.clip_action_objects,
        );
        health::reset(&mut self.world);
        animation::update_invulnerability(&mut self.world, &self.resources);
        health::update_cooldown(GAME_TICKRATE, &mut self.world);

        self.col_solver.import_colliders(&mut self.world);
        self.col_solver.export_kinematic_moves(&mut self.world);

        game.plan_collision_queries(
            GAME_TICKRATE,
            &self.resources,
            &mut self.world,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        self.col_solver.compute_collisions(&mut self.world);

        health::collect_damage(&mut self.world, &self.col_solver);
        health::apply_damage(&mut self.world);
        health::apply_cooldown(&mut self.world);
        attack::update_grazing(GAME_TICKRATE, &mut self.world, &self.col_solver);
        health::despawn_on_zero_health(&mut self.world, &mut self.cmds);

        let new_state = game.update(
            GAME_TICKRATE,
            &self.resources,
            &mut self.world,
            &self.col_solver,
            &mut self.cmds,
        );
        self.cmds.run_on(&mut self.world);

        animation::delete_clip_action_objects(
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
            miniquad::window::set_window_size(self.old_size.0, self.old_size.1);
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

    fn next_state(&mut self, input: &InputModel) {
        if self.state == AppState::DebugFreeze {
            return;
        }

        /* Normal state transitions */
        match self.state {
            AppState::GameDone | AppState::GameOver if input.confirmation_detected => {
                self.state = AppState::Start;
            }
            AppState::Win if input.confirmation_detected => {
                self.state = AppState::GameDone;
            }
            AppState::Start if input.confirmation_detected => {
                self.state = AppState::Active { paused: false };
                self.queued_level = Some(LevelId::TestRoom);
            }
            AppState::Active { paused } if input.pause_requested => {
                self.state = AppState::Active { paused: !paused };
            }
            _ => (),
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
    pub cfg: GameCfg,
    pub resolver: FsResolver,
    pub level: LevelDef,
    pub animations: HashMap<AnimationId, Animation>,
    pub textures: HashMap<TextureId, Texture2D>,
    pub fonts: HashMap<FontId, Font>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            cfg: GameCfg::default(),
            resolver: FsResolver::new(),
            level: LevelDef::default(),
            animations: HashMap::new(),
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub async fn load_texture(&mut self, texture_id: TextureId) {
        let texture = self.resolver.load(texture_id).await.unwrap();
        self.textures.insert(texture_id, texture);
    }

    pub async fn load_font(&mut self, font_id: FontId) {
        let font = self.resolver.load(font_id).await.unwrap();
        self.fonts.insert(font_id, font);
    }

    /// **ADDITIVLY** loads an animations pack
    pub async fn load_animation_pack(&mut self, pack_id: AnimationPackId) {
        let pack: lib_asset::animation_manifest::AnimationPack =
            self.resolver.load(pack_id).await.unwrap();
        for (id, manifest) in pack {
            let anim = Animation::from_manifest(&manifest).unwrap();
            self.animations.insert(id, anim);
        }
    }
}

impl Default for Resources {
    fn default() -> Self {
        Resources::new()
    }
}
