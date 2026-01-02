use glam::Vec2;

use crate::SHAPE_TOI_EPSILON;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    pub fn overlaps(self, other: Self) -> bool {
        (self.min.x <= other.max.x && self.max.x >= other.min.x)
            && (self.min.y <= other.max.y && self.max.y >= other.min.y)
    }

    pub fn contains(self, point: Vec2) -> bool {
        self.min.x <= point.x
            && self.min.y <= point.y
            && point.x <= self.max.x
            && point.y <= self.max.y
    }

    pub fn size(self) -> Vec2 {
        self.max - self.min
    }

    pub fn cast_rect(self, aabb: Self, dir: Vec2, t_max: f32) -> bool {
        let half_ext = aabb.size() / 2.0;
        let new_rect = self.expand(half_ext);
        let point = aabb.min + half_ext;
        if new_rect
            .expand(Vec2::splat(SHAPE_TOI_EPSILON))
            .contains(point)
        {
            return true;
        }
        new_rect.cast_point(point, dir, t_max)
    }

    pub fn cast_point(self, origin: Vec2, dir: Vec2, t_max: f32) -> bool {
        let recip = dir.recip();
        let vmin = (self.min - origin) * recip;
        let vmax = (self.max - origin) * recip;
        let tmin = vmin.min(vmax).extend(0.0);
        let tmax = vmin.max(vmax).extend(t_max);
        tmin.max_element() <= tmax.min_element()
    }

    pub fn expand(self, delta: Vec2) -> Aabb {
        Aabb {
            min: self.min - delta,
            max: self.max + delta,
        }
    }
}
