#[cfg(feature = "dev-env")]
mod animation_edit;
mod cmd;
mod screendump;

use hashbrown::{HashMap, HashSet};
use hecs::World;
use log::set_logger;

#[cfg(feature = "dev-env")]
pub use animation_edit::*;
pub use cmd::*;
pub use screendump::*;

pub(crate) struct DebugStuff {
    pub cmd_center: CommandCenter,
    pub debug_draws: HashMap<String, fn(&World)>,
    pub enabled_debug_draws: HashSet<String>,
}

impl DebugStuff {
    pub(crate) fn new() -> Self {
        set_logger(&*GLOBAL_CON as &dyn log::Log).expect("failed to init logger");

        Self {
            cmd_center: CommandCenter::new(),
            debug_draws: HashMap::new(),
            enabled_debug_draws: HashSet::new(),
        }
    }

    pub(crate) fn should_pause(&self) -> bool {
        self.cmd_center.should_pause()
    }
}
