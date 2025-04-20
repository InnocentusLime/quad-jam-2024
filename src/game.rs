use jam_macro::method_system;
use macroquad::prelude::*;
use rapier2d::prelude::InteractionGroups;
use shipyard::{EntityId, Get, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use crate::{inline_tilemap, physics::{groups, BeamTag, BodyTag, ColliderTy, ForceApplier, KinematicControl, OneSensorTag, PhysicsInfo, PhysicsState}, ui::UiModel, AppState, BallState, BoxTag, BruteTag, BulletTag, DamageTag, DeltaTime, EnemyState, Health, PlayerDamageSensorTag, PlayerDamageState, PlayerGunState, PlayerScore, PlayerTag, RayTag, RewardInfo, RewardState, TileStorage, TileType, Transform};

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

        let ty = **world.get::<&TileType>(tile).unwrap();

        match ty {
            TileType::Wall => world.add_component(
                tile,
                (
                    PhysicsInfo::new(
                        InteractionGroups {
                            memberships: groups::LEVEL,
                            filter: groups::LEVEL_INTERACT,
                        },
                        ColliderTy::Box { width: 32.0, height: 32.0, },
                        1.0,
                        true,
                    ),
                    BodyTag::Static,
                )
            ),
            TileType::Ground => (),
        }
    }

    world.add_entity(storage)
}

#[derive(Unique)]
pub struct Game {
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
            PhysicsInfo::new(
                InteractionGroups {
                    memberships: groups::NPCS,
                    filter: groups::NPCS_INTERACT,
                },
                ColliderTy::Circle { radius: 8.0 },
                5.0,
                true,
            ),
            BodyTag::Dynamic,
            ForceApplier { force: Vec2::ZERO },
            DamageTag,
        ));
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
            OneSensorTag::new(),
            PhysicsInfo::new(
                InteractionGroups {
                    memberships: groups::LEVEL,
                    filter: groups::PLAYER,
                },
                ColliderTy::Box { width: 16.0, height: 16.0 },
                1.0,
                true,
            ),
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
                BodyTag::Dynamic,
                PhysicsInfo::new(
                    InteractionGroups {
                        memberships: groups::LEVEL,
                        filter: groups::LEVEL_INTERACT,
                    },
                    ColliderTy::Box {
                        width: 32.0,
                        height: 32.0,
                    },
                    1.0,
                    true,
                ),
            ));

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
            BodyTag::Kinematic,
            KinematicControl::new(),
            PhysicsInfo::new(
                InteractionGroups {
                    memberships: groups::PLAYER,
                    filter: groups::PLAYER_INTERACT,
                },
                ColliderTy::Box {
                    width: 16.0,
                    height: 16.0,
                },
                1.0,
                true,
            ),
        ));

        // TODO: use a sensor with many collisions
        let player_damage_sensor = world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            PhysicsInfo::new(
                InteractionGroups {
                    memberships: groups::LEVEL,
                    filter: groups::NPCS,
                },
                ColliderTy::Box {
                    width: 16.0,
                    height: 16.0,
                },
                1.0,
                true,
            ),
            OneSensorTag::new(),
            PlayerDamageSensorTag,
        ));

        world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            RayTag {
                len: 10.0,
                life_left: 0.0,
            },
            BeamTag::new(InteractionGroups {
                memberships: groups::PROJECTILES,
                filter: groups::NPCS,
            }),
            PhysicsInfo::new(
                InteractionGroups {
                    memberships: groups::PROJECTILES,
                    filter: groups::PROJECTILES_INTERACT,
                },
                ColliderTy::Box { width: 0.0, height: PLAYER_RAY_WIDTH },
                0.0,
                true,
            ),
        ));

        // let brute_pos = [
        //     vec2(280.0, 240.0); 25
        // ];
        // for pos in brute_pos {
        //     Self::spawn_brute(pos, world);
        // }

        for x in 0..5 {
            for y in 0..5 {
                let pos = vec2(
                    x as f32 * 16.0 + 100.0,
                    y as f32 * 16.0 + 200.0,
                );

                Self::spawn_brute(pos, world);
            }
        }

        // for x in 0..5 {
        //     for y in 0..5 {
        //         let a = x as f32 + y as f32 * 2.0;
        //         let (dx, dy) = a.sin_cos();
        //         let pos = vec2(
        //             x as f32 * 8.0 * 4.0 + 100.0 + dx * 13.0,
        //             y as f32 * 3.0 + 200.0 + dy * 7.0,
        //         );

        //         Self::spawn_brute(pos, world);
        //     }
        // }

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
    pub fn player_sensor_pose(
        &mut self,
        mut tf: ViewMut<Transform>,
        sense_tag: View<PlayerDamageSensorTag>,
        player_tag: View<PlayerTag>,
    ) {
        let (&player_tf, _) = (&tf, &player_tag)
            .iter()
            .next()
            .unwrap();

        for (tf, _) in (&mut tf, &sense_tag).iter() {
            tf.pos = player_tf.pos;
        }
    }

    #[method_system]
    pub fn player_ammo_pickup(
        &mut self,
        mut player_amo: ViewMut<PlayerGunState>,
        mut bullet: ViewMut<BulletTag>,
        bul_sensor: View<OneSensorTag>,
    ) {
        for (bul, sens) in (&mut bullet, &bul_sensor).iter() {
            if bul.is_picked { continue; }

            let mut pl = (&mut player_amo).get(self.player)
                .unwrap();
            let Some(col) = sens.col else { continue; };

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
    pub fn player_ray_align(
        &mut self,
        mut tf: ViewMut<Transform>,
        mut ray_tag: ViewMut<RayTag>,
        ui_model: UniqueView<UiModel>,
    ) {
        let player_tf = *(&tf).get(self.player)
            .unwrap();
        let player_pos = player_tf.pos;
        let mpos = self.mouse_pos();
        let shootdir = mpos - player_pos;

        if shootdir.length() <= DISTANCE_EPS { return; }

        let rayang = shootdir.to_angle();
        for (tf, tag) in (&mut tf, &ray_tag).iter() {
            if tag.life_left > 0.0 { continue; }

            tf.pos = player_pos;
            tf.angle = rayang;
        }
    }

    #[method_system]
    pub fn player_shooting(
        &mut self,
        beam_tag: View<BeamTag>,
        mut tf: ViewMut<Transform>,
        mut player_amo: ViewMut<PlayerGunState>,
        mut ray_tag: ViewMut<RayTag>,
        mut enemy_state: ViewMut<EnemyState>,
        ui_model: UniqueView<UiModel>,
        mut score: UniqueViewMut<PlayerScore>,
        mut bullet: ViewMut<BulletTag>,
        pinfo: View<PhysicsInfo>,
    ) {
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
        let player_tf = *(&tf).get(self.player).unwrap();
        let mut raylen = 0.0;
        let player_pos = player_tf.pos;
        let mut shootdir = Vec2::ZERO;
        let mut amo = (&mut player_amo).get(self.player)
            .unwrap();

        if !ui_model.attack_down() { return; }

        if *amo != PlayerGunState::Full { return; }

        *amo = PlayerGunState::Empty;

        for (tf, ray_tag, beam_tag, pinfo) in (&tf, &mut ray_tag, &beam_tag, &pinfo).iter() {
            let ColliderTy::Box { width, .. } = pinfo.shape()
                else { continue; };

            shootdir = Vec2::from_angle(tf.angle);
            raylen = *width;
            ray_tag.len = raylen;
            ray_tag.life_left = PLAYER_RAY_LINGER;

            let cols = beam_tag.overlaps.as_slice();

            score.0 += (cols.len() as u32) * mul_table[cols.len().clamp(0, mul_table.len() - 1)];

            for col in cols {
                *(&mut enemy_state).get(*col).unwrap() = EnemyState::Stunned {
                    left: PLAYER_HIT_COOLDOWN,
                };
            }
        }

        for (pos, _) in (&mut tf, &mut bullet).iter() {
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
        dt: UniqueView<DeltaTime>,
    ) {
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
        pos: View<Transform>,
        state: View<EnemyState>,
        dt: UniqueView<DeltaTime>,
        mut force: ViewMut<ForceApplier>,
    ) {
        let player_pos = pos.get(self.player).unwrap().pos;

        for (enemy_tf, _, enemy_state, force) in (&pos, &brute_tag, &state, &mut force).iter() {
            if !matches!(enemy_state, EnemyState::Free) {
                continue;
            }

            for (fella_tf, _, fella_state) in (&pos, &brute_tag, &state).iter() {
                if !matches!(fella_state, EnemyState::Free) {
                    continue;
                }

                let dr = fella_tf.pos - enemy_tf.pos;

                force.force += dr * BRUTE_GROUP_FORCE;
            }

            let dr = player_pos - enemy_tf.pos;

            force.force += dr.normalize_or_zero() * BRUTE_CHASE_FORCE;
        }
    }

    #[method_system]
    pub fn player_damage(
        &mut self,
        pl_sense_tag: View<PlayerDamageSensorTag>,
        sense_tag: View<OneSensorTag>,
        mut player_dmg: ViewMut<PlayerDamageState>,
        mut health: ViewMut<Health>,
        enemy_state: View<EnemyState>,
    ) {
        let (mut player_dmg, mut player_health) = (&mut player_dmg, &mut health).get(self.player)
            .unwrap();
        let (sens, _) = (&sense_tag, &pl_sense_tag)
            .iter().next().unwrap();

        if sens.col.is_none() {
            return;
        }

        if matches!(&*player_dmg, PlayerDamageState::Cooldown(_)) {
            return;
        }

        info!("You got kicked");
        player_health.0 -= 1;
        *player_dmg = PlayerDamageState::Cooldown(PLAYER_HIT_COOLDOWN);
    }

    #[method_system]
    pub fn player_controls(
        &mut self,
        dt: UniqueView<DeltaTime>,
        ui_model: UniqueView<UiModel>,
        player: View<PlayerTag>,
        mut control: ViewMut<KinematicControl>,
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

        for (control, _) in (&mut control, &player).iter() {
            control.slide = true;
            control.dr = dir.normalize_or_zero() * dt.0 * PLAYER_SPEED;
        }
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