use super::prelude::*;

pub fn spawn(world: &mut World, pos: Vec2) {
    world.spawn((
        Transform::from_pos(pos),
        GoalTag { achieved: false },
        col_query::Pickup::new_one(
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
