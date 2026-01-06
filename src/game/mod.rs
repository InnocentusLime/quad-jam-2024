mod basic_bullet;
mod components;
mod damager;
mod goal;
mod player;
mod prelude;
mod render;
mod stabber;

use lib_asset::level::*;
use lib_asset::{AnimationPackId, FontId, TextureId};
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
        let xy = vec2(x as f32, y as f32);
        world
            .insert(
                tile,
                (Transform::from_pos(
                    xy * TILE_SIDE_F32 + Vec2::splat(TILE_SIDE_F32 / 2.0),
                ),),
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
                            width: TILE_SIDE_F32,
                            height: TILE_SIDE_F32,
                        },
                    },),
                )
                .unwrap(),
            TileType::Ground => (),
        }
    }

    world.spawn((storage,))
}

fn init_level(world: &mut World, level_def: &LevelDef) {
    let tile_data = level_def
        .map
        .tilemap
        .iter()
        .map(|idx| level_def.map.tiles[idx])
        .map(|tile| match tile.ty {
            TileTy::Ground => TileType::Ground,
            TileTy::Wall => TileType::Wall,
        })
        .collect::<Vec<_>>();

    spawn_tiles(
        level_def.map.width as usize,
        level_def.map.height as usize,
        tile_data,
        world,
    );
    for entity in level_def.characters.iter() {
        let pos = entity.tf.pos.to_vec2();
        match entity.info {
            CharacterInfo::Player {} => player::spawn(world, pos),
            CharacterInfo::Goal {} => goal::spawn(world, pos),
            CharacterInfo::Damager {} => damager::spawn(world, pos),
            CharacterInfo::Stabber {} => stabber::spawn(world, pos),
            CharacterInfo::BasicBullet {} => basic_bullet::spawn(world, pos, entity.tf.look_angle),
        }
    }
}

fn decide_next_state(world: &mut World) -> Option<AppState> {
    let player_dead = world
        .query_mut::<&Health>()
        .with::<player::PlayerData>()
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

async fn load_resources(resources: &mut Resources) {
    set_default_filter_mode(FilterMode::Nearest);

    resources.load_font(FontId::Quaver).await;
    resources.load_texture(TextureId::BunnyAtlas).await;
    build_textures_atlas();

    resources.load_animation_pack(AnimationPackId::Bunny).await;
    resources
        .load_animation_pack(AnimationPackId::Stabber)
        .await;
}

pub struct Project {
    do_ai: bool,
    do_player: bool,
}

impl Project {
    pub async fn new(app: &mut App) -> Project {
        load_resources(&mut app.resources).await;
        Project {
            do_ai: true,
            do_player: true,
        }
    }
}

impl Game for Project {
    fn handle_command(&mut self, _app: &mut App, cmd: &DebugCommand) -> bool {
        match cmd.command.as_str() {
            "noai" => self.do_ai = false,
            "ai" => self.do_ai = true,
            "nopl" => self.do_player = false,
            "pl" => self.do_player = true,
            _ => return false,
        }
        true
    }

    fn debug_draws(&self) -> &[(&'static str, fn(&World, &Resources))] {
        &[
            ("phys", draw_physics_debug),
            ("ch", draw_char_state),
            ("dmg", debug_damage_boxes),
        ]
    }

    fn init(&mut self, resources: &lib_game::Resources, world: &mut World, _render: &mut Render) {
        if let Some(level_data) = &resources.level {
            init_level(world, level_data);
        }
    }

    fn input_phase(
        &mut self,
        input: &lib_game::InputModel,
        dt: f32,
        resources: &lib_game::Resources,
        world: &mut World,
    ) {
        if self.do_player {
            do_auto_state_transition::<player::PlayerData>(world, resources);
            player::controls(dt, input, world, resources);
        }
        if self.do_ai {
            do_auto_state_transition::<&mut StabberState>(world, resources);
            stabber::ai(dt, world, resources);
        }
    }

    fn plan_collision_queries(
        &mut self,
        _dt: f32,
        resources: &lib_game::Resources,
        world: &mut World,
        _cmds: &mut CommandBuffer,
    ) {
        if self.do_player {
            state_to_anim::<player::PlayerData>(world, resources);
        }
        if self.do_ai {
            state_to_anim::<&mut StabberState>(world, resources);
        }
    }

    fn update(
        &mut self,
        dt: f32,
        resources: &lib_game::Resources,
        world: &mut World,
        _collisions: &CollisionSolver,
        cmds: &mut CommandBuffer,
    ) -> Option<lib_game::AppState> {
        goal::check(world);

        stabber::die_on_zero_health(world, cmds);
        basic_bullet::update(dt, world, resources, cmds);

        decide_next_state(world)
    }

    fn render_export(
        &self,
        app_state: &AppState,
        _resources: &lib_game::Resources,
        world: &World,
        render: &mut Render,
    ) {
        if app_state.is_presentable() {
            render::stabber_hp(render, world);
            render::game_ui(render, world);
        }

        render::toplevel_ui(app_state, render);
    }
}

fn debug_damage_boxes(world: &World, _resources: &Resources) {
    for (_, (tf, tag)) in &mut world.query::<(&Transform, &col_query::Damage)>() {
        draw_shape_lines(tf, &tag.collider, RED);
    }
}
