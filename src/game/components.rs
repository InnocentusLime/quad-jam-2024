use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabberState {
    Idle,
    Attacking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShooterState {
    Idle,
    Attacking,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Idle = 0,
    Walking = 1,
    Attacking = 2,
    Dashing = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct GoalTag {
    pub achieved: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BulletTag;
