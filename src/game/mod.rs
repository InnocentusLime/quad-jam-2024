mod components;
mod damager;
mod goal;
mod health;
mod level;
mod player;
mod prelude;
mod render;
mod tile;

use prelude::*;

fn spawn_tiles(width: usize, height: usize, data: Vec<TileType>, world: &mut World) -> Entity {
    assert_eq!(data.len(), width * height);

    let storage = TileStorage::from_data(
        width,
        height,
        data.into_iter().map(|ty| world.spawn((ty,))).collect(),
    )
    .unwrap();

    for (x, y, tile) in storage.iter_poses() {
        world
            .insert(
                tile,
                (Transform {
                    pos: vec2(x as f32 * 32.0 + 16.0, y as f32 * 32.0 + 16.0),
                    angle: 0.0,
                },),
            )
            .unwrap();

        let ty = *world.get::<&TileType>(tile).unwrap();

        match ty {
            TileType::Wall => world
                .insert(
                    tile,
                    (BodyTag {
                        groups: col_group::LEVEL,
                        shape: Shape::Rect {
                            width: 32.0,
                            height: 32.0,
                        },
                    },),
                )
                .unwrap(),
            TileType::Ground => world.insert(tile, (TileSmell { time_left: 0.0 },)).unwrap(),
        }
    }

    world.spawn((storage,))
}

fn init_level(world: &mut World, level_def: level::LevelDef) {
    let tile_data = level_def
        .map
        .tiles
        .into_iter()
        .map(|x: level::TileDef| match x {
            level::TileDef::Wall => TileType::Wall,
            level::TileDef::Ground => TileType::Ground,
        })
        .collect::<Vec<_>>();

    spawn_tiles(level_def.map.width, level_def.map.height, tile_data, world);
    for entity in level_def.entities {
        match entity {
            level::EntityDef::Player(pos) => player::spawn(world, pos),
            level::EntityDef::Goal(pos) => goal::spawn(world, pos),
            level::EntityDef::Damager(pos) => damager::spawn(world, pos),
        }
    }
}

fn decide_next_state(world: &mut World) -> Option<AppState> {
    let player_dead = world
        .query_mut::<&Health>()
        .with::<&PlayerTag>()
        .into_iter()
        .all(|(_, hp)| hp.value <= 0);
    let goal_achieved = world
        .query_mut::<&GoalTag>()
        .into_iter()
        .any(|(_, goal)| goal.achieved);

    if player_dead {
        return Some(AppState::GameOver);
    }

    if goal_achieved {
        return Some(AppState::Win);
    }

    None
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

pub struct Project {
    do_ai: bool,
}

impl Project {
    pub async fn new(app: &mut App) -> Project {
        load_graphics(&mut app.render).await.unwrap();
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
            ("smell", tile::debug_draw_tile_smell),
        ]
    }

    async fn next_level(
        &mut self,
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
        let level = ron::from_str::<level::LevelDef>(prev_level.as_str()).unwrap();

        match level.next_level {
            Some(x) => NextState::Load(x),
            None => NextState::AppState(AppState::GameDone),
        }
    }

    async fn init(&mut self, path: &str, world: &mut World) {
        let level_data = load_string(path).await.unwrap();
        let level = ron::from_str::<level::LevelDef>(level_data.as_str()).unwrap();

        init_level(world, level);
    }

    fn input_phase(&mut self, input: &lib_game::InputModel, _dt: f32, world: &mut World) {
        player::controls(input, world);
        if self.do_ai { /* No enemies yet */ }
    }

    fn plan_physics_queries(&mut self, dt: f32, world: &mut World, cmds: &mut CommandBuffer) {
        player::update(dt, world, cmds);
    }

    fn update(
        &mut self,
        dt: f32,
        world: &mut World,
        _cmds: &mut CommandBuffer,
    ) -> Option<lib_game::AppState> {
        tile::tick_smell(dt, world);
        tile::player_step_smell(world);
        goal::check(world);
        health::collect_damage(world);
        health::update_cooldown(dt, world);
        health::apply_damage(world);

        decide_next_state(world)
    }

    fn render_export(&self, app_state: &AppState, world: &World, render: &mut Render) {
        if app_state.is_presentable() {
            render::tiles(render, world);
            render::player(render, world);
            render::goal(render, world);
            render::game_ui(render, world);
        }

        render::toplevel_ui(app_state, render);
    }
}
