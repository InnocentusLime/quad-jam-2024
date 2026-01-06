use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2) {
    builder.add_bundle((
        Transform::from_pos(pos),
        GoalTag { achieved: false },
        col_query::Pickup::new(
            Shape::Rect {
                width: 16.0,
                height: 16.0,
            },
            col_group::PLAYER,
            col_group::NONE,
        ),
    ));
}

pub fn check(world: &mut World) {
    for (_, (sens, goal)) in world.query_mut::<(&col_query::Pickup, &mut GoalTag)>() {
        if !sens.has_collided() {
            continue;
        }

        goal.achieved = true;
    }
}
