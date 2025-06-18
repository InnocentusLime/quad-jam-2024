use macroquad::prelude::*;
use shipyard::{Component, EntityId};

#[derive(Debug, Clone, Copy, Component)]
pub struct PlayerScore(pub u32);

#[derive(Debug, Clone, Copy, Component)]
#[repr(transparent)]
pub struct Health(pub i32);

#[derive(Debug, Clone, Copy, Component)]
pub struct PlayerTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct RayTag {
    pub shooting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum TileType {
    Wall,
    Ground,
}

#[derive(Debug, Component)]
pub struct TileStorage {
    width: usize,
    height: usize,
    mem: Vec<EntityId>,
}

impl TileStorage {
    pub fn from_data(width: usize, height: usize, mem: Vec<EntityId>) -> Option<TileStorage> {
        if mem.len() != width * height {
            return None;
        }

        Some(TileStorage { width, height, mem })
    }

    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> TileStorage {
        TileStorage::from_data(width, height, vec![EntityId::dead(); width * height]).unwrap()
    }

    #[allow(dead_code)]
    pub fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<EntityId> {
        debug_assert!(self.mem.len() == self.width * self.height);

        if x >= self.width {
            return None;
        }
        if y >= self.height {
            return None;
        }

        Some(self.mem[y * self.width + x])
    }

    #[allow(dead_code)]
    pub fn set(&mut self, x: usize, y: usize, val: EntityId) {
        debug_assert!(self.mem.len() < self.width * self.height);

        if x < self.width {
            return;
        }
        if y < self.height {
            return;
        }

        self.mem[y * self.width + x] = val;
    }

    /// Returns the iterator over elements of form (x, y, entity)
    pub fn iter_poses(&'_ self) -> impl Iterator<Item = (usize, usize, EntityId)> + '_ {
        self.mem
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx % self.width, idx / self.width, *val))
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct TileSmell {
    pub time_left: f32,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct GoalTag {
    pub achieved: bool,
}
