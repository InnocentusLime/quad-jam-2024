use lib_game::*;
use macroquad::prelude::*;
use shipyard::{EntityId, IntoIter, View, ViewMut, World};

use crate::enemy::spawn_brute;
use crate::enemy::spawn_main_cell;
use crate::goal::spawn_goal;

use crate::components::*;
use crate::level::LevelDef;
use crate::player::spawn_player;

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
                    InteractionGroups {
                        memberships: groups::LEVEL,
                        filter: groups::LEVEL_INTERACT,
                    },
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
        .map(|x| match x {
            crate::level::TileDef::Wall => TileType::Wall,
            crate::level::TileDef::Ground => TileType::Ground,
        })
        .collect::<Vec<_>>();

    spawn_tiles(level_def.map.width, level_def.map.height, tile_data, world);
    for entity in level_def.entities {
        match entity {
            crate::level::EntityDef::Player(pos) => spawn_player(world, pos),
            crate::level::EntityDef::MainCell(pos) => spawn_main_cell(world, pos),
            crate::level::EntityDef::Brute(pos) => spawn_brute(world, pos),
            crate::level::EntityDef::Goal(pos) => spawn_goal(world, pos),
            crate::level::EntityDef::Box(pos) => spawn_box(world, pos),
            crate::level::EntityDef::Bullet(pos) => spawn_bullet(world, pos),
        }
    }
}

fn spawn_box(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        BoxTag,
        BodyTag::new(
            InteractionGroups {
                memberships: groups::LEVEL,
                filter: groups::LEVEL_INTERACT,
            },
            ColliderTy::Box {
                width: 32.0,
                height: 32.0,
            },
            1.0,
            true,
            BodyKind::Dynamic,
        ),
    ));
}

fn spawn_bullet(world: &mut World, pos: Vec2) {
    world.add_entity((
        Transform::from_pos(pos),
        BulletTag::Dropped,
        OneSensorTag::new(
            ColliderTy::Box {
                width: 16.0,
                height: 16.0,
            },
            InteractionGroups {
                memberships: groups::LEVEL,
                filter: groups::PLAYER,
            },
        ),
    ));
    world.add_entity((
        Transform::from_pos(pos),
        BulletHitterTag,
        OneSensorTag::new(
            ColliderTy::Box {
                width: 24.0,
                height: 24.0,
            },
            InteractionGroups {
                memberships: groups::PROJECTILES,
                filter: groups::MAINCELL,
            },
        ),
    ));
    world.add_entity((
        Transform::from_pos(pos),
        BulletWallHitterTag,
        OneSensorTag::new(
            ColliderTy::Box {
                width: 16.0,
                height: 16.0,
            },
            InteractionGroups {
                memberships: groups::PROJECTILES,
                filter: groups::LEVEL.union(groups::NPCS),
            },
        ),
    ));
}

pub fn reward_enemies(enemy: View<EnemyState>, mut reward: ViewMut<RewardInfo>) {
    for (state, reward) in (&enemy, &mut reward).iter() {
        if !matches!(
            (state, reward.state),
            (EnemyState::Dead, RewardState::Locked)
        ) {
            continue;
        }

        reward.state = RewardState::Pending;
    }
}

pub fn count_rewards(mut reward: ViewMut<RewardInfo>, mut score: ViewMut<PlayerScore>) {
    for reward in (&mut reward).iter() {
        if !matches!(reward.state, RewardState::Pending) {
            continue;
        }

        reward.state = RewardState::Counted;
        for score in (&mut score).iter() {
            score.0 += reward.amount;
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
