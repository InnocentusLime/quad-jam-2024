use macroquad::prelude::*;
use rapier2d::prelude::InteractionGroups;
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
    fn into_group(self) -> rapier2d::prelude::Group {
        use rapier2d::prelude::*;

        pub const LEVEL: Group = Group::GROUP_1;
        pub const NPCS: Group = Group::GROUP_2;
        pub const PLAYER: Group = Group::GROUP_3;
        pub const PROJECTILES: Group = Group::GROUP_4;
        pub const MAINCELL: Group = Group::GROUP_5;
        pub const ITEMS: Group = Group::GROUP_6;

        let mut filter = Group::NONE;
        if self.level {
            filter |= LEVEL;
        }
        if self.npcs {
            filter |= NPCS;
        }
        if self.player {
            filter |= PLAYER;
        }
        if self.projectiles {
            filter |= PROJECTILES;
        }
        if self.maincell {
            filter |= MAINCELL
        }
        if self.items {
            filter |= ITEMS
        }

        filter
    }

    pub(crate) fn into_interaction_groups(self) -> InteractionGroups {
        InteractionGroups {
            memberships: self.into_group(),
            filter: self.into_group(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColliderTy {
    Box { width: f32, height: f32 },
    Circle { radius: f32 },
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
    pub(crate) mass: f32,
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
            mass,
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
