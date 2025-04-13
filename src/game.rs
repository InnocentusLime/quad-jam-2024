use jam_macro::method_system;
use macroquad::prelude::*;
use rapier2d::prelude::InteractionGroups;
use shipyard::{EntityId, Get, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use crate::{inline_tilemap, physics::{groups, physics_spawn, BodyKind, ColliderTy, PhysicsInfo, PhysicsState}, ui::UiModel, AppState, BallState, BoxTag, BruteTag, BulletTag, DeltaTime, EnemyState, Health, PlayerDamageState, PlayerGunState, PlayerScore, PlayerTag, RayTag, RewardInfo, RewardState, TileStorage, TileType, Transform};

pub const PLAYER_SPEED: f32 = 128.0;
pub const BALL_THROW_TIME: f32 = 0.2;
pub const BALL_PICK_TIME: f32 = 0.5;
pub const MAX_BALL_DIST: f32 = 256.0;
pub const BALL_THROW_SPEED: f32 = 512.0;
pub const BALL_RETRACT_SPEED: f32 = 1024.0;
pub const DISTANCE_EPS: f32 = 0.01;

pub const PLAYER_RAY_LINGER: f32 = 2.0;
pub const PLAYER_MAX_RAY_LEN: f32 = 360.0;
pub const PLAYER_RAY_LEN_NUDGE: f32 = 8.0;
pub const PLAYER_RAY_WIDTH: f32 = 3.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 4;
pub const PLAYER_HIT_COOLDOWN: f32 = 2.0;
pub const BRUTE_SPAWN_HEALTH: i32 = 3;

pub const BRUTE_GROUP_FORCE: f32 = 0.01 * 22.0;
pub const BRUTE_CHASE_FORCE: f32 = 40.0 * 24.0;

pub const REWARD_PER_ENEMY: u32 = 10;

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
                1.0,
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
    camera: Camera2D,
}

impl Game {
    fn spawn_brute(pos: Vec2, world: &mut World) {
        let brute = world.add_entity((
            Transform {
                pos,
                angle: 0.0,
            },
            RewardInfo {
                state: RewardState::Locked,
                amount: REWARD_PER_ENEMY,
            },
            BruteTag,
            EnemyState::Free,
            Health(BRUTE_SPAWN_HEALTH),
        ));
        physics_spawn(
            world,
            brute,
            ColliderTy::Circle { radius: 8.0 },
            BodyKind::Dynamic,
            InteractionGroups {
                memberships: groups::NPCS,
                filter: groups::NPCS_INTERACT,
            },
            5.0,
        );
    }

    fn spawn_bullet(pos: Vec2, world: &mut World) {
        world.add_entity((
            Transform {
                pos,
                angle: 0.0,
            },
            BulletTag {
                is_picked: false,
            },
        ));
    }

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
                BoxTag,
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
                1.0,
            );

            the_box
        });

        let player = world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            PlayerTag,
            Health(PLAYER_SPAWN_HEALTH),
            PlayerDamageState::Hittable,
            PlayerGunState::Empty,
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
            1.0,
        );

        world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            RayTag {
                len: 10.0,
                life_left: 0.0,
             },
        ));

        let weapon = world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            BallState::InPocket,
        ));
        physics_spawn(
            world,
            weapon,
            ColliderTy::Circle {
                radius: 4.0
            },
            BodyKind::Kinematic,
            InteractionGroups {
                memberships: groups::PROJECTILES,
                filter: groups::PROJECTILES_INTERACT,
            },
            1.0,
        );

        let brute_pos = [
            vec2(280.0, 240.0),
        ];
        for pos in brute_pos {
            Self::spawn_brute(pos, world);
        }

        for x in 0..5 {
            for y in 0..5 {
                let pos = vec2(
                    x as f32 * 16.0 + 100.0,
                    y as f32 * 16.0 + 200.0,
                );

                Self::spawn_brute(pos, world);
            }
        }

        Self::spawn_bullet(vec2(100.0, 100.0), world);

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
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w
            ],
            world,
        );

        world.add_unique(PlayerScore(0));

        Self {
            player,
            weapon,
            boxes,
            tilemap,
            camera: Camera2D::default(),
        }
    }

    pub fn mouse_pos(
        &self,
    ) -> Vec2 {
        let (mx, my) = mouse_position();
        self.camera.screen_to_world(vec2(mx, my))
    }

    #[method_system]
    pub fn update_camera(
        &mut self,
    ) {
        let view_height = 19.0 * 32.0;
        let view_width = (screen_width() / screen_height()) * view_height;
        self.camera = Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: view_width,
            h: view_height,
        });
        self.camera.zoom.y *= -1.0;
    }

    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    #[method_system]
    pub fn reset_amo_pickup(
        &mut self,
        mut bullet: ViewMut<BulletTag>,
        mut player_amo: ViewMut<PlayerGunState>,
    ) {
        let mut pl = (&mut player_amo).get(self.player)
            .unwrap();

        if *pl != PlayerGunState::Empty {
            return;
        }

        for bul in (&mut bullet).iter() {
            bul.is_picked = false;
        }
    }

    #[method_system]
    pub fn player_ammo_pickup(
        &mut self,
        mut phys: UniqueViewMut<PhysicsState>,
        mut player_amo: ViewMut<PlayerGunState>,
        pos: View<Transform>,
        mut bullet: ViewMut<BulletTag>,
    ) {
        for (bul, tf) in (&mut bullet, &pos).iter() {
            if bul.is_picked { continue; }

            let mut pl = (&mut player_amo).get(self.player)
                .unwrap();
            let Some(col) = phys.any_collisions(
                *tf,
                // FIXME: dirty hack
                InteractionGroups {
                    memberships: groups::LEVEL,
                    filter: groups::PLAYER,
                },
                ColliderTy::Box { width: 16.0, height: 16.0 },
                None,
            )
            else { continue; };

            if col != self.player { continue; }

            if *pl == PlayerGunState::Full { continue; }

            *pl = PlayerGunState::Full;
            bul.is_picked = true;
        }
    }

    #[method_system]
    pub fn ray_tick(
        &mut self,
        mut ray_tag: ViewMut<RayTag>,
        dt: UniqueView<DeltaTime>,
    ) {
        for ray in (&mut ray_tag).iter() {
            ray.life_left -= dt.0;
            ray.life_left = ray.life_left.max(0.0);
        }
    }

    #[method_system]
    pub fn player_shooting(
        &mut self,
        mut phys: UniqueViewMut<PhysicsState>,
        mut pos: ViewMut<Transform>,
        mut player_amo: ViewMut<PlayerGunState>,
        mut ray_tag: ViewMut<RayTag>,
        mut enemy_state: ViewMut<EnemyState>,
        ui_model: UniqueView<UiModel>,
        mut score: UniqueViewMut<PlayerScore>,
        mut bullet: ViewMut<BulletTag>,
    ) {
        if !ui_model.attack_down() { return; }

        let player_tf = *(&pos).get(self.player)
            .unwrap();
        let player_pos = player_tf.pos;
        let mpos = self.mouse_pos();
        let shootdir = mpos - player_pos;
        let mut amo = (&mut player_amo).get(self.player)
            .unwrap();

        if *amo != PlayerGunState::Full { return; }
        if shootdir.length() <= DISTANCE_EPS { return; }

        *amo = PlayerGunState::Empty;

        let cast = phys.cast_shape(
            player_tf,
            InteractionGroups {
                memberships: groups::PROJECTILES,
                filter: groups::PROJECTILES_INTERACT,
            },
            shootdir,
            ColliderTy::Box { width: PLAYER_RAY_WIDTH, height: PLAYER_RAY_WIDTH },
            None
        );
        let raylen = match cast {
            Some((_, len)) => len + PLAYER_RAY_LEN_NUDGE,
            None => PLAYER_MAX_RAY_LEN,
        };
        let rayang = shootdir.to_angle();

        for (ray_tag, pos) in (&mut ray_tag, &mut pos).iter() {
            ray_tag.len = raylen;
            ray_tag.life_left = PLAYER_RAY_LINGER;
            pos.pos = player_pos;
            pos.angle = rayang;
        }

        let shootdir = shootdir.normalize_or_zero();
        let cols = phys.all_collisions(
            Transform {
                pos: player_pos + shootdir * (raylen / 2.0),
                angle: rayang,
            },
            // FIXME: dirty hack to interact with NPCS
            InteractionGroups {
                memberships: groups::PROJECTILES,
                filter: groups::NPCS,
            },
            ColliderTy::Box {
                width: raylen,
                height: PLAYER_RAY_WIDTH,
            },
            None,
        );

        let mul_table = [
            0,
            1,
            1,
            1,
            2,
            2,
            2,
            10,
            10,
            20,
        ];

        // info!("cols: {}", cols.len());
        score.0 += (cols.len() as u32) * mul_table[cols.len().clamp(0, mul_table.len() - 1)];

        for col in cols {
            *(&mut enemy_state).get(col).unwrap() = EnemyState::Stunned {
                left: PLAYER_HIT_COOLDOWN,
            };
        }

        for (pos, _) in (&mut pos, &mut bullet).iter() {
            pos.pos = player_pos + shootdir * (raylen - 2.0 * PLAYER_RAY_LEN_NUDGE);
        }
    }

    // #[method_system]
    // pub fn ball_logic(
    //     &mut self,
    //     mut pos: ViewMut<Transform>,
    //     mut state: ViewMut<BallState>,
    //     mut rbs: ViewMut<PhysicsInfo>,
    //     mut enemy_state: ViewMut<EnemyState>,
    //     ui_model: UniqueView<UiModel>,
    //     mut phys: UniqueViewMut<PhysicsState>,
    //     dt: UniqueView<DeltaTime>,
    // ) {
    //     let player_pos = pos.get(self.player).unwrap().pos;
    //     let mut upd_ent = None;

    //     for (state, pos, bod) in (&mut state, &mut pos, &mut rbs).iter() {
    //         bod.enabled = matches!(state, BallState::Throwing { .. });

    //         match state {
    //             BallState::InPocket => if ui_model.attack_down() {
    //                 let (mx, my) = mouse_position();
    //                 let mpos = vec2(mx, my);
    //                 let off = (mpos - player_pos).clamp_length(0.0, MAX_BALL_DIST);

    //                 *state = BallState::Throwing { to: player_pos + off };
    //             } else {
    //                 pos.pos = player_pos;
    //             },
    //             BallState::Throwing { to } => {
    //                 let this_pos = pos.pos;
    //                 let target_pos = *to;
    //                 let diff = target_pos - this_pos;
    //                 let step = (diff.normalize_or_zero() * BALL_THROW_SPEED * dt.0)
    //                     .clamp_length_max(diff.length());

    //                 if to.distance(pos.pos) < DISTANCE_EPS {
    //                     *state = BallState::Retracting;
    //                 }

    //                 if phys.move_kinematic(bod, step, false) {
    //                     *state = BallState::Retracting;
    //                 }

    //                 if let Some(enemy) = phys.any_collisions(
    //                     *pos,
    //                     InteractionGroups {
    //                         memberships: groups::PROJECTILES,
    //                         filter: groups::NPCS,
    //                     },
    //                     ColliderTy::Circle { radius: 16.0 },
    //                     None,
    //                 ) {
    //                     upd_ent = Some((
    //                         enemy,
    //                         EnemyState::Captured
    //                     ));
    //                     *state = BallState::Capturing { enemy };
    //                 };
    //             },
    //             BallState::Retracting => {
    //                 let this_pos = pos.pos;
    //                 let target_pos = player_pos;
    //                 let diff = target_pos - this_pos;
    //                 let step = (diff.normalize_or_zero() * BALL_THROW_SPEED * dt.0)
    //                     .clamp_length_max(diff.length());

    //                 pos.pos += step;

    //                 if player_pos.distance(pos.pos) < DISTANCE_EPS {
    //                     *state = BallState::InPocket;
    //                 }
    //             },
    //             BallState::Capturing { enemy} => {
    //                 let this_pos = pos.pos;
    //                 let target_pos = player_pos;
    //                 let diff = target_pos - this_pos;
    //                 let step = (diff.normalize_or_zero() * BALL_THROW_SPEED * dt.0)
    //                     .clamp_length_max(diff.length());

    //                 pos.pos += step;

    //                 if player_pos.distance(pos.pos) < DISTANCE_EPS {
    //                     *state = BallState::Spinning { enemy: *enemy };
    //                 }
    //             },
    //             BallState::Spinning { enemy } => if ui_model.attack_down() {
    //                 pos.pos = player_pos +
    //                     Vec2::from_angle(get_time() as f32 * 5.0 * std::f32::consts::PI) * 32.0;
    //             } else {
    //                 let (mx, my) = mouse_position();
    //                 let mpos = vec2(mx, my);
    //                 let dir = (mpos - player_pos)
    //                     .normalize_or(vec2(0.0, 1.0));

    //                 upd_ent = Some((
    //                     *enemy,
    //                     EnemyState::Launched { dir, by_player: true },
    //                 ));
    //                 *state = BallState::InPocket;
    //             },
    //         }
    //     }

    //     match upd_ent {
    //         Some((enemy, EnemyState::Captured)) => {
    //             let (mut enemy_state, _) = (&mut enemy_state, &mut pos).get(enemy).unwrap();
    //             *enemy_state = EnemyState::Captured;
    //         },
    //         Some((enemy, EnemyState::Launched { dir, by_player })) => {
    //             let (mut enemy_state, mut pos) = (&mut enemy_state, &mut pos).get(enemy).unwrap();
    //             *enemy_state = EnemyState::Launched { dir, by_player };
    //             pos.pos = player_pos + dir * 16.0;
    //         },
    //         _ => (),
    //     }
    // }

    #[method_system]
    pub fn enemy_states(
        &mut self,
        rbs: View<PhysicsInfo>,
        mut enemy: ViewMut<EnemyState>,
        pos: View<Transform>,
        mut phys: UniqueViewMut<PhysicsState>,
        mut hp: ViewMut<Health>,
        dt: UniqueView<DeltaTime>,
    ) {
        for (rb, enemy, pos, hp) in (&rbs, &mut enemy, &pos, &mut hp).iter() {
            match enemy {
                EnemyState::Stunned { left } => {
                    *left -= dt.0;
                    if *left < 0.0 {
                        hp.0 -= 1;
                        *enemy = EnemyState::Free;
                    }
                },
                EnemyState::Free => {
                    if hp.0 <= 0 { *enemy = EnemyState::Dead; }
                },
                _ => (),
            }
        }
    }

    #[method_system]
    pub fn enemy_state_data(
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
                    rb.groups = InteractionGroups {
                        memberships: groups::NPCS,
                        filter: groups::NPCS_INTERACT,
                    };
                },
                EnemyState::Captured => {
                    rb.enabled = false;
                    rb.groups = InteractionGroups {
                        memberships: groups::NPCS,
                        filter: groups::NPCS_INTERACT,
                    };
                    pos.pos = ball_pos;
                },
                EnemyState::Stunned { .. } => {
                    rb.enabled = false;
                    rb.groups = InteractionGroups {
                        memberships: groups::NPCS,
                        filter: groups::NPCS_INTERACT,
                    };
                },
                EnemyState::Dead => {
                    rb.enabled = false;
                    rb.groups = InteractionGroups {
                        memberships: groups::NPCS,
                        filter: groups::NPCS_INTERACT,
                    };
                },
            }
        }
    }

    #[method_system]
    pub fn brute_ai(
        &mut self,
        brute_tag: View<BruteTag>,
        mut phys: UniqueViewMut<PhysicsState>,
        rbs: View<PhysicsInfo>,
        pos: View<Transform>,
        state: View<EnemyState>,
        dt: UniqueView<DeltaTime>,
    ) {
        let player_pos = pos.get(self.player).unwrap().pos;

        for (enemy_tf, _, enemy_info, enemy_state) in (&pos, &brute_tag, &rbs, &state).iter() {
            if !matches!(enemy_state, EnemyState::Free) {
                continue;
            }

            for (fella_tf, _, fella_state) in (&pos, &brute_tag, &state).iter() {
                if !matches!(fella_state, EnemyState::Free) {
                    continue;
                }

                let dr = fella_tf.pos - enemy_tf.pos;

                phys.apply_force(enemy_info, dr * BRUTE_GROUP_FORCE);
            }

            let dr = player_pos - enemy_tf.pos;

            phys.apply_force(enemy_info, dr.normalize_or_zero() * BRUTE_CHASE_FORCE);
        }

        // for (enemy_tf, _, info, state) in (&pos, &brute_tag, &rbs, &state).iter() {
        //     if !matches!(state, EnemyState::Free) {
        //         continue;
        //     }

        //     let dr = (player_pos - enemy_tf.pos).normalize_or_zero() * 32.0 * dt.0;
        //     phys.move_kinematic(
        //         info,
        //         dr,
        //         true,
        //     );
        // }
    }

    #[method_system]
    pub fn brute_damage(
        &mut self,
        mut phys: UniqueViewMut<PhysicsState>,
        mut rbs: ViewMut<PhysicsInfo>,
        brute: View<BruteTag>,
        mut player_dmg: ViewMut<PlayerDamageState>,
        mut health: ViewMut<Health>,
        state: View<EnemyState>,
        pos: View<Transform>,
    ) {
        let (mut player_dmg, mut player_health) = (&mut player_dmg, &mut health).get(self.player)
            .unwrap();

        for (_, brute_rb, brute_pos, brute_state) in (&brute, &rbs, &pos, &state).iter() {
            if matches!(&*player_dmg, PlayerDamageState::Cooldown(_)) { return; }
            if player_health.0 <= 0 { return; }

            if !matches!(brute_state, EnemyState::Free) { continue; }
            let Some(collision) = phys.any_collisions(
                *brute_pos,
                InteractionGroups {
                    // FIXME: a very dirty hack to have our cast collide with the player
                    memberships: groups::LEVEL,
                    filter: groups::PLAYER
                },
                *brute_rb.col(),
                Some(brute_rb),
            )
            else { continue; };

            if collision != self.player { continue; }

            info!("You got kicked");
            player_health.0 -= 1;
            *player_dmg = PlayerDamageState::Cooldown(PLAYER_HIT_COOLDOWN);
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
        // FIXME: the game now ticks at the fixed rate. Here (and in many other instances)
        // dt.0 is no longer appropiate.
        phys.move_kinematic(
            &mut rb,
            // dir.normalize_or_zero() * dt.0 * PLAYER_SPEED,
            dir.normalize_or_zero() * (1.0 / 60.0) * PLAYER_SPEED,
            true,
        );
    }

    #[method_system]
    pub fn reward_enemies(
        &mut self,
        enemy: View<EnemyState>,
        mut reward: ViewMut<RewardInfo>,
    ) {
        for (state, reward) in (&enemy, &mut reward).iter() {
            if !matches!((state, reward.state), (EnemyState::Dead, RewardState::Locked)) { continue; }

            reward.state = RewardState::Pending;
        }
    }

    #[method_system]
    pub fn count_rewards(
        &mut self,
        mut reward: ViewMut<RewardInfo>,
        mut score: UniqueViewMut<PlayerScore>,
    ) {
        for reward in (&mut reward).iter() {
            if !matches!(reward.state, RewardState::Pending) { continue; }

            reward.state = RewardState::Counted;
            score.0 += reward.amount;
        }
    }

    #[method_system]
    pub fn player_damage_state(
        &mut self,
        mut player_dmg: ViewMut<PlayerDamageState>,
        dt: UniqueView<DeltaTime>,
    ) {
        for player_dmg in (&mut player_dmg).iter() {
            let PlayerDamageState::Cooldown(time) = player_dmg
                else { continue; };

            *time -= dt.0;
            if *time > 0.0 { continue; }

            *player_dmg = PlayerDamageState::Hittable;
        }
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

pub fn decide_next_state(
    player: View<PlayerTag>,
    health: View<Health>,
    enemy_state: View<EnemyState>,
) -> Option<AppState> {
    let player_dead = (&player, &health).iter()
        .all(|(_, hp)| hp.0 <= 0);
    let enemies_dead = enemy_state.iter()
        .all(|state| matches!(state, EnemyState::Dead));

    if player_dead { return Some(AppState::GameOver); }

    if enemies_dead { return Some(AppState::Win); }

    None
}