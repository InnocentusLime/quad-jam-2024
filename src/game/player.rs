use hecs::EntityBuilder;

use super::prelude::*;

pub const PLAYER_SPEED: f32 = 48.0;
pub const PLAYER_DASH_SPEED: f32 = 432.0;
pub const PLAYER_SPAWN_HEALTH: i32 = 3;
pub const PLAYER_HIT_COOLDOWN: f32 = 1.0;
pub const PLAYER_SHAPE: Shape = Shape::Rect {
    width: 16.0,
    height: 16.0,
};

pub const PLAYER_MAX_STAMINA: f32 = 100.0;
pub const PLAYER_STAMINA_REGEN_RATE: f32 = 20.0;
pub const PLAYER_STAMINA_REGEN_COOLDOWN: f32 = 0.8;
pub const PLAYER_ATTACK_COST: f32 = 10.0;
pub const PLAYER_DASH_COST: f32 = 25.0;

pub fn spawn(world: &mut World, pos: Vec2) {
    let mut builder = EntityBuilder::new();
    builder.add_bundle(CharacterBundle::new_player(
        pos,
        PLAYER_SHAPE,
        PLAYER_SPAWN_HEALTH,
    ));
    builder.add_bundle((
        PlayerData {
            state: PlayerState::Idle,
            stamina: PLAYER_MAX_STAMINA,
            stamina_cooldown: 0.0,
        },
        DamageCooldown::new(PLAYER_HIT_COOLDOWN),
    ));
    world.spawn(builder.build());
}

pub fn controls(dt: f32, input: &InputModel, world: &mut World, resources: &Resources) {
    let mut walk_dir = Vec2::ZERO;
    let mut do_walk = false;
    if input.left_movement_down {
        walk_dir += vec2(-1.0, 0.0);
        do_walk = true;
    }
    if input.up_movement_down {
        walk_dir += vec2(0.0, -1.0);
        do_walk = true;
    }
    if input.right_movement_down {
        walk_dir += vec2(1.0, 0.0);
        do_walk = true;
    }
    if input.down_movement_down {
        walk_dir += vec2(0.0, 1.0);
        do_walk = true;
    }
    walk_dir = walk_dir.normalize_or_zero();

    for_each_character::<&mut PlayerData>(world, resources, |_, mut character| {
        let look_dir = (input.aim - character.pos()).normalize_or(vec2(0.0, 1.0));
        character.set_walk_step(Vec2::ZERO);
        let (allow_walk_input, allow_look_input) = character.get_input_flags();
        if matches!(
            character.get_state(),
            PlayerState::Idle | PlayerState::Walking
        ) && input.attack_down
            && character.data.can_do_action(PLAYER_ATTACK_COST)
        {
            character.set_state(PlayerState::Attacking);
            character.data.do_action(PLAYER_ATTACK_COST);
        } else if matches!(
            character.get_state(),
            PlayerState::Idle | PlayerState::Walking
        ) && input.dash_pressed
            && character.data.can_do_action(PLAYER_DASH_COST)
        {
            character.set_state(PlayerState::Dashing);
            character.data.do_action(PLAYER_DASH_COST);
        } else if !matches!(character.get_state(), PlayerState::Walking)
            && allow_walk_input
            && do_walk
        {
            character.set_state(PlayerState::Walking);
        } else if matches!(character.get_state(), PlayerState::Walking) && !do_walk {
            character.set_state(PlayerState::Idle);
        }

        if matches!(character.get_state(), PlayerState::Walking) {
            character.set_walk_step(walk_dir * PLAYER_SPEED * dt);
        } else if matches!(character.get_state(), PlayerState::Dashing) {
            character.set_walk_step(character.look_direction() * PLAYER_DASH_SPEED * dt);
        }
        if allow_look_input {
            character.set_look_direction(look_dir);
        }
    });
}

pub fn update_stamina(dt: f32, world: &mut World) {
    for (_, data) in world.query_mut::<&mut PlayerData>() {
        if data.stamina_cooldown >= 0.0 {
            data.stamina_cooldown -= dt;
            continue;
        }
        data.stamina += dt * PLAYER_STAMINA_REGEN_RATE;
        data.stamina = data.stamina.min(PLAYER_MAX_STAMINA);
    }
}

impl CharacterData for &mut PlayerData {
    type StateId = PlayerState;

    fn get_state(&self) -> Self::StateId {
        self.state
    }

    fn set_state(&mut self, new_state: Self::StateId) {
        self.state = new_state
    }

    fn state_to_anim(character: &Character<Self>) -> AnimationId {
        match (character.get_state(), character.look_dir_enum()) {
            (PlayerState::Idle, Direction::Right) => AnimationId::BunnyIdleR,
            (PlayerState::Idle, Direction::Down) => AnimationId::BunnyIdleD,
            (PlayerState::Idle, Direction::Left) => AnimationId::BunnyIdleL,
            (PlayerState::Idle, Direction::Up) => AnimationId::BunnyIdleU,
            (PlayerState::Walking, Direction::Right) => AnimationId::BunnyWalkR,
            (PlayerState::Walking, Direction::Down) => AnimationId::BunnyWalkD,
            (PlayerState::Walking, Direction::Left) => AnimationId::BunnyWalkL,
            (PlayerState::Walking, Direction::Up) => AnimationId::BunnyWalkU,
            (PlayerState::Attacking, _) => AnimationId::BunnyAttackD,
            (PlayerState::Dashing, _) => AnimationId::BunnyDash,
        }
    }

    fn on_anim_end(character: &mut Character<Self>) {
        match character.get_state() {
            PlayerState::Attacking | PlayerState::Dashing => character.set_state(PlayerState::Idle),
            _ => (),
        }
    }
}

impl PlayerData {
    fn can_do_action(&self, cost: f32) -> bool {
        self.stamina >= cost
    }

    fn do_action(&mut self, cost: f32) {
        self.stamina -= cost;
        self.stamina_cooldown = PLAYER_STAMINA_REGEN_COOLDOWN;
    }
}
