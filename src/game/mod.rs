mod prelude;

use prelude::*;

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
    fn handle_command(&mut self, _app: &mut App, cmd: &DebugCommand) -> bool {
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
        _resources: &lib_game::Resources,
        _world: &mut World,
        _cmds: &mut CommandBuffer,
    ) {
    }

    fn update(
        &mut self,
        _dt: f32,
        _resources: &lib_game::Resources,
        _world: &mut World,
        _collisions: &CollisionSolver,
        _cmds: &mut CommandBuffer,
    ) -> Option<Box<dyn State>> {
        None
    }
}
