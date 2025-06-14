use crate::components::*;
use lib_game::*;
use macroquad::prelude::*;
use shipyard::{IntoIter, View, ViewMut, World};

pub fn spawn_goal(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        GoalTag { achieved: false },
        OneSensorTag::new(
            ColliderTy::Box {
                width: 16.0,
                height: 16.0,
            },
            PhysicsFilter(
                PhysicsGroup {
                    items: true,
                    ..PhysicsGroup::empty()
                },
                PhysicsGroup {
                    player: true,
                    ..PhysicsGroup::empty()
                },
            ),
        ),
    ));
}

pub fn check_goal(mut goal: ViewMut<GoalTag>, sens: View<OneSensorTag>) {
    for (sens, goal) in (&sens, &mut goal).iter() {
        if sens.col.is_none() {
            continue;
        }

        goal.achieved = true;
    }
}
