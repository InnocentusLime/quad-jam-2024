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

use glam::{Affine2, Vec2, vec2};
use hecs::Entity;

pub use group::*;
pub use shape::*;

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub tf: Affine2,
    pub shape: Shape,
    pub group: Group,
}

#[derive(Clone, Copy)]
struct ColliderSlice {
    verts_start: usize,
    normals_start: usize,
    verts_end: usize,
    normals_end: usize,
    group: Group,
}

impl ColliderSlice {
    pub fn satisfies_filter(&self, filter: Group) -> bool {
        self.group.includes(filter)
    }
}

struct GroupRefs(Vec<(Entity, ColliderSlice)>, Group);

const BUFFER_CAPACITY: usize = 10_000;

pub struct CollisionSolver {
    groups: [GroupRefs; GROUP_COUNT],
    vertices: Vec<Vec2>,
    normals: Vec<Vec2>,
}

impl CollisionSolver {
    pub fn new() -> CollisionSolver {
        debug_assert!(GROUP_COUNT == u32::BITS as usize);
        let groups = std::array::from_fn(|idx| {
            let group = Group::from_id(idx as u32);
            GroupRefs(Vec::new(), group)
        });
        CollisionSolver {
            groups,
            vertices: Vec::with_capacity(BUFFER_CAPACITY),
            normals: Vec::with_capacity(BUFFER_CAPACITY),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.normals.clear();
        self.groups.iter_mut().for_each(|x| x.0.clear());
    }

    pub fn fill(&mut self, entities: impl IntoIterator<Item = (Entity, Collider)>) {
        for (ent, collider) in entities {
            let collider = self.put_collider(collider);
            let matches = self
                .groups
                .iter_mut()
                .filter(|group| collider.group.includes(group.1));
            for group in matches {
                group.0.push((ent, collider));
            }
        }
    }

    fn put_collider(&mut self, collider: Collider) -> ColliderSlice {
        let verts_start = self.vertices.len();
        let normals_start = self.normals.len();

        collider
            .shape
            .write_vertices(collider.tf, &mut self.vertices);
        collider.shape.write_normals(collider.tf, &mut self.normals);

        let verts_end = self.vertices.len();
        let normals_end = self.normals.len();

        ColliderSlice {
            verts_start,
            normals_start,
            verts_end,
            normals_end,
            group: collider.group,
        }
    }

    pub fn query_overlaps(
        &mut self,
        query: Collider,
        filter: Group,
    ) -> impl Iterator<Item = Entity> {
        let query_slice = self.put_collider(query);
        self.do_query_overlaps(query_slice, filter)
    }

    fn do_query_overlaps(
        &self,
        query_slice: ColliderSlice,
        filter: Group,
    ) -> impl Iterator<Item = Entity> {
        self.groups
            .iter()
            .filter(move |group| query_slice.group.includes(group.1))
            .flat_map(|group| group.0.iter())
            .filter(move |(_, col)| col.satisfies_filter(filter))
            .filter(move |(_, col)| self.slices_collide(&query_slice, col))
            .map(|(e, _)| *e)
    }

    pub fn query_shape_cast(
        &mut self,
        query: Collider,
        direction: Vec2,
        t_max: f32,
    ) -> Option<(Entity, f32, Vec2)> {
        let query_slice = self.put_collider(query);
        let (mut toi, mut normal, mut entity) = (f32::INFINITY, Vec2::ZERO, Entity::DANGLING);
        for group in &self.groups {
            if !query_slice.group.includes(group.1) {
                continue;
            }
            for (cand_entity, collider_slice) in &group.0 {
                let (cand_toi, cand_normal) =
                    self.time_of_impact_slice(&query_slice, collider_slice, direction, t_max);
                if cand_toi < toi {
                    toi = cand_toi;
                    normal = cand_normal;
                    entity = *cand_entity;
                }
            }
        }

        if toi == f32::INFINITY {
            None
        } else {
            Some((entity, toi, normal))
        }
    }

    fn time_of_impact_slice(
        &self,
        slice1: &ColliderSlice,
        slice2: &ColliderSlice,
        direction: Vec2,
        t_max: f32,
    ) -> (f32, Vec2) {
        let v_slice1 = &self.vertices[slice1.verts_start..slice1.verts_end];
        let v_slice2 = &self.vertices[slice2.verts_start..slice2.verts_end];
        let (mut toi, mut push_normal) = (-f32::INFINITY, Vec2::ZERO);
        for normal in &self.normals[slice1.normals_start..slice1.normals_end] {
            let (cand_toi, cand_push) =
                self.candidate_time_of_impact_slice(v_slice1, v_slice2, *normal, direction, t_max);
            if cand_toi == f32::INFINITY {
                continue;
            }
            if toi < cand_toi {
                toi = cand_toi;
                push_normal = cand_push;
            }
        }
        for normal in &self.normals[slice2.normals_start..slice2.normals_end] {
            let (cand_toi, cand_push) =
                self.candidate_time_of_impact_slice(v_slice1, v_slice2, *normal, direction, t_max);
            if cand_toi == f32::INFINITY {
                continue;
            }
            if toi < cand_toi {
                toi = cand_toi;
                push_normal = cand_push;
            }
        }

        if toi == f32::INFINITY {
            return (toi, push_normal);
        }
        if self.is_separated_slice(slice1, slice2, (toi + SHAPE_TOI_EPSILON * 10.0) * direction) {
            (f32::INFINITY, Vec2::ZERO)
        } else {
            (toi, push_normal)
        }
    }

    /// Computes the time of impact for a fixed axis.
    /// The axis is encoded with its normal: axis_normal.
    /// `axis_normal` must be a normalized vector.
    fn candidate_time_of_impact_slice(
        &self,
        v_slice1: &[Vec2],
        v_slice2: &[Vec2],
        axis_normal: Vec2,
        direction: Vec2,
        t_max: f32,
    ) -> (f32, Vec2) {
        let dproj = axis_normal.dot(direction);
        // Do not process cases when movement is parallel to the
        // separation axis.
        if dproj <= SHAPE_TOI_EPSILON {
            return (f32::INFINITY, Vec2::ZERO);
        }

        let proj1 = self.project_slice(v_slice1, axis_normal);
        let proj2 = self.project_slice(v_slice2, axis_normal);
        let t = if proj1.x < proj2.x {
            (proj2.x - proj1.y) / dproj
        } else {
            (proj1.x - proj2.y) / dproj
        };

        if t <= 0.0 || t > t_max {
            (f32::INFINITY, Vec2::ZERO)
        } else {
            (t, -axis_normal)
        }
    }

    fn slices_collide(&self, slice1: &ColliderSlice, slice2: &ColliderSlice) -> bool {
        if slice1.group.intersection(slice2.group).is_empty() {
            return false;
        }

        !self.is_separated_slice(slice1, slice2, Vec2::ZERO)
    }

    fn is_separated_slice(
        &self,
        slice1: &ColliderSlice,
        slice2: &ColliderSlice,
        offset_slice1: Vec2,
    ) -> bool {
        let v_slice1 = &self.vertices[slice1.verts_start..slice1.verts_end];
        let v_slice2 = &self.vertices[slice2.verts_start..slice2.verts_end];

        for normal in &self.normals[slice1.normals_start..slice1.normals_end] {
            if self.try_separating_axis_slice(v_slice1, v_slice2, *normal, offset_slice1) {
                return true;
            }
        }

        for normal in &self.normals[slice2.normals_start..slice2.normals_end] {
            if self.try_separating_axis_slice(v_slice1, v_slice2, *normal, offset_slice1) {
                return true;
            }
        }

        false
    }

    fn try_separating_axis_slice(
        &self,
        slice1: &[Vec2],
        slice2: &[Vec2],
        axis: Vec2,
        offset_slice1: Vec2,
    ) -> bool {
        let offset_slice1_proj = offset_slice1.dot_into_vec(axis);
        let proj1 = self.project_slice(slice1, axis) + offset_slice1_proj;
        let proj2 = self.project_slice(slice2, axis);
        let (l_proj, r_proj) = if proj1.x < proj2.x {
            (proj1, proj2)
        } else {
            (proj2, proj1)
        };

        l_proj.y < r_proj.x
    }

    fn project_slice(&self, slice: &[Vec2], axis: Vec2) -> Vec2 {
        let mut max = -f32::INFINITY;
        let mut min = f32::INFINITY;
        for v in slice {
            let proj = v.dot(axis);
            max = vec2(proj, max).max_element();
            min = vec2(proj, min).min_element();
        }
        vec2(min, max)
    }
}

impl Default for CollisionSolver {
    fn default() -> Self {
        CollisionSolver::new()
    }
}
