use hecs::Entity;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PlayerScore(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct PlayerData {
    pub stamina: f32,
    pub stamina_cooldown: f32,
    pub state: PlayerState,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Idle = 0,
    Walking = 1,
    Attacking = 2,
    Dashing = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    Wall,
    Ground,
}

#[derive(Debug)]
pub struct TileStorage {
    width: usize,
    height: usize,
    mem: Vec<Entity>,
}

impl TileStorage {
    pub fn from_data(width: usize, height: usize, mem: Vec<Entity>) -> Option<TileStorage> {
        if mem.len() != width * height {
            return None;
        }

        Some(TileStorage { width, height, mem })
    }

    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> TileStorage {
        TileStorage::from_data(width, height, vec![Entity::DANGLING; width * height]).unwrap()
    }

    #[allow(dead_code)]
    pub fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.height
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> Option<Entity> {
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
    pub fn set(&mut self, x: usize, y: usize, val: Entity) {
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
    pub fn iter_poses(&'_ self) -> impl Iterator<Item = (usize, usize, Entity)> + '_ {
        self.mem
            .iter()
            .enumerate()
            .map(|(idx, val)| (idx % self.width, idx / self.width, *val))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GoalTag {
    pub achieved: bool,
}
