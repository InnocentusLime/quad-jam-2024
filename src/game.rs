use lib_game::*;
use macroquad::prelude::*;
use shipyard::{EntityId, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};

use crate::goal::spawn_goal;
use crate::inline_tilemap;

use crate::components::*;

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
            TileType::Ground => world.add_component(
                tile, 
                (
                    TileSmell { time_left: 0.0 },
                ),
            ),
        }
    }

    world.add_entity(storage)
}

#[derive(Unique)]
pub struct Game {
    pub do_ai: bool,
    pub player: EntityId,
    pub _boxes: [EntityId; 4],
    pub _tilemap: EntityId,
    pub goal_achieved: bool,
    camera: Camera2D,
}

impl Game {
    fn spawn_bullet(pos: Vec2, world: &mut World) {
        world.add_entity((
            Transform { pos, angle: 0.0 },
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
            Transform { pos, angle: 0.0 },
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
            Transform { pos, angle: 0.0 },
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

    pub fn new(world: &mut World) -> Self {
        let mut angle = 0.0;
        let poses = [
            vec2(200.0, 160.0),
            vec2(64.0, 250.0),
            vec2(128.0, 150.0),
            vec2(300.0, 250.0),
        ];
        let boxes = poses.map(|pos| {
            angle += 0.2;
            let the_box = world.add_entity((
                Transform { pos, angle },
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

            the_box
        });

        for x in 0..8 {
            for y in 0..8 {
                let pos = vec2(x as f32 * 12.0 + 100.0, y as f32 * 12.0 + 200.0);

                // if x < 5 {
                    crate::enemy::spawn_brute(pos, world);
                // } else {
                    // crate::enemy::spawn_stalker(pos, world);
                // }
            }
        }

        crate::enemy::spawn_main_cell(vec2(64.0, 128.0), world);

        /* Uncomment for a tough topology */
        // for x in 0..5 {
        //     for y in 0..5 {
        //         let a = x as f32 + y as f32 * 2.0;
        //         let (dx, dy) = a.sin_cos();
        //         let pos = vec2(
        //             x as f32 * 8.0 * 4.0 + 100.0 + dx * 13.0,
        //             y as f32 * 3.0 + 200.0 + dy * 7.0,
        //         );

        //         Self::spawn_brute(pos, world);
        //     }
        // }

        Self::spawn_bullet(vec2(100.0, 100.0), world);

        let tilemap = spawn_tiles(
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

        Self {
            do_ai: true,
            player: crate::player::spawn_player(world),
            _boxes: boxes,
            _tilemap: tilemap,
            goal_achieved: false,
            camera: Camera2D::default(),
        }
    }

    pub fn should_ai(this: UniqueView<Game>) -> bool {
        this.do_ai
    }

    pub fn mouse_pos(&self) -> Vec2 {
        let (mx, my) = mouse_position();
        self.camera.screen_to_world(vec2(mx, my))
    }

    pub fn update_camera(mut this: UniqueViewMut<Game>, tile_storage: View<TileStorage>) {
        let view_height = 19.0 * 32.0;
        let view_width = (screen_width() / screen_height()) * view_height;
        this.camera = Camera2D::from_display_rect(Rect {
            x: 0.0,
            y: 0.0,
            w: view_width,
            h: view_height,
        });
        this.camera.zoom.y *= -1.0;

        // FIXME: macroquad's camera is super confusing. Just like this math
        for storage in tile_storage.iter() {
            this.camera.target = vec2(
                (0.5 * 32.0) * (storage.width() as f32),
                (0.5 * 32.0) * (storage.height() as f32),
            );
        }
    }

    pub fn camera(&self) -> &Camera2D {
        &self.camera
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
}

pub fn decide_next_state(
    game: UniqueView<Game>,
    player: View<PlayerTag>,
    health: View<Health>,
) -> Option<AppState> {
    let player_dead = (&player, &health).iter().all(|(_, hp)| hp.0 <= 0);

    if player_dead {
        return Some(AppState::GameOver);
    }

    if game.goal_achieved {
        return Some(AppState::Win);
    }

    None
}
