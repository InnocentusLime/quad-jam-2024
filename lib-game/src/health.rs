use crate::{CollisionSolver, DamageCooldown, Health, Team, col_query};

use hecs::{CommandBuffer, World};

pub fn reset(world: &mut World) {
    for (_, hp) in world.query_mut::<&mut Health>() {
        hp.is_invulnerable = false;
        hp.damage = 0;
    }
}

pub fn update_cooldown(dt: f32, world: &mut World) {
    for (_, (cooldown, hp)) in world.query_mut::<(&mut DamageCooldown, &mut Health)>() {
        hp.is_invulnerable = hp.is_invulnerable || cooldown.remaining > 0.0;
        if cooldown.remaining > 0.0 {
            cooldown.remaining -= dt;
        }
    }
}

pub fn apply_cooldown(world: &mut World) {
    for (_, (cooldown, hp)) in world.query_mut::<(&mut DamageCooldown, &mut Health)>() {
        if hp.damage > 0 && !hp.is_invulnerable {
            cooldown.remaining = cooldown.max_value;
        }
    }
}

pub fn apply_damage(world: &mut World) {
    for (_, hp) in world.query_mut::<&mut Health>() {
        if hp.is_invulnerable {
            continue;
        }

        hp.value -= hp.damage;
    }
}

pub fn collect_damage(world: &mut World, col_solver: &CollisionSolver) {
    let mut hp_query = world.query::<(&mut Health, &Team)>();
    let mut hp_query = hp_query.view();
    for (_, (damage_q, attack_team)) in &mut world.query::<(&col_query::Damage, &Team)>() {
        for entity in col_solver.collisions_for(damage_q) {
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

pub fn despawn_on_zero_health(world: &mut World, cmds: &mut CommandBuffer) {
    for (entity, health) in world.query_mut::<&Health>() {
        if health.value <= 0 {
            cmds.despawn(entity);
        }
    }
}
