//! The crate for detecting collisions between shapes.
//! The coordinate system is as follows:
//! * `X` points right
//! * `Y` point up
//!
//! While the crate does use [glam::Affine2] to encode shape transforms, using the scale
//! for shapes is not allowed.

pub mod conv;
pub mod shape_util;

use glam::{Affine2, Vec2};
use shipyard::EntityId;

pub const GROUP_COUNT: usize = 32;

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Rect { size: Vec2 },
    Circle { radius: f32 },
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Group(u32);

impl Group {
    pub const fn empty() -> Group {
        Group(0)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn from_id(x: u32) -> Group {
        Group(1u32.unbounded_shl(x))
    }

    pub const fn union(self, other: Group) -> Group {
        Group(self.0 | other.0)
    }

    pub const fn intersection(self, other: Group) -> Group {
        Group(self.0 & other.0)
    }

    pub const fn enabled(self, idx: u32) -> bool {
        self.includes(Group::from_id(idx))
    }

    pub const fn includes(self, target: Group) -> bool {
        self.0 & target.0 == target.0
    }
}

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
}

struct GroupRefs(Vec<(EntityId, Collider)>, Group);

pub struct CollisionSolver {
    groups: [GroupRefs; GROUP_COUNT],
}

impl CollisionSolver {
    pub fn new() -> CollisionSolver {
        debug_assert!(GROUP_COUNT <= u32::BITS as usize);
        let groups = std::array::from_fn(|idx| {
            let group = Group::from_id(idx as u32);
            GroupRefs(Vec::new(), group)
        });
        CollisionSolver { groups }
    }

    pub fn clear(&mut self) {
        self.groups.iter_mut().for_each(|x| x.0.clear());
    }

    pub fn fill(&mut self, entities: impl IntoIterator<Item = (EntityId, Collider)>) {
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
    ) -> impl Iterator<Item = &'_ (EntityId, Collider)> {
        let group = self
            .groups
            .iter()
            .filter(|x| x.1.includes(query.group))
            .min_by_key(|x| x.0.len())
            .expect("ill-formed group");
        group.0.iter().filter(move |(_, col)| col.collides(&query))
    }
}
