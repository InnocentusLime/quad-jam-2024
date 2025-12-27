use super::prelude::*;

pub fn spawn(world: &mut World, pos: Vec2) {
    world.spawn((
        Transform::from_pos(pos),
        Team::Enemy,
        col_query::Damage::new(
            Shape::Rect {
                width: 16.0,
                height: 16.0,
            },
            col_group::CHARACTERS,
            col_group::NONE,
        ),
    ));
}
