mod cmd;
mod screendump;

use egui::Context;
use hashbrown::{HashMap, HashSet};
use hecs::World;
use log::set_logger;

pub use cmd::*;
pub use screendump::*;

use crate::{App, DebugCommand, Resources, State, dump};

pub(crate) struct DebugStuff {
    pub cmd_center: CommandCenter,
    pub debug_draws: HashMap<String, fn(&World, &Resources)>,
    pub enabled_debug_draws: HashSet<String>,
    pub force_freeze: bool,
}

impl DebugStuff {
    pub(crate) fn new() -> Self {
        // set_logger(&*GLOBAL_CON as &dyn log::Log).expect("failed to init logger");

        Self {
            cmd_center: CommandCenter::new(),
            debug_draws: HashMap::new(),
            enabled_debug_draws: HashSet::new(),
            force_freeze: false,
        }
    }

    pub fn should_pause(&self) -> bool {
        self.cmd_center.should_pause() || self.force_freeze
    }

    pub fn new_update(&mut self) {
        GLOBAL_DUMP.reset();
    }
}

impl App {
    pub fn debug_draw(&mut self) {
        let ent_count = self.world.iter().count();

        // dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
        self.dump_archetypes();
        GLOBAL_DUMP.lock();

        // app.render.debug_render(|| {
        //     for debug_draw_name in self.enabled_debug_draws.iter() {
        //         let draw = self.debug_draws[debug_draw_name];
        //         draw(&app.world, &app.resources);
        //     }
        // });
    }

    fn dump_archetypes(&self) {
        let mut total_archetypes = 0;
        for _arch in self.world.archetypes() {
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
            "hw" => self.render_world = false,
            "sw" => self.render_world = true,
            "dde" => {
                if cmd.args.is_empty() {
                    // error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !self.debug.debug_draws.contains_key(dd_name) {
                    // error!("No such debug draw: {:?}", dd_name);
                    return;
                }
                self.debug.enabled_debug_draws.insert(dd_name.to_owned());
            }
            "ddd" => {
                if cmd.args.is_empty() {
                    // error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !self.debug.enabled_debug_draws.contains(dd_name) {
                    // error!("No enabled debug draw: {:?}", dd_name);
                    return;
                }
                self.debug.enabled_debug_draws.remove(dd_name);
            }
            unmatched => {
                // if !state.handle_command(app, &cmd) {
                // error!("Unknown command: {unmatched:?}");
                // }
            }
        }
    }
}
