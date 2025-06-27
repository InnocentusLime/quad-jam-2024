use super::prelude::*;

pub fn spawn(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        GoalTag { achieved: false },
        OneSensorTag::new(
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

pub fn check(mut goal: ViewMut<GoalTag>, sens: View<OneSensorTag>) {
    for (sens, goal) in (&sens, &mut goal).iter() {
        if sens.col.is_none() {
            continue;
        }

        goal.achieved = true;
    }
}
