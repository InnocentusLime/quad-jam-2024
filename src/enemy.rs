use crate::components::*;
use lib_game::*;
use macroquad::prelude::*;
use quad_dbg::dump;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

pub const BRUTE_SPAWN_HEALTH: i32 = 2;
pub const REWARD_PER_ENEMY: u32 = 10;
pub const MAIN_CELL_IMPULSE: f32 = 3000.0;
pub const BRUTE_GROUP_IMPULSE: f32 = 20.0;
pub const MAIN_CELL_DIR_ADJUST_SPEED: f32 = std::f32::consts::PI / 20.0;
pub const MAIN_CELL_WALK_TIME: f32 = 2.0;

pub fn spawn_brute(world: &mut World, pos: Vec2) {
    let _brute = world.add_entity((
        Transform { pos, angle: 0.0 },
        RewardInfo {
            state: RewardState::Locked,
            amount: REWARD_PER_ENEMY,
        },
        BruteTag,
        EnemyState::Free,
        Health(BRUTE_SPAWN_HEALTH),
        BodyTag::new(
            InteractionGroups {
                memberships: groups::NPCS,
                filter: groups::NPCS_INTERACT,
            },
            ColliderTy::Circle { radius: 6.0 },
            5.0,
            true,
            BodyKind::Dynamic,
        ),
        ImpulseApplier {
            impulse: Vec2::ZERO,
        },
        DamageTag,
    ));
}

#[allow(dead_code)]
pub fn spawn_stalker(world: &mut World, pos: Vec2) {
    let _brute = world.add_entity((
        Transform { pos, angle: 0.0 },
        RewardInfo {
            state: RewardState::Locked,
            amount: REWARD_PER_ENEMY,
        },
        StalkerTag,
        EnemyState::Free,
        Health(BRUTE_SPAWN_HEALTH),
        BodyTag::new(
            InteractionGroups {
                memberships: groups::NPCS,
                filter: groups::NPCS_INTERACT,
            },
            ColliderTy::Circle { radius: 6.0 },
            2.0,
            true,
            BodyKind::Dynamic,
        ),
        ForceApplier { force: Vec2::ZERO },
        DamageTag,
    ));
}

pub fn spawn_main_cell(world: &mut World, pos: Vec2) {
    let _brute = world.add_entity((
        Transform { pos, angle: 0.0 },
        RewardInfo {
            state: RewardState::Locked,
            amount: REWARD_PER_ENEMY,
        },
        MainCellTag::Wait {
            think: 2.0,
        },
        EnemyState::Free,
        Health(5),
        BodyTag::new(
            InteractionGroups {
                memberships: groups::NPCS.union(groups::MAINCELL),
                filter: groups::NPCS_INTERACT,
            },
            ColliderTy::Circle { radius: 12.0 },
            1000.0,
            true,
            BodyKind::Dynamic,
        ),
        ImpulseApplier {
            impulse: Vec2::ZERO,
        },
        DamageTag,
    ));
}

pub fn enemy_states(dt: f32, mut enemy: ViewMut<EnemyState>, mut hp: ViewMut<Health>) {
    for (enemy, hp) in (&mut enemy, &mut hp).iter() {
        match enemy {
            EnemyState::Stunned { left } => {
                *left -= dt;
                if *left < 0.0 {
                    hp.0 -= 1;
                    *enemy = EnemyState::Free;
                }
            }
            EnemyState::Free => {
                if hp.0 <= 0 {
                    *enemy = EnemyState::Dead;
                }
            }
            _ => (),
        }
    }
}

pub fn cell_phys_data(mut rbs: ViewMut<BodyTag>, mut enemy: ViewMut<EnemyState>) {
    for (rb, enemy) in (&mut rbs, &mut enemy).iter() {
        if matches!(enemy, EnemyState::Dead) {
            rb.enabled = false;
        }
    }
}

pub fn brute_ai(
    main_tag: View<MainCellTag>,
    brute_tag: View<BruteTag>,
    pos: View<Transform>,
    state: View<EnemyState>,
    mut impulse: ViewMut<ImpulseApplier>,
) {
    let target = match (&pos, &main_tag).iter().next() {
        Some((x, _)) => x.pos,
        _ => return,
    };

    for (enemy_tf, _, enemy_state, impulse) in (&pos, &brute_tag, &state, &mut impulse).iter() {
        if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
            continue;
        }

        let dr = target - enemy_tf.pos;
        // let k = (dr.length() / 64.0).powf(1.4);
        impulse.impulse += dr.normalize_or_zero() * BRUTE_GROUP_IMPULSE;
    }
}

pub fn stalker_ai(
    _brute_tag: View<StalkerTag>,
    _pos: View<Transform>,
    _state: View<EnemyState>,
    _force: ViewMut<ForceApplier>,
    _tile_storage: View<TileStorage>,
    _smell: View<TileSmell>,
) {
}

#[allow(dead_code)]
fn sample_spot(storage: &TileStorage, smell: &View<TileSmell>, sx: usize, sy: usize) -> bool {
    let sample = storage
        .get(sx, sy)
        .and_then(|x| smell.get(x).ok())
        .map(|x| *x);

    if let Some(sample) = sample {
        if sample.time_left <= 2.5 && sample.time_left >= 1.0 {
            return true;
        }
    }

    false
}

pub fn main_cell_ai(
    dt: f32,
    pos: View<Transform>,
    player_tag: View<PlayerTag>,
    mut main_tag: ViewMut<MainCellTag>,
    state: View<EnemyState>,
    mut impulse: ViewMut<ImpulseApplier>,
) {
    let target_tf = (&pos, &player_tag).iter().next().unwrap();
    let target_pos = target_tf.0.pos;

    for (this_tf, main_tag, enemy_state, impulse) in (&pos, &mut main_tag, &state, &mut impulse).iter() {
        if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
            continue;
        }

        let this_pos = this_tf.pos;
        let real_dir = (target_pos - this_pos).normalize_or_zero();
        *main_tag = match *main_tag {
            MainCellTag::Wait { think } if think <= 0.0 =>  MainCellTag::Walk {
                think: MAIN_CELL_WALK_TIME,
                dir: (target_pos - this_pos).normalize_or_zero(),
            },
            MainCellTag::Walk { think, .. } if think <= 0.0 => MainCellTag::Wait {
                think: 3.0,
            },
            MainCellTag::Wait { think } => MainCellTag::Wait {
                think: think - dt,
            },
            MainCellTag::Walk { think, dir } => MainCellTag::Walk {
                think: think - dt,
                dir: if think < 0.2 * MAIN_CELL_WALK_TIME {
                    dir
                } else {
                    real_dir
                },
            }
        };
        let MainCellTag::Walk { dir, .. } = *main_tag
            else { continue; };

        impulse.impulse += dir.normalize_or_zero() * MAIN_CELL_IMPULSE;
    }
}