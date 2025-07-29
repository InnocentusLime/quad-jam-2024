use super::prelude::*;

pub const PLAYER_SPEED: f32 = 132.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 3;
pub const PLAYER_HIT_COOLDOWN: f32 = 1.0;
pub const PLAYER_SIZE: f32 = 16.0;

pub fn spawn(world: &mut World, pos: Vec2) {
    world.spawn((
        Transform::from_pos(pos),
        PlayerTag::default(),
        PlayerScore(0),
        Health::new(PLAYER_SPAWN_HEALTH),
        DamageCooldown::new(PLAYER_HIT_COOLDOWN),
        KinematicControl::new(col_group::LEVEL),
        BodyTag {
            groups: col_group::PLAYER,
            shape: Shape::Rect {
                width: PLAYER_SIZE,
                height: PLAYER_SIZE,
            },
        },
    ));
}

pub fn controls(input: &InputModel, world: &mut World) {
    for (_, (tf, tag)) in world.query_mut::<(&Transform, &mut PlayerTag)>() {
        if input.attack_down {
            tag.action = PlayerAction::Attack;
            continue;
        }

        let mut do_walk = false;
        let mut walk_direction = Vec2::ZERO;
        if input.left_movement_down {
            walk_direction += vec2(-1.0, 0.0);
            do_walk = true;
        }
        if input.up_movement_down {
            walk_direction += vec2(0.0, -1.0);
            do_walk = true;
        }
        if input.right_movement_down {
            walk_direction += vec2(1.0, 0.0);
            do_walk = true;
        }
        if input.down_movement_down {
            walk_direction += vec2(0.0, 1.0);
            do_walk = true;
        }

        tag.action = PlayerAction::Move {
            look_direction: (input.aim - tf.pos).normalize_or(vec2(0.0, 1.0)),
            walk_direction: do_walk.then_some(walk_direction),
        };
    }
}

pub fn update(dt: f32, world: &mut World, cmds: &mut CommandBuffer) {
    use PlayerAction as Action;
    use PlayerState as State;

    // TODO: implement action queueing, because player
    // may press something on the last frame.
    for (_, (tf, tag)) in &mut world.query::<(&Transform, &mut PlayerTag)>() {
        let action = std::mem::replace(&mut tag.action, Action::None);
        let new_state = match action {
            Action::None => continue,
            Action::Move {
                look_direction,
                walk_direction,
            } => do_move(tag.state, look_direction, walk_direction),
            Action::Attack => do_attack(tag.state, *tf, world, cmds),
        };
        if let Some(new_state) = new_state {
            tag.state = new_state;
        }
    }

    for (_, (tag, control)) in world.query_mut::<(&mut PlayerTag, &mut KinematicControl)>() {
        control.dr = Vec2::ZERO;
        let new_state = match &mut tag.state {
            State::Idle { .. } => continue,
            State::Walking {
                walk_direction,
                look_direction,
            } => update_walking(dt, control, *walk_direction, *look_direction),
            State::Attacking {
                time_left,
                direction,
                attack_entity,
            } => update_attacking(dt, time_left, *direction, *attack_entity, cmds),
        };
        if let Some(new_state) = new_state {
            tag.state = new_state;
        }
    }
}

fn update_walking(
    dt: f32,
    control: &mut KinematicControl,
    walk_direction: Vec2,
    look_direction: Vec2,
) -> Option<PlayerState> {
    use PlayerState as State;

    control.dr = walk_direction * PLAYER_SPEED * dt;
    Some(State::Idle { look_direction })
}

fn update_attacking(
    dt: f32,
    time_left: &mut f32,
    direction: Vec2,
    attack_entity: Entity,
    cmds: &mut CommandBuffer,
) -> Option<PlayerState> {
    use PlayerState as State;

    if *time_left > 0.0f32 {
        *time_left -= dt;
        return None;
    }
    cmds.despawn(attack_entity);
    Some(State::Idle {
        look_direction: direction,
    })
}

fn do_move(
    state: PlayerState,
    look_direction: Vec2,
    walk_direction: Option<Vec2>,
) -> Option<PlayerState> {
    use PlayerState as State;

    // Do not let him walk if he is attacking
    match state {
        State::Attacking { .. } => return None,
        _ => (),
    };
    let new_state = match walk_direction {
        Some(walk_direction) => State::Walking {
            look_direction,
            walk_direction,
        },
        None => State::Idle { look_direction },
    };
    Some(new_state)
}

fn do_attack(
    state: PlayerState,
    tf: Transform,
    world: &World,
    cmds: &mut CommandBuffer,
) -> Option<PlayerState> {
    use PlayerState as State;

    // Do not let him attack if he is already attacking
    let attack_direction = match state {
        State::Attacking { .. } => return None,
        State::Idle { look_direction } | State::Walking { look_direction, .. } => look_direction,
    };
    let attack_entity = spawn_attack(tf, attack_direction, world, cmds);
    Some(State::Attacking {
        time_left: 0.5,
        direction: attack_direction,
        attack_entity,
    })
}

fn spawn_attack(
    tf: Transform,
    attack_direction: Vec2,
    world: &World,
    cmds: &mut CommandBuffer,
) -> Entity {
    let damage_entity = world.reserve_entity();
    let components = (
        Transform {
            pos: 32.0 * attack_direction + tf.pos,
            angle: attack_direction.to_angle(),
        },
        col_query::Damage {
            group: col_group::ENEMY,
            collision_list: CollisionList::many(5),
            collider: Shape::Rect {
                width: 64.0,
                height: 8.0,
            },
        },
        PlayerAttackTag,
    );
    cmds.insert(damage_entity, components);
    damage_entity
}
