mod components;
mod damager;
mod goal;
mod health;
mod player;
mod prelude;
mod render;
mod tile;

use hashbrown::HashMap;
use lib_anim::{Animation, AnimationId};
use prelude::*;

pub const ANIMATION_TIME_UNIT: f32 = 1.0 / 1000.0;

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
            TileType::Ground => world.insert(tile, (TileSmell { time_left: 0.0 },)).unwrap(),
        }
    }

    world.spawn((storage,))
}

fn init_level(world: &mut World, level_def: lib_level::LevelDef) {
    let tile_data = level_def
        .map
        .tilemap
        .into_iter()
        .map(|idx| level_def.map.tiles[&idx])
        .map(|tile| match tile.ty {
            lib_level::TileTy::Ground => TileType::Ground,
            lib_level::TileTy::Wall => TileType::Wall,
        })
        .collect::<Vec<_>>();

    spawn_tiles(
        level_def.map.width as usize,
        level_def.map.height as usize,
        tile_data,
        world,
    );
    for entity in level_def.entities {
        let pos = vec2(
            entity.tf.pos.x + entity.width / 2.0,
            entity.tf.pos.y + entity.height / 2.0,
        );
        match entity.info {
            lib_level::EntityInfo::Player {} => player::spawn(world, pos),
            lib_level::EntityInfo::Goal {} => goal::spawn(world, pos),
            lib_level::EntityInfo::Damager {} => damager::spawn(world, pos),
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

    render.add_font(
        FontKey("quaver"),
        &load_ttf_font("assets/quaver.ttf").await?,
    );
    render.ui_font = FontKey("quaver");

    render.add_texture(
        TextureKey("bnuuy"),
        &load_texture("assets/bnuuy.png").await?,
    );

    build_textures_atlas();

    Ok(())
}

pub struct Project {
    do_ai: bool,
    animations: HashMap<AnimationId, Animation>,
}

impl Project {
    pub async fn new(app: &mut App) -> Project {
        load_graphics(&mut app.render).await.unwrap();
        let mut animations = lib_anim::load_animation_pack("bnuuy-anims").await.unwrap();

        // There is no way to specify offsets right now.
        // So we patch them in
        patch_bunny_attack_animation(animations.get_mut(&AnimationId::BunnyAttackD).unwrap());

        Project {
            do_ai: true,
            animations,
        }
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

        NextState::AppState(AppState::GameDone)
    }

    async fn init(&mut self, _path: &str, world: &mut World, render: &mut Render) {
        let level_data = lib_level::load_level("test_room").await.unwrap();
        let atlas_path = "./assets/".to_owned() + &level_data.map.atlas_path;
        render.add_texture(
            TextureKey("atlas"),
            &load_texture(&atlas_path).await.unwrap(),
        );
        render.set_atlas(
            TextureKey("atlas"),
            level_data.map.atlas_margin,
            level_data.map.atlas_spacing,
        );
        render.set_tilemap(&level_data);

        init_level(world, level_data);
    }

    fn input_phase(&mut self, input: &lib_game::InputModel, _dt: f32, world: &mut World) {
        player::controls(input, world);
        if self.do_ai { /* No enemies yet */ }
    }

    fn plan_collision_queries(&mut self, dt: f32, world: &mut World, cmds: &mut CommandBuffer) {
        player::update(dt, world, cmds);
    }

    fn update(
        &mut self,
        dt: f32,
        world: &mut World,
        _cmds: &mut CommandBuffer,
    ) -> Option<lib_game::AppState> {
        update_anims(dt, world, &self.animations);
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
            render::player_attack(render, world);
            render::goal(render, world);
            render::game_ui(render, world);
            render::anims(world, render, &self.animations);
        }

        render::toplevel_ui(app_state, render);
    }
}

fn update_anims(dt: f32, world: &mut World, animations: &HashMap<AnimationId, Animation>) {
    for (_, play) in world.query::<&mut AnimationPlay>().iter() {
        let Some(anim) = animations.get(&play.animation) else {
            warn!("No such anim: {:?}", play.animation);
            continue;
        };
        let max_pos = anim.max_pos();
        if max_pos == 0 {
            continue;
        }

        play.total_dt += dt;
        if play.total_dt < ANIMATION_TIME_UNIT {
            continue;
        }

        let cursor_delta = play.total_dt.div_euclid(ANIMATION_TIME_UNIT);
        play.total_dt -= cursor_delta * ANIMATION_TIME_UNIT;

        play.cursor += cursor_delta as u32;
        if anim.is_looping {
            play.cursor = play.cursor % max_pos;
        } else {
            play.cursor = play.cursor.min(max_pos);
        }
    }
}

fn patch_bunny_attack_animation(animation: &mut Animation) {
    use lib_anim::ClipAction::DrawSprite;

    let hit_off = 14.0;
    match &mut animation.clips[1].action {
        DrawSprite { local_pos, .. } => local_pos.y -= 3.0,
    }
    match &mut animation.clips[2].action {
        DrawSprite { local_pos, .. } => local_pos.y += hit_off,
    }
    match &mut animation.clips[3].action {
        DrawSprite { local_pos, .. } => local_pos.y += hit_off,
    }
    match &mut animation.clips[4].action {
        DrawSprite { local_pos, .. } => local_pos.y += hit_off,
    }
}
