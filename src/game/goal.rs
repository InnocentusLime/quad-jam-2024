use super::prelude::*;

pub fn spawn(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        GoalTag { achieved: false },
        col_query::Pickup::new_one(
            ColliderTy::Box {
                width: 16.0,
                height: 16.0,
            },
            PhysicsGroup {
                player: true,
                ..PhysicsGroup::empty()
            },
        ),
    ));
}

pub fn check(mut goal: ViewMut<GoalTag>, sens: View<col_query::Pickup>) {
    for (sens, goal) in (&sens, &mut goal).iter() {
        if !sens.has_collided() {
            continue;
        }

        goal.achieved = true;
    }
}
