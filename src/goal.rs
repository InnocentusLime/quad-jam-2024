use lib_game::*;
use crate::components::*;
use macroquad::prelude::*;
use crate::game::Game;
use shipyard::{IntoIter, UniqueViewMut, View, World};

pub fn spawn_goal(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        GoalTag,
        OneSensorTag::new(
            ColliderTy::Box {
                width: 16.0,
                height: 16.0,
            },
            InteractionGroups {
                memberships: groups::ITEMS,
                filter: groups::PLAYER,
            },
        ),
    ));
}

pub fn check_goal(
    mut game: UniqueViewMut<Game>,
    goal: View<GoalTag>,
    sens: View<OneSensorTag>,
) {
    for (sens, _) in (&sens, &goal).iter() {
        if sens.col.is_none() {
            continue;
        }

        game.goal_achieved = true;
    }
}