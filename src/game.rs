use jam_macro::method_system;
use macroquad::prelude::*;
use rapier2d::prelude::InteractionGroups;
use shipyard::{EntityId, Get, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use crate::{inline_tilemap, physics::{groups, physics_spawn, BodyKind, ColliderTy, PhysicsInfo, PhysicsState}, ui::UiModel, BallState, DeltaTime, EnemyState, MobType, TileStorage, TileType, Transform};

pub const PLAYER_SPEED: f32 = 128.0;
pub const BALL_THROW_TIME: f32 = 0.2;
pub const BALL_PICK_TIME: f32 = 0.5;
pub const MAX_BALL_DIST: f32 = 256.0;
pub const BALL_THROW_SPEED: f32 = 512.0;
pub const BALL_RETRACT_SPEED: f32 = 1024.0;
pub const DISTANCE_EPS: f32 = 0.01;

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
                    memberships: groups::LEVEL,
                    filter: groups::LEVEL_INTERACT,
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
                    memberships: groups::LEVEL,
                    filter: groups::LEVEL_INTERACT,
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
                memberships: groups::PLAYER,
                filter: groups::PLAYER_INTERACT,
            },
        );

        let weapon = world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            BallState::InPocket,
            MobType::BallOfHurt,
        ));
        physics_spawn(
            world,
            weapon,
            ColliderTy::Circle {
                radius: 4.0
            },
            BodyKind::Kinematic,
            InteractionGroups {
                memberships: groups::NPCS,
                filter: groups::NPCS_INTERACT,
            },
        );

        let brute = world.add_entity((
            Transform {
                pos: vec2(200.0, 80.0),
                angle: 0.0,
            },
            MobType::Brute,
            EnemyState::Free,
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
                memberships: groups::NPCS,
                filter: groups::NPCS_INTERACT | groups::PROJECTILES,
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

    #[method_system]
    pub fn ball_logic(
        &mut self,
        mut pos: ViewMut<Transform>,
        mut state: ViewMut<BallState>,
        mut rbs: ViewMut<PhysicsInfo>,
        mut enemy_state: ViewMut<EnemyState>,
        ui_model: UniqueView<UiModel>,
        mut phys: UniqueViewMut<PhysicsState>,
        dt: UniqueView<DeltaTime>,
    ) {
        let player_pos = pos.get(self.player).unwrap().pos;
        let mut upd_ent = None;

        for (state, pos, bod) in (&mut state, &mut pos, &mut rbs).iter() {
            bod.enabled = matches!(state, BallState::Throwing { .. });

            match state {
                BallState::InPocket => if ui_model.attack_down() {
                    let (mx, my) = mouse_position();
                    let mpos = vec2(mx, my);
                    let off = (mpos - player_pos).clamp_length(0.0, MAX_BALL_DIST);

                    *state = BallState::Throwing { to: player_pos + off };
                } else {
                    pos.pos = player_pos;
                },
                BallState::Throwing { to } => {
                    let this_pos = pos.pos;
                    let target_pos = *to;
                    let diff = target_pos - this_pos;
                    let step = (diff.normalize_or_zero() * BALL_THROW_SPEED * dt.0)
                        .clamp_length_max(diff.length());

                    if to.distance(pos.pos) < DISTANCE_EPS {
                        *state = BallState::Retracting;
                    }

                    if phys.move_kinematic(bod, step, false) {
                        *state = BallState::Retracting;
                    }

                    if let Some(enemy) = phys.any_collisions(
                        *pos,
                        InteractionGroups {
                            memberships: groups::PROJECTILES,
                            filter: groups::NPCS,
                        },
                        ColliderTy::Circle { radius: 16.0 },
                    ) {
                        upd_ent = Some((
                            enemy,
                            EnemyState::Captured
                        ));
                        *state = BallState::Capturing { enemy };
                    };
                },
                BallState::Retracting => {
                    let this_pos = pos.pos;
                    let target_pos = player_pos;
                    let diff = target_pos - this_pos;
                    let step = (diff.normalize_or_zero() * BALL_THROW_SPEED * dt.0)
                        .clamp_length_max(diff.length());

                    pos.pos += step;

                    if player_pos.distance(pos.pos) < DISTANCE_EPS {
                        *state = BallState::InPocket;
                    }
                },
                BallState::Capturing { enemy} => {
                    let this_pos = pos.pos;
                    let target_pos = player_pos;
                    let diff = target_pos - this_pos;
                    let step = (diff.normalize_or_zero() * BALL_THROW_SPEED * dt.0)
                        .clamp_length_max(diff.length());

                    pos.pos += step;

                    if player_pos.distance(pos.pos) < DISTANCE_EPS {
                        *state = BallState::Spinning { enemy: *enemy };
                    }
                },
                BallState::Spinning { enemy } => if ui_model.attack_down() {
                    pos.pos = player_pos +
                        Vec2::from_angle(get_time() as f32 * 5.0 * std::f32::consts::PI) * 32.0;
                } else {
                    let (mx, my) = mouse_position();
                    let mpos = vec2(mx, my);
                    let dir = (mpos - player_pos)
                        .normalize_or(vec2(0.0, 1.0));

                    upd_ent = Some((
                        *enemy,
                        EnemyState::Launched { dir },
                    ));
                    *state = BallState::InPocket;
                },
            }
        }

        match upd_ent {
            Some((enemy, EnemyState::Captured)) => {
                let (mut enemy_state, _) = (&mut enemy_state, &mut pos).get(enemy).unwrap();
                *enemy_state = EnemyState::Captured;
            },
            Some((enemy, EnemyState::Launched { dir })) => {
                let (mut enemy_state, mut pos) = (&mut enemy_state, &mut pos).get(enemy).unwrap();
                *enemy_state = EnemyState::Launched { dir };
                pos.pos = player_pos + dir * 16.0;
            },
            _ => (),
        }
    }

    #[method_system]
    pub fn captured_enemy(
        &mut self,
        mut rbs: ViewMut<PhysicsInfo>,
        mut enemy: ViewMut<EnemyState>,
        mut pos: ViewMut<Transform>,
        mut phys: UniqueViewMut<PhysicsState>,
        dt: UniqueView<DeltaTime>,
    ) {
        let ball_pos = pos.get(self.weapon)
            .unwrap()
            .pos;

        for (rb, enemy, pos) in (&mut rbs, &mut enemy, &mut pos).iter() {
            match enemy {
                EnemyState::Free => {
                    rb.enabled = true;
                },
                EnemyState::Captured => {
                    rb.enabled = false;
                    pos.pos = ball_pos;
                },
                EnemyState::Launched { dir } => {
                    rb.enabled = true;
                    phys.move_kinematic(rb, *dir * 256.0 * dt.0, false);
                },
            }
        }
    }

    #[method_system]
    pub fn brute_ai(
        &mut self,
        mob_ty: View<MobType>,
        mut phys: UniqueViewMut<PhysicsState>,
        rbs: View<PhysicsInfo>,
        pos: View<Transform>,
        state: View<EnemyState>,
        dt: UniqueView<DeltaTime>,
    ) {
        let player_pos = pos.get(self.player).unwrap().pos;

        for (enemy_tf, mob_ty, info, state) in (&pos, &mob_ty, &rbs, &state).iter() {
            if !matches!(mob_ty, MobType::Brute) {
                continue;
            }

            if !matches!(state, EnemyState::Free) {
                continue;
            }

            let dr = (player_pos - enemy_tf.pos).normalize_or_zero() * 32.0 * dt.0;
            phys.move_kinematic(
                info,
                dr,
                true,
            );
        }
    }

    #[method_system]
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

        let mut rb = (&mut rbs).get(self.player).unwrap();
        phys.move_kinematic(
            &mut rb,
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

// method_as_system!(
//     Game::box_deleter as game_box_deleter(
//         this: Game,
//         stores: AllStoragesViewMut
//     )
// );