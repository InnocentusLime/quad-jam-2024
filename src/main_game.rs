use crate::prelude::*;

pub struct MainGame {
    do_ai: bool,
    do_player_controls: bool,
}

impl MainGame {
    pub fn make_state_request() -> StateRequest {
        StateRequest {
            name: "main game",
            constructor: Box::new(Self::new_state),
            dependencies: vec!["prefab/player.json".into()],
        }
    }

    pub fn new_state(_resources: &mut Resources) -> Box<dyn State> {
        Box::new(MainGame { do_player_controls: true, do_ai: true })
    }
}

impl State for MainGame {
    fn handle_command(&mut self, _resources: &mut Resources, cmd: &DebugCommand) -> bool {
        match cmd.command.as_str() {
            "nopl" => self.do_player_controls = false,
            "pl" => self.do_player_controls = true,
            "noai" => self.do_ai = false,
            "ai" => self.do_ai = true,
            _ => return false,
        }
        true
    }

    fn plan_collision_queries(
        &mut self,
        _dt: f32,
        _resources: &mut Resources,
        _cmds: &mut CommandBuffer,
    ) {
    }

    fn update(
        &mut self,
        _dt: f32,
        _resources: &mut Resources,
        _collisions: &CollisionSolver,
        _cmds: &mut CommandBuffer,
    ) -> Option<StateRequest> {
        None
    }

    fn input(
        &mut self,
        dt: f32,
        input_model: &InputModel,
        resources: &mut Resources,
        cmds: &mut CommandBuffer,
    ) {
        for (_, (tf, tag)) in &mut resources.world.query::<(&mut Transform, &PlayerTag)>() {
            tf.pos += 13.0 * dt * input_model.player_move_direction;
            let pos = tf.pos + vec2(32.0, 0.0);

            if input_model.shoot_pressed {
                info!("shoot");
                spawn_prefab(cmds, resources, tag.bullet_prefab, Transform::from_pos(pos));
            }
        }

        for (_, tf) in &mut resources
            .world
            .query::<&mut Transform>()
            .with::<&BulletTag>()
        {
            tf.pos += 34.0 * dt * Vec2::X;
        }
    }
}
