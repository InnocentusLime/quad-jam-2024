use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2, resources: &Resources) {
    builder.add_bundle(CharacterBundle::new_player(
        pos,
        resources.cfg.player.shape,
        resources.cfg.player.max_hp,
    ));
    builder.add_bundle((
        PlayerState::Idle,
        DamageCooldown::new(resources.cfg.player.hit_cooldown),
        GrazeGain {
            value: 0.0,
            max_value: resources.cfg.player.max_stamina,
        },
        col_query::Grazing::new(
            resources.cfg.player.graze_shape,
            col_group::ATTACKS,
            col_group::NONE,
        ),
    ));
}

pub fn controls(dt: f32, input: &InputModel, world: &mut World, resources: &Resources) {
    let cfg = &resources.cfg;
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

    for_each_character::<PlayerData>(world, resources, |_, mut c| {
        let look_dir = (input.aim - c.pos()).normalize_or(vec2(0.0, 1.0));

        if input.attack_down && can_attack(&c, cfg) {
            c.set_state(PlayerState::Attacking);
            c.data.substract_stamina(cfg.player.attack_cost);
        } else if input.dash_pressed && can_dash(&c, cfg) {
            c.set_state(PlayerState::Dashing);
            c.data.substract_stamina(cfg.player.dash_cost);
        } else if do_walk && can_walk(&c) {
            c.set_state(PlayerState::Walking);
        } else if !do_walk && matches!(c.get_state(), PlayerState::Walking) {
            c.set_state(PlayerState::Idle);
        }

        c.set_walk_step(Vec2::ZERO);
        match c.get_state() {
            PlayerState::Walking => c.set_walk_step(walk_dir * cfg.player.speed * dt),
            PlayerState::Dashing => {
                c.set_walk_step(c.look_direction() * cfg.player.dash_speed * dt)
            }
            _ => (),
        }

        if c.get_input_flags().1 {
            c.set_look_direction(look_dir);
        }
    });
}

fn can_attack(c: &Character<PlayerData>, cfg: &GameCfg) -> bool {
    matches!(c.get_state(), PlayerState::Idle | PlayerState::Walking)
        && c.data.can_do_action(cfg.player.attack_cost)
}

fn can_dash(c: &Character<PlayerData>, cfg: &GameCfg) -> bool {
    matches!(c.get_state(), PlayerState::Idle | PlayerState::Walking)
        && c.data.can_do_action(cfg.player.dash_cost)
}

fn can_walk(c: &Character<PlayerData>) -> bool {
    let (allow_walk_input, _) = c.get_input_flags();
    !matches!(c.get_state(), PlayerState::Walking) && allow_walk_input
}

impl CharacterData for PlayerData<'_> {
    type StateId = PlayerState;

    fn get_state(&self) -> Self::StateId {
        *self.state
    }

    fn set_state(&mut self, new_state: Self::StateId) {
        *self.state = new_state
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

#[derive(Query)]
pub struct PlayerData<'a> {
    pub state: &'a mut PlayerState,
    pub graze_gain: &'a mut GrazeGain,
}

impl<'a> PlayerData<'a> {
    fn can_do_action(&self, cost: f32) -> bool {
        self.graze_gain.value >= cost
    }

    fn substract_stamina(&mut self, cost: f32) {
        self.graze_gain.value -= cost;
        self.graze_gain.value = self.graze_gain.value.max(0.0);
    }
}
