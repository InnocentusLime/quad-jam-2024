use hashbrown::HashMap;
use lib_asset::animation::{Animation, AnimationId, ClipAction};

use super::prelude::*;

pub const PLAYER_SPEED: f32 = 48.0;
pub const PLAYER_DASH_SPEED: f32 = 432.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 3;
pub const PLAYER_HIT_COOLDOWN: f32 = 1.0;
pub const PLAYER_SIZE: f32 = 16.0;

pub const PLAYER_MAX_STAMINA: f32 = 100.0;
pub const PLAYER_STAMINA_REGEN_RATE: f32 = 20.0;
pub const PLAYER_STAMINA_REGEN_COOLDOWN: f32 = 0.8;
pub const PLAYER_ATTACK_COST: f32 = 10.0;
pub const PLAYER_DASH_COST: f32 = 25.0;

struct PlayerContext<'a> {
    kinematic: &'a mut KinematicControl,
    play: &'a mut AnimationPlay,
    animation: &'a Animation,
    data: &'a mut PlayerData,
    look: &'a mut CharacterLook,
}

impl<'a> PlayerContext<'a> {
    fn set_state(&mut self, new_state: PlayerState) {
        self.data.state = new_state;
        self.play.cursor = 0;
        self.play.total_dt = 0.0f32;
    }

    fn set_look_direction(&mut self, dir: Vec2) {
        self.look.0 = dir.to_angle();
    }

    fn look_direction(&self) -> Vec2 {
        Vec2::from_angle(self.look.0)
    }

    fn set_walk_step(&mut self, step: Vec2) {
        if self.can_move() {
            self.kinematic.dr = step;
        } else {
            self.kinematic.dr = Vec2::ZERO;
        }
    }

    fn do_auto_state_transition(&mut self) {
        match self.data.state {
            PlayerState::Attacking if self.is_anim_done() => {
                self.set_state(PlayerState::Idle);
            }
            PlayerState::Dashing if self.is_anim_done() => {
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

    fn get_input_flags(&self) -> (bool, bool) {
        for clip in self.animation.active_clips(self.play.cursor) {
            let ClipAction::LockInput {
                allow_walk_input,
                allow_look_input,
            } = clip.action
            else {
                continue;
            };
            return (allow_walk_input, allow_look_input);
        }
        (true, true)
    }

    fn can_move(&self) -> bool {
        for clip in self.animation.active_clips(self.play.cursor) {
            let ClipAction::Move = clip.action else {
                continue;
            };
            return true;
        }
        false
    }

    fn can_do_action(&self, cost: f32) -> bool {
        self.data.stamina >= cost
    }

    fn do_action(&mut self, cost: f32) {
        self.data.stamina -= cost;
        self.data.stamina_cooldown = PLAYER_STAMINA_REGEN_COOLDOWN;
    }
}

pub fn draw_player_state(world: &World) {
    for (_, (tf, kinematic, play, data, look)) in world
        .query::<(
            &Transform,
            &mut KinematicControl,
            &mut AnimationPlay,
            &mut PlayerData,
            &CharacterLook,
        )>()
        .iter()
    {
        let debug_texts = [
            format!("{:?}", data.state),
            format!("{:?}", play.animation),
            format!("cursor (ms): {}", play.cursor),
            format!("look: {:.2}", look.0.to_degrees()),
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
        CharacterLook(0.0),
        PlayerData {
            state: PlayerState::Idle,
            stamina: PLAYER_MAX_STAMINA,
            stamina_cooldown: 0.0,
        },
        PlayerScore(0),
        Team::Player,
        Health::new(PLAYER_SPAWN_HEALTH),
        DamageCooldown::new(PLAYER_HIT_COOLDOWN),
        KinematicControl::new(col_group::LEVEL),
        BodyTag {
            groups: col_group::CHARACTERS.union(col_group::PLAYER),
            shape: Shape::Rect {
                width: PLAYER_SIZE,
                height: PLAYER_SIZE,
            },
        },
        AnimationPlay {
            pause: false,
            animation: AnimationId::BunnyWalkD,
            total_dt: 0.0,
            cursor: 0,
        },
    ));
}

pub fn auto_state_transition(world: &mut World, animations: &HashMap<AnimationId, Animation>) {
    for (_, (data, play, kinematic, look)) in world.query_mut::<(
        &mut PlayerData,
        &mut AnimationPlay,
        &mut KinematicControl,
        &mut CharacterLook,
    )>() {
        let Some(animation) = animations.get(&play.animation) else {
            warn!("Animation {:?} is not loaded", play.animation);
            continue;
        };
        let mut ctx = PlayerContext {
            animation,
            play,
            kinematic,
            data,
            look,
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
    for (_, (tf, data, play, kinematic, look)) in world.query_mut::<(
        &Transform,
        &mut PlayerData,
        &mut AnimationPlay,
        &mut KinematicControl,
        &mut CharacterLook,
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
            look,
        };
        ctx.set_walk_step(Vec2::ZERO);
        let (allow_walk_input, allow_look_input) = ctx.get_input_flags();

        if matches!(
            ctx.current_state(),
            PlayerState::Idle | PlayerState::Walking
        ) && input.attack_down
            && ctx.can_do_action(PLAYER_ATTACK_COST)
        {
            ctx.set_state(PlayerState::Attacking);
            ctx.do_action(PLAYER_ATTACK_COST);
        } else if matches!(
            ctx.current_state(),
            PlayerState::Idle | PlayerState::Walking
        ) && input.dash_pressed
            && ctx.can_do_action(PLAYER_DASH_COST)
        {
            ctx.set_state(PlayerState::Dashing);
            ctx.do_action(PLAYER_DASH_COST);
        } else if !matches!(ctx.current_state(), PlayerState::Walking)
            && allow_walk_input
            && do_walk
        {
            ctx.set_state(PlayerState::Walking);
        } else if matches!(ctx.current_state(), PlayerState::Walking) && !do_walk {
            ctx.set_state(PlayerState::Idle);
        }

        if matches!(ctx.current_state(), PlayerState::Walking) {
            ctx.set_walk_step(walk_dir * PLAYER_SPEED * dt);
        } else if matches!(ctx.current_state(), PlayerState::Dashing) {
            ctx.set_walk_step(ctx.look_direction() * PLAYER_DASH_SPEED * dt);
        }
        if allow_look_input {
            ctx.set_look_direction(look_dir);
        }
    }
}

pub fn update_stamina(dt: f32, world: &mut World) {
    for (_, data) in world.query_mut::<&mut PlayerData>() {
        if data.stamina_cooldown >= 0.0 {
            data.stamina_cooldown -= dt;
            continue;
        }
        data.stamina += dt * PLAYER_STAMINA_REGEN_RATE;
        data.stamina = data.stamina.min(PLAYER_MAX_STAMINA);
    }
}

pub fn state_to_anim(world: &mut World) {
    for (_, (data, look, kin, play)) in world.query_mut::<(
        &PlayerData,
        &CharacterLook,
        &KinematicControl,
        &mut AnimationPlay,
    )>() {
        let animation = match data.state {
            PlayerState::Idle => match angle_to_direction(look.0) {
                Direction::Right => AnimationId::BunnyIdleR,
                Direction::Down => AnimationId::BunnyIdleD,
                Direction::Left => AnimationId::BunnyIdleL,
                Direction::Up => AnimationId::BunnyIdleU,
            },
            PlayerState::Walking => match angle_to_direction(kin.dr.to_angle()) {
                Direction::Right => AnimationId::BunnyWalkR,
                Direction::Down => AnimationId::BunnyWalkD,
                Direction::Left => AnimationId::BunnyWalkL,
                Direction::Up => AnimationId::BunnyWalkU,
            },
            PlayerState::Attacking => AnimationId::BunnyAttackD,
            PlayerState::Dashing => AnimationId::BunnyDash,
        };
        play.animation = animation;
    }
}

fn angle_to_direction(angle: f32) -> Direction {
    let increment = std::f32::consts::FRAC_PI_4;
    if (-3.0 * increment..=-increment).contains(&angle) {
        Direction::Up
    } else if (-increment..increment).contains(&angle) {
        Direction::Right
    } else if (increment..=3.0 * increment).contains(&angle) {
        Direction::Down
    } else {
        Direction::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Right,
    Down,
    Left,
    Up,
}
