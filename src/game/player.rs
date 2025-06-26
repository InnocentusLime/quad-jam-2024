use super::prelude::*;

pub const PLAYER_SPEED: f32 = 132.0;
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

pub fn spawn(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        PlayerTag,
        PlayerScore(0),
        Health(PLAYER_SPAWN_HEALTH),
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

pub fn controls(
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
