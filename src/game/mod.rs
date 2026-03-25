mod prelude;

use glam::Vec2;
use log::{error, info};
use prelude::*;

use crate::{BulletTag, PlayerTag};

pub struct MainGame {
    do_ai: bool,
    do_player_controls: bool,
}

impl MainGame {
    pub fn new() -> MainGame {
        MainGame {
            do_player_controls: true,
            do_ai: true,
        }
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
        _resources: &mut lib_game::Resources,
        _cmds: &mut CommandBuffer,
    ) {
    }

    fn update(
        &mut self,
        _dt: f32,
        _resources: &mut lib_game::Resources,
        _collisions: &CollisionSolver,
        _cmds: &mut CommandBuffer,
    ) -> Option<Box<dyn State>> {
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
            let pos = tf.pos + glam::vec2(32.0, 0.0);

            if input_model.shoot_pressed {
                info!("shoot");
                spawn_prefab(cmds, resources, tag.bullet_prefab, Transform::from_pos(pos));
            }
        }
        
        for (_, tf) in &mut resources.world.query::<&mut Transform>().with::<&BulletTag>() {
            tf.pos += 34.0 * dt * glam::Vec2::X;
        }
    }
}
