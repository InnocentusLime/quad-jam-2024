use super::prelude::*;

impl CharacterData for &mut StabberState {
    type StateId = StabberState;

    fn get_state(&self) -> Self::StateId {
        **self
    }
    fn set_state(&mut self, new_state: Self::StateId) {
        **self = new_state
    }
    fn state_to_anim(character: &Character<Self>) -> AnimationId {
        match character.get_state() {
            StabberState::Idle => AnimationId::StabberIdle,
            StabberState::Attacking => AnimationId::StabberAttack,
        }
    }
    fn on_anim_end(character: &mut Character<Self>) {
        if character.get_state() == StabberState::Attacking {
            character.set_state(StabberState::Idle);
        }
    }
}

pub fn init(builder: &mut EntityBuilder, pos: Vec2, resources: &Resources) {
    builder.add_bundle(CharacterBundle::new_enemy(
        pos,
        resources.cfg.stabber.shape,
        resources.cfg.stabber.max_hp,
    ));
    builder.add_bundle((
        DamageCooldown::new(resources.cfg.stabber.hit_cooldown),
        StabberState::Idle,
    ));
}

pub fn ai(dt: f32, world: &mut World, resources: &Resources) {
    let cfg = &resources.cfg;
    let Some((_, (player_tf, _))) = world
        .query_mut::<(&Transform, &PlayerState)>()
        .into_iter()
        .next()
    else {
        return;
    };
    let player_tf = *player_tf;

    for_each_character::<&mut StabberState>(world, resources, |_, mut character| {
        let off_to_player = player_tf.pos - character.pos();
        let dir = off_to_player.normalize_or(Vec2::Y);

        character.set_walk_step(Vec2::ZERO);
        if character.get_state() == StabberState::Idle {
            character.set_look_direction(dir);
            character.set_walk_step(dir * cfg.stabber.speed * dt);
            if off_to_player.length() <= cfg.stabber.attack_range {
                character.set_state(StabberState::Attacking);
            }
        }
    });
}

pub fn die_on_zero_health(world: &mut World, cmds: &mut CommandBuffer) {
    for (entity, (health, _)) in world.query_mut::<(&Health, &StabberState)>() {
        if health.value > 0 {
            continue;
        }
        cmds.despawn(entity);
    }
}
