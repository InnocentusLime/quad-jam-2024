use super::prelude::*;

pub const SHOOTER_SPAWN_HEALTH: i32 = 3;
pub const SHOOTER_HIT_COOLDOWN: f32 = 3.0;
pub const SHOOTER_SHAPE: Shape = Shape::Rect {
    width: 16.0,
    height: 16.0,
};

impl CharacterData for &mut ShooterState {
    type StateId = ShooterState;

    fn get_state(&self) -> Self::StateId {
        **self
    }
    fn set_state(&mut self, new_state: Self::StateId) {
        **self = new_state
    }
    fn state_to_anim(character: &Character<Self>) -> AnimationId {
        match character.get_state() {
            ShooterState::Idle => AnimationId::ShooterIdle,
            ShooterState::Attacking => AnimationId::ShooterAttack,
        }
    }
    fn on_anim_end(character: &mut Character<Self>) {
        if character.get_state() == ShooterState::Attacking {
            character.set_state(ShooterState::Idle);
        }
    }
}

pub fn init(builder: &mut EntityBuilder, pos: Vec2) {
    builder.add_bundle(CharacterBundle::new_enemy(
        pos,
        SHOOTER_SHAPE,
        SHOOTER_SPAWN_HEALTH,
    ));
    builder.add_bundle((
        DamageCooldown::new(SHOOTER_HIT_COOLDOWN),
        ShooterState::Idle,
    ));
}

pub fn ai(_dt: f32, world: &mut World, resources: &Resources) {
    let Some((_, (player_tf, _))) = world
        .query_mut::<(&Transform, &PlayerState)>()
        .into_iter()
        .next()
    else {
        return;
    };
    let player_tf = *player_tf;

    for_each_character::<&mut ShooterState>(world, resources, |_, mut character| {
        let off_to_player = player_tf.pos - character.pos();
        let dir = off_to_player.normalize_or(Vec2::Y);

        character.set_walk_step(Vec2::ZERO);
        if character.get_state() == ShooterState::Idle {
            character.set_look_direction(dir);
            character.set_state(ShooterState::Attacking);
        }
    });
}

pub fn die_on_zero_health(world: &mut World, cmds: &mut CommandBuffer) {
    for (entity, (health, _)) in world.query_mut::<(&Health, &ShooterState)>() {
        if health.value > 0 {
            continue;
        }
        cmds.despawn(entity);
    }
}
