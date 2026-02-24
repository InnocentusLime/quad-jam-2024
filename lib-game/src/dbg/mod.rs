mod cmd;
mod screendump;

use hashbrown::{HashMap, HashSet};
use hecs::World;
use log::set_logger;
use macroquad::prelude::*;

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
        set_logger(&*GLOBAL_CON as &dyn log::Log).expect("failed to init logger");

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

    pub fn ui(&mut self, app: &mut App, state: &mut dyn State) {
        egui_macroquad::ui(|egui_ctx| {
            let cmd = self.cmd_center.show(egui_ctx, get_char_pressed());
            if let Some(cmd) = cmd {
                self.handle_command(app, state, cmd);
            }
            GLOBAL_DUMP.show(egui_ctx);
        });
    }

    fn handle_command(&mut self, app: &mut App, state: &mut dyn State, cmd: DebugCommand) {
        match cmd.command.as_str() {
            "f" => self.force_freeze = true,
            "uf" => self.force_freeze = false,
            "hw" => app.render_world = false,
            "sw" => app.render_world = true,
            "dde" => {
                if cmd.args.is_empty() {
                    error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !self.debug_draws.contains_key(dd_name) {
                    error!("No such debug draw: {:?}", dd_name);
                    return;
                }
                self.enabled_debug_draws.insert(dd_name.to_owned());
            }
            "ddd" => {
                if cmd.args.is_empty() {
                    error!("Not enough args");
                    return;
                }

                let dd_name = &cmd.args[0];
                if !self.enabled_debug_draws.contains(dd_name) {
                    error!("No enabled debug draw: {:?}", dd_name);
                    return;
                }
                self.enabled_debug_draws.remove(dd_name);
            }
            unmatched => {
                if !state.handle_command(app, &cmd) {
                    error!("Unknown command: {unmatched:?}");
                }
            }
        }
    }

    pub fn new_update(&mut self) {
        GLOBAL_DUMP.reset();
    }

    pub fn draw(&self, app: &mut App) {
        let ent_count = app.world.iter().count();

        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
        self.dump_archetypes(app);
        GLOBAL_DUMP.lock();

        app.render.debug_render(|| {
            for debug_draw_name in self.enabled_debug_draws.iter() {
                let draw = self.debug_draws[debug_draw_name];
                draw(&app.world, &app.resources);
            }
        });

        egui_macroquad::draw();
    }

    fn dump_archetypes(&self, app: &mut App) {
        let mut total_archetypes = 0;
        for _arch in app.world.archetypes() {
            total_archetypes += 1;
        }

        dump!("Total archetypes: {total_archetypes}");
    }
}
