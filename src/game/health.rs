use super::prelude::*;

pub fn update_cooldown(dt: f32, world: &mut World) {
    for (_, (cooldown, hp)) in world.query_mut::<(&mut DamageCooldown, &mut Health)>() {
        if hp.value <= 0 {
            continue;
        }

        hp.block_damage = cooldown.remaining > 0.0;
        if cooldown.remaining > 0.0 {
            cooldown.remaining -= dt;
            continue;
        }

        if hp.damage > 0 {
            cooldown.remaining = cooldown.max_value;
        }
    }
}

pub fn apply_damage(world: &mut World) {
    for (_, hp) in world.query_mut::<&mut Health>() {
        let damage = std::mem::replace(&mut hp.damage, 0);
        if hp.block_damage {
            continue;
        }

        hp.value -= damage;
    }
}

pub fn collect_damage(world: &mut World) {
    let mut hp_query = world.query::<&mut Health>();
    let mut hp_query = hp_query.view();
    for (_, damage_q) in &mut world.query::<&col_query::Damage>() {
        for entity in damage_q.collisions() {
            let Some(health) = hp_query.get_mut(*entity) else {
                continue;
            };
            health.damage += 1;
        }
    }
}
