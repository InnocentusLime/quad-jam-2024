use crate::components::*;
use lib_game::*;
use macroquad::prelude::*;
use quad_dbg::dump;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

pub const BRUTE_SPAWN_HEALTH: i32 = 2;
pub const REWARD_PER_ENEMY: u32 = 10;
pub const MAIN_CELL_SPEED: f32 = 124.0;
pub const BRUTE_GROUP_IMPULSE: f32 = 12.0;
pub const MAIN_CELL_DIR_ADJUST_SPEED: f32 = std::f32::consts::PI / 20.0;
pub const MAIN_CELL_WALK_TIME: f32 = 2.0;
pub const MAIN_CELL_TARGET_NUDGE: f32 = 64.0;
pub const MAIN_CELL_WANDER_STEPS: u32 = 2;
pub const MAIN_CELL_ATTACK_IMPULSE: f32 = 1300.0;

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
        MainCellTag {
            state: MainCellState::Wait {
                think: 2.0,
                counter: None,
            },
            step: 0,
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
        VelocityProxy(Vec2::ZERO),
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

// FIXIME: boost brutes when the main cell is attacking
pub fn brute_ai(
    main_tag: View<MainCellTag>,
    brute_tag: View<BruteTag>,
    pos: View<Transform>,
    state: View<EnemyState>,
    mut impulse: ViewMut<ImpulseApplier>,
) {
    let (target, speedup) = match (&pos, &main_tag).iter().next() {
        Some((x, main)) => (x.pos, matches!(main.state, MainCellState::Walk { .. })),
        _ => return,
    };

    for (enemy_tf, _, enemy_state, impulse) in (&pos, &brute_tag, &state, &mut impulse).iter() {
        if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
            continue;
        }

        let dr = target - enemy_tf.pos;
        // let k = (dr.length() / 64.0).powf(1.4);
        impulse.impulse += dr.normalize_or_zero() * BRUTE_GROUP_IMPULSE;
        if speedup {
            impulse.impulse *= 2.0;
        }
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
    mut vel: ViewMut<VelocityProxy>,
) {
    let player_tf = (&pos, &player_tag).iter().next().unwrap();
    let player_pos = player_tf.0.pos;

    for (this_tf, main_tag, enemy_state, vel) in (&pos, &mut main_tag, &state, &mut vel).iter() {
        if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
            continue;
        }

        let this_pos = this_tf.pos;
        let player_dir = (player_pos - this_pos).normalize_or_zero();
        if matches!(main_tag.state, MainCellState::Wait { counter: None, .. }) {
            main_tag.step += 1;
        }
        main_tag.state = match main_tag.state {
            MainCellState::Wander { counter, target, .. } if this_pos.distance(target) <= 32.0 => MainCellState::Wait { 
                think: 0.4, 
                counter: Some(counter),
            },
            MainCellState::Wait { think, counter: None } if think <= 0.0 =>  MainCellState::Wander {
                target: pick_new_destination(this_pos, MAIN_CELL_WANDER_STEPS, main_tag.step),
                counter: counter_value(main_tag.step),
            },
            MainCellState::Wait { think, counter: Some(0) } if think <= 0.0 =>  MainCellState::Walk {
                think: MAIN_CELL_WALK_TIME,
                dir: (player_pos - this_pos).normalize_or_zero(),
            },
            MainCellState::Wait { think, counter: Some(n) } if think <= 0.0 =>  MainCellState::Wander {
                target: pick_new_destination(this_pos, n, main_tag.step),
                counter: n - 1,
            },
            MainCellState::Wait { think, counter: Some(n) } if think <= 0.0 =>  MainCellState::Wander {
                target: pick_new_destination(this_pos, n, main_tag.step),
                counter: n - 1,
            },
            MainCellState::Wander { target, counter } => MainCellState::Wander { 
                target: target, 
                counter,
            },
            // TODO: wait for collision instead
            MainCellState::Walk { think, .. } if think <= 0.0 => MainCellState::Wait {
                think: 1.0,
                counter: None,
            },
            // TODO: wait for cells to gather
            MainCellState::Wait { think, counter } => MainCellState::Wait {
                think: think - dt,
                counter,
            },
            MainCellState::Walk { think, dir } => MainCellState::Walk {
                think: think - dt,
                dir,
            }
        };
        let (dir, k) = match main_tag.state {
            MainCellState::Walk { dir, .. } => {
                (dir, 2.0)
            },
            MainCellState::Wander { target, .. } => {
                let dr = target - this_pos;
                let k = if dr.length() <= 64.0 {
                    ((dr.length() + 16.0) / 64.0).powf(2.0).min(1.0)
                } else {
                    1.0
                };
                (dr.normalize_or_zero(), k)
            },
            _ => {
                continue;
            },
        };

        // assert!(dir.length() <= 1.1, "{}");
        if k >= 1.0 {
            vel.0 += dir.normalize_or_zero() * 200.0 * dt * k;
            vel.0 = vel.0.clamp_length_max(MAIN_CELL_SPEED * k);
        }
    }
}

static FUZZ_TABLE: [i32; 13] = [
    -1,
    1,
    1,
    1,
    0,
    0,
    -1,
    1,
    1,
    -1,
    0,
    0,
    0,
];

fn counter_value(step: u32) -> u32 {
    ((MAIN_CELL_WANDER_STEPS as i32) + FUZZ_TABLE[step as usize % FUZZ_TABLE.len()]) as u32
}

fn pick_new_destination(main_pos: Vec2, counter: u32, step: u32) -> Vec2 {
    let poses = [
        vec2(MAIN_CELL_TARGET_NUDGE, MAIN_CELL_TARGET_NUDGE),
        vec2(16.0*32.0 - MAIN_CELL_TARGET_NUDGE, MAIN_CELL_TARGET_NUDGE),
        vec2(16.0*32.0 - MAIN_CELL_TARGET_NUDGE, 16.0*32.0 - MAIN_CELL_TARGET_NUDGE),
        vec2(MAIN_CELL_TARGET_NUDGE, 16.0*32.0 - MAIN_CELL_TARGET_NUDGE),
    ];
    let (idx, _) = poses.iter()
        .enumerate()
        .map(|(idx, pos)| (idx, pos.distance(main_pos)))
        .min_by(|(_, l), (_, r)| f32::total_cmp(l, r))
        .unwrap();
    let next_idx = if counter == counter_value(step) {
        idx
    } else if step % 2 == 0  {
        (idx + poses.len() - 1) % poses.len()
    } else {
        (idx + 1) % poses.len()
    };
    poses[next_idx]
}