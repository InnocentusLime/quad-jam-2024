use super::prelude::*;

const BULLET_SPEED: f32 = 32.0;
const BULLET_SHAPE: Shape = Shape::Rect {
    width: 16.0,
    height: 16.0,
};

pub fn init(builder: &mut EntityBuilder, pos: Vec2, look_angle: f32) {
    builder.add_bundle(AttackBundle::new(
        Transform::from_pos(pos),
        Team::Enemy,
        BULLET_SHAPE,
        2.0,
        col_group::PLAYER,
    ));
    builder.add_bundle(CharacterBundle {
        look: CharacterLook(look_angle),
        ..CharacterBundle::new_projectile(pos, BULLET_SHAPE)
    });
    builder.add(BulletTag);
}

pub fn update(dt: f32, world: &mut World, resources: &Resources, cmds: &mut CommandBuffer) {
    for_each_character::<(&BulletTag, &mut col_query::Damage)>(
        world,
        resources,
        |entity, mut character| {
            character.set_walk_step(dt * character.look_direction() * BULLET_SPEED);
            if character.data.1.has_collided() || character.collided() {
                cmds.despawn(entity);
            }
        },
    );
}
