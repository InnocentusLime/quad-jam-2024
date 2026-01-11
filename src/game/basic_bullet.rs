use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2, look_angle: f32, resources: &Resources) {
    builder.add_bundle(AttackBundle::new(
        Transform::from_pos(pos),
        Team::Enemy,
        resources.cfg.basic_bullet.shape,
        resources.cfg.basic_bullet.graze_value,
        col_group::PLAYER,
    ));
    builder.add_bundle(CharacterBundle {
        look: CharacterLook(look_angle),
        ..CharacterBundle::new_projectile(pos, resources.cfg.basic_bullet.shape)
    });
    builder.add(BulletTag);
}

pub fn update(dt: f32, world: &mut World, resources: &Resources, cmds: &mut CommandBuffer) {
    let cfg = &resources.cfg;
    for_each_character::<(&BulletTag, &mut col_query::Damage)>(
        world,
        resources,
        |entity, mut character| {
            character.set_walk_step(dt * character.look_direction() * cfg.basic_bullet.speed);
            if character.data.1.has_collided() || character.collided() {
                cmds.despawn(entity);
            }
        },
    );
}
