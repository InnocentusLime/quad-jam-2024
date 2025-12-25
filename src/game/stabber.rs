use hecs::EntityBuilder;

use super::prelude::*;

pub const STABBER_SPAWN_HEALTH: i32 = 3;
pub const STABBER_HIT_COOLDOWN: f32 = 3.0;
pub const STABBER_WALK_SPEED: f32 = 18.0;
pub const STABBER_AGRO_RANGE: f32 = 36.0;
pub const STABBER_SHAPE: Shape = Shape::Rect {
    width: 16.0,
    height: 16.0,
};

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

pub fn spawn(world: &mut World, pos: Vec2) {
    let mut builder = EntityBuilder::new();
    builder.add_bundle(CharacterBundle::new_enemy(
        pos,
        STABBER_SHAPE,
        STABBER_SPAWN_HEALTH,
    ));
    builder.add_bundle((
        DamageCooldown::new(STABBER_HIT_COOLDOWN),
        StabberState::Idle,
    ));
    world.spawn(builder.build());
}

pub fn ai(dt: f32, world: &mut World, resources: &Resources) {
    let Some((_, (player_tf, _))) = world
        .query_mut::<(&Transform, &PlayerData)>()
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
            character.set_walk_step(dir * STABBER_WALK_SPEED * dt);
            if off_to_player.length() <= STABBER_AGRO_RANGE {
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
