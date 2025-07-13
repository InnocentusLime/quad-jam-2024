use super::prelude::*;

pub fn spawn(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        GoalTag { achieved: false },
        col_query::Pickup::new_one(
            Shape::Rect {
                width: 16.0,
                height: 16.0,
            },
            col_group::PLAYER,
        ),
    ));
}

pub fn check(world: &mut World) {
    for (sens, goal) in world.iter::<(&col_query::Pickup, &mut GoalTag)>().iter() {
        if !sens.has_collided() {
            continue;
        }

        goal.achieved = true;
    }
}
