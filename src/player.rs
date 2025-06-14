use crate::components::*;
use lib_game::*;
use macroquad::prelude::*;
use shipyard::{Get, IntoIter, View, ViewMut, World};

pub const PLAYER_SPEED: f32 = 132.0;
pub const PLAYER_RAY_LINGER: f32 = 2.0;
pub const PLAYER_RAY_WIDTH: f32 = 3.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 3;
pub const PLAYER_HIT_COOLDOWN: f32 = 2.0;
pub const PLAYER_SIZE: f32 = 16.0;

pub const PLAYER_GROUP: PhysicsGroup = PhysicsGroup {
    player: true,
    ..PhysicsGroup::empty()
};
pub const PLAYER_INTERACT: PhysicsGroup = PhysicsGroup {
    level: true,
    items: true,
    ..PhysicsGroup::empty()
};
pub const PLAYER_DMG_INTERACT: PhysicsGroup = PhysicsGroup {
    npcs: true,
    ..PhysicsGroup::empty()
};

pub fn spawn_player(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        PlayerTag,
        PlayerScore(0),
        Health(crate::player::PLAYER_SPAWN_HEALTH),
        PlayerDamageState::Hittable,
        KinematicControl::new(),
        BodyTag::new(
            PhysicsFilter(PLAYER_GROUP, PLAYER_INTERACT),
            ColliderTy::Box {
                width: PLAYER_SIZE,
                height: PLAYER_SIZE,
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
            PhysicsFilter(PLAYER_GROUP, PLAYER_DMG_INTERACT),
        ),
        PlayerDamageSensorTag,
    ));
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
    mut bullet: ViewMut<BulletTag>,
    bul_sensor: View<OneSensorTag>,
    player_tag: View<PlayerTag>,
) {
    let (player_id, _) = player_tag.iter().with_id().next().unwrap();
    for (bul, sens) in (&mut bullet, &bul_sensor).iter() {
        if !matches!(bul, BulletTag::Dropped) {
            continue;
        }

        let Some(col) = sens.col else {
            continue;
        };

        if col != player_id {
            continue;
        }

        *bul = BulletTag::PickedUp;
    }
}

pub fn player_damage(
    player_tag: View<PlayerTag>,
    pl_sense_tag: View<PlayerDamageSensorTag>,
    sense_tag: View<OneSensorTag>,
    mut player_dmg: ViewMut<PlayerDamageState>,
    mut health: ViewMut<Health>,
) {
    let (player_dmg, player_health, _) = (&mut player_dmg, &mut health, &player_tag)
        .iter()
        .next()
        .unwrap();
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

pub fn player_throw(
    input: &InputModel,
    mut bullet: ViewMut<BulletTag>,
    player_tag: View<PlayerTag>,
    mut tf: ViewMut<Transform>,
) {
    if !input.attack_down {
        return;
    }

    let (&player_tf, _) = (&tf, &player_tag).iter().next().unwrap();
    for (bullet, tf) in (&mut bullet, &mut tf).iter() {
        if !matches!(bullet, BulletTag::PickedUp) {
            continue;
        }

        let mpos = input.aim;
        let dir = (mpos - player_tf.pos).normalize_or_zero();
        if dir.length() < LENGTH_EPSILON {
            continue;
        }

        tf.pos = player_tf.pos;
        *bullet = BulletTag::Thrown { dir };
    }
}

pub fn bullet_parts(
    mut bullet: ViewMut<BulletTag>,
    bullet_hitter: ViewMut<BulletHitterTag>,
    bullet_wall_hitter: ViewMut<BulletWallHitterTag>,
    mut pos: ViewMut<Transform>,
) {
    let mut bullet_pos = Vec2::ZERO;
    for (_, pos) in (&mut bullet, &mut pos).iter() {
        bullet_pos = pos.pos;
    }

    for (pos, _) in (&mut pos, &bullet_hitter).iter() {
        pos.pos = bullet_pos;
    }

    for (pos, _) in (&mut pos, &bullet_wall_hitter).iter() {
        pos.pos = bullet_pos;
    }
}

pub fn thrown_damage(
    bullet: View<BulletTag>,
    bullet_hitter: ViewMut<BulletHitterTag>,
    sense_tag: View<OneSensorTag>,
    mut enemy_state: ViewMut<EnemyState>,
) {
    let bullet = bullet.iter().next().unwrap();
    if !matches!(bullet, BulletTag::Thrown { .. }) {
        return;
    }

    for (sens, _) in (&sense_tag, &bullet_hitter).iter() {
        let Some(hit) = sens.col else {
            continue;
        };

        *(&mut enemy_state).get(hit).unwrap() = EnemyState::Stunned {
            left: PLAYER_HIT_COOLDOWN,
        };
    }
}

pub fn thrown_logic(
    dt: f32,
    mut bullet: ViewMut<BulletTag>,
    bullet_wall_hitter: ViewMut<BulletWallHitterTag>,
    mut pos: ViewMut<Transform>,
    sense_tag: View<OneSensorTag>,
) {
    let hit_wall = (&bullet_wall_hitter, &sense_tag)
        .iter()
        .next()
        .unwrap()
        .1
        .col
        .is_some();

    for (bullet, pos) in (&mut bullet, &mut pos).iter() {
        let BulletTag::Thrown { dir } = *bullet else {
            continue;
        };

        if hit_wall {
            *bullet = BulletTag::Dropped;
            continue;
        }

        pos.pos += dir * dt * 636.0;
    }
}
