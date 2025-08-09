//! The crate for detecting collisions between shapes.
//! The coordinate system is as follows:
//! * `X` points right
//! * `Y` point up
//!
//! While the crate does use [glam::Affine2] to encode shape transforms, using the scale
//! for shapes is not allowed.

pub mod conv;
mod group;
mod shape;

use glam::{Affine2, Vec2};
use hecs::Entity;

pub use group::*;
pub use shape::*;

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub tf: Affine2,
    pub shape: Shape,
    pub group: Group,
}

impl Collider {
    pub fn collides(&self, other: &Self) -> bool {
        if self.group.intersection(other.group).is_empty() {
            return false;
        }

        !self.shape.is_separated(&other.shape, self.tf, other.tf)
    }

    pub fn satisfies_filter(&self, filter: Group) -> bool {
        self.group.includes(filter)
    }
}

struct GroupRefs(Vec<(Entity, Collider)>, Group);

pub struct CollisionSolver {
    groups: [GroupRefs; GROUP_COUNT],
}

impl CollisionSolver {
    pub fn new() -> CollisionSolver {
        debug_assert!(GROUP_COUNT == u32::BITS as usize);
        let groups = std::array::from_fn(|idx| {
            let group = Group::from_id(idx as u32);
            GroupRefs(Vec::new(), group)
        });
        CollisionSolver { groups }
    }

    pub fn clear(&mut self) {
        self.groups.iter_mut().for_each(|x| x.0.clear());
    }

    pub fn fill(&mut self, entities: impl IntoIterator<Item = (Entity, Collider)>) {
        for (ent, collider) in entities {
            let matches = self
                .groups
                .iter_mut()
                .filter(|group| collider.group.includes(group.1));
            for group in matches {
                group.0.push((ent, collider));
            }
        }
    }

    pub fn query_overlaps(
        &self,
        query: Collider,
        filter: Group,
    ) -> impl Iterator<Item = &'_ (Entity, Collider)> {
        self.groups
            .iter()
            .filter(move |group| query.group.includes(group.1))
            .flat_map(|group| group.0.iter())
            .filter(move |(_, col)| col.satisfies_filter(filter))
            .filter(move |(_, col)| col.collides(&query))
    }

    pub fn query_shape_cast(
        &self,
        query: Collider,
        direction: Vec2,
        t_max: f32,
    ) -> Option<(Entity, f32, Vec2)> {
        self.groups
            .iter()
            .filter(move |group| query.group.includes(group.1))
            .flat_map(|group| group.0.iter())
            .filter_map(move |(entity, collider)| {
                Self::query_shape_cast_do_shapecast(*entity, collider, query, direction, t_max)
            })
            .min_by(|(_, toi1, _), (_, toi2, _)| f32::total_cmp(toi1, toi2))
    }

    fn query_shape_cast_do_shapecast(
        entity: Entity,
        collider: &Collider,
        query: Collider,
        direction: Vec2,
        t_max: f32,
    ) -> Option<(Entity, f32, Vec2)> {
        let (toi, normal) =
            query
                .shape
                .time_of_impact(&collider.shape, query.tf, collider.tf, direction, t_max)?;
        Some((entity, toi, normal))
    }
}
