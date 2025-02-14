use macroquad::prelude::*;
use shipyard::{EntityId, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut, World};
use crate::{inline_tilemap, method_as_system, physics::{physics_spawn, BodyKind, ColliderTy, PhysicsInfo, PhysicsState}, ui::UiModel, DeltaTime, Follower, Speed, TileStorage, TileType, Transform};

const PLAYER_SPEED_MAX: f32 = 128.0;
const PLAYER_ACC: f32 = 128.0;
pub const PLAYER_SPEED: f32 = 128.0;

fn spawn_walls(world: &mut World) {
    const WALL_THICK: f32 = 32.0;
    const WALL_SIDE: f32 = 480.0;

    let wall_data = [
        (WALL_SIDE / 2.0, WALL_SIDE - WALL_THICK / 2.0, WALL_SIDE, WALL_THICK),
        (WALL_SIDE / 2.0, WALL_THICK / 2.0, WALL_SIDE, WALL_THICK),
        (WALL_SIDE - WALL_THICK / 2.0, WALL_SIDE / 2.0, WALL_THICK, WALL_SIDE),
        (WALL_THICK / 2.0, WALL_SIDE / 2.0, WALL_THICK, WALL_SIDE),
    ];

    for (x, y, width, height) in wall_data {
        let wall = world.add_entity((
            Transform {
                pos: vec2(x, y),
                angle: 0.0f32,
            },
        ));
        physics_spawn(
            world,
            wall,
            ColliderTy::Box {
                width,
                height,
            },
            BodyKind::Static,
        );
    }
}

fn spawn_tiles(
    width: usize,
    height: usize,
    data: Vec<TileType>,
    world: &mut World,
) -> EntityId {
    assert_eq!(data.len(), width * height);

    let storage = TileStorage::from_data(
        width,
        height,
        data.into_iter()
            .map(|ty| world.add_entity(ty))
            .collect()
    ).unwrap();

    world.add_entity(storage)
}

#[derive(Unique)]
pub struct Game {
    player: EntityId,
    boxes: [EntityId; 4],
    tilemap: EntityId,
}

impl Game {
    pub fn new(world: &mut World) -> Self {
        let _follower = world.add_entity((
            Speed(Vec2::ZERO),
            Transform {
                pos: Vec2::ZERO,
                angle: 0.0f32,
            },
            Follower,
        ));
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
                Transform {
                    pos,
                    angle,
                },
            ));
            physics_spawn(
                world,
                the_box,
                ColliderTy::Box {
                    width: 32.0,
                    height: 32.0,
                },
                BodyKind::Dynamic,
            );

            the_box
        });

        spawn_walls(world);

        let player = world.add_entity(
            Transform {
                pos: vec2(300.0, 300.0),
                angle: 0.0,
            }
        );
        physics_spawn(
            world,
            player,
            ColliderTy::Box {
                width: 16.0,
                height: 16.0,
            },
            BodyKind::Kinematic,
        );

        let tilemap = spawn_tiles(
            16,
            16,
            inline_tilemap![
                w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, g, g, g, g, g, g, g, g, g, g, g, g, g, g, w,
                w, w, w, w, w, w, w, w, w, w, w, w, w, w, w, w
            ],
            world,
        );

        Self {
            player,
            boxes,
            tilemap,
        }
    }

    pub fn update_follower(
        &mut self,
        follow: View<Follower>,
        mut pos: ViewMut<Transform>,
        mut speed: ViewMut<Speed>,
        dt: UniqueView<DeltaTime>,
    ) {
        let dt = dt.0;
        // TODO: do not use here
        let (mx, my) = mouse_position();

        for (_, pos, speed) in (&follow, &mut pos, &mut speed).iter() {
            let dv = (vec2(mx, my) - pos.pos).normalize_or_zero();

            speed.0 += dv * PLAYER_ACC * dt;
            speed.0 = speed.0.clamp_length(0.0, PLAYER_SPEED_MAX);

            pos.pos += speed.0 * dt;
        }
    }

    pub fn player_controls(
        &mut self,
        mut phys: UniqueViewMut<PhysicsState>,
        mut rbs: ViewMut<PhysicsInfo>,
        dt: UniqueView<DeltaTime>,
        ui_model: UniqueView<UiModel>,
    ) {
        let mut dir = Vec2::ZERO;
        if ui_model.move_left() {
            dir += vec2(-1.0, 0.0);
        }
        if ui_model.move_up() {
            dir += vec2(0.0, -1.0);
        }
        if ui_model.move_right() {
            dir += vec2(1.0, 0.0);
        }
        if ui_model.move_down() {
            dir += vec2(0.0, 1.0);
        }

        phys.move_kinematic(
            &mut rbs,
            self.player,
            dir.normalize_or_zero() * dt.0 * PLAYER_SPEED,
        );
    }

    // Doesn't work because we end up doing 2 borrows
    // pub fn box_deleter(
    //     &mut self,
    //     mut stores: AllStoragesViewMut,
    // ) {
    //     let map = [
    //         (KeyCode::Key1, 0),
    //         (KeyCode::Key2, 1),
    //         (KeyCode::Key3, 2),
    //         (KeyCode::Key4, 3),
    //     ];

    //     for (key, idx) in map {
    //         if is_key_pressed(key) {
    //             stores.delete_entity(self.boxes[idx]);
    //         }
    //     }
    // }
}

method_as_system!(
    Game::update_follower as game_update_follower(
        this: Game,
        follow: View<Follower>,
        pos: ViewMut<Transform>,
        speed: ViewMut<Speed>,
        dt: UniqueView<DeltaTime>
    )
);

method_as_system!(
    Game::player_controls as game_player_controls(
        this: Game,
        phys: UniqueViewMut<PhysicsState>,
        rbs: ViewMut<PhysicsInfo>,
        dt: UniqueView<DeltaTime>,
        ui_model: UniqueView<UiModel>
    )
);

// method_as_system!(
//     Game::box_deleter as game_box_deleter(
//         this: Game,
//         stores: AllStoragesViewMut
//     )
// );