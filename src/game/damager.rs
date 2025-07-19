use super::prelude::*;

pub fn spawn(world: &mut World, pos: Vec2) {
    world.spawn((
        Transform::from_pos(pos),
        col_query::Damage::new_one(
            Shape::Rect {
                width: 32.0,
                height: 32.0,
            },
            col_group::HITTABLE,
        ),
    ));
}
