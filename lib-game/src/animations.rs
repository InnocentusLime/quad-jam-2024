use hashbrown::HashMap;
use hecs::{CommandBuffer, Entity, World};
use log::warn;

use crate::{AnimationEvent, AnimationPlay, Resources};

pub const ANIMATION_TIME_UNIT: f32 = 1.0 / 1000.0;

pub(crate) fn update_anims(dt: f32, world: &mut World, resources: &Resources) {
    for (_, play) in world.query::<&mut AnimationPlay>().iter() {
        let Some(anim) = resources.animations.get(&play.animation) else {
            warn!("No such anim: {:?}", play.animation);
            continue;
        };
        let max_pos = anim.max_pos();
        if max_pos == 0 {
            continue;
        }
        if play.pause {
            continue;
        }

        play.total_dt += dt;
        if play.total_dt < ANIMATION_TIME_UNIT {
            continue;
        }

        let cursor_delta = play.total_dt.div_euclid(ANIMATION_TIME_UNIT);
        play.total_dt -= cursor_delta * ANIMATION_TIME_UNIT;

        play.cursor += cursor_delta as u32;
        if anim.is_looping {
            play.cursor = play.cursor % max_pos;
        } else {
            play.cursor = play.cursor.min(max_pos);
        }
    }
}

pub(crate) fn collect_active_events(
    world: &mut World,
    active_events: &mut HashMap<AnimationEvent, Entity>,
) {
    active_events.clear();
    for (ent, event) in world.query_mut::<&AnimationEvent>() {
        active_events.insert(*event, ent);
    }
}

// Assumption: active_events contains alive entities
pub(crate) fn delete_animation_events(
    world: &mut World,
    resources: &Resources,
    cmds: &mut CommandBuffer,
    active_events: &mut HashMap<AnimationEvent, Entity>,
) {
    for (event, entity) in active_events.iter() {
        let Ok(play) = world.get::<&AnimationPlay>(event.parent) else {
            cmds.despawn(*entity);
            continue;
        };
        if play.animation != event.animation {
            cmds.despawn(*entity);
        }
    }

    for (entity, play) in world.query_mut::<&mut AnimationPlay>() {
        let Some(anim) = resources.animations.get(&play.animation) else {
            warn!("No such anim: {:?}", play.animation);
            continue;
        };
        let to_despawn = anim
            .clips
            .iter()
            .filter(|x| !(x.start <= play.cursor && play.cursor < x.start + x.len))
            .filter_map(|clip| {
                active_events.get(&AnimationEvent {
                    parent: entity,
                    animation: play.animation,
                    clip_id: clip.id,
                })
            });
        for entity in to_despawn {
            cmds.despawn(*entity);
        }
    }
}
