use macroquad::prelude::*;
use shipyard::{Component, EntityId, Unique};

#[derive(Debug, Clone, Copy)]
pub enum RewardState {
    Locked,
    Pending,
    Counted,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct RewardInfo {
    pub state: RewardState,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy, Unique)]
pub struct PlayerScore(pub u32);

#[derive(Debug, Clone, Copy, Component)]
#[repr(transparent)]
pub struct Health(pub i32);

#[derive(Debug, Clone, Copy, Component)]
pub enum EnemyState {
    Free,
    Stunned { left: f32 },
    Dead,
}

// TODO: this is a hack, because deleting entities
// in shipyard is unreasonably difficult
#[derive(Debug, Clone, Copy, Component)]
pub enum BulletTag {
    Dropped,
    PickedUp,
    Thrown {
        dir: Vec2,
    },
}

#[derive(Debug, Clone, Copy, Component)]
pub struct BoxTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct PlayerTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct DamageTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct PlayerDamageSensorTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct BruteTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct StalkerTag;

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

    pub fn width(&self) -> usize {
        self.width
    }

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
pub enum PlayerDamageState {
    Hittable,
    Cooldown(f32),
}

#[derive(Debug, Clone, Copy, Unique)]
pub enum SwarmBrain {
    Walk {
        think: f32,
        dir: Vec2,
    },
    Wait {
        think: f32,
    },
}

#[derive(Debug, Clone, Copy, Component)]
pub struct MainCellTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct TileSmell {
    pub time_left: f32,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct BulletHitterTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct BulletWallHitterTag;

#[derive(Debug, Clone, Copy, Component)]
pub struct GoalTag;