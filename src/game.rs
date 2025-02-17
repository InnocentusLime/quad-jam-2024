use macroquad::prelude::*;
use rapier2d::prelude::{Group, InteractionGroups};
use shipyard::{EntityId, Get, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use crate::{inline_tilemap, method_as_system, physics::{physics_spawn, BodyKind, ColliderTy, PhysicsInfo, PhysicsState}, ui::UiModel, BallState, DeltaTime, EnemyState, MobType, Speed, TileStorage, TileType, Transform};

pub const PLAYER_SPEED: f32 = 128.0;
pub const BALL_THROW_TIME: f32 = 0.2;
pub const BALL_PICK_TIME: f32 = 0.5;

fn spawn_tiles(
    width: usize,
    height: usize,
    data: Vec<TileType>,
    world: &mut World,
) -> EntityId {
    assert_eq!(data.len(), width * height);

    let storage = TileStorage::from_data(
        width,
        height,
        data.into_iter()
            .map(|ty| world.add_entity(ty))
            .collect()
    ).unwrap();

    for (x, y, tile) in storage.iter_poses() {
        world.add_component(
            tile,
            Transform {
                pos: vec2(x as f32 * 32.0 + 16.0, y as f32 * 32.0 + 16.0),
                angle: 0.0,
            }
        );

        let ty = world.get::<&TileType>(tile).unwrap();

        match ty.as_ref() {
            TileType::Wall => physics_spawn(
                world,
                tile,
                ColliderTy::Box { width: 32.0, height: 32.0, },
                BodyKind::Static,
                InteractionGroups {
                    memberships: Group::GROUP_1,
                    filter: Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_3,
                },
            ),
            TileType::Ground => (),
        }
    }

    world.add_entity(storage)
}

#[derive(Unique)]
pub struct Game {
    weapon: EntityId,
    player: EntityId,
    boxes: [EntityId; 4],
    tilemap: EntityId,
}

impl Game {
    pub fn new(world: &mut World) -> Self {
        let mut angle = 0.0;
        let poses = [
            vec2(200.0, 160.0),
            vec2(64.0, 250.0),
            vec2(128.0, 150.0),
            vec2(300.0, 250.0),
        ];
        let boxes = poses.map(|pos| {
            angle += 0.2;
            let the_box = world.add_entity((
                Transform {
                    pos,
                    angle,
                },
                MobType::Box,
            ));
            physics_spawn(
                world,
                the_box,
                ColliderTy::Box {
                    width: 32.0,
                    height: 32.0,
                },
                BodyKind::Dynamic,
                InteractionGroups {
                    memberships: Group::GROUP_1,
                    filter: Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_3,
                },
            );

            the_box
        });

        let player = world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            MobType::Player,
        ));
        physics_spawn(
            world,
            player,
            ColliderTy::Box {
                width: 16.0,
                height: 16.0,
            },
            BodyKind::Kinematic,
            InteractionGroups {
                memberships: Group::GROUP_3,
                filter: Group::GROUP_1,
            },
        );

        let weapon = world.add_entity((
            Transform {
                pos: vec2(0.0, 0.0),
                angle: 0.0,
            },
            BallState::InPocket,
            MobType::BallOfHurt,
        ));
        physics_spawn(
            world,
            weapon,
            ColliderTy::Circle {
                radius: 16.0
            },
            BodyKind::Kinematic,
            InteractionGroups {
                memberships: Group::GROUP_2,
                filter: Group::GROUP_1,
            },
        );

        let brute = world.add_entity((
            Transform {
                pos: vec2(200.0, 80.0),
                angle: 0.0,
            },
            MobType::Brute,
            EnemyState {
                captured: false,
            },
        ));
        physics_spawn(
            world,
            brute,
            ColliderTy::Box {
                width: 32.0,
                height: 32.0,
            },
            BodyKind::Kinematic,
            InteractionGroups {
                memberships: Group::GROUP_2,
                filter: Group::GROUP_1,
            },
        );

        let tilemap = spawn_tiles(
            16,
            16,
            inline_tilemap![
                w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, w, w, w, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, w, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, w, g, g, w, g, g, g, g, w,
                w, g, g, w, g, g, g, g, g, g, w, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, w, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, w, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, w, g, g, g, g, w,
                w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w
            ],
            world,
        );

        Self {
            player,
            weapon,
            boxes,
            tilemap,
        }
    }

    pub fn ball_logic(
        &mut self,
        mut pos: ViewMut<Transform>,
        mut state: ViewMut<BallState>,
        mut rbs: ViewMut<PhysicsInfo>,
        ui_model: UniqueView<UiModel>,
        mut phys: UniqueViewMut<PhysicsState>,
        dt: UniqueView<DeltaTime>,
    ) {
        let mut dr = None;
        let player_pos = pos.get(self.player).unwrap().pos;

        for (state, pos, bod) in (&mut state, &mut pos, &mut rbs).iter() {
            dr = match state {
                BallState::InProgress {
                    from,
                    to,
                    time_left
                } => if ui_model.attack_down() {
                    let k = 1.0 - *time_left / BALL_THROW_TIME;
                    let dr = *to - *from;

                    *time_left -= dt.0;
                    let to = *to;
                    let new_pos = if *time_left <= 0.0 {
                        *state = BallState::Deployed;
                        to
                    } else {
                        *from + dr * k
                    };

                    Some(new_pos - pos.pos)
                } else {
                    let k = 1.0 - *time_left / BALL_THROW_TIME;

                    info!("rollbacktime: {}", k * BALL_PICK_TIME);

                    *state = BallState::RollingBack {
                        from: pos.pos,
                        total: k * BALL_PICK_TIME,
                        time_left: k * BALL_PICK_TIME,
                    };

                    None
                },
                BallState::RollingBack {
                    total,
                    from,
                    time_left,
                } => {
                    let k = 1.0 - *time_left / *total;
                    let dr = player_pos - *from;

                    *time_left -= dt.0;
                    let new_pos = *from + dr * k;

                    if *time_left <= 0.0 {
                        *state = BallState::InPocket;
                    }

                    Some(new_pos - pos.pos)
                },
                BallState::InPocket => if ui_model.attack_down() {
                    let (mx, my) = mouse_position();
                    bod.enabled = true;
                    *state = BallState::InProgress {
                        from: player_pos,
                        to: vec2(mx, my),
                        time_left: BALL_THROW_TIME,
                    };

                    None
                } else {
                    bod.enabled = false;
                    pos.pos = player_pos;

                    None
                },
                BallState::Deployed => {
                    *state = BallState::RollingBack {
                        from: pos.pos,
                        total: BALL_PICK_TIME,
                        time_left: BALL_PICK_TIME,
                    };

                    None
                },
            };
        }

        if let Some(dr) = dr {
            phys.move_kinematic(
                &mut rbs,
                self.weapon,
                dr,
                false,
            );
        }
    }

    pub fn update_enemy_internals(
        &mut self,
        mut rbs: ViewMut<PhysicsInfo>,
        mut enemy: ViewMut<EnemyState>,
        mut pos: ViewMut<Transform>,
        ball_state: View<BallState>,
    ) {
        let ball_pos = pos.get(self.weapon)
            .unwrap()
            .pos;
        let ball_state = ball_state.get(self.weapon)
            .unwrap();
        let ball_can_capture = matches!(ball_state, BallState::InProgress { .. });

        for (rb, enemy, pos) in (&mut rbs, &mut enemy, &mut pos).iter() {
            if !ball_can_capture && enemy.captured {
                enemy.captured = false;
            }

            rb.enabled = !enemy.captured;

            if enemy.captured {
                pos.pos = ball_pos;
            }
        }
    }

    pub fn player_controls(
        &mut self,
        mut phys: UniqueViewMut<PhysicsState>,
        mut rbs: ViewMut<PhysicsInfo>,
        dt: UniqueView<DeltaTime>,
        ui_model: UniqueView<UiModel>,
    ) {
        let mut dir = Vec2::ZERO;
        if ui_model.move_left() {
            dir += vec2(-1.0, 0.0);
        }
        if ui_model.move_up() {
            dir += vec2(0.0, -1.0);
        }
        if ui_model.move_right() {
            dir += vec2(1.0, 0.0);
        }
        if ui_model.move_down() {
            dir += vec2(0.0, 1.0);
        }

        phys.move_kinematic(
            &mut rbs,
            self.player,
            dir.normalize_or_zero() * dt.0 * PLAYER_SPEED,
            true,
        );
    }

    // Doesn't work because we end up doing 2 borrows
    // pub fn box_deleter(
    //     &mut self,
    //     mut stores: AllStoragesViewMut,
    // ) {
    //     let map = [
    //         (KeyCode::Key1, 0),
    //         (KeyCode::Key2, 1),
    //         (KeyCode::Key3, 2),
    //         (KeyCode::Key4, 3),
    //     ];

    //     for (key, idx) in map {
    //         if is_key_pressed(key) {
    //             stores.delete_entity(self.boxes[idx]);
    //         }
    //     }
    // }
}

method_as_system!(
    Game::update_enemy_internals as game_enemy_internals(
        this: Game,
        rbs: ViewMut<PhysicsInfo>,
        enemy: ViewMut<EnemyState>,
        pos: ViewMut<Transform>,
        ball_state: View<BallState>
    )
);

method_as_system!(
    Game::player_controls as game_player_controls(
        this: Game,
        phys: UniqueViewMut<PhysicsState>,
        rbs: ViewMut<PhysicsInfo>,
        dt: UniqueView<DeltaTime>,
        ui_model: UniqueView<UiModel>
    )
);

method_as_system!(
    Game::ball_logic as game_ball_logic(
        this: Game,
        pos: ViewMut<Transform>,
        state: ViewMut<BallState>,
        rbs: ViewMut<PhysicsInfo>,
        ui_model: UniqueView<UiModel>,
        phys: UniqueViewMut<PhysicsState>,
        dt: UniqueView<DeltaTime>
    )
);

// method_as_system!(
//     Game::box_deleter as game_box_deleter(
//         this: Game,
//         stores: AllStoragesViewMut
//     )
// );