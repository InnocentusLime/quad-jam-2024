use glam::{Vec2, vec2};
use hashbrown::HashSet;
use hecs::*;
use lib_col::*;
use lib_game::*;
use macroquad::prelude::*;

struct ControlTag;

struct EntityCollision {
    shape: Shape,
    group: Group,
}

struct CollisionTestGame {
    solver: CollisionSolver,
    collided: HashSet<Entity>,
    colliders: Vec<(Entity, Collider)>,
}

impl CollisionTestGame {
    pub fn new() -> Self {
        Self {
            solver: CollisionSolver::new(),
            collided: HashSet::new(),
            colliders: Vec::new(),
        }
    }

    fn dump_detected_colliders(&mut self, _world: &mut World, _args: &[&str]) {
        for (ent, collider) in &self.colliders {
            info!("ENT: {ent:?}");
            info!("SHAPE: {:?}", collider.shape);
            info!("GROUP: {:?}", collider.group);
            info!("POS: {}", collider.tf.translation);
            info!("MAT: {}", collider.tf.matrix2);
        }
    }
}

impl Game for CollisionTestGame {
    fn debug_commands(
        &self,
    ) -> &[(
        &'static str,
        &'static str,
        fn(&mut Self, &mut World, &[&str]),
    )] {
        &[(
            "dc",
            "dumps all detected colliders to stdout",
            Self::dump_detected_colliders,
        )]
    }

    fn debug_draws(&self) -> &[(&'static str, fn(&World))] {
        &[]
    }

    async fn init(&mut self, _data: &str, world: &mut World, _render: &mut Render) {
        info!("Init");
        world.spawn((
            ControlTag,
            Transform {
                pos: vec2(32.0, 16.0),
                angle: 0.0,
            },
            EntityCollision {
                group: Group::from_id(0),
                shape: Shape::Rect {
                    width: 32.0,
                    height: 16.0,
                },
            },
        ));
        world.spawn((
            Transform {
                pos: vec2(128.0, 60.0),
                angle: std::f32::consts::FRAC_PI_6,
            },
            EntityCollision {
                group: Group::from_id(0),
                shape: Shape::Rect {
                    width: 64.0,
                    height: 16.0,
                },
            },
        ));
        world.spawn((
            Transform {
                pos: vec2(64.0, 128.0),
                angle: std::f32::consts::FRAC_PI_3,
            },
            EntityCollision {
                group: Group::from_id(0),
                shape: Shape::Rect {
                    width: 100.0,
                    height: 45.0,
                },
            },
        ));
        world.spawn((
            Transform {
                pos: vec2(97.0, 128.0),
                angle: std::f32::consts::FRAC_PI_3,
            },
            EntityCollision {
                group: Group::from_id(0),
                shape: Shape::Circle { radius: 32.0 },
            },
        ));
        world.spawn((
            Transform {
                pos: vec2(256.0, 97.0),
                angle: std::f32::consts::FRAC_PI_3,
            },
            EntityCollision {
                group: Group::from_id(0),
                shape: Shape::Circle { radius: 128.0 },
            },
        ));
    }

    async fn next_level(
        &mut self,
        _prev: Option<&str>,
        _app_state: &AppState,
        _world: &World,
    ) -> NextState {
        info!("next state");
        NextState::Load("test".to_string())
    }

    fn input_phase(&mut self, input: &InputModel, dt: f32, world: &mut World) {
        for (_, (tf, _)) in world.query_mut::<(&mut Transform, &ControlTag)>() {
            let mut dir = Vec2::ZERO;
            if input.down_movement_down {
                dir += vec2(0.0, 1.0);
            }
            if input.up_movement_down {
                dir += vec2(0.0, -1.0);
            }
            if input.right_movement_down {
                dir += vec2(1.0, 0.0);
            }
            if input.left_movement_down {
                dir += vec2(-1.0, 0.0);
            }
            dir = dir.normalize_or_zero();

            tf.pos += dir * dt * 64.0;
            if input.attack_down {
                tf.angle += dt * std::f32::consts::PI / 180.0 * 12.0;
            }
        }
    }

    fn plan_collision_queries(&mut self, _dt: f32, _world: &mut World, _cmds: &mut CommandBuffer) {
        /* NOOP */
    }

    fn update(
        &mut self,
        _dt: f32,
        world: &mut World,
        _cmds: &mut CommandBuffer,
    ) -> Option<AppState> {
        self.collided.clear();
        self.solver.clear();
        self.colliders.clear();

        let cold = world
            .query_mut::<(&mut EntityCollision, &Transform)>()
            .into_iter()
            .map(|(ent, (info, tf))| (ent, get_entity_collider(tf, info)));
        self.solver.fill(cold);

        let cold = world
            .query_mut::<(&mut EntityCollision, &Transform)>()
            .into_iter()
            .map(|(ent, (info, tf))| (ent, get_entity_collider(tf, info)));
        self.colliders.extend(cold);

        for (_, (_, tf, info)) in world.query_mut::<(&ControlTag, &Transform, &EntityCollision)>() {
            let query = get_entity_collider(tf, info);
            self.collided.extend(
                self.solver
                    .query_overlaps(query, Group::empty())
                    .map(|(ent, _)| *ent),
            );
        }

        None
    }

    fn render_export(&self, _state: &AppState, world: &World, render: &mut Render) {
        for (ent, (tf, col)) in &mut world.query::<(&Transform, &EntityCollision)>() {
            let color = if self.collided.contains(&ent) {
                RED
            } else {
                WHITE
            };
            match col.shape {
                Shape::Rect { width, height } => render.world.spawn((
                    *tf,
                    RectShape {
                        width,
                        height,
                        origin: vec2(0.5, 0.5),
                    },
                    Tint(color),
                )),
                Shape::Circle { radius } => {
                    render
                        .world
                        .spawn((*tf, CircleShape { radius }, Tint(color)))
                }
            };
        }
    }
}

#[macroquad::test]
async fn test_shape_collisions() {
    let mut app = lib_game::App::new(&Conf::default()).await.unwrap();
    let font_bytes = include_bytes!("../assets/quaver.ttf");
    let font = load_ttf_font_from_bytes(font_bytes).unwrap();
    app.render.add_font(FontKey("undefined"), &font);
    let mut game = CollisionTestGame::new();

    set_max_level(STATIC_MAX_LEVEL);

    app.run(&mut game).await;
}

fn get_entity_collider(tf: &Transform, info: &EntityCollision) -> Collider {
    let col_tf = conv::topleft_corner_tf_to_crate(tf.pos, tf.angle);
    Collider {
        shape: info.shape,
        group: info.group,
        tf: col_tf,
    }
}
