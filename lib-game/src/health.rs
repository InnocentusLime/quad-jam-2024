use crate::{col_query, components::*};

use hecs::World;

pub fn reset(world: &mut World) {
    for (_, hp) in world.query_mut::<&mut Health>() {
        hp.block_damage = false;
        hp.damage = 0;
    }
}

pub fn update_cooldown(dt: f32, world: &mut World) {
    for (_, (cooldown, hp)) in world.query_mut::<(&mut DamageCooldown, &mut Health)>() {
        hp.block_damage = hp.block_damage || cooldown.remaining > 0.0;
        if cooldown.remaining > 0.0 {
            cooldown.remaining -= dt;
        }
    }
}

pub fn apply_cooldown(world: &mut World) {
    for (_, (cooldown, hp)) in world.query_mut::<(&mut DamageCooldown, &mut Health)>() {
        if hp.damage > 0 && !hp.block_damage {
            cooldown.remaining = cooldown.max_value;
        }
    }
}

pub fn apply_damage(world: &mut World) {
    for (_, hp) in world.query_mut::<&mut Health>() {
        if hp.block_damage {
            continue;
        }

        hp.value -= hp.damage;
    }
}

pub fn collect_damage(world: &mut World) {
    let mut hp_query = world.query::<(&mut Health, &Team)>();
    let mut hp_query = hp_query.view();
    for (_, (damage_q, attack_team)) in &mut world.query::<(&col_query::Damage, &Team)>() {
        for entity in damage_q.collisions() {
            let Some((health, target_team)) = hp_query.get_mut(*entity) else {
                continue;
            };
            if *attack_team == *target_team {
                continue;
            }
            health.damage += 1;
        }
    }
}
