use hashbrown::HashMap;
use lib_anim::{Animation, AnimationId};

use super::prelude::*;

pub const STABBER_SIZE: f32 = 16.0;
pub const STABBER_SPAWN_HEALTH: i32 = 3;
pub const STABBER_HIT_COOLDOWN: f32 = 3.0;
pub const STABBER_WALK_SPEED: f32 = 18.0;
pub const STABBER_AGRO_RANGE: f32 = 36.0;

struct StabberContext<'a> {
    kinematic: &'a mut KinematicControl,
    play: &'a mut AnimationPlay,
    animation: &'a Animation,
    state: &'a mut StabberState,
    look: &'a mut CharacterLook,
}

impl<'a> StabberContext<'a> {
    fn set_state(&mut self, new_state: StabberState) {
        *self.state = new_state;
        self.play.cursor = 0;
        self.play.total_dt = 0.0f32;
    }

    fn set_look_direction(&mut self, dir: Vec2) {
        self.look.0 = std::f32::consts::PI - dir.angle_to(-Vec2::Y);
    }

    fn set_walk_step(&mut self, step: Vec2) {
        self.kinematic.dr = step;
    }

    fn do_auto_state_transition(&mut self) {
        match self.state {
            StabberState::Attacking if self.is_anim_done() => {
                self.set_state(StabberState::Idle);
            }
            _ => (),
        }
    }

    fn current_state(&self) -> StabberState {
        *self.state
    }

    fn is_anim_done(&self) -> bool {
        self.play.is_done(self.animation)
    }
}

pub fn spawn(world: &mut World, pos: Vec2) {
    world.spawn((
        Transform::from_pos(pos),
        CharacterLook(0.0),
        Team::Enemy,
        Health::new(STABBER_SPAWN_HEALTH),
        DamageCooldown::new(STABBER_HIT_COOLDOWN),
        KinematicControl::new(col_group::LEVEL),
        BodyTag {
            groups: col_group::CHARACTERS,
            shape: Shape::Rect {
                width: STABBER_SIZE,
                height: STABBER_SIZE,
            },
        },
        AnimationPlay {
            pause: false,
            animation: AnimationId::BunnyWalkD,
            total_dt: 0.0,
            cursor: 0,
        },
        StabberState::Idle,
    ));
}

pub fn auto_state_transition(world: &mut World, animations: &HashMap<AnimationId, Animation>) {
    for (_, (state, play, kinematic, look)) in world.query_mut::<(
        &mut StabberState,
        &mut AnimationPlay,
        &mut KinematicControl,
        &mut CharacterLook,
    )>() {
        let Some(animation) = animations.get(&play.animation) else {
            warn!("Animation {:?} is not loaded", play.animation);
            continue;
        };
        let mut ctx = StabberContext {
            animation,
            play,
            kinematic,
            state,
            look,
        };
        ctx.do_auto_state_transition();
    }
}

pub fn ai(dt: f32, world: &mut World, animations: &HashMap<AnimationId, Animation>) {
    let Some((_, (player_tf, _))) = world
        .query_mut::<(&Transform, &PlayerData)>()
        .into_iter()
        .next()
    else {
        return;
    };
    let player_tf = *player_tf;

    for (_, (tf, state, play, kinematic, look)) in world.query_mut::<(
        &Transform,
        &mut StabberState,
        &mut AnimationPlay,
        &mut KinematicControl,
        &mut CharacterLook,
    )>() {
        let off_to_player = player_tf.pos - tf.pos;
        let dir = off_to_player.normalize_or(Vec2::Y);
        let Some(animation) = animations.get(&play.animation) else {
            warn!("Animation {:?} is not loaded", play.animation);
            continue;
        };
        let mut ctx = StabberContext {
            animation,
            play,
            kinematic,
            state,
            look,
        };

        ctx.set_walk_step(Vec2::ZERO);
        match ctx.current_state() {
            StabberState::Idle => {
                ctx.set_look_direction(dir);
                ctx.set_walk_step(dir * STABBER_WALK_SPEED * dt);
                if off_to_player.length() <= STABBER_AGRO_RANGE {
                    ctx.set_state(StabberState::Attacking);
                }
            }
            StabberState::Attacking => (),
        }
    }
}

pub fn state_to_anim(world: &mut World) {
    for (_, (state, play)) in world.query_mut::<(&StabberState, &mut AnimationPlay)>() {
        let animation = match *state {
            StabberState::Idle => AnimationId::StabberIdle,
            StabberState::Attacking => AnimationId::StabberAttack,
        };
        play.animation = animation;
    }
}

pub fn die_on_zero_health(world: &mut World, cmds: &mut CommandBuffer) {
    for (entity, (health, _)) in world.query_mut::<(&Health, &StabberState)>() {
        if health.value > 0 {
            continue;
        }
        cmds.despawn(entity);
    }
}
