use lib_game::*;
use crate::{components::*, player::PLAYER_SIZE};
use macroquad::prelude::*;
use crate::game::Game;
use shipyard::{EntityId, Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

pub const SMELL_AFTER_PLAYER: f32 = 4.0;

pub fn debug_draw_tile_smell(
    world: &World
) {
    world.run(do_debug_tile_smell_draw)
}

pub fn tick_smell(
    dt: f32,
    mut smell: ViewMut<TileSmell>,
) {
    for smell in (&mut smell).iter() {
        smell.time_left = (smell.time_left - dt).max(0.0);
    }
}

pub fn player_step_smell(
    game: UniqueView<Game>,
    mut smell: ViewMut<TileSmell>,
    pos: View<Transform>,
    tile_storage: View<TileStorage>,
) {
    let player_pos = pos.get(game.player).unwrap().pos;
    let Some(storage) = tile_storage.iter().next() else {
        return;
    };

    for (x, y, id) in storage.iter_poses() {
        let px = ((player_pos.x + PLAYER_SIZE / 2.0) / 32.0) as usize;
        let py = ((player_pos.y + PLAYER_SIZE / 2.0) / 32.0) as usize;
        let Ok(mut smell) = (&mut smell).get(id)
            else { continue; };

        if x != px || y != py { continue; }
        smell.time_left = SMELL_AFTER_PLAYER;
    }
}

fn do_debug_tile_smell_draw(tile_storage: View<TileStorage>, tiles: View<TileSmell>) {
    let Some(storage) = tile_storage.iter().next() else {
        return;
    };
    let iter = storage
        .iter_poses()
        .filter_map(|(x, y, id)| Some((x, y, tiles.get(id).ok()?)));

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