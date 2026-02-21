use crate::{LockInput, Move, animation::Animation};
use hecs::{Bundle, Entity, Query, World};
use lib_asset::animation_manifest::AnimationId;
use lib_col::{Group, Shape};
use log::warn;
use macroquad::prelude::*;

use crate::{
    AnimationPlay, BodyTag, CharacterLook, Direction, Health, KinematicControl, Resources, Team,
    Transform, col_group, draw_shape_lines,
};

pub fn draw_char_state(world: &World, resources: &Resources) {
    for_each_character::<()>(world, resources, |ent, character| {
        let debug_texts = [
            format!("ID {ent:?}"),
            format!("{:?}", character.animation_id()),
            format!("cursor (ms): {}", character.character_q.play.cursor),
            format!("look: {:.2}", character.character_q.look.0.to_degrees()),
            format!("dr: {:.2}", character.character_q.kinematic.dr),
        ];

        let pos = character.pos() + vec2(8.0, 0.0);
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

        draw_shape_lines(
            character.character_q.tf,
            &character.character_q.body.shape,
            YELLOW,
        );
        let end = character.pos() + 20.0 * character.look_direction();
        draw_line(
            character.pos().x,
            character.pos().y,
            end.x,
            end.y,
            1.0,
            YELLOW,
        );
    });
}

pub fn state_to_anim<Q: Query>(world: &mut World, resources: &Resources)
where
    for<'a> Q::Item<'a>: CharacterData,
{
    for_each_character::<Q>(world, resources, |_, character| {
        character.character_q.play.animation = Q::Item::state_to_anim(&character);
    });
}

pub fn do_auto_state_transition<Q: Query>(world: &mut World, resources: &Resources)
where
    for<'a> Q::Item<'a>: CharacterData,
{
    for_each_character::<Q>(world, resources, |_, mut character| {
        if !character.is_anim_done() {
            return;
        }
        Q::Item::on_anim_end(&mut character);
    });
}

pub fn for_each_character<Q: Query>(
    world: &World,
    resources: &Resources,
    mut body: impl for<'c> FnMut(Entity, Character<Q::Item<'c>>),
) {
    let mut world_q = world.query::<(CharacterQuery, Q)>();
    for (ent, (character_q, state)) in &mut world_q {
        let Some(animation) = resources.animations.get(&character_q.play.animation) else {
            warn!(
                "Entity {ent:?}: Animation {:?} is not loaded",
                character_q.play.animation
            );
            continue;
        };
        body(
            ent,
            Character {
                character_q,
                data: state,
                animation,
            },
        );
    }
}

pub struct Character<'a, T> {
    pub character_q: CharacterQuery<'a>,
    pub data: T,
    pub animation: &'a Animation,
}

impl<'a, T> Character<'a, T> {
    pub fn pos(&self) -> Vec2 {
        self.character_q.tf.pos
    }

    pub fn set_look_direction(&mut self, dir: Vec2) {
        *self.character_q.look = CharacterLook::from_direction(dir)
    }

    pub fn look_direction(&self) -> Vec2 {
        self.character_q.look.to_direction()
    }

    pub fn set_walk_step(&mut self, step: Vec2) {
        if self.can_move() {
            self.character_q.kinematic.dr = step;
        } else {
            self.character_q.kinematic.dr = Vec2::ZERO;
        }
    }

    pub fn is_anim_done(&self) -> bool {
        self.character_q.play.is_done(self.animation)
    }

    pub fn get_input_flags(&self) -> (bool, bool) {
        self.animation
            .active_clips::<LockInput>(self.anim_cursor())
            .map(|(_, x)| (x.allow_walk_input, x.allow_look_input))
            .next()
            .unwrap_or((true, true))
    }

    pub fn can_move(&self) -> bool {
        self.animation
            .active_clips::<Move>(self.anim_cursor())
            .next()
            .is_some()
    }

    pub fn anim_cursor(&self) -> u32 {
        self.character_q.play.cursor
    }

    pub fn look_angle(&self) -> f32 {
        self.character_q.look.0
    }

    pub fn look_dir_enum(&self) -> Direction {
        self.character_q.look.to_dir_enum()
    }

    pub fn transform_child(&self, rotate: bool, pos: Vec2, angle: f32) -> Transform {
        if rotate {
            Transform {
                pos: self.pos() + self.look_direction().rotate(pos),
                angle: angle + self.look_angle(),
            }
        } else {
            Transform {
                pos: self.pos() + pos,
                angle,
            }
        }
    }

    pub fn transform_character(&self, rotate: bool, pos: Vec2, look_angle: f32) -> (Vec2, f32) {
        if rotate {
            (
                self.pos() + self.look_direction().rotate(pos),
                look_angle + self.look_angle(),
            )
        } else {
            (self.pos() + pos, look_angle)
        }
    }

    pub fn collided(&self) -> bool {
        self.character_q.kinematic.collided
    }

    pub fn animation_id(&self) -> AnimationId {
        self.character_q.play.animation
    }
}

impl<'a, T: CharacterData> Character<'a, T> {
    pub fn set_state(&mut self, state_id: T::StateId) {
        self.character_q.play.cursor = 0;
        self.character_q.play.total_dt = 0.0f32;
        self.data.set_state(state_id);
    }

    pub fn get_state(&self) -> T::StateId {
        self.data.get_state()
    }
}

pub trait CharacterData: Sized {
    type StateId;
    fn get_state(&self) -> Self::StateId;
    fn set_state(&mut self, new_state: Self::StateId);
    fn state_to_anim(character: &Character<Self>) -> AnimationId;
    fn on_anim_end(character: &mut Character<Self>);
}

#[derive(Query)]
pub struct CharacterQuery<'a> {
    pub tf: &'a mut Transform,
    pub kinematic: &'a mut KinematicControl,
    pub play: &'a mut AnimationPlay,
    pub look: &'a mut CharacterLook,
    pub hp: &'a mut Health,
    pub team: &'a Team,
    pub body: &'a BodyTag,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub tf: Transform,
    pub kinematic: KinematicControl,
    pub play: AnimationPlay,
    pub look: CharacterLook,
    pub hp: Health,
    pub team: Team,
    pub body: BodyTag,
}

impl CharacterBundle {
    pub fn new_enemy(pos: Vec2, shape: Shape, spawn_health: i32) -> Self {
        Self::new_creature(pos, col_group::NONE, shape, spawn_health, Team::Enemy)
    }

    pub fn new_player(pos: Vec2, shape: Shape, spawn_health: i32) -> Self {
        Self::new_creature(pos, col_group::PLAYER, shape, spawn_health, Team::Player)
    }

    pub fn new_creature(
        pos: Vec2,
        group: Group,
        shape: Shape,
        spawn_health: i32,
        team: Team,
    ) -> Self {
        CharacterBundle {
            team,
            tf: Transform::from_pos(pos),
            look: CharacterLook(0.0),
            hp: Health::new(spawn_health),
            kinematic: KinematicControl::new_slide(col_group::LEVEL),
            body: BodyTag {
                groups: col_group::CHARACTERS.union(group),
                shape,
            },
            play: AnimationPlay {
                pause: false,
                animation: AnimationId::BnuuyWalkD,
                total_dt: 0.0,
                cursor: 0,
            },
        }
    }

    pub fn new_projectile(pos: Vec2, shape: Shape) -> Self {
        CharacterBundle {
            team: Team::Enemy,
            tf: Transform::from_pos(pos),
            look: CharacterLook(0.0),
            hp: Health::new(1),
            kinematic: KinematicControl::new_nonslide(col_group::LEVEL),
            body: BodyTag {
                groups: col_group::ATTACKS,
                shape,
            },
            play: AnimationPlay {
                pause: false,
                animation: AnimationId::BnuuyWalkD,
                total_dt: 0.0,
                cursor: 0,
            },
        }
    }
}
