use hecs::{Bundle, CommandBuffer, World};
use lib_col::Shape;
use macroquad::math::Vec2;

use crate::{
    AttackBundle, BodyTag, GrazeValue, KinematicControl, ProjectileData, Team,
    Transform, col_group, col_query,
};

pub(crate) fn ai(dt: f32, world: &mut World) {
    for (_, (kinematic, data)) in world.query_mut::<(&mut KinematicControl, &ProjectileData)>() {
        kinematic.dr = data.dir * data.speed * dt;
    }
}

pub(crate) fn despawn_on_hit(world: &mut World, cmds: &mut CommandBuffer) {
    for (entity, (kinematic, attack)) in world.query_mut::<(&mut KinematicControl, &col_query::Damage)>() {
        if kinematic.collided || attack.has_collided() {
            cmds.despawn(entity);
        }
    }
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub kinematic: KinematicControl,
    pub data: ProjectileData,
}

impl ProjectileBundle {
    pub fn new_enemy(pos: Vec2, shape: Shape, dir: Vec2, graze_value: f32, speed: f32) -> (AttackBundle, Self) {
        let attack = AttackBundle {
            tf: Transform::from_pos(pos),
            team: Team::Enemy,
            query: col_query::Damage::new(shape, col_group::PLAYER, col_group::NONE),
            hitbox: BodyTag {
                groups: col_group::ATTACKS,
                shape,
            },
            graze_value: GrazeValue(graze_value),
        };
        let this = ProjectileBundle {
            kinematic: KinematicControl::new_nonslide(col_group::LEVEL),
            data: ProjectileData { dir, speed },
        };
        (attack, this)
    }
}
