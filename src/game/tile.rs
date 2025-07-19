use super::prelude::*;

pub const SMELL_AFTER_PLAYER: f32 = 4.0;

pub fn debug_draw_tile_smell(world: &World) {
    let mut storage_q = world.query::<&TileStorage>();
    let Some((_, storage)) = storage_q.into_iter().next() else {
        return;
    };
    let iter = storage
        .iter_poses()
        .filter_map(|(x, y, id)| Some((x, y, world.get::<&TileSmell>(id).ok()?)));

    for (x, y, tile) in iter {
        draw_text(
            &format!("{:.2}", tile.time_left),
            x as f32 * 32.0 + 8.0,
            y as f32 * 32.0 + 16.0,
            10.0,
            YELLOW,
        );
    }
}

pub fn tick_smell(dt: f32, world: &mut World) {
    for (_, smell) in world.query_mut::<&mut TileSmell>() {
        smell.time_left = (smell.time_left - dt).max(0.0);
    }
}

pub fn player_step_smell(world: &mut World) {
    let (_, player_tf) = world
        .query_mut::<&Transform>()
        .with::<&PlayerTag>()
        .into_iter()
        .next()
        .unwrap();
    let player_pos = player_tf.pos;

    let mut storage_q = world.query::<&TileStorage>();
    let Some((_, storage)) = storage_q.into_iter().next() else {
        return;
    };

    let px = ((player_pos.x) / 32.0) as usize;
    let py = ((player_pos.y) / 32.0) as usize;

    for sx in (px.saturating_sub(1))..(px + 1) {
        for sy in (py.saturating_sub(1))..(py + 1) {
            let Some(id) = storage.get(sx, sy) else {
                continue;
            };
            let Ok(mut smell) = world.get::<&mut TileSmell>(id) else {
                continue;
            };

            smell.time_left = SMELL_AFTER_PLAYER;
        }
    }
}
