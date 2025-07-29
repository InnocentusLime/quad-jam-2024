pub use super::components::*;
pub use hecs::{CommandBuffer, Entity, World};
pub use lib_game::*;
pub use macroquad::prelude::*;

pub mod col_group {
    use quad_col::Group;

    pub const NONE: Group = Group::empty();
    pub const LEVEL: Group = Group::from_id(0);
    pub const PLAYER: Group = Group::from_id(1);
    pub const ENEMY: Group = Group::from_id(2);
    pub const DAMAGABLE: Group = Group::from_id(3);
}

pub mod col_query {
    pub const LEVEL: usize = 0;
    pub const DAMAGE: usize = 1;
    pub const PICKUP: usize = 2;
    pub const INTERACTION: usize = 3;

    #[allow(dead_code)]
    pub type Level = lib_game::CollisionQuery<LEVEL>;
    pub type Damage = lib_game::CollisionQuery<DAMAGE>;
    pub type Pickup = lib_game::CollisionQuery<PICKUP>;
    #[allow(dead_code)]
    pub type Interaction = lib_game::CollisionQuery<INTERACTION>;
}
