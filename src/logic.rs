use lib_game::*;
use macroquad::prelude::*;
use shipyard::{EntityId, Get, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};

use crate::inline_tilemap;

use crate::components::*;

pub const PLAYER_SPEED: f32 = 128.0;
pub const DISTANCE_EPS: f32 = 0.01;

pub const PLAYER_RAY_LINGER: f32 = 2.0;
pub const PLAYER_RAY_LEN_NUDGE: f32 = 8.0;
pub const PLAYER_RAY_WIDTH: f32 = 3.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 4;
pub const PLAYER_HIT_COOLDOWN: f32 = 2.0;
pub const BRUTE_SPAWN_HEALTH: i32 = 2;

pub const BRUTE_GROUP_FORCE: f32 = 0.01 * 22.0;
pub const BRUTE_CHASE_FORCE: f32 = 40.0 * 24.0;

pub const REWARD_PER_ENEMY: u32 = 10;

fn spawn_tiles(width: usize, height: usize, data: Vec<TileType>, world: &mut World) -> EntityId {
    assert_eq!(data.len(), width * height);

    let storage = TileStorage::from_data(
        width,
        height,
        data.into_iter().map(|ty| world.add_entity(ty)).collect(),
    )
    .unwrap();

    for (x, y, tile) in storage.iter_poses() {
        world.add_component(
            tile,
            Transform {
                pos: vec2(x as f32 * 32.0 + 16.0, y as f32 * 32.0 + 16.0),
                angle: 0.0,
            },
        );

        let ty = **world.get::<&TileType>(tile).unwrap();

        match ty {
            TileType::Wall => world.add_component(
                tile,
                (BodyTag::new(
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
                    BodyKind::Static,
                ),),
            ),
            TileType::Ground => (),
        }
    }

    world.add_entity(storage)
}

#[derive(Unique)]
pub struct Game {
    player: EntityId,
    _boxes: [EntityId; 4],
    _tilemap: EntityId,
    camera: Camera2D,
}

impl Game {
    fn spawn_brute(pos: Vec2, world: &mut World) {
        let _brute = world.add_entity((
            Transform { pos, angle: 0.0 },
            RewardInfo {
                state: RewardState::Locked,
                amount: REWARD_PER_ENEMY,
            },
            BruteTag,
            EnemyState::Free,
            Health(BRUTE_SPAWN_HEALTH),
            BodyTag::new(
                InteractionGroups {
                    memberships: groups::NPCS,
                    filter: groups::NPCS_INTERACT,
                },
                ColliderTy::Circle { radius: 6.0 },
                5.0,
                true,
                BodyKind::Dynamic,
            ),
            ForceApplier { force: Vec2::ZERO },
            DamageTag,
        ));
    }

    fn spawn_bullet(pos: Vec2, world: &mut World) {
        world.add_entity((
            Transform { pos, angle: 0.0 },
            BulletTag { is_picked: false },
            OneSensorTag::new(
                ColliderTy::Box {
                    width: 16.0,
                    height: 16.0,
                },
                InteractionGroups {
                    memberships: groups::LEVEL,
                    filter: groups::PLAYER,
                },
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
                Transform { pos, angle },
                BoxTag,
                BodyTag::new(
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
                    BodyKind::Dynamic,
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
            KinematicControl::new(),
            BodyTag::new(
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
                BodyKind::Kinematic,
            ),
        ));

        let _player_damage_sensor = world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            OneSensorTag::new(
                ColliderTy::Box {
                    width: 16.0,
                    height: 16.0,
                },
                InteractionGroups {
                    memberships: groups::LEVEL,
                    filter: groups::NPCS,
                },
            ),
            PlayerDamageSensorTag,
        ));

        world.add_entity((
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            },
            RayTag { shooting: false },
            BeamTag::new(
                InteractionGroups {
                    memberships: groups::PROJECTILES,
                    filter: groups::NPCS,
                },
                InteractionGroups {
                    memberships: groups::PROJECTILES,
                    filter: groups::PROJECTILES_INTERACT,
                },
                PLAYER_RAY_WIDTH,
            ),
        ));

        for x in 0..8 {
            for y in 0..8 {
                let pos = vec2(x as f32 * 12.0 + 100.0, y as f32 * 12.0 + 200.0);

                Self::spawn_brute(pos, world);
            }
        }

        /* Uncomment for a tough topology */
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
                w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, g, g, g, g, g, g, g, g, g, g, g,
                g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g,
                g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g,
                g, g, w, w, w, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g,
                g, w, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, w,
                g, g, w, g, g, g, g, w, w, g, g, w, g, g, g, g, g, g, w, g, g, g, g, w, w, g, g, g,
                g, g, g, g, g, g, w, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, w, w, w, w, w, w, w, w, w, w, w,
                w, w, w, w
            ],
            world,
        );

        world.add_unique(PlayerScore(0));

        Self {
            player,
            _boxes: boxes,
            _tilemap: tilemap,
            camera: Camera2D::default(),
        }
    }

    pub fn mouse_pos(&self) -> Vec2 {
        let (mx, my) = mouse_position();
        self.camera.screen_to_world(vec2(mx, my))
    }

    pub fn update_camera(mut this: UniqueViewMut<Game>, tile_storage: View<TileStorage>) {
        let view_height = 19.0 * 32.0;
        let view_width = (screen_width() / screen_height()) * view_height;
        this.camera = Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: view_width,
            h: view_height,
        });
        this.camera.zoom.y *= -1.0;

        // FIXME: macroquad's camera is super confusing. Just like this math
        for storage in tile_storage.iter() {
            this.camera.target = vec2(
                (0.5 * 32.0) * (storage.width() as f32),
                (0.5 * 32.0) * (storage.height() as f32),
            );
        }
    }

    pub fn camera(&self) -> &Camera2D {
        &self.camera
    }

    pub fn reset_amo_pickup(
        this: UniqueView<Game>,
        mut bullet: ViewMut<BulletTag>,
        player_amo: View<PlayerGunState>,
    ) {
        let pl = (&player_amo).get(this.player).unwrap();

        if *pl != PlayerGunState::Empty {
            return;
        }

        for bul in (&mut bullet).iter() {
            bul.is_picked = false;
        }
    }

    pub fn player_sensor_pose(
        mut tf: ViewMut<Transform>,
        sense_tag: View<PlayerDamageSensorTag>,
        player_tag: View<PlayerTag>,
    ) {
        let (&player_tf, _) = (&tf, &player_tag).iter().next().unwrap();

        for (tf, _) in (&mut tf, &sense_tag).iter() {
            tf.pos = player_tf.pos;
        }
    }

    pub fn player_ammo_pickup(
        this: UniqueView<Game>,
        mut player_amo: ViewMut<PlayerGunState>,
        mut bullet: ViewMut<BulletTag>,
        bul_sensor: View<OneSensorTag>,
    ) {
        for (bul, sens) in (&mut bullet, &bul_sensor).iter() {
            if bul.is_picked {
                continue;
            }

            let mut pl = (&mut player_amo).get(this.player).unwrap();
            let Some(col) = sens.col else {
                continue;
            };

            if col != this.player {
                continue;
            }

            if *pl == PlayerGunState::Full {
                continue;
            }

            *pl = PlayerGunState::Full;
            bul.is_picked = true;
        }
    }

    pub fn player_ray_controls(
        input: &InputModel,
        this: UniqueView<Game>,
        mut tf: ViewMut<Transform>,
        mut ray_tag: ViewMut<RayTag>,
        mut player_amo: ViewMut<PlayerGunState>,
    ) {
        let player_tf = *(&tf).get(this.player).unwrap();
        let player_pos = player_tf.pos;
        let mut amo = (&mut player_amo).get(this.player).unwrap();
        let mpos = this.mouse_pos();
        let shootdir = mpos - player_pos;

        if shootdir.length() <= DISTANCE_EPS {
            return;
        }

        for (tf, tag) in (&mut tf, &mut ray_tag).iter() {
            tag.shooting = false;
            tf.pos = player_pos;
            tf.angle = shootdir.to_angle();

            if input.attack_down && *amo == PlayerGunState::Full {
                tag.shooting = true;
                *amo = PlayerGunState::Empty;
            }
        }
    }

    pub fn player_ray_effect(
        this: UniqueView<Game>,
        beam_tag: View<BeamTag>,
        mut tf: ViewMut<Transform>,
        mut ray_tag: ViewMut<RayTag>,
        mut enemy_state: ViewMut<EnemyState>,
        mut score: UniqueViewMut<PlayerScore>,
        mut bullet: ViewMut<BulletTag>,
    ) {
        let player_tf = *(&tf).get(this.player).unwrap();
        let mul_table = [0, 1, 1, 1, 2, 2, 2, 10, 10, 20];
        let mut off = Vec2::ZERO;

        for (tf, ray_tag, beam_tag) in (&tf, &mut ray_tag, &beam_tag).iter() {
            if !ray_tag.shooting {
                return;
            }

            let shootdir = Vec2::from_angle(tf.angle);
            let hitcount = beam_tag.overlaps.len();
            score.0 += (hitcount as u32) * mul_table[hitcount.clamp(0, mul_table.len() - 1)];

            for col in &beam_tag.overlaps {
                *(&mut enemy_state).get(*col).unwrap() = EnemyState::Stunned {
                    left: PLAYER_HIT_COOLDOWN,
                };
            }

            off = shootdir * (beam_tag.length - 2.0 * PLAYER_RAY_LEN_NUDGE)
        }

        for (pos, _) in (&mut tf, &mut bullet).iter() {
            pos.pos = player_tf.pos + off;
        }
    }

    pub fn enemy_states(dt: f32, mut enemy: ViewMut<EnemyState>, mut hp: ViewMut<Health>) {
        for (enemy, hp) in (&mut enemy, &mut hp).iter() {
            match enemy {
                EnemyState::Stunned { left } => {
                    *left -= dt;
                    if *left < 0.0 {
                        hp.0 -= 1;
                        *enemy = EnemyState::Free;
                    }
                }
                EnemyState::Free => {
                    if hp.0 <= 0 {
                        *enemy = EnemyState::Dead;
                    }
                }
                _ => (),
            }
        }
    }

    pub fn enemy_state_data(mut rbs: ViewMut<BodyTag>, mut enemy: ViewMut<EnemyState>) {
        for (rb, enemy) in (&mut rbs, &mut enemy).iter() {
            match enemy {
                EnemyState::Free => {
                    rb.enabled = true;
                    rb.groups = InteractionGroups {
                        memberships: groups::NPCS,
                        filter: groups::NPCS_INTERACT,
                    };
                }
                EnemyState::Stunned { .. } => {
                    rb.enabled = true;
                    rb.groups = InteractionGroups {
                        memberships: groups::NPCS,
                        filter: groups::NPCS_INTERACT,
                    };
                }
                EnemyState::Dead => {
                    rb.enabled = false;
                    rb.groups = InteractionGroups {
                        memberships: groups::NPCS,
                        filter: groups::NPCS_INTERACT,
                    };
                }
            }
        }
    }

    pub fn brute_ai(
        this: UniqueView<Game>,
        brute_tag: View<BruteTag>,
        pos: View<Transform>,
        state: View<EnemyState>,
        mut force: ViewMut<ForceApplier>,
    ) {
        let player_pos = pos.get(this.player).unwrap().pos;

        for (enemy_tf, _, enemy_state, force) in (&pos, &brute_tag, &state, &mut force).iter() {
            if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
                continue;
            }

            for (fella_tf, _, fella_state) in (&pos, &brute_tag, &state).iter() {
                if !matches!(fella_state, EnemyState::Free | EnemyState::Stunned { .. }) {
                    continue;
                }

                let dr = fella_tf.pos - enemy_tf.pos;

                force.force += dr * BRUTE_GROUP_FORCE;
            }

            let dr = player_pos - enemy_tf.pos;

            force.force += dr.normalize_or_zero() * BRUTE_CHASE_FORCE;
        }
    }

    pub fn player_damage(
        this: UniqueView<Game>,
        pl_sense_tag: View<PlayerDamageSensorTag>,
        sense_tag: View<OneSensorTag>,
        mut player_dmg: ViewMut<PlayerDamageState>,
        mut health: ViewMut<Health>,
    ) {
        let (mut player_dmg, mut player_health) =
            (&mut player_dmg, &mut health).get(this.player).unwrap();
        let (sens, _) = (&sense_tag, &pl_sense_tag).iter().next().unwrap();

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

    pub fn player_controls(
        (input, dt): (&InputModel, f32),
        player: View<PlayerTag>,
        mut control: ViewMut<KinematicControl>,
    ) {
        let mut dir = Vec2::ZERO;
        if input.left_movement_down {
            dir += vec2(-1.0, 0.0);
        }
        if input.up_movement_down {
            dir += vec2(0.0, -1.0);
        }
        if input.right_movement_down {
            dir += vec2(1.0, 0.0);
        }
        if input.down_movement_down {
            dir += vec2(0.0, 1.0);
        }

        for (control, _) in (&mut control, &player).iter() {
            control.slide = true;
            control.dr = dir.normalize_or_zero() * dt * PLAYER_SPEED;
        }
    }

    pub fn reward_enemies(enemy: View<EnemyState>, mut reward: ViewMut<RewardInfo>) {
        for (state, reward) in (&enemy, &mut reward).iter() {
            if !matches!(
                (state, reward.state),
                (EnemyState::Dead, RewardState::Locked)
            ) {
                continue;
            }

            reward.state = RewardState::Pending;
        }
    }

    pub fn count_rewards(mut reward: ViewMut<RewardInfo>, mut score: UniqueViewMut<PlayerScore>) {
        for reward in (&mut reward).iter() {
            if !matches!(reward.state, RewardState::Pending) {
                continue;
            }

            reward.state = RewardState::Counted;
            score.0 += reward.amount;
        }
    }

    pub fn player_damage_state(dt: f32, mut player_dmg: ViewMut<PlayerDamageState>) {
        for player_dmg in (&mut player_dmg).iter() {
            let PlayerDamageState::Cooldown(time) = player_dmg else {
                continue;
            };

            *time -= dt;
            if *time > 0.0 {
                continue;
            }

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
    let player_dead = (&player, &health).iter().all(|(_, hp)| hp.0 <= 0);
    let enemies_dead = enemy_state
        .iter()
        .all(|state| matches!(state, EnemyState::Dead));

    if player_dead {
        return Some(AppState::GameOver);
    }

    if enemies_dead {
        return Some(AppState::Win);
    }

    None
}
