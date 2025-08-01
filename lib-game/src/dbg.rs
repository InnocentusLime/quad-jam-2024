use hashbrown::{HashMap, HashSet};
use hecs::World;
use log::{error, info};
use macroquad::{camera::Camera, input::get_char_pressed};
use quad_dbg::{CommandCenter, ScreenCons, ScreenDump};

use crate::{App, AppState, InputModel, Render};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsoleMode {
    Hidden,
    Dump,
    Console,
}

impl ConsoleMode {
    fn scroll(self) -> Self {
        match self {
            ConsoleMode::Hidden => ConsoleMode::Dump,
            ConsoleMode::Dump => ConsoleMode::Console,
            ConsoleMode::Console => ConsoleMode::Hidden,
        }
    }
}

struct DebugState {
    debug_draws: HashMap<String, fn(&World)>,
    enabled_debug_draws: HashSet<String>,
}

pub(crate) struct DebugStuff<T> {
    cmd_center: CommandCenter<App, T, DebugState>,
    console_mode: ConsoleMode,
    state: DebugState,
}

impl<T: 'static> DebugStuff<T> {
    pub(crate) fn new(
        debug_draws: impl Iterator<Item = (String, fn(&World))>,
        user_cmds: impl Iterator<Item = (&'static str, &'static str, fn(&mut T, &mut World, &[&str]))>,
    ) -> Self {
        let mut cmd_center = CommandCenter::new();

        ScreenCons::init_log();
        init_debug_commands(&mut cmd_center);
        for (cmd, description, payload) in user_cmds {
            cmd_center.add_command(cmd, description, move |app, ex, _, args| {
                payload(ex, &mut app.world, args)
            });
        }

        Self {
            cmd_center,
            console_mode: ConsoleMode::Hidden,
            state: DebugState {
                debug_draws: debug_draws.collect(),
                enabled_debug_draws: HashSet::new(),
            },
        }
    }

    pub(crate) fn draw(&self, camera: &dyn Camera, render: &mut Render, world: &World) {
        render.debug_render(camera, || {
            for debug in self.state.enabled_debug_draws.iter() {
                (self.state.debug_draws[debug])(world)
            }
        });

        let mut console_mode = self.console_mode;
        if self.cmd_center.should_pause() {
            console_mode = ConsoleMode::Console;
        }

        match console_mode {
            ConsoleMode::Hidden => (),
            ConsoleMode::Dump => ScreenDump::draw(),
            ConsoleMode::Console => ScreenCons::draw(),
        }

        self.cmd_center.draw();
    }

    pub(crate) fn input(&mut self, input: &InputModel, app: &mut App, ex: &mut T) {
        if input.scroll_down {
            ScreenCons::scroll_forward();
        }
        if input.scroll_up {
            ScreenCons::scroll_back();
        }

        if let Some(ch) = get_char_pressed() {
            self.cmd_center.input(ch, app, ex, &mut self.state);
        }

        if input.console_toggle_requested {
            self.console_mode = self.console_mode.scroll();
        }
    }

    pub(crate) fn should_pause(&self) -> bool {
        self.cmd_center.should_pause()
    }
}

fn init_debug_commands<T>(cmds: &mut CommandCenter<App, T, DebugState>) {
    cmds.add_command("f", "freeze the app", |app, _, _, _| app.freeze = true);
    cmds.add_command("uf", "unfreeze the app", |app, _, _, _| app.freeze = false);
    cmds.add_command("hw", "hide the world rendering", |app, _, _, _| {
        app.draw_world = false
    });
    cmds.add_command("sw", "show the world rendering", |app, _, _, _| {
        app.draw_world = true
    });
    cmds.add_command(
        "reset",
        "reset app back to the start state",
        |app, _, _, _| {
            app.state = AppState::Start;
            app.loaded_level = None;
        },
    );
    cmds.add_command(
        "dde",
        "enable a debug draw. Usage: dde [NAME]",
        |_app, _, state, args| {
            if args.len() < 1 {
                error!("Not enough args");
                return;
            }

            let dd_name = args[0];
            if !state.debug_draws.contains_key(dd_name) {
                error!("No such debug draw: {:?}", dd_name);
                return;
            }

            state.enabled_debug_draws.insert(dd_name.to_owned());
        },
    );
    cmds.add_command(
        "ddd",
        "disable a debug draw. Usage: ddd [NAME]",
        |_app, _, state, args| {
            if args.len() < 1 {
                error!("Not enough args");
                return;
            }

            let dd_name = args[0];
            if !state.enabled_debug_draws.contains(dd_name) {
                error!("No enabled debug draw: {:?}", dd_name);
                return;
            }

            state.enabled_debug_draws.remove(dd_name);
        },
    );
    cmds.add_command("ddl", "list all debug draws", |_app, _, state, _| {
        for key in state.debug_draws.keys() {
            info!("{}", key);
        }
    });
}
