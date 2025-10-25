use hashbrown::HashMap;
use hecs::World;
use lib_anim::{Animation, AnimationId};
use log::warn;

use crate::AnimationPlay;

pub const ANIMATION_TIME_UNIT: f32 = 1.0 / 1000.0;

pub(crate) fn update_anims(
    dt: f32,
    world: &mut World,
    animations: &HashMap<AnimationId, Animation>,
) {
    for (_, play) in world.query::<&mut AnimationPlay>().iter() {
        let Some(anim) = animations.get(&play.animation) else {
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

pub(crate) fn patch_bunny_attack_animation(animation: &mut Animation) {
    use lib_anim::ClipAction::DrawSprite;

    let hit_off = 14.0;
    match &mut animation.clips[1].action {
        DrawSprite { local_pos, .. } => local_pos.y -= 3.0,
    }
    match &mut animation.clips[2].action {
        DrawSprite { local_pos, .. } => local_pos.y += hit_off,
    }
    match &mut animation.clips[3].action {
        DrawSprite { local_pos, .. } => local_pos.y += hit_off,
    }
    match &mut animation.clips[4].action {
        DrawSprite { local_pos, .. } => local_pos.y += hit_off,
    }
}
