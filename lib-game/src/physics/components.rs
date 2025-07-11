use macroquad::prelude::*;
use shipyard::{Component, EntityId};

use crate::Transform;

pub const MAX_COLLISION_QUERIES: usize = 8;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PhysicsGroup {
    pub level: bool,
    pub npcs: bool,
    pub player: bool,
    pub projectiles: bool,
    pub maincell: bool,
    pub items: bool,
}

impl PhysicsGroup {
    pub const fn empty() -> PhysicsGroup {
        PhysicsGroup {
            level: false,
            npcs: false,
            player: false,
            projectiles: false,
            maincell: false,
            items: false,
        }
    }
}

impl PhysicsGroup {
    pub(crate) fn into_group(self) -> quad_col::Group {
        use quad_col::Group;

        pub const LEVEL: Group = Group::from_id(0);
        pub const NPCS: Group = Group::from_id(1);
        pub const PLAYER: Group = Group::from_id(2);
        pub const PROJECTILES: Group = Group::from_id(3);
        pub const MAINCELL: Group = Group::from_id(4);
        pub const ITEMS: Group = Group::from_id(5);

        let mut filter = Group::empty();
        if self.level {
            filter = filter.union(LEVEL);
        }
        if self.npcs {
            filter = filter.union(NPCS);
        }
        if self.player {
            filter = filter.union(PLAYER);
        }
        if self.projectiles {
            filter = filter.union(PROJECTILES);
        }
        if self.maincell {
            filter = filter.union(MAINCELL);
        }
        if self.items {
            filter = filter.union(ITEMS);
        }

        filter
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColliderTy {
    Box { width: f32, height: f32 },
    Circle { radius: f32 },
}

impl ColliderTy {
    pub(crate) fn into_shape(self) -> quad_col::Shape {
        match self {
            ColliderTy::Box { width, height } => quad_col::Shape::Rect { width, height },
            ColliderTy::Circle { radius } => quad_col::Shape::Circle { radius },
        }
    }
}

#[derive(Clone, Debug)]
pub enum CollisionList {
    One(Option<EntityId>),
    Many(Vec<EntityId>),
}

impl CollisionList {
    pub fn one() -> Self {
        CollisionList::One(None)
    }

    pub fn many() -> Self {
        CollisionList::Many(Vec::new())
    }

    pub fn clear(&mut self) {
        match self {
            CollisionList::One(entity_id) => {
                entity_id.take();
            }
            CollisionList::Many(entity_ids) => entity_ids.clear(),
        }
    }

    pub fn collisions(&self) -> &[EntityId] {
        match self {
            CollisionList::One(None) => &[],
            CollisionList::One(Some(entity_id)) => std::slice::from_ref(entity_id),
            CollisionList::Many(entity_ids) => &entity_ids,
        }
    }
}

impl Extend<EntityId> for CollisionList {
    fn extend<I: IntoIterator<Item = EntityId>>(&mut self, iter: I) {
        match self {
            // We aren't required to consume all iterator items
            CollisionList::One(entity_id) => *entity_id = iter.into_iter().next(),
            CollisionList::Many(entity_ids) => entity_ids.extend(iter),
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct CollisionQuery<const ID: usize> {
    /// The collision filter. Setting it to an empty group
    /// will make the collision engine skip this query.
    pub group: PhysicsGroup,
    /// The buffer to put the collisions into.
    pub collision_list: CollisionList,
    /// The collider to use for the check.
    pub collider: ColliderTy,
    /// Extra transform for the query. Gets applied before
    /// the transform of the containing entity: `entity_tf * extra_tf`.
    pub extra_tf: Transform,
}

impl<const ID: usize> CollisionQuery<ID> {
    pub fn new_one(collider: ColliderTy, group: PhysicsGroup) -> Self {
        Self {
            collider,
            group,
            collision_list: CollisionList::one(),
            extra_tf: Transform::IDENTITY,
        }
    }

    pub fn new_many(collider: ColliderTy, group: PhysicsGroup) -> Self {
        Self {
            collider,
            group,
            collision_list: CollisionList::many(),
            extra_tf: Transform::IDENTITY,
        }
    }

    pub fn collisions(&self) -> &[EntityId] {
        self.collision_list.collisions()
    }

    pub fn has_collided(&self) -> bool {
        !self.collisions().is_empty()
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct KinematicControl {
    pub dr: Vec2,
    pub slide: bool,
}

impl KinematicControl {
    pub fn new() -> Self {
        Self {
            dr: Vec2::ZERO,
            slide: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyKind {
    Static,
    Kinematic,
}

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal, Insertion)]
pub struct BodyTag {
    pub enabled: bool,
    pub groups: PhysicsGroup,
    pub(crate) shape: ColliderTy,
    pub(crate) _mass: f32,
    pub(crate) kind: BodyKind,
}

impl BodyTag {
    pub fn new(
        groups: PhysicsGroup,
        shape: ColliderTy,
        mass: f32,
        enabled: bool,
        kind: BodyKind,
    ) -> Self {
        Self {
            enabled,
            groups,
            _mass: mass,
            shape,
            kind,
        }
    }

    pub fn shape(&self) -> &ColliderTy {
        &self.shape
    }

    pub fn kind(&self) -> BodyKind {
        self.kind
    }
}
