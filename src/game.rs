use lib_game::*;
use macroquad::prelude::*;
use shipyard::{EntityId, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};

use crate::enemy::spawn_brute;
use crate::enemy::spawn_main_cell;
use crate::goal::spawn_goal;
use crate::inline_tilemap;

use crate::components::*;
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

#[derive(Unique)]
pub struct GameState {
    pub do_ai: bool,
}

impl GameState {
    pub fn new(world: &mut World) -> Self {
        let poses = [
            vec2(200.0, 160.0),
            vec2(64.0, 250.0),
            vec2(128.0, 150.0),
            vec2(300.0, 250.0),
        ];
        for pos in poses {
            spawn_box(pos, world);
        }

        for x in 0..8 {
            for y in 0..8 {
                let pos = vec2(x as f32 * 12.0 + 100.0, y as f32 * 12.0 + 200.0);
                spawn_brute(pos, world);
            }
        }

        spawn_main_cell(vec2(64.0, 128.0), world);
        spawn_bullet(vec2(100.0, 100.0), world);

        spawn_tiles(
            16,
            16,
            inline_tilemap![
                w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, g, g, g, g, g, g, g, g, g, g, g,
                g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g,
                g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g,
                g, g, w, w, w, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g,
                g, w, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, g, g, g, g, g, g, w,
                g, g, w, g, g, g, g, w, w, g, g, w, g, g, g, g, g, g, w, g, g, g, g, w, w, g, g, g,
                g, g, g, g, g, g, w, g, g, g, g, w, w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w, w, w, w, w, w, w, w, w, w, w, w, w,
                w, w, w, w
            ],
            world,
        );

        spawn_goal(world, vec2(400.0, 64.0));
        spawn_player(world);

        Self {
            do_ai: true,
        }
    }

    pub fn should_ai(this: UniqueView<GameState>) -> bool {
        this.do_ai
    }
}

fn spawn_box(pos: Vec2, world: &mut World) {
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
    
fn spawn_bullet(pos: Vec2, world: &mut World) {
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

pub fn count_rewards(mut reward: ViewMut<RewardInfo>, mut score: UniqueViewMut<PlayerScore>) {
    for reward in (&mut reward).iter() {
        if !matches!(reward.state, RewardState::Pending) {
            continue;
        }

        reward.state = RewardState::Counted;
        score.0 += reward.amount;
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
