use lib_game::*;
use crate::components::*;
use macroquad::prelude::*;
use crate::game::Game;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

pub const BRUTE_SPAWN_HEALTH: i32 = 2;
pub const BRUTE_GROUP_FORCE: f32 = 0.01 * 22.0;
pub const BRUTE_CHASE_FORCE: f32 = 40.0 * 24.0;
pub const REWARD_PER_ENEMY: u32 = 10;
    
pub fn spawn_brute(pos: Vec2, world: &mut World) {
    world.add_unique(SwarmKnowledge {
        last_hit: SwarmLastHit { 
            anger_time: 0.0, 
            anger_dir: Vec2::ZERO,
        },
    });
    
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
        ForceApplier { force: Vec2::ZERO },
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

pub fn enemy_state_data(mut rbs: ViewMut<BodyTag>, mut enemy: ViewMut<EnemyState>) {
    for (rb, enemy) in (&mut rbs, &mut enemy).iter() {
        match enemy {
            EnemyState::Free => {
                rb.enabled = true;
                rb.groups = InteractionGroups {
                    memberships: groups::NPCS,
                    filter: groups::NPCS_INTERACT,
                };
            }
            EnemyState::Stunned { .. } => {
                rb.enabled = true;
                rb.groups = InteractionGroups {
                    memberships: groups::NPCS,
                    filter: groups::NPCS_INTERACT,
                };
            }
            EnemyState::Dead => {
                rb.enabled = false;
                rb.groups = InteractionGroups {
                    memberships: groups::NPCS,
                    filter: groups::NPCS_INTERACT,
                };
            }
        }
    }
}

pub fn brute_ai(
    dt: f32,
    this: UniqueView<Game>,
    mut know: UniqueViewMut<SwarmKnowledge>,
    brute_tag: View<BruteTag>,
    pos: View<Transform>,
    state: View<EnemyState>,
    mut force: ViewMut<ForceApplier>,
) {
    let player_pos = pos.get(this.player).unwrap().pos;

    for (enemy_tf, _, enemy_state, force) in (&pos, &brute_tag, &state, &mut force).iter() {
        if !matches!(enemy_state, EnemyState::Free | EnemyState::Stunned { .. }) {
            continue;
        }

        for (fella_tf, _, fella_state) in (&pos, &brute_tag, &state).iter() {
            if !matches!(fella_state, EnemyState::Free | EnemyState::Stunned { .. }) {
                continue;
            }

            let dr = fella_tf.pos - enemy_tf.pos;

            force.force += dr * BRUTE_GROUP_FORCE;
        }

        let dr = player_pos - enemy_tf.pos;

        if know.last_hit.anger_time <= 0.0 {
            force.force += dr.normalize_or_zero() * BRUTE_CHASE_FORCE;
        } else {
            force.force += know.last_hit.anger_dir * (BRUTE_CHASE_FORCE * 3.0);
        }
    }
            
    if know.last_hit.anger_time > 0.0 {
        know.last_hit.anger_time -= dt;
    }
}