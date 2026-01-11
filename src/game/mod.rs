mod basic_bullet;
mod components;
mod damager;
mod goal;
mod player;
mod prelude;
mod render;
mod shooter;
mod stabber;

use lib_asset::level::*;
use lib_asset::{AnimationPackId, FontId, TextureId};
use prelude::*;

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
    resources
        .load_animation_pack(AnimationPackId::Shooter)
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

            do_auto_state_transition::<&mut ShooterState>(world, resources);
            shooter::ai(dt, world, resources);
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
            state_to_anim::<&mut ShooterState>(world, resources);
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

    fn init_tile(
        &self,
        resources: &Resources,
        builder: &mut hecs::EntityBuilder,
        tile_x: u32,
        tile_y: u32,
        tile: TileIdx,
    ) {
        let ty = resources.level.map.tiles[&tile].ty;
        let tile_pos =
            vec2(tile_x as f32, tile_y as f32) * TILE_SIDE_F32 + Vec2::splat(TILE_SIDE_F32 / 2.0);

        builder.add(Transform::from_pos(tile_pos));
        builder.add(ty);
        builder.add(tile);
        if ty == TileTy::Wall {
            builder.add(BodyTag {
                groups: col_group::LEVEL,
                shape: Shape::Rect {
                    width: TILE_SIDE_F32,
                    height: TILE_SIDE_F32,
                },
            });
        }
    }

    fn init_character(
        &self,
        resources: &Resources,
        builder: &mut hecs::EntityBuilder,
        def: CharacterDef,
    ) {
        let pos = def.tf.pos;
        let look = def.tf.look_angle;
        match def.info {
            CharacterInfo::Player {} => player::init(builder, pos, resources),
            CharacterInfo::Goal {} => goal::init(builder, pos),
            CharacterInfo::Damager {} => damager::init(builder, pos),
            CharacterInfo::Stabber {} => stabber::init(builder, pos, resources),
            CharacterInfo::BasicBullet {} => basic_bullet::init(builder, pos, look, resources),
            CharacterInfo::Shooter {} => shooter::init(builder, pos, resources),
        }
    }
}

fn debug_damage_boxes(world: &World, _resources: &Resources) {
    for (_, (tf, tag)) in &mut world.query::<(&Transform, &col_query::Damage)>() {
        draw_shape_lines(tf, &tag.collider, RED);
    }
}
