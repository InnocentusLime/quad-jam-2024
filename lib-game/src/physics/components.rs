use macroquad::prelude::*;
use rapier2d::prelude::InteractionGroups;
use shipyard::{Component, EntityId};

pub mod groups {
    use rapier2d::prelude::*;

    pub const LEVEL: Group = Group::GROUP_1;
    pub const NPCS: Group = Group::GROUP_2;
    pub const PLAYER: Group = Group::GROUP_3;
    pub const PROJECTILES: Group = Group::GROUP_4;
    pub const MAINCELL: Group = Group::GROUP_5;

    pub const LEVEL_INTERACT: Group = LEVEL.union(NPCS).union(PLAYER).union(PROJECTILES);
    pub const PLAYER_INTERACT: Group = LEVEL;
    pub const NPCS_INTERACT: Group = LEVEL.union(PROJECTILES).union(NPCS);
    pub const PROJECTILES_INTERACT: Group = LEVEL;
}

#[derive(Clone, Copy, Debug)]
pub enum ColliderTy {
    Box { width: f32, height: f32 },
    Circle { radius: f32 },
}

#[derive(Clone, Debug, Component)]
pub struct BeamTag {
    pub width: f32,
    pub length: f32,
    pub cast_filter: InteractionGroups,
    pub overlap_filter: InteractionGroups,
    pub overlaps: Vec<EntityId>,
}

impl BeamTag {
    pub fn new(
        overlap_filter: InteractionGroups,
        cast_filter: InteractionGroups,
        width: f32,
    ) -> Self {
        Self {
            width,
            length: 0.0f32,
            cast_filter,
            overlap_filter,
            overlaps: Vec::with_capacity(32),
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct OneSensorTag {
    pub shape: ColliderTy,
    pub groups: InteractionGroups,
    pub col: Option<EntityId>,
}

impl OneSensorTag {
    pub fn new(shape: ColliderTy, groups: InteractionGroups) -> Self {
        Self {
            shape,
            groups,
            col: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct KinematicControl {
    pub dr: Vec2,
    pub slide: bool,
}

impl KinematicControl {
    pub fn new() -> Self {
        Self {
            dr: Vec2::ZERO,
            slide: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct ForceApplier {
    pub force: Vec2,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct ImpulseApplier {
    pub impulse: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyKind {
    Static,
    Dynamic,
    Kinematic,
}

#[derive(Clone, Copy, Debug, Component)]
#[track(Deletion, Removal, Insertion)]
pub struct BodyTag {
    pub enabled: bool,
    pub groups: InteractionGroups,
    pub(crate) shape: ColliderTy,
    pub(crate) mass: f32,
    pub(crate) kind: BodyKind,
}

impl BodyTag {
    pub fn new(
        groups: InteractionGroups,
        shape: ColliderTy,
        mass: f32,
        enabled: bool,
        kind: BodyKind,
    ) -> Self {
        Self {
            enabled,
            groups,
            mass,
            shape,
            kind,
        }
    }

    pub fn shape(&self) -> &ColliderTy {
        &self.shape
    }

    pub fn kind(&self) -> BodyKind {
        self.kind
    }
}
