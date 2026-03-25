mod cmd;
mod screendump;

use crate::components::*;
use crate::prelude::*;
use crate::{App, DebugCommand};

pub use cmd::*;
pub use screendump::*;

use egui::Context;

pub(crate) struct DebugStuff {
    pub cmd_center: CommandCenter,
    pub force_freeze: bool,
}

impl DebugStuff {
    pub(crate) fn new() -> Self {
        Self { cmd_center: CommandCenter::new(), force_freeze: false }
    }

    pub fn game_freeze_active(&self) -> bool {
        self.cmd_center.should_pause() || self.force_freeze
    }

    pub fn new_update(&mut self) {
        GLOBAL_DUMP.reset();
    }
}

impl App {
    pub fn dump_common_info(&mut self) {
        let ent_count = self.resources.world.iter().count();

        // dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
        self.dump_archetypes();
        GLOBAL_DUMP.lock();
    }

    fn dump_archetypes(&self) {
        let mut total_archetypes = 0;
        for _arch in self.resources.world.archetypes() {
            total_archetypes += 1;
        }

        dump!("Total archetypes: {total_archetypes}");
    }

    pub fn debug_ui(&mut self, egui_ctx: &Context) {
        if let Some(cmd) = self.debug.cmd_center.show(egui_ctx) {
            self.handle_command(cmd);
        }
        GLOBAL_DUMP.show(egui_ctx);
    }

    fn handle_command(&mut self, cmd: DebugCommand) {
        match cmd.command.as_str() {
            "f" => self.debug.force_freeze = true,
            "uf" => self.debug.force_freeze = false,
            "hw" => self.render.render_world = false,
            "sw" => self.render.render_world = true,
            "dde" => {
                if cmd.args.is_empty() {
                    error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !self.render.debug_draws.contains_key(dd_name) {
                    error!("No such debug draw: {:?}", dd_name);
                    return;
                }
                self.render.enabled_debug_draws.insert(dd_name.to_owned());
                info!("Enabled debug draw {dd_name:?}");
            }
            "ddd" => {
                if cmd.args.is_empty() {
                    error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !self.render.enabled_debug_draws.contains(dd_name) {
                    error!("No enabled debug draw: {:?}", dd_name);
                    return;
                }
                self.render.enabled_debug_draws.remove(dd_name);
                info!("Disabled debug draw {dd_name:?}");
            }
            "spawn" => {
                if cmd.args.len() < 3 {
                    error!("Not enough args");
                    return;
                }

                let prefab_path = &cmd.args[0];
                let Ok(x) = cmd.args[1].trim().parse::<f32>() else {
                    error!("Second argument is not a number");
                    return;
                };
                let Ok(y) = cmd.args[2].trim().parse::<f32>() else {
                    error!("Third argument is not a number");
                    return;
                };
                let Some(prefab_handle) = self.resources.prefabs.resolve(prefab_path) else {
                    error!("No such prefab: {prefab_path:?}");
                    return;
                };
                let prefab = self.resources.prefabs.get(prefab_handle).unwrap();
                let entity = self.resources.world.spawn(prefab);
                if prefab.has::<Transform>() {
                    info!("prefab has Transform. Override to ({x:.2}, {y:.2})");
                    self.resources
                        .world
                        .insert_one(entity, Transform::from_xy(x, y))
                        .unwrap();
                }
                info!("Spawned {prefab_path:?} as {entity:?}");
            }
            unmatched => {
                if !self.state.handle_command(&mut self.resources, &cmd) {
                    error!("Unknown command: {unmatched:?}");
                }
            }
        }
    }
}
