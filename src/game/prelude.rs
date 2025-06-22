pub use super::components::*;
pub use lib_game::*;
pub use macroquad::prelude::*;
pub use shipyard::{EntityId, Get, IntoIter, View, ViewMut, World};

pub const LEVEL_GROUP: PhysicsGroup = PhysicsGroup {
    level: true,
    ..PhysicsGroup::empty()
};
#[allow(dead_code)]
pub const PROJECTILES_GROUP: PhysicsGroup = PhysicsGroup {
    projectiles: true,
    ..PhysicsGroup::empty()
};
pub const LEVEL_INTERACT: PhysicsGroup = PhysicsGroup {
    npcs: true,
    player: true,
    projectiles: true,
    ..PhysicsGroup::empty()
};
