use lib_game::*;
use crate::components::*;
use macroquad::prelude::*;
use crate::game::Game;
use shipyard::{EntityId, Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

pub const PLAYER_SPEED: f32 = 128.0;
pub const PLAYER_RAY_LINGER: f32 = 2.0;
pub const PLAYER_RAY_LEN_NUDGE: f32 = 8.0;
pub const PLAYER_RAY_WIDTH: f32 = 3.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 4;
pub const PLAYER_HIT_COOLDOWN: f32 = 2.0;

pub const DISTANCE_EPS: f32 = 0.01;

pub fn spawn_player(world: &mut World) -> EntityId {
    world.add_unique(PlayerScore(0));
    
    let player = world.add_entity((
        Transform {
            pos: vec2(300.0, 300.0),
            angle: 0.0,
        },
        PlayerTag,
        Health(crate::player::PLAYER_SPAWN_HEALTH),
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
            crate::player::PLAYER_RAY_WIDTH,
        ),
    ));

    player
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
    
pub fn player_ray_effect(
    this: UniqueView<Game>,
    mut know: UniqueViewMut<SwarmKnowledge>,
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

    for (ray_tf, ray_tag, beam_tag) in (&tf, &mut ray_tag, &beam_tag).iter() {
        if !ray_tag.shooting {
            return;
        }

        let shootdir = Vec2::from_angle(ray_tf.angle);
        let hitcount = beam_tag.overlaps.len();
        score.0 += (hitcount as u32) * mul_table[hitcount.clamp(0, mul_table.len() - 1)];

        let mut overall_dir = Vec2::ZERO; 
        for col in &beam_tag.overlaps {
            let (mut enemy_state, enemy_tf) = (&mut enemy_state, &tf).get(*col).unwrap();
            *enemy_state = EnemyState::Stunned {
                left: PLAYER_HIT_COOLDOWN,
            };
            let dir = (player_tf.pos - enemy_tf.pos).normalize_or_zero();
            overall_dir = (overall_dir + dir).normalize_or_zero();
        }
            
        know.last_hit.anger_time = 0.8;
        know.last_hit.anger_dir = overall_dir;
        off = shootdir * (beam_tag.length - 2.0 * PLAYER_RAY_LEN_NUDGE)
    }

    for (pos, _) in (&mut tf, &mut bullet).iter() {
        pos.pos = player_tf.pos + off;
    }
}