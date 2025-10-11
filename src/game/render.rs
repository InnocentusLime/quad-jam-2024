use hashbrown::HashMap;
use lib_anim::{Animation, AnimationId, ClipAction};
use lib_asset::{FontId, TextureId};

use super::prelude::*;

use std::borrow::Cow;

static WIN_TEXT: &str = "Congratulations!";
static GAMEOVER_TEXT: &str = "Game Over";
static COMPLETE_TEXT: &str = "Congratulations! You beat the game!";
static PAUSE_TEXT: &str = "Paused";
static PAUSE_HINT: &str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPress escape to resume";

static RESTART_HINT_DESK: &str = "Press Space to restart";
static RESTART_HINT_MOBILE: &str = "Tap the screen to restart";
static CONTINUE_HINT_DESK: &str = "Press Space to continue";
static CONTINUE_HINT_MOBILE: &str = "Tap the screen to continue";

static START_TEXT_DESK: &str = "Controls";
static START_HINT: &str = "Move: WASD\nShoot: Mouse + Left Button\nYou get extra score for hitting multiple enemies at once\nPRESS SPACE TO START\nGet ready to run!";
static START_TEXT_MOBILE: &str = "Tap to start";

pub fn anims(world: &World, render: &mut Render, animations: &HashMap<AnimationId, Animation>) {
    for (_, (tf, play)) in world.query::<(&Transform, &mut AnimationPlay)>().iter() {
        let Some(anim) = animations.get(&play.animation) else {
            warn!("No such anim: {:?}", play.animation);
            continue;
        };
        let matching_clips = anim
            .clips
            .iter()
            .filter(|x| x.start <= play.cursor && play.cursor < x.start + x.len);
        for clip in matching_clips {
            match &clip.action {
                ClipAction::DrawSprite {
                    layer,
                    local_pos,
                    local_rotation: _,
                    texture: _,
                    rect,
                    origin,
                    sort_offset,
                } => render.sprite_buffer.push(SpriteData {
                    layer: *layer,
                    tf: Transform::from_pos(tf.pos + vec2(local_pos.x, local_pos.y)),
                    texture: TextureId::BunnyAtlas,
                    rect: Rect {
                        x: rect.x as f32,
                        y: rect.y as f32,
                        w: rect.w as f32,
                        h: rect.h as f32,
                    },
                    origin: vec2(origin.x, origin.y),
                    color: WHITE,
                    sort_offset: *sort_offset,
                }),
            }
        }
    }
}

pub fn player_attack(render: &mut Render, world: &World) {
    use super::player::{PLAYER_ATTACK_LENGTH, PLAYER_ATTACK_WIDTH};
    for (_, pos) in &mut world.query::<&Transform>().with::<&PlayerAttackTag>() {
        render.world.spawn((
            *pos,
            RectShape {
                origin: vec2(0.5, 0.5),
                width: PLAYER_ATTACK_LENGTH,
                height: PLAYER_ATTACK_WIDTH,
            },
            Tint(RED),
        ));
    }
}

pub fn game_ui(render: &mut Render, world: &World) {
    let world_font_size = 16f32;
    let off_y = world_font_size;
    let ui_x = TILE_SIDE_F32 * 16.0;
    let (font_size, font_scale, font_scale_aspect) = camera_font_scale(world_font_size);
    let font = FontId::Quaver;

    let mut player_q = world.query::<(&PlayerScore, &Health)>();
    let (_, (score, player_health)) = player_q.into_iter().next().unwrap();
    let (game_state, game_state_color) = if player_health.value <= 0 {
        ("You are dead", RED)
    } else {
        ("", BLANK)
    };

    render.world.spawn((
        GlyphText {
            font,
            string: Cow::Owned(format!("Score:{}", score.0)),
            font_size,
            font_scale,
            font_scale_aspect,
        },
        Tint(YELLOW),
        Transform::from_xy(ui_x, off_y * 1.0),
    ));
    render.world.spawn((
        GlyphText {
            font,
            string: Cow::Owned(format!("Health:{}", player_health.value)),
            font_size,
            font_scale,
            font_scale_aspect,
        },
        Tint(YELLOW),
        Transform::from_xy(ui_x, off_y * 2.0),
    ));
    render.world.spawn((
        GlyphText {
            font,
            string: Cow::Borrowed(game_state),
            font_size,
            font_scale,
            font_scale_aspect,
        },
        Tint(game_state_color),
        Transform::from_xy(ui_x, off_y * 5.0),
    ));
}

pub fn goal(render: &mut Render, world: &World) {
    for (_, pos) in &mut world.query::<&Transform>().with::<&GoalTag>() {
        render.world.spawn((
            *pos,
            Tint(GREEN),
            RectShape {
                origin: vec2(0.5, 0.5),
                width: 16.0,
                height: 16.0,
            },
        ));
    }
}

pub fn toplevel_ui(app_state: &AppState, render: &mut Render) {
    match app_state {
        AppState::Start => {
            render.world.spawn((AnnouncementText {
                heading: start_text(),
                body: Some(START_HINT),
            },));
        }
        AppState::GameOver => {
            render.world.spawn((AnnouncementText {
                heading: GAMEOVER_TEXT,
                body: Some(game_restart_hint()),
            },));
        }
        AppState::Win => {
            render.world.spawn((AnnouncementText {
                heading: WIN_TEXT,
                body: Some(game_continue_hint()),
            },));
        }
        AppState::Active { paused: true } => {
            render.world.spawn((AnnouncementText {
                heading: PAUSE_TEXT,
                body: Some(PAUSE_HINT),
            },));
        }
        AppState::PleaseRotate => {
            render.world.spawn((AnnouncementText {
                heading: ORIENTATION_TEXT,
                body: Some(ORIENTATION_HINT),
            },));
        }
        AppState::GameDone => {
            render.world.spawn((AnnouncementText {
                heading: COMPLETE_TEXT,
                body: Some(game_restart_hint()),
            },));
        }
        _ => (),
    }
}

fn start_text() -> &'static str {
    if lib_game::sys::on_mobile() {
        START_TEXT_MOBILE
    } else {
        START_TEXT_DESK
    }
}

fn game_restart_hint() -> &'static str {
    if lib_game::sys::on_mobile() {
        RESTART_HINT_MOBILE
    } else {
        RESTART_HINT_DESK
    }
}

fn game_continue_hint() -> &'static str {
    if lib_game::sys::on_mobile() {
        CONTINUE_HINT_MOBILE
    } else {
        CONTINUE_HINT_DESK
    }
}
