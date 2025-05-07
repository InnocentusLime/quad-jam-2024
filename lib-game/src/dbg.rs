use log::{error, info};
use quad_dbg::CommandCenter;

use crate::{App, AppState};

pub fn init_debug_commands(cmds: &mut CommandCenter<App>) {
    cmds.add_command(
        "f", 
        "freeze the app", 
        |app, _| app.freeze = true,
    );
    cmds.add_command(
        "uf", 
        "unfreeze the app",
        |app, _| app.freeze = false,
    );
    cmds.add_command(
        "hw", 
        "hide the world rendering",
        |app, _| app.draw_world = false,
    );
    cmds.add_command(
        "sw", 
        "show the world rendering", 
        |app, _| app.draw_world = true,
    );
    cmds.add_command(
        "reset", 
        "reset app back to the start state",
        |app, _| app.state = AppState::Start,
    );
    cmds.add_command(
        "dde", 
        "enable a debug draw. Usage: dde [NAME]", 
        |app, args| {
            if args.len() < 1 {
                error!("Not enough args");
                return;
            }   

            let dd_name = args[0];
            if !app.debug_draws.contains_key(dd_name) {
                error!("No such debug draw: {:?}", dd_name);
                return;
            }

            app.enabled_debug_draws.insert(dd_name.to_owned());
        }
    );
    cmds.add_command(
        "ddd", 
        "disable a debug draw. Usage: ddd [NAME]", 
        |app, args| {
            if args.len() < 1 {
                error!("Not enough args");
                return;
            }   

            let dd_name = args[0];
            if !app.enabled_debug_draws.contains(dd_name) {
                error!("No enabled debug draw: {:?}", dd_name);
                return;
            }

            app.enabled_debug_draws.remove(dd_name);
        }
    );
    cmds.add_command(
        "ddl", 
        "list all debug draws", 
        |app, _| for key in app.debug_draws.keys() {
            info!("{}", key);
        }
    );
}