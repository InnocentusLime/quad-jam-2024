use hecs::Entity;
use macroquad::prelude::*;

use crate::Transform;

pub use quad_col::{Group, Shape};

pub const MAX_COLLISION_QUERIES: usize = 8;

#[derive(Clone, Debug)]
pub enum CollisionList {
    One(Option<Entity>),
    Many(Vec<Entity>),
}

impl CollisionList {
    pub fn one() -> Self {
        CollisionList::One(None)
    }

    pub fn many(capacity: usize) -> Self {
        CollisionList::Many(Vec::with_capacity(capacity))
    }

    pub fn clear(&mut self) {
        match self {
            CollisionList::One(entity_id) => {
                entity_id.take();
            }
            CollisionList::Many(entity_ids) => entity_ids.clear(),
        }
    }

    pub fn collisions(&self) -> &[Entity] {
        match self {
            CollisionList::One(None) => &[],
            CollisionList::One(Some(entity_id)) => std::slice::from_ref(entity_id),
            CollisionList::Many(entity_ids) => &entity_ids,
        }
    }
}

impl Extend<Entity> for CollisionList {
    fn extend<I: IntoIterator<Item = Entity>>(&mut self, iter: I) {
        match self {
            // We aren't required to consume all iterator items
            CollisionList::One(entity_id) => *entity_id = iter.into_iter().next(),
            CollisionList::Many(entity_ids) => entity_ids.extend(iter),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollisionQuery<const ID: usize> {
    /// The collision filter. Setting it to an empty group
    /// will make the collision engine skip this query.
    pub group: Group,
    /// The buffer to put the collisions into.
    pub collision_list: CollisionList,
    /// The collider to use for the check.
    pub collider: Shape,
    /// Extra transform for the query. Gets applied before
    /// the transform of the containing entity: `entity_tf * extra_tf`.
    pub extra_tf: Transform,
}

impl<const ID: usize> CollisionQuery<ID> {
    pub fn new_one(collider: Shape, group: Group) -> Self {
        Self {
            collider,
            group,
            collision_list: CollisionList::one(),
            extra_tf: Transform::IDENTITY,
        }
    }

    pub fn new_many(collider: Shape, group: Group, capacity: usize) -> Self {
        Self {
            collider,
            group,
            collision_list: CollisionList::many(capacity),
            extra_tf: Transform::IDENTITY,
        }
    }

    pub fn collisions(&self) -> &[Entity] {
        self.collision_list.collisions()
    }

    pub fn has_collided(&self) -> bool {
        !self.collisions().is_empty()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KinematicControl {
    pub dr: Vec2,
    pub collision: Group,
}

impl KinematicControl {
    /// Creates a new [KinematicControl].
    /// * `collision` -- the layer which the body will collide against
    pub fn new(collision: Group) -> Self {
        Self {
            dr: Vec2::ZERO,
            collision,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BodyTag {
    pub groups: Group,
    pub shape: Shape,
}
