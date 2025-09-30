use hashbrown::HashMap;
use lib_anim::{Animation, AnimationId};

use super::prelude::*;

pub const PLAYER_SPEED: f32 = 48.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 3;
pub const PLAYER_HIT_COOLDOWN: f32 = 1.0;
pub const PLAYER_SIZE: f32 = 16.0;
pub const PLAYER_ATTACK_LENGTH: f32 = TILE_SIDE_F32 * 3.0;
pub const PLAYER_ATTACK_WIDTH: f32 = 8.0;

struct PlayerContext<'a> {
    kinematic: &'a mut KinematicControl,
    play: &'a mut AnimationPlay,
    animation: &'a Animation,
    data: &'a mut PlayerData,
}

impl<'a> PlayerContext<'a> {
    fn set_state(&mut self, new_state: PlayerState) {
        self.data.state = new_state;
        self.play.cursor = 0;
        self.play.total_dt = 0.0f32;
    }

    fn set_look_direction(&mut self, dir: Vec2) {
        self.data.look_direction = dir;
    }

    fn set_walk_step(&mut self, step: Vec2) {
        self.kinematic.dr = step;
    }

    fn do_auto_state_transition(&mut self) {
        match self.data.state {
            PlayerState::Attacking if self.is_anim_done() => {
                self.set_state(PlayerState::Idle);
            }
            _ => (),
        }
    }

    fn current_state(&self) -> PlayerState {
        self.data.state
    }

    fn is_anim_done(&self) -> bool {
        self.play.is_done(self.animation)
    }
}

pub fn draw_player_state(world: &World) {
    for (_, (tf, kinematic, play, data)) in world
        .query::<(
            &Transform,
            &mut KinematicControl,
            &mut AnimationPlay,
            &mut PlayerData,
        )>()
        .iter()
    {
        let debug_texts = [
            format!("{:?}", data.state),
            format!("{:?}", play.animation),
            format!("cursor (ms): {}", play.cursor),
            format!("look: {}", data.look_direction),
            format!("dr: {}", kinematic.dr),
        ];

        let pos = tf.pos + vec2(8.0, 0.0);
        let debug_text_size = 8.0;
        for (idx, text) in debug_texts.into_iter().enumerate() {
            draw_text(
                &text,
                pos.x,
                pos.y + (idx as f32) * debug_text_size,
                debug_text_size,
                YELLOW,
            );
        }
    }
}

pub fn spawn(world: &mut World, pos: Vec2) {
    world.spawn((
        Transform::from_pos(pos),
        PlayerData {
            look_direction: Vec2::Y,
            state: PlayerState::Idle,
        },
        PlayerScore(0),
        Health::new(PLAYER_SPAWN_HEALTH),
        DamageCooldown::new(PLAYER_HIT_COOLDOWN),
        KinematicControl::new(col_group::LEVEL),
        BodyTag {
            groups: col_group::PLAYER.union(col_group::DAMAGABLE),
            shape: Shape::Rect {
                width: PLAYER_SIZE,
                height: PLAYER_SIZE,
            },
        },
        AnimationPlay {
            animation: AnimationId::BunnyWalkD,
            total_dt: 0.0,
            cursor: 0,
        },
    ));
}

pub fn auto_state_transition(world: &mut World, animations: &HashMap<AnimationId, Animation>) {
    for (_, (data, play, kinematic)) in
        world.query_mut::<(&mut PlayerData, &mut AnimationPlay, &mut KinematicControl)>()
    {
        let Some(animation) = animations.get(&play.animation) else {
            warn!("Animation {:?} is not loaded", play.animation);
            continue;
        };
        let mut ctx = PlayerContext {
            animation,
            play,
            kinematic,
            data,
        };
        ctx.do_auto_state_transition();
    }
}

pub fn controls(
    dt: f32,
    input: &InputModel,
    world: &mut World,
    animations: &HashMap<AnimationId, Animation>,
) {
    for (_, (tf, data, play, kinematic)) in world.query_mut::<(
        &Transform,
        &mut PlayerData,
        &mut AnimationPlay,
        &mut KinematicControl,
    )>() {
        let look_dir = (input.aim - tf.pos).normalize_or(vec2(0.0, 1.0));
        let mut walk_dir = Vec2::ZERO;
        let mut do_walk = false;
        if input.left_movement_down {
            walk_dir += vec2(-1.0, 0.0);
            do_walk = true;
        }
        if input.up_movement_down {
            walk_dir += vec2(0.0, -1.0);
            do_walk = true;
        }
        if input.right_movement_down {
            walk_dir += vec2(1.0, 0.0);
            do_walk = true;
        }
        if input.down_movement_down {
            walk_dir += vec2(0.0, 1.0);
            do_walk = true;
        }
        walk_dir = walk_dir.normalize_or_zero();

        let Some(animation) = animations.get(&play.animation) else {
            warn!("Animation {:?} is not loaded", play.animation);
            continue;
        };
        let mut ctx = PlayerContext {
            animation,
            play,
            kinematic,
            data,
        };
        let new_state = match ctx.current_state() {
            PlayerState::Idle if input.attack_down => Some(PlayerState::Attacking),
            PlayerState::Idle if do_walk => Some(PlayerState::Walking),
            PlayerState::Walking if input.attack_down => Some(PlayerState::Attacking),
            PlayerState::Walking if !do_walk => Some(PlayerState::Idle),
            _ => None,
        };
        if let Some(new_state) = new_state {
            ctx.set_state(new_state);
        }

        if matches!(ctx.current_state(), PlayerState::Walking) {
            ctx.set_walk_step(walk_dir * PLAYER_SPEED * dt);
        } else {
            ctx.set_walk_step(Vec2::ZERO);
        }
        if matches!(
            ctx.current_state(),
            PlayerState::Walking | PlayerState::Idle
        ) {
            ctx.set_look_direction(look_dir);
        }
    }
}

pub fn state_to_anim(world: &mut World) {
    for (_, (data, play)) in world.query_mut::<(&PlayerData, &mut AnimationPlay)>() {
        let animation = match data.state {
            PlayerState::Idle => AnimationId::BunnyIdleD,
            PlayerState::Walking => AnimationId::BunnyWalkD,
            PlayerState::Attacking => AnimationId::BunnyAttackD,
        };
        play.animation = animation;
    }
}
