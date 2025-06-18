use crate::components::*;
use lib_game::*;
use macroquad::prelude::*;
use shipyard::{IntoIter, View, ViewMut, World};

pub const PLAYER_SPEED: f32 = 132.0;
pub const PLAYER_RAY_LINGER: f32 = 2.0;
pub const PLAYER_RAY_WIDTH: f32 = 3.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 3;
#[allow(dead_code)]
pub const PLAYER_HIT_COOLDOWN: f32 = 2.0;
pub const PLAYER_SIZE: f32 = 16.0;

pub const PLAYER_GROUP: PhysicsGroup = PhysicsGroup {
    player: true,
    ..PhysicsGroup::empty()
};
pub const PLAYER_INTERACT: PhysicsGroup = PhysicsGroup {
    level: true,
    items: true,
    ..PhysicsGroup::empty()
};

pub fn spawn_player(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        PlayerTag,
        PlayerScore(0),
        Health(crate::player::PLAYER_SPAWN_HEALTH),
        KinematicControl::new(),
        BodyTag::new(
            PhysicsFilter(PLAYER_GROUP, PLAYER_INTERACT),
            ColliderTy::Box {
                width: PLAYER_SIZE,
                height: PLAYER_SIZE,
            },
            1.0,
            true,
            BodyKind::Kinematic,
        ),
    ));
}

pub fn player_controls(
    (input, dt): (&InputModel, f32),
    player: View<PlayerTag>,
    mut control: ViewMut<KinematicControl>,
) {
    let mut dir = Vec2::ZERO;
    if input.left_movement_down {
        dir += vec2(-1.0, 0.0);
    }
    if input.up_movement_down {
        dir += vec2(0.0, -1.0);
    }
    if input.right_movement_down {
        dir += vec2(1.0, 0.0);
    }
    if input.down_movement_down {
        dir += vec2(0.0, 1.0);
    }

    for (control, _) in (&mut control, &player).iter() {
        control.slide = true;
        control.dr = dir.normalize_or_zero() * dt * PLAYER_SPEED;
    }
}
