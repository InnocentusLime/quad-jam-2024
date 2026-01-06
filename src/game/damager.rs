use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2) {
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
}
