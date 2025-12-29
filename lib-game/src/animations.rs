use hashbrown::HashMap;
use hecs::{CommandBuffer, Entity, EntityBuilder, World};
use macroquad::prelude::*;

use crate::{
    AnimationPlay, AttackBundle, ClipActionObject, Render, Resources, SpriteData, Transform,
    col_group, col_query, for_each_character,
};

pub const ANIMATION_TIME_UNIT: f32 = 1.0 / 1000.0;

pub(crate) fn update(dt: f32, world: &mut World, resources: &Resources) {
    for_each_character::<()>(world, resources, |_, character| {
        let play = character.character_q.play;
        let anim = character.animation;

        let max_pos = anim.max_pos();
        if max_pos == 0 || play.pause {
            return;
        }

        play.total_dt += dt;
        if play.total_dt < ANIMATION_TIME_UNIT {
            return;
        }

        let cursor_delta = play.total_dt.div_euclid(ANIMATION_TIME_UNIT);
        play.total_dt -= cursor_delta * ANIMATION_TIME_UNIT;
        play.cursor += cursor_delta as u32;
        if anim.is_looping {
            play.cursor %= max_pos;
        } else {
            play.cursor = play.cursor.min(max_pos);
        }
    });
}

pub(crate) fn collect_clip_action_objects(
    world: &mut World,
    clip_action_objects: &mut HashMap<ClipActionObject, Entity>,
) {
    clip_action_objects.clear();
    for (ent, event) in world.query_mut::<&ClipActionObject>() {
        clip_action_objects.insert(*event, ent);
    }
}

// Assumption: active_events contains alive entities
pub(crate) fn delete_clip_action_objects(
    world: &mut World,
    resources: &Resources,
    cmds: &mut CommandBuffer,
    clip_action_objects: &mut HashMap<ClipActionObject, Entity>,
) {
    for (event, entity) in clip_action_objects.iter() {
        let Ok(play) = world.get::<&AnimationPlay>(event.parent) else {
            cmds.despawn(*entity);
            continue;
        };
        if play.animation != event.animation {
            cmds.despawn(*entity);
        }
    }

    for_each_character::<()>(world, resources, |parent, character| {
        let to_despawn = character
            .animation
            .inactive_clips(character.anim_cursor())
            .filter_map(|clip| {
                clip_action_objects.get(&ClipActionObject {
                    parent,
                    animation: character.animation_id(),
                    clip_id: clip.id,
                })
            });
        for entity in to_despawn {
            cmds.despawn(*entity);
        }
    });
}

pub(crate) fn update_attack_boxes(
    world: &mut World,
    resources: &Resources,
    cmds: &mut CommandBuffer,
    active_events: &HashMap<ClipActionObject, Entity>,
) {
    for_each_character::<()>(world, resources, |parent, character| {
        for clip in character
            .animation
            .active_attack_box(character.anim_cursor())
        {
            let event = ClipActionObject {
                parent,
                animation: character.animation_id(),
                clip_id: clip.id,
            };
            let new_col_tf = character.transform_child(
                clip.action.rotate_with_parent,
                clip.action.local_pos.to_vec2(),
                clip.action.local_rotation,
            );

            match active_events.get(&event).copied() {
                Some(ent) => {
                    let mut query = world
                        .query_one::<(&mut Transform, &mut col_query::Damage)>(ent)
                        .expect("incomplete attach box components");
                    let (col_tf, col_q) = query.get().unwrap();
                    *col_tf = new_col_tf;
                    col_q.collider = clip.action.shape;
                    col_q.group = clip.action.group;
                }
                None => {
                    let mut builder = EntityBuilder::new();
                    builder.add_bundle(AttackBundle::new(
                        new_col_tf,
                        clip.action.team,
                        clip.action.shape,
                        1.0,
                        col_group::NONE,
                    ));
                    builder.add(event);
                    cmds.spawn(builder.build());
                }
            }
        }
    });
}

pub(crate) fn update_invulnerability(world: &mut World, resources: &Resources) {
    for_each_character::<()>(world, resources, |_, character| {
        let is_invulnerable = character
            .animation
            .active_invulnerability(character.anim_cursor())
            .next()
            .is_some();
        character.character_q.hp.is_invulnerable = is_invulnerable;
    });
}

pub(crate) fn draw_sprites(world: &mut World, resources: &Resources, render: &mut Render) {
    for_each_character::<()>(world, resources, |_, character| {
        for clip in character
            .animation
            .active_draw_sprite(character.anim_cursor())
        {
            let tf = character.transform_child(
                clip.action.rotate_with_parent,
                clip.action.local_pos.to_vec2(),
                clip.action.local_rotation,
            );
            render.sprite_buffer.push(SpriteData {
                layer: clip.action.layer,
                tf,
                texture: clip.action.texture_id,
                rect: Rect {
                    x: clip.action.rect.x as f32,
                    y: clip.action.rect.y as f32,
                    w: clip.action.rect.w as f32,
                    h: clip.action.rect.h as f32,
                },
                color: WHITE,
                sort_offset: clip.action.sort_offset,
            })
        }
    });
}
