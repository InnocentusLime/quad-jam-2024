#[cfg(feature = "dev-env")]
mod animation_edit;
mod cmd;
mod screendump;

use hashbrown::{HashMap, HashSet};
use hecs::World;
use lib_asset::{Asset, LevelId, level::LevelDef};
use log::set_logger;
use macroquad::prelude::*;

#[cfg(feature = "dev-env")]
pub use animation_edit::*;
pub use cmd::*;
pub use screendump::*;
use strum::VariantArray;

use crate::{App, AppState, DebugCommand, Game, Resources, dump, level_utils};

pub(crate) struct DebugStuff {
    pub cmd_center: CommandCenter,
    pub debug_draws: HashMap<String, fn(&World, &Resources)>,
    pub enabled_debug_draws: HashSet<String>,
    #[cfg(feature = "dev-env")]
    anim_edit: AnimationEdit,
}

impl DebugStuff {
    pub(crate) fn new<G: Game>(game: &mut G) -> Self {
        set_logger(&*GLOBAL_CON as &dyn log::Log).expect("failed to init logger");

        let debug_draws = game
            .debug_draws()
            .iter()
            .map(|(name, payload)| (name.to_string(), *payload))
            .collect::<HashMap<_, _>>();

        Self {
            cmd_center: CommandCenter::new(),
            debug_draws,
            enabled_debug_draws: HashSet::new(),
            #[cfg(feature = "dev-env")]
            anim_edit: AnimationEdit::new(),
        }
    }

    pub fn ui<G: Game>(&mut self, app: &mut App, game: &mut G) {
        egui_macroquad::ui(|egui_ctx| {
            #[cfg(feature = "dev-env")]
            egui::Window::new("animation_edit").show(egui_ctx, |ui| {
                self.anim_edit.ui(
                    &app.resources.resolver,
                    ui,
                    &mut app.resources.animations,
                    &mut app.world,
                );
            });
            let cmd = self.cmd_center.show(egui_ctx, get_char_pressed());
            if let Some(cmd) = cmd {
                self.handle_command(app, game, cmd);
            }
            GLOBAL_DUMP.show(egui_ctx);
        });

        if (self.should_pause() || app.freeze) && app.state == (AppState::Active { paused: false })
        {
            app.state = AppState::DebugFreeze;
        }
        if !(self.should_pause() || app.freeze) && app.state == AppState::DebugFreeze {
            app.state = AppState::Active { paused: false };
        }
    }

    fn should_pause(&self) -> bool {
        self.cmd_center.should_pause()
    }

    fn handle_command<G: Game>(&mut self, app: &mut App, game: &mut G, cmd: DebugCommand) {
        match cmd.command.as_str() {
            "f" => app.freeze = true,
            "uf" => app.freeze = false,
            "hw" => app.render_world = false,
            "sw" => app.render_world = true,
            "reset" => app.state = AppState::Start,
            "l" => {
                for level_id in LevelId::VARIANTS {
                    let l_name: &'static str = level_id.into();
                    info!("Level {l_name}: {}", LevelDef::filename(*level_id));
                }
            }
            "load" => {
                if cmd.args.is_empty() {
                    error!("Not enough args");
                    return;
                }

                let l_str = &cmd.args[0];
                let Some(level_id) = level_utils::resolve_level(l_str) else {
                    error!("{l_str:?} does not match any known level");
                    return;
                };

                app.queued_level = Some(level_id);
            }
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
            "get" => {
                if cmd.args.len() < 2 {
                    error!("Not enough args. Need 2");
                    return;
                }

                let section = &cmd.args[0];
                let field = &cmd.args[1];
                match app.resources.cfg.get_field(section, field) {
                    Ok(x) => info!("{section}.{field} = {x}"),
                    Err(e) => error!("{e:#}"),
                }
            }
            "set" => {
                if cmd.args.len() < 3 {
                    error!("Not enough args. Need 3");
                    return;
                }

                let section = &cmd.args[0];
                let field = &cmd.args[1];
                let val = &cmd.args[2];
                match app.resources.cfg.set_field(section, field, val) {
                    Ok(_) => info!("{section}.{field} updated"),
                    Err(e) => error!("{e:#}"),
                }
            }
            unmatched => {
                if !game.handle_command(app, &cmd) {
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

        dump!("Dt: {:.2}", app.accumelated_time);
        dump!("FPS: {:?}", get_fps());
        dump!("Entities: {ent_count}");
        GLOBAL_DUMP.lock();

        app.render.debug_render(&app.camera, || {
            for debug_draw_name in self.enabled_debug_draws.iter() {
                let draw = self.debug_draws[debug_draw_name];
                draw(&app.world, &app.resources);
            }
        });

        egui_macroquad::draw();
    }
}
