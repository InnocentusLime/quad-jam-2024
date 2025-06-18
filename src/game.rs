use lib_game::*;
use macroquad::prelude::*;
use shipyard::{EntityId, IntoIter, View, World};

use crate::goal::spawn_goal;

use crate::components::*;
use crate::level::LevelDef;
use crate::player::spawn_player;

pub const LEVEL_GROUP: PhysicsGroup = PhysicsGroup {
    level: true,
    ..PhysicsGroup::empty()
};
#[allow(dead_code)]
pub const PROJECTILES_GROUP: PhysicsGroup = PhysicsGroup {
    projectiles: true,
    ..PhysicsGroup::empty()
};
pub const LEVEL_INTERACT: PhysicsGroup = PhysicsGroup {
    npcs: true,
    player: true,
    projectiles: true,
    ..PhysicsGroup::empty()
};

fn spawn_tiles(width: usize, height: usize, data: Vec<TileType>, world: &mut World) -> EntityId {
    assert_eq!(data.len(), width * height);

    let storage = TileStorage::from_data(
        width,
        height,
        data.into_iter().map(|ty| world.add_entity(ty)).collect(),
    )
    .unwrap();

    for (x, y, tile) in storage.iter_poses() {
        world.add_component(
            tile,
            Transform {
                pos: vec2(x as f32 * 32.0 + 16.0, y as f32 * 32.0 + 16.0),
                angle: 0.0,
            },
        );

        let ty = **world.get::<&TileType>(tile).unwrap();

        match ty {
            TileType::Wall => world.add_component(
                tile,
                (BodyTag::new(
                    PhysicsFilter(LEVEL_GROUP, LEVEL_INTERACT),
                    ColliderTy::Box {
                        width: 32.0,
                        height: 32.0,
                    },
                    1.0,
                    true,
                    BodyKind::Static,
                ),),
            ),
            TileType::Ground => world.add_component(tile, (TileSmell { time_left: 0.0 },)),
        }
    }

    world.add_entity(storage)
}

pub fn init_level(world: &mut World, level_def: LevelDef) {
    let tile_data = level_def
        .map
        .tiles
        .into_iter()
        .map(|x: crate::level::TileDef| match x {
            crate::level::TileDef::Wall => TileType::Wall,
            crate::level::TileDef::Ground => TileType::Ground,
        })
        .collect::<Vec<_>>();

    spawn_tiles(level_def.map.width, level_def.map.height, tile_data, world);
    for entity in level_def.entities {
        match entity {
            crate::level::EntityDef::Player(pos) => spawn_player(world, pos),
            crate::level::EntityDef::Goal(pos) => spawn_goal(world, pos),
        }
    }
}

pub fn decide_next_state(
    player: View<PlayerTag>,
    health: View<Health>,
    goal: View<GoalTag>,
) -> Option<AppState> {
    let player_dead = (&player, &health).iter().all(|(_, hp)| hp.0 <= 0);
    let goal_achieved = goal.iter().any(|x| x.achieved);

    if player_dead {
        return Some(AppState::GameOver);
    }

    if goal_achieved {
        return Some(AppState::Win);
    }

    None
}
