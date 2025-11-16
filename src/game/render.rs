use lib_asset::FontId;

use super::prelude::*;

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

pub fn game_ui(render: &mut Render, world: &World) {
    let off_y = 16.0;
    let ui_x = TILE_SIDE_F32 * 16.0;
    let font = FontId::Quaver;

    let mut player_q = world.query::<(&PlayerScore, &Health, &PlayerData)>();
    let (_, (score, player_health, player_data)) = player_q.into_iter().next().unwrap();
    let (game_state, game_state_color) = if player_health.value <= 0 {
        ("You are dead", RED)
    } else {
        ("", BLANK)
    };

    put_text_fmt!(
        render,
        vec2(ui_x, off_y * 1.0),
        YELLOW,
        font,
        16.0,
        "Score: {}",
        score.0
    );
    put_text_fmt!(
        render,
        vec2(ui_x, off_y * 2.0),
        YELLOW,
        font,
        16.0,
        "Health: {}",
        player_health.value
    );
    put_text_fmt!(
        render,
        vec2(ui_x, off_y * 3.0),
        YELLOW,
        font,
        16.0,
        "Stamina: {:3.2}",
        player_data.stamina,
    );
    render.put_text(
        vec2(ui_x, off_y * 6.0),
        game_state_color,
        font,
        16.0,
        game_state,
    );
}

pub fn stabber_hp(render: &mut Render, world: &World) {
    for (_, (pos, health)) in &mut world
        .query::<(&Transform, &Health)>()
        .with::<&StabberState>()
    {
        if health.is_invulnerable {
            put_text_fmt!(
                render,
                pos.pos + vec2(0.0, -16.0),
                RED,
                FontId::Quaver,
                8.0,
                "{} (invulnerable)",
                health.value
            );
        } else {
            put_text_fmt!(
                render,
                pos.pos + vec2(0.0, -16.0),
                RED,
                FontId::Quaver,
                8.0,
                "{}",
                health.value
            );
        }
    }
}

pub fn toplevel_ui(app_state: &AppState, render: &mut Render) {
    match app_state {
        AppState::Start => {
            render.announcement_text = Some(AnnouncementText {
                heading: start_text(),
                body: Some(START_HINT),
            })
        }
        AppState::GameOver => {
            render.announcement_text = Some(AnnouncementText {
                heading: GAMEOVER_TEXT,
                body: Some(game_restart_hint()),
            })
        }
        AppState::Win => {
            render.announcement_text = Some(AnnouncementText {
                heading: WIN_TEXT,
                body: Some(game_continue_hint()),
            })
        }
        AppState::Active { paused: true } => {
            render.announcement_text = Some(AnnouncementText {
                heading: PAUSE_TEXT,
                body: Some(PAUSE_HINT),
            })
        }
        AppState::PleaseRotate => {
            render.announcement_text = Some(AnnouncementText {
                heading: ORIENTATION_TEXT,
                body: Some(ORIENTATION_HINT),
            })
        }
        AppState::GameDone => {
            render.announcement_text = Some(AnnouncementText {
                heading: COMPLETE_TEXT,
                body: Some(game_restart_hint()),
            })
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
