mod basic_bullet;
mod components;
mod damager;
mod goal;
mod player;
mod prelude;
mod render;
mod shooter;
mod stabber;

use lib_asset::AnimationPackId;
use lib_asset::level::*;
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

async fn load_resources(resources: &mut Resources) -> AssetKey {
    set_default_filter_mode(FilterMode::Nearest);

    let ui_font = resources.load_font("quaver.ttf").await;
    resources.load_texture("bnuuy.png").await;
    resources.load_texture("world.png").await;
    build_textures_atlas();

    resources.load_animation_pack(AnimationPackId::Bunny).await;
    resources
        .load_animation_pack(AnimationPackId::Stabber)
        .await;
    resources
        .load_animation_pack(AnimationPackId::Shooter)
        .await;
    ui_font
}

pub struct Project {
    do_ai: bool,
    ui_font: AssetKey,
    do_player_controls: bool,
    transitions: Vec<fn(&mut World, &Resources)>,
    ais: Vec<fn(f32, &mut World, &Resources)>,
    anim_syncs: Vec<fn(&mut World, &Resources)>,
}

impl Project {
    pub async fn new(app: &mut App) -> Project {
        let ui_font = load_resources(&mut app.resources).await;
        app.render.ui_font = ui_font;

        let mut proj = Project {
            do_player_controls: true,
            do_ai: true,
            transitions: Vec::new(),
            ais: Vec::new(),
            anim_syncs: Vec::new(),
            ui_font,
        };
        proj.register_character::<player::PlayerData>(None);
        proj.register_character::<&mut StabberState>(Some(stabber::ai));
        proj.register_character::<&mut ShooterState>(Some(shooter::ai));

        proj
    }

    pub fn register_character<Q: Query>(&mut self, ai: Option<fn(f32, &mut World, &Resources)>)
    where
        for<'a> Q::Item<'a>: CharacterData,
    {
        self.transitions.push(do_auto_state_transition::<Q>);
        if let Some(ai) = ai {
            self.ais.push(ai);
        }
        self.anim_syncs.push(state_to_anim::<Q>);
    }
}

impl Game for Project {
    fn handle_command(&mut self, _app: &mut App, cmd: &DebugCommand) -> bool {
        match cmd.command.as_str() {
            "nopl" => self.do_player_controls = false,
            "pl" => self.do_player_controls = true,
            "noai" => self.do_ai = false,
            "ai" => self.do_ai = true,
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
        if self.do_ai {
            for transition in &self.transitions {
                transition(world, resources)
            }
            for ai in &self.ais {
                ai(dt, world, resources)
            }
        }

        if self.do_player_controls {
            player::controls(dt, input, world, resources);
        }
    }

    fn plan_collision_queries(
        &mut self,
        _dt: f32,
        resources: &lib_game::Resources,
        world: &mut World,
        _cmds: &mut CommandBuffer,
    ) {
        if self.do_ai {
            for anim_sync in &self.anim_syncs {
                anim_sync(world, resources);
            }
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
            render::stabber_hp(render, world, self.ui_font);
            render::game_ui(render, world, self.ui_font);
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
        let ty = resources.level.map.tiles[&tile.0].ty;
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
        match def.info {
            CharacterInfo::Player {} => player::init(builder, def.pos, resources),
            CharacterInfo::Goal {} => goal::init(builder, def.pos),
            CharacterInfo::Damager {} => damager::init(builder, def.pos),
            CharacterInfo::Stabber {} => stabber::init(builder, def.pos, resources),
            CharacterInfo::BasicBullet {} => {
                basic_bullet::init(builder, def.pos, def.look_angle, resources)
            }
            CharacterInfo::Shooter {} => shooter::init(builder, def.pos, resources),
        }
    }
}

fn debug_damage_boxes(world: &World, _resources: &Resources) {
    for (_, (tf, tag)) in &mut world.query::<(&Transform, &col_query::Damage)>() {
        draw_shape_lines(tf, &tag.collider, RED);
    }
}
