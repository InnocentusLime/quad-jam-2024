use macroquad::prelude::*;

pub use lib_col::{Group, Shape};

pub const MAX_COLLISION_QUERIES: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct CollisionQuery<const ID: usize> {
    /// The group membership.
    /// The engine will pick all entities with
    /// their group intersecting with this field.
    ///
    /// Setting it to an empty group will make the
    /// collision engine skip this query.
    pub group: Group,
    /// The group filter.
    /// The engine will pick all entities inside
    /// that group.
    pub filter: Group,
    /// The collider to use for the check.
    pub collider: Shape,
    pub collision_slice: CollisionQuerySlice,
}

impl<const ID: usize> CollisionQuery<ID> {
    pub fn new(collider: Shape, group: Group, filter: Group) -> Self {
        Self {
            collider,
            group,
            filter,
            collision_slice: CollisionQuerySlice { off: 0, len: 0 },
        }
    }

    pub fn has_collided(&self) -> bool {
        self.collision_slice.len > 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionQuerySlice {
    pub off: usize,
    pub len: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct KinematicControl {
    pub dr: Vec2,
    pub collision: Group,
    pub slide: bool,
    pub collided: bool,
}

impl KinematicControl {
    /// Creates a new [KinematicControl].
    /// * `collision` -- the layer which the body will collide against
    pub fn new_slide(collision: Group) -> Self {
        Self {
            dr: Vec2::ZERO,
            collision,
            slide: true,
            collided: false,
        }
    }

    pub fn new_nonslide(collision: Group) -> Self {
        Self {
            dr: Vec2::ZERO,
            collision,
            slide: false,
            collided: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BodyTag {
    pub groups: Group,
    pub shape: Shape,
}

pub mod col_group {
    use lib_col::Group;

    pub const NONE: Group = Group::empty();
    pub const LEVEL: Group = Group::from_id(0);
    pub const CHARACTERS: Group = Group::from_id(1);
    pub const PLAYER: Group = Group::from_id(2);
    pub const PROJECTILES: Group = Group::from_id(3);
}

pub mod col_query {
    pub const LEVEL: usize = 0;
    pub const DAMAGE: usize = 1;
    pub const PICKUP: usize = 2;
    pub const INTERACTION: usize = 3;

    #[allow(dead_code)]
    pub type Level = super::CollisionQuery<LEVEL>;
    pub type Damage = super::CollisionQuery<DAMAGE>;
    pub type Pickup = super::CollisionQuery<PICKUP>;
    #[allow(dead_code)]
    pub type Interaction = super::CollisionQuery<INTERACTION>;
}
