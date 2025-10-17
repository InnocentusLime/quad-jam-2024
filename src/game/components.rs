use hecs::Entity;
use lib_anim::{Animation, AnimationId};
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PlayerScore(pub u32);

/// [Health] component stores entity's health.
/// Normally, to do damage, you should just put it into the `damage` field.
/// `damage` is zeroed every frame and is substracted to `value`.
/// When the `block_damage` flag is raised, `damage` is ignored this frame.
#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub value: i32,
    pub damage: i32,
    pub block_damage: bool,
}

impl Health {
    pub fn new(value: i32) -> Self {
        Self {
            value,
            damage: 0,
            block_damage: false,
        }
    }
}

/// [DamageCooldown] enables cooldown on damage.
/// When [Health] contains more than zero damage and the entity
/// has [DamageCooldown] component, the game will raise the `block_damage`
/// flag. It will remain raised for the duration of `max_value`.
/// `remaining` is used to track the remaining invulnerability time.
#[derive(Debug, Clone, Copy)]
pub struct DamageCooldown {
    pub remaining: f32,
    pub max_value: f32,
}

impl DamageCooldown {
    pub fn new(max_value: f32) -> Self {
        Self {
            max_value,
            remaining: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerData {
    pub look_direction: Vec2,
    pub state: PlayerState,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Idle = 0,
    Walking = 1,
    Attacking = 2,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PlayerAttackTag;

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

pub struct AnimationPlay {
    pub animation: AnimationId,
    pub total_dt: f32,
    pub cursor: u32,
}

impl AnimationPlay {
    pub fn is_done(&self, animation: &Animation) -> bool {
        if animation.is_looping {
            return false;
        }
        self.cursor == animation.max_pos()
    }
}
