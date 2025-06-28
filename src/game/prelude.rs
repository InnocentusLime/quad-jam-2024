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

pub mod col_query {
    pub const LEVEL: usize = 0;
    pub const PLAYER_DAMAGE: usize = 1;
    pub const ENEMY_DAMAGE: usize = 2;
    pub const PICKUP: usize = 3;
    pub const INTERACTION: usize = 4;

    #[allow(dead_code)]
    pub type Level = lib_game::CollisionQuery<LEVEL>;
    #[allow(dead_code)]
    pub type PlayerDamage = lib_game::CollisionQuery<PLAYER_DAMAGE>;
    #[allow(dead_code)]
    pub type EnemyDamage = lib_game::CollisionQuery<ENEMY_DAMAGE>;
    pub type Pickup = lib_game::CollisionQuery<PICKUP>;
    #[allow(dead_code)]
    pub type Interaction = lib_game::CollisionQuery<INTERACTION>;
}
