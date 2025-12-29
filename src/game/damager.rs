use hecs::EntityBuilder;

use super::prelude::*;

pub fn spawn(world: &mut World, pos: Vec2) {
    let mut builder = EntityBuilder::new();
    builder.add_bundle(AttackBundle::new(
        Transform::from_pos(pos),
        Team::Enemy,
        Shape::Rect {
            width: 16.0,
            height: 16.0,
        },
        50.0,
        col_group::NONE,
    ));
    world.spawn(builder.build());
}
