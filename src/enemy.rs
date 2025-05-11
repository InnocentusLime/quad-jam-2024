use lib_game::*;
use crate::components::*;
use macroquad::prelude::*;
use crate::game::Game;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

pub const BRUTE_SPAWN_HEALTH: i32 = 2;
pub const BRUTE_GROUP_FORCE: f32 = 0.01 * 22.0;
pub const BRUTE_CHASE_FORCE: f32 = 900.0;
pub const REWARD_PER_ENEMY: u32 = 10;
pub const BRAIN_TARGET_SPEED: f32 = 130.0;
    
pub fn spawn_brute(pos: Vec2, world: &mut World) {
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
        ImpulseApplier { impulse: Vec2::ZERO },
        DamageTag,
    ));
}

pub fn spawn_stalker(pos: Vec2, world: &mut World) {
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

pub fn spawn_main_cell(pos: Vec2, world: &mut World) {
    world.add_unique(SwarmBrain::Walk { 
        dir: Vec2::ZERO,
        think: 0.0,
    });
    
    let _brute = world.add_entity((
        Transform { pos, angle: 0.0 },
        RewardInfo {
            state: RewardState::Locked,
            amount: REWARD_PER_ENEMY,
        },
        MainCellTag,
        EnemyState::Free,
        Health(BRUTE_SPAWN_HEALTH),
        BodyTag::new(
            InteractionGroups {
                memberships: groups::NPCS,
                filter: groups::NPCS_INTERACT,
            },
            ColliderTy::Circle { radius: 12.0 },
            1000.0,
            true,
            BodyKind::Dynamic,
        ),
        ImpulseApplier { impulse: Vec2::ZERO },
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

pub fn main_cell_ai(
    brain: UniqueView<SwarmBrain>,
    main_tag: View<MainCellTag>,
    // player_tag:  View<PlayerTag>,
    mut pos: ViewMut<Transform>,
    state: View<EnemyState>,
    mut impulse: ViewMut<ImpulseApplier>,
) {
    let dir = match &*brain {
        SwarmBrain::Walk { dir, .. } => *dir,
        _ => return,
    };
    // let target_tf = (&pos, &player_tag).iter()
    //     .next().unwrap();
    // let target_pos = target_tf.0.pos;
    
    for (enemy_tf, _, enemy_state, impulse) in (&mut pos, &main_tag, &state, &mut impulse).iter() {
        if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
            continue;
        }

        // let dr = target_pos - enemy_tf.pos;
        impulse.impulse += dir.normalize_or_zero() * 1600.0;
    }
}

pub fn update_brain(
    dt: f32,
    pos: View<Transform>,
    player_tag: View<PlayerTag>,
    main_tag: View<MainCellTag>,
    mut brain: UniqueViewMut<SwarmBrain>,
) {
    let target_tf = (&pos, &player_tag).iter()
        .next().unwrap();
    let this_tf = (&pos, &main_tag).iter()
        .next().unwrap();
    let target_pos = target_tf.0.pos;
    let this_pos = this_tf.0.pos;

    match &mut *brain {
        SwarmBrain::Wait { think } if *think <= 0.0 => {
            let dr = target_pos - this_pos;
            *brain = SwarmBrain::Walk { 
                think: 2.0, 
                dir: dr.normalize_or_zero(), 
            };
        },
        SwarmBrain::Wait { think } => {
            *think -= dt;
        }
        SwarmBrain::Walk { think, .. } if *think <= 0.0 => {
            *brain = SwarmBrain::Wait { think: 3.0 };
        },
        SwarmBrain::Walk { think, .. } => {
            *think -= dt;
        },
        _ => (),
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
        impulse.impulse += dr.normalize_or_zero() * 10.0;
    }
}

pub fn stalker_ai(
    brain: UniqueView<SwarmBrain>,
    brute_tag: View<StalkerTag>,
    pos: View<Transform>,
    state: View<EnemyState>,
    mut force: ViewMut<ForceApplier>,
    tile_storage: View<TileStorage>,
    smell: View<TileSmell>,
) {
    // let Some(storage) = tile_storage.iter().next() else {
    //     return;
    // };
    // let target = match &*brain {
    //     SwarmBrain::Chase { pos } => *pos,
    //     SwarmBrain::Panic { .. } => return,
    // };

    // for (enemy_tf, _, enemy_state, force) in (&pos, &brute_tag, &state, &mut force).iter() {
    //     if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
    //         continue;
    //     }

    //     let mut found = false;
    //     let sx = (enemy_tf.pos.x / 32.0) as usize;
    //     let sy = (enemy_tf.pos.y / 32.0) as usize;
    //     'outer: for sx in ((sx.saturating_sub(1))..(sx+1)) {
    //         for sy in ((sy.saturating_sub(1))..(sy+1)) {
    //             found = sample_spot(storage, &smell, sx, sy);
    //             if found { break 'outer; }
    //         }
    //     }

    //     if found { continue; }

    //     let dr = target - enemy_tf.pos;
    //     force.force += dr.normalize_or_zero() * BRUTE_CHASE_FORCE;
    // }
}

fn sample_spot(
    storage: &TileStorage,
    smell: &View<TileSmell>,
    sx: usize,
    sy: usize
) -> bool {
    let sample = storage.get(sx, sy)
        .and_then(|x| smell.get(x).ok())
        .map(|x| *x);

    if let Some(sample) = sample {
        if sample.time_left <= 2.5 && sample.time_left >= 1.0 {
            return true;
        }
    }    

    false
}

// pub fn brute_ai(
//     dt: f32,
//     this: UniqueView<Game>,
//     mut know: UniqueViewMut<SwarmBrain>,
//     brute_tag: View<BruteTag>,
//     pos: View<Transform>,
//     state: View<EnemyState>,
//     mut force: ViewMut<ForceApplier>,
// ) {
//     let player_pos = pos.get(this.player).unwrap().pos;

//     for (enemy_tf, _, enemy_state, force) in (&pos, &brute_tag, &state, &mut force).iter() {
//         if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
//             continue;
//         }

//         for (fella_tf, _, fella_state) in (&pos, &brute_tag, &state).iter() {
//             if !matches!(fella_state, EnemyState::Free | EnemyState::Stunned { .. }) {
//                 continue;
//             }

//             let dr = fella_tf.pos - enemy_tf.pos;

//             force.force += dr * BRUTE_GROUP_FORCE;
//         }

//         let dr = player_pos - enemy_tf.pos;

//         if know.last_hit.anger_time <= 0.0 {
//             force.force += dr.normalize_or_zero() * BRUTE_CHASE_FORCE;
//         } else {
//             force.force += know.last_hit.anger_dir * (BRUTE_CHASE_FORCE * 3.0);
//         }
//     }
            
//     if know.last_hit.anger_time > 0.0 {
//         know.last_hit.anger_time -= dt;
//     }
// }