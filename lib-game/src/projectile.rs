use hecs::{CommandBuffer, EntityBuilder, World};
use lib_col::Shape;
use macroquad::math::Vec2;

use crate::{
    KinematicControl, ProjectileData, Team, Transform, build_attack, col_group, col_query,
};

pub(crate) fn ai(dt: f32, world: &mut World) {
    for (_, (kinematic, data)) in world.query_mut::<(&mut KinematicControl, &ProjectileData)>() {
        kinematic.dr = data.dir * data.speed * dt;
    }
}

pub(crate) fn despawn_on_hit(world: &mut World, cmds: &mut CommandBuffer) {
    for (entity, (kinematic, attack)) in
        world.query_mut::<(&mut KinematicControl, &col_query::Damage)>()
    {
        if kinematic.collided || attack.has_collided() {
            cmds.despawn(entity);
        }
    }
}

pub fn build_projectile(
    builder: &mut EntityBuilder,
    pos: Vec2,
    shape: Shape,
    dir: Vec2,
    graze_value: f32,
    speed: f32,
) {
    build_attack(
        builder,
        Transform::from_pos(pos),
        Team::Enemy,
        shape,
        graze_value,
        col_group::PLAYER,
    );
    builder.add_bundle((
        KinematicControl::new_nonslide(col_group::LEVEL),
        ProjectileData { dir, speed },
    ));
}
